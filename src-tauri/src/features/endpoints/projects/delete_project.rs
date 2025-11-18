/// 删除 Project API - 单文件组件
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    features::shared::{repositories::ProjectRepository, TransactionHelper},
    infra::{
        core::{AppError, AppResult},
        http::error_handler::no_content_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `delete_project`

## 1. 端点签名 (Endpoint Signature)

DELETE /api/projects/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要删除一个不再使用的项目，
> 以便清理项目列表。

### 2.2. 核心业务逻辑 (Core Business Logic)

软删除数据库中指定 ID 的项目（设置 `is_deleted = true`，更新 `updated_at`）。
同时软删除所有关联的 sections 和 tasks。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 项目ID

### 3.2. 响应 (Responses)

**204 No Content:**

*   成功删除，无响应体。

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Project not found: {id}"
}
```

## 4. 验证规则 (Validation Rules)

- `project_id`:
    - **必须**是有效的 UUID 格式（由 Axum 路径提取器自动验证）。
    - **必须**存在于数据库中。
    - **必须**未被删除（`is_deleted = false`）。
    - 违反时返回 `404 NOT_FOUND`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1. 从路径参数中提取 `project_id`。
2. 获取写入许可。
3. 启动数据库事务。
4. 检查项目是否存在。
5. 如果项目不存在或已删除，返回 `404 Not Found`。
6. 调用 `ProjectRepository::soft_delete` 软删除项目：
    - 设置 `is_deleted = true`
    - 更新 `updated_at`
    - 软删除所有关联的 sections
    - 软删除所有关联的 tasks
7. 写入 Event Outbox：发送 `project.deleted` 事件。
8. 提交数据库事务。
9. 返回 `204 No Content`。

## 6. 边界情况 (Edge Cases)

- **项目不存在:** 返回 `404` 错误。
- **项目已删除:** 幂等，返回 `404` 错误。
- **项目有关联 sections:** 级联软删除所有 sections。
- **项目有关联 tasks:** 级联软删除所有 tasks。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次，查询项目是否存在。
    - **`UPDATE`:** 3次，软删除项目、sections 和 tasks。
    - **`INSERT`:** 1条记录到 `event_outbox` 表。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **级联影响:**
    - **关联 sections:** 设置 `is_deleted = true`。
    - **关联 tasks:** 设置 `deleted_at = now()`。
- **SSE 事件:**
    - **事件类型:** `project.deleted`
    - **聚合类型:** `project`
    - **聚合ID:** 项目 UUID
    - **载荷:** `{ "id": "uuid" }`

## 8. 契约 (Contract)

- 软删除保留数据用于恢复或审计
- 级联软删除所有关联的 sections 和 tasks
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>, Path(project_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, project_id).await {
        Ok(()) => no_content_response().into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, project_id: Uuid) -> AppResult<()> {
        // 获取写入许可
        let _permit = app_state.acquire_write_permit().await;

        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 检查项目是否存在
        let exists = ProjectRepository::find_by_id(&mut *tx, project_id).await?;
        if exists.is_none() {
            return Err(AppError::not_found("Project", project_id.to_string()));
        }

        let now = app_state.clock().now_utc();

        // 软删除项目及关联数据
        ProjectRepository::soft_delete(&mut tx, project_id, now).await?;

        // 写入 Event Outbox
        events::write_project_deleted_event(app_state, &mut tx, project_id, now).await?;

        // 提交事务
        TransactionHelper::commit(tx).await?;

        Ok(())
    }
}

// ==================== 事件层 ====================
mod events {
    use super::*;
    use crate::infra::events::{
        models::DomainEvent,
        outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
    };
    use chrono::{DateTime, Utc};

    pub async fn write_project_deleted_event(
        app_state: &AppState,
        tx: &mut Transaction<'_, Sqlite>,
        project_id: Uuid,
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());
        let payload = serde_json::json!({
            "id": project_id.to_string(),
        });

        let event = DomainEvent::new(
            "project.deleted",
            "project",
            project_id.to_string(),
            payload,
        )
        .with_aggregate_version(now.timestamp_millis());

        outbox_repo.append_in_tx(tx, &event).await?;

        Ok(())
    }
}
