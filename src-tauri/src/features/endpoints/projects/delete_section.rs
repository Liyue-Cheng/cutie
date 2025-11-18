/// 删除 ProjectSection API - 单文件组件
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    features::shared::{repositories::ProjectSectionRepository, TransactionHelper},
    infra::{
        core::{AppError, AppResult},
        http::error_handler::no_content_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `delete_section`

## 1. 端点签名 (Endpoint Signature)

DELETE /api/projects/{project_id}/sections/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要删除一个不再使用的章节，
> 以便清理项目结构。

### 2.2. 核心业务逻辑 (Core Business Logic)

软删除数据库中指定 ID 的章节（设置 `is_deleted = true`，更新 `updated_at`）。
同时清除相关任务的 section_id（任务保留在项目中）。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `project_id` (UUID, required): 项目ID
- `id` (UUID, required): 章节ID

### 3.2. 响应 (Responses)

**204 No Content:**

*   成功删除，无响应体。

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "ProjectSection not found: {id}"
}
```

## 4. 验证规则 (Validation Rules)

- `section_id`:
    - **必须**是有效的 UUID 格式（由 Axum 路径提取器自动验证）。
    - **必须**存在于数据库中。
    - **必须**未被删除（`is_deleted = false`）。
    - **必须**属于指定项目。
    - 违反时返回 `404 NOT_FOUND`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1. 从路径参数中提取 `section_id` 和 `project_id`。
2. 获取写入许可。
3. 启动数据库事务。
4. 检查章节是否存在并属于指定项目。
5. 如果章节不存在或已删除或不属于指定项目，返回 `404 Not Found`。
6. 调用 `ProjectSectionRepository::soft_delete` 软删除章节：
    - 设置 `is_deleted = true`
    - 更新 `updated_at`
    - 清除相关任务的 section_id（`SET section_id = NULL`）
7. 写入 Event Outbox：发送 `project_section.deleted` 事件。
8. 提交数据库事务。
9. 返回 `204 No Content`。

## 6. 边界情况 (Edge Cases)

- **章节不存在:** 返回 `404` 错误。
- **章节已删除:** 幂等，返回 `404` 错误。
- **章节不属于指定项目:** 返回 `404` 错误。
- **章节有关联任务:** 清除任务的 section_id，任务保留在项目中。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次，查询章节是否存在。
    - **`UPDATE`:** 2次，软删除章节和清除任务的 section_id。
    - **`INSERT`:** 1条记录到 `event_outbox` 表。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **级联影响:**
    - **关联任务:** 清除 `section_id`，任务保留在项目中。
- **SSE 事件:**
    - **事件类型:** `project_section.deleted`
    - **聚合类型:** `project_section`
    - **聚合ID:** 章节 UUID
    - **载荷:** `{ "id": "uuid", "project_id": "uuid" }`

## 8. 契约 (Contract)

- 软删除保留数据用于恢复或审计
- 相关任务清除 section_id 但保留在项目中
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path((project_id, section_id)): Path<(Uuid, Uuid)>,
) -> Response {
    match logic::execute(&app_state, project_id, section_id).await {
        Ok(()) => no_content_response().into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        project_id: Uuid,
        section_id: Uuid,
    ) -> AppResult<()> {
        // 获取写入许可
        let _permit = app_state.acquire_write_permit().await;

        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 检查章节是否存在并属于指定项目
        let section = ProjectSectionRepository::find_by_id(&mut *tx, section_id).await?;
        match section {
            Some(s) if s.project_id == project_id => {
                // 章节存在且属于指定项目
            }
            _ => {
                return Err(AppError::not_found("ProjectSection", section_id.to_string()));
            }
        }

        let now = app_state.clock().now_utc();

        // 软删除章节
        ProjectSectionRepository::soft_delete(&mut tx, section_id, now).await?;

        // 写入 Event Outbox
        events::write_section_deleted_event(app_state, &mut tx, section_id, project_id, now).await?;

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

    pub async fn write_section_deleted_event(
        app_state: &AppState,
        tx: &mut Transaction<'_, Sqlite>,
        section_id: Uuid,
        project_id: Uuid,
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());
        let payload = serde_json::json!({
            "id": section_id.to_string(),
            "project_id": project_id.to_string(),
        });

        let event = DomainEvent::new(
            "project_section.deleted",
            "project_section",
            section_id.to_string(),
            payload,
        )
        .with_aggregate_version(now.timestamp_millis());

        outbox_repo.append_in_tx(tx, &event).await?;

        Ok(())
    }
}

