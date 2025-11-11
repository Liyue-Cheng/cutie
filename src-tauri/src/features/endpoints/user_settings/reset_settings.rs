/// 重置所有用户设置 API - 单文件组件
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};

use crate::{
    entities::user_setting::ResetResponse,
    features::user_settings::shared::{create_all_default_entities, UserSettingRepository},
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
CABC for `reset_settings`

## 1. 端点签名 (Endpoint Signature)

POST /api/user-settings/reset

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要重置所有设置为默认值，
> 以便我能恢复应用的初始配置（如遇到配置错误或想重新开始）。

### 2.2. 核心业务逻辑 (Core Business Logic)

删除数据库中所有用户自定义设置，使应用恢复到默认配置状态。
返回系统默认设置列表供前端重新加载。发送 SSE 事件通知前端设置已重置。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**无请求参数或请求体**

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `ResetResponse`

```json
{
  "reset_count": 9,
  "settings": [
    {
      "setting_key": "appearance.language",
      "setting_value": "\"en\"",
      "value_type": "string",
      "category": "appearance",
      "updated_at": "2025-01-11T12:00:00Z",
      "created_at": "2025-01-11T12:00:00Z"
    }
  ]
}
```

**500 Internal Server Error:**

```json
{
  "error_code": "INTERNAL_ERROR",
  "message": "Failed to reset settings"
}
```

## 4. 验证规则 (Validation Rules)

- 无输入验证需求（无请求参数）

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  获取写入许可（`acquire_write_permit`），确保写操作串行执行。
2.  删除数据库中所有用户设置（`UserSettingRepository::delete_all`）。
3.  获取系统默认设置列表（`create_all_default_entities`）。
4.  在事务中写入 SSE 事件到 EventOutbox（`user_settings.reset`）。
5.  返回重置数量和默认设置列表。

## 6. 边界情况 (Edge Cases)

- **数据库已为空:** `reset_count` 为 0，仍返回默认设置列表。
- **数据库连接失败:** 返回 `500` 错误。
- **事务提交失败:** 返回 `500` 错误，但删除操作已回滚。

## 7. 预期副作用 (Expected Side Effects)

- **数据库操作:**
    - **`DELETE`:** 1次，删除 `user_settings` 表的所有记录。
    - **`INSERT`:** 1次，写入事件到 `event_outbox` 表。
    - **(事务):** 事件写入操作包含在事务内。
    - **(写入许可):** 通过 semaphore 确保写操作串行执行。
- **SSE 事件:**
    - **`user_settings.reset`:** 发送默认设置列表。
- **日志记录:**
    - 成功时，可能以 `INFO` 级别记录 "All settings reset to default"。
    - 失败时（数据库错误），以 `ERROR` 级别记录错误信息。

*（无其他已知副作用）*
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>) -> Response {
    match logic::execute(&app_state).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState) -> AppResult<ResetResponse> {
        let pool = app_state.db_pool();

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 1. 删除所有设置
        let deleted_count = UserSettingRepository::delete_all(pool).await?;

        // 2. 获取默认设置
        let default_settings = create_all_default_entities();
        let dtos = default_settings
            .into_iter()
            .map(|s| s.into())
            .collect::<Vec<_>>();

        // 3. 在事务中写入 SSE 事件到 outbox
        database::emit_reset_event(pool, &dtos).await?;

        Ok(ResetResponse {
            reset_count: deleted_count as usize,
            settings: dtos,
        })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::entities::user_setting::UserSettingDto;

    pub async fn emit_reset_event(
        pool: &sqlx::SqlitePool,
        dtos: &[UserSettingDto],
    ) -> AppResult<()> {
        let outbox_repo = SqlxEventOutboxRepository::new(pool.clone());
        let payload = serde_json::json!({ "settings": dtos });
        let event = DomainEvent::new(
            "user_settings.reset",
            "user_setting",
            "reset".to_string(),
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
// - UserSettingRepository::delete_all
