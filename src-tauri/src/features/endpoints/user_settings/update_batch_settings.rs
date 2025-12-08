/// 批量更新用户设置 API - 单文件组件
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};

use crate::{
    entities::user_setting::{BatchUpdateResponse, UpdateBatchSettingsRequest, UserSetting},
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
CABC for `update_batch_settings`

## 1. 端点签名 (Endpoint Signature)

PUT /api/user-settings

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要批量更新多个设置项，
> 以便我能一次性保存多个配置更改（如同时调整语言、缩放、主题等）。

### 2.2. 核心业务逻辑 (Core Business Logic)

批量更新多个设置项（UPSERT 语义）。所有更新操作在一个事务内执行，
保证原子性（要么全部成功，要么全部失败）。成功后发送 SSE 事件通知前端。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**请求体 (Request Body):** `application/json`

```json
{
  "settings": [
    {
      "key": "appearance.language",
      "value": "zh-CN",
      "value_type": "string"
    },
    {
      "key": "appearance.display_scale",
      "value": 110,
      "value_type": "number"
    }
  ]
}
```

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `BatchUpdateResponse`

```json
{
  "updated_count": 2,
  "settings": [
    {
      "setting_key": "appearance.language",
      "setting_value": "\"zh-CN\"",
      "value_type": "string",
      "updated_at": "2025-01-11T12:30:00Z",
      "created_at": "2025-01-11T12:00:00Z"
    },
    {
      "setting_key": "appearance.display_scale",
      "setting_value": "110",
      "value_type": "number",
      "updated_at": "2025-01-11T12:30:00Z",
      "created_at": "2025-01-11T12:00:00Z"
    }
  ]
}
```

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "key", "code": "UNKNOWN_KEY", "message": "Unknown setting key 'xxx'" }
  ]
}
```

## 4. 验证规则 (Validation Rules)

- `settings`:
    - **必须**至少包含1个设置项。
- 每个设置项的 `key`:
    - **必须**存在于默认设置列表中或已存在于数据库中。
    - 违反时返回错误码：`UNKNOWN_KEY`
- 每个设置项的 `value`:
    - **必须**是有效的 JSON 值（可序列化）。
    - 违反时返回错误码：`INVALID_JSON`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  验证 `settings` 列表非空。
2.  对每个设置项：
    - 验证 key 是否在数据库或默认设置列表中。
    - 如果 key 无效，返回 `422` 错误并终止。
    - 序列化 `value` 为 JSON 字符串。
3.  获取写入许可（`acquire_write_permit`），确保写操作串行执行。
4.  在事务内批量 UPSERT 所有设置（`UserSettingRepository::upsert_batch`）。
5.  在同一事务内写入 SSE 事件到 EventOutbox（`user_settings.batch_updated`）。
6.  提交事务。
7.  返回更新后的设置列表和更新数量。

## 6. 边界情况 (Edge Cases)

- **settings 列表为空:** 返回 `422` 错误。
- **某个 key 无效:** 整个事务回滚，返回 `422` 错误。
- **某个 value 无法序列化:** 整个事务回滚，返回 `422` 错误。
- **数据库事务失败:** 所有更新回滚，返回 `500` 错误。

## 7. 预期副作用 (Expected Side Effects)

- **数据库操作:**
    - **`SELECT`:** N次（N = settings 数量），查询现有设置或验证 key。
    - **`INSERT/UPDATE`:** N次，批量 UPSERT 设置到 `user_settings` 表。
    - **`INSERT`:** 1次，写入事件到 `event_outbox` 表。
    - **(事务):** 所有写操作包含在一个事务内，保证原子性。
    - **(写入许可):** 通过 semaphore 确保写操作串行执行。
- **SSE 事件:**
    - **`user_settings.batch_updated`:** 发送所有更新后的设置 DTO 列表。
- **日志记录:**
    - 成功时，可能以 `INFO` 级别记录 "Batch settings updated successfully"。
    - 失败时（验证失败或数据库错误），以 `WARN` 或 `ERROR` 级别记录详细错误信息。

*（无其他已知副作用）*
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<UpdateBatchSettingsRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        request: UpdateBatchSettingsRequest,
    ) -> AppResult<BatchUpdateResponse> {
        let pool = app_state.db_pool();

        // 1. 验证 settings 列表非空
        if request.settings.is_empty() {
            return Err(AppError::validation_error(
                "settings",
                "Settings list cannot be empty",
                "EMPTY_LIST",
            ));
        }

        // 2. 准备所有设置实体
        let mut settings = Vec::new();
        for update in &request.settings {
            // 验证 key 是否存在于数据库或默认列表中
            let existing = UserSettingRepository::find_by_key(pool, &update.key).await?;
            if existing.is_none() && get_default_value(&update.key).is_none() {
                return Err(AppError::validation_error(
                    "key",
                    format!("Unknown setting key '{}' and no default found", update.key),
                    "UNKNOWN_KEY",
                ));
            }

            // 序列化值
            let setting_value = serde_json::to_string(&update.value).map_err(|e| {
                AppError::validation_error(
                    "value",
                    format!("Invalid JSON value: {}", e),
                    "INVALID_JSON",
                )
            })?;

            let setting = UserSetting::new(
                update.key.clone(),
                setting_value,
                update.value_type.clone(),
            );
            settings.push(setting);
        }

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 3. 批量 UPSERT
        let updated = UserSettingRepository::upsert_batch(pool, &settings).await?;

        let dtos = updated.into_iter().map(|s| s.into()).collect::<Vec<_>>();
        let count = dtos.len();

        // 4. 在事务中写入 SSE 事件到 outbox
        database::emit_batch_update_event(pool, &dtos).await?;

        Ok(BatchUpdateResponse {
            updated_count: count,
            settings: dtos,
        })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::entities::user_setting::UserSettingDto;

    pub async fn emit_batch_update_event(
        pool: &sqlx::SqlitePool,
        dtos: &[UserSettingDto],
    ) -> AppResult<()> {
        let outbox_repo = SqlxEventOutboxRepository::new(pool.clone());
        let payload = serde_json::json!({ "settings": dtos });
        let event = DomainEvent::new(
            "user_settings.batch_updated",
            "user_setting",
            "batch".to_string(),
            payload,
        );

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
// - UserSettingRepository::upsert_batch
