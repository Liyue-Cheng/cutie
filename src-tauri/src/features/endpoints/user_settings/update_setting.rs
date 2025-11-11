/// 更新单个用户设置 API - 单文件组件
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::{Sqlite, Transaction};

use crate::{
    entities::user_setting::{UpdateSettingRequest, UserSetting, UserSettingDto},
    features::user_settings::shared::{get_default_value, UserSettingRepository},
    infra::{
        core::{AppError, AppResult},
        events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        },
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `update_setting`

## 1. 端点签名 (Endpoint Signature)

PUT /api/user-settings/{key}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要更新单个设置项的值，
> 以便我能自定义应用的行为和外观（如切换语言、调整缩放等）。

### 2.2. 核心业务逻辑 (Core Business Logic)

更新或创建指定 key 的设置项（UPSERT 语义）。系统会验证 key 是否在默认设置列表中，
并根据 value_type 序列化值为 JSON 字符串。更新成功后发送 SSE 事件通知前端。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `key` (String, required): 设置键，例如 `appearance.language`

**请求体 (Request Body):** `application/json`

```json
{
  "value": "zh-CN",
  "value_type": "string"
}
```

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `UserSettingDto`

```json
{
  "setting_key": "appearance.language",
  "setting_value": "\"zh-CN\"",
  "value_type": "string",
  "category": "appearance",
  "updated_at": "2025-01-11T12:30:00Z",
  "created_at": "2025-01-11T12:00:00Z"
}
```

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "key", "code": "UNKNOWN_KEY", "message": "Unknown setting key 'xxx' and no default found" }
  ]
}
```

## 4. 验证规则 (Validation Rules)

- `key`:
    - **必须**存在于默认设置列表中或已存在于数据库中。
    - 违反时返回错误码：`UNKNOWN_KEY`
- `value`:
    - **必须**是有效的 JSON 值（可序列化）。
    - 违反时返回错误码：`INVALID_JSON`
- `value_type`:
    - **必须**是枚举值之一：`string`, `number`, `boolean`, `object`, `array`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  从路径参数提取 `key`，从请求体提取 `value` 和 `value_type`。
2.  查询数据库获取该设置的 category，如果不存在则从默认设置列表获取。
3.  如果 key 既不在数据库中也不在默认列表中，返回 `422` 错误。
4.  将 `value` 序列化为 JSON 字符串。
5.  获取写入许可（`acquire_write_permit`），确保写操作串行执行。
6.  创建或更新设置实体（`UserSettingRepository::upsert`）。
7.  在事务中写入 SSE 事件到 EventOutbox（`user_settings.updated`）。
8.  提交事务。
9.  返回更新后的设置 DTO。

## 6. 边界情况 (Edge Cases)

- **key 不在默认列表中且数据库中也不存在:** 返回 `422` 错误。
- **value 无法序列化为 JSON:** 返回 `422` 错误。
- **首次设置某个 key:** 执行 INSERT 操作。
- **更新已存在的 key:** 执行 UPDATE 操作。

## 7. 预期副作用 (Expected Side Effects)

- **数据库操作:**
    - **`SELECT`:** 1次，查询现有设置或验证 key。
    - **`INSERT/UPDATE`:** 1次，UPSERT 设置到 `user_settings` 表。
    - **`INSERT`:** 1次，写入事件到 `event_outbox` 表。
    - **(事务):** 所有写操作包含在一个事务内。
    - **(写入许可):** 通过 semaphore 确保写操作串行执行。
- **SSE 事件:**
    - **`user_settings.updated`:** 发送更新后的设置 DTO。
- **日志记录:**
    - 成功时，可能以 `INFO` 级别记录 "Setting updated successfully"。
    - 失败时（验证失败或数据库错误），以 `WARN` 或 `ERROR` 级别记录详细错误信息。

*（无其他已知副作用）*
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(key): Path<String>,
    Json(request): Json<UpdateSettingRequest>,
) -> Response {
    match logic::execute(&app_state, key, request).await {
        Ok(setting) => success_response(setting).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        key: String,
        request: UpdateSettingRequest,
    ) -> AppResult<UserSettingDto> {
        let pool = app_state.db_pool();

        // 1. 获取 category（从默认值或现有记录）
        let category = if let Some(existing) = UserSettingRepository::find_by_key(pool, &key).await?
        {
            existing.category
        } else {
            get_default_value(&key)
                .map(|d| d.category)
                .ok_or_else(|| {
                    AppError::validation_error(
                        "key",
                        format!("Unknown setting key '{}' and no default found", key),
                        "UNKNOWN_KEY",
                    )
                })?
        };

        // 2. 序列化值为 JSON 字符串
        let setting_value = serde_json::to_string(&request.value).map_err(|e| {
            AppError::validation_error("value", format!("Invalid JSON value: {}", e), "INVALID_JSON")
        })?;

        // 3. 创建设置实体
        let setting = UserSetting::new(key.clone(), setting_value, request.value_type, category);

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 4. UPSERT 设置
        let updated = UserSettingRepository::upsert(pool, &setting).await?;

        let dto: UserSettingDto = updated.clone().into();

        // 5. 在事务中写入 SSE 事件到 outbox
        database::emit_update_event(pool, &dto, &key).await?;

        Ok(dto)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;

    pub async fn emit_update_event(
        pool: &sqlx::SqlitePool,
        dto: &UserSettingDto,
        key: &str,
    ) -> AppResult<()> {
        let outbox_repo = SqlxEventOutboxRepository::new(pool.clone());
        let payload = serde_json::to_value(dto)?;
        let event =
            DomainEvent::new("user_settings.updated", "user_setting", key.to_string(), payload);

        let mut tx = pool.begin().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::TransactionFailed {
                message: format!("Failed to start transaction: {}", e),
            })
        })?;

        outbox_repo.append_in_tx(&mut tx, &event).await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::TransactionFailed {
                message: format!("Failed to commit transaction: {}", e),
            })
        })?;

        Ok(())
    }
}

// ✅ 已使用共享 Repository：
// - UserSettingRepository::find_by_key
// - UserSettingRepository::upsert
