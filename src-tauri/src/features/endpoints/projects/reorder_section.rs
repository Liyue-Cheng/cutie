/// 重排序 ProjectSection API - 单文件组件
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::ProjectSectionDto,
    features::shared::{repositories::ProjectSectionRepository, TransactionHelper},
    infra::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
        LexoRankService,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `reorder_section`

## 1. 端点签名 (Endpoint Signature)

POST /api/projects/{project_id}/sections/{section_id}/reorder

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要通过拖放来重新排序项目中的章节，
> 以便更好地组织项目结构。

### 2.2. 核心业务逻辑 (Core Business Logic)

根据前后邻居章节的 ID，使用 LexoRank 算法计算新的 sort_order，
并更新被拖动章节的排序位置。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `project_id` (UUID, required): 项目ID
- `section_id` (UUID, required): 被拖动的章节ID

**请求体 (Request Body):** `application/json`

```json
{
  "prev_section_id": "uuid | null",  // 前一个章节ID（null表示移到最前面）
  "next_section_id": "uuid | null"   // 后一个章节ID（null表示移到最后面）
}
```

### 3.2. 响应 (Responses)

**200 OK:**

```json
{
  "id": "uuid",
  "project_id": "uuid",
  "title": "string",
  "description": "string | null",
  "sort_order": "string",
  "created_at": "ISO8601 timestamp",
  "updated_at": "ISO8601 timestamp"
}
```

**404 Not Found:** 章节不存在或不属于该项目
**422 Unprocessable Entity:** 验证失败

## 4. 验证规则 (Validation Rules)

- 被拖动的章节必须存在且属于指定项目
- 如果提供 prev_section_id，该章节必须存在且属于同一项目
- 如果提供 next_section_id，该章节必须存在且属于同一项目
- prev_section_id 和 next_section_id 不能同时为 null（除非是第一个章节）

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1. 验证输入。
2. 获取写入许可。
3. 开启事务。
4. 查询被拖动的章节，验证存在性和归属。
5. 查询前后邻居章节的 sort_order（如果提供了ID）。
6. 使用 LexoRankService 计算新的 sort_order。
7. 更新章节的 sort_order。
8. 写入 Event Outbox：发送 `project_section.reordered` 事件。
9. 提交事务。
10. 返回更新后的 ProjectSectionDto。

## 6. 边界情况 (Edge Cases)

- **章节不存在:** 返回 `404`。
- **章节不属于指定项目:** 返回 `404`。
- **邻居章节不存在:** 返回 `404`。
- **邻居章节不属于同一项目:** 返回 `422`。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 查询章节信息。
    - **`UPDATE`:** 1条记录到 `project_sections` 表。
    - **`INSERT`:** 1条记录到 `event_outbox` 表。
- **SSE 事件:**
    - **事件类型:** `project_section.reordered`
    - **载荷:** 完整的 `ProjectSectionDto` 对象

## 8. 契约 (Contract)

- HTTP 响应与 SSE 事件载荷完全一致
*/

// ==================== 请求/响应 DTOs ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorderSectionRequest {
    pub prev_section_id: Option<Uuid>,
    pub next_section_id: Option<Uuid>,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path((project_id, section_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<ReorderSectionRequest>,
) -> Response {
    match logic::execute(&app_state, project_id, section_id, request).await {
        Ok(section_dto) => success_response(section_dto).into_response(),
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
        request: ReorderSectionRequest,
    ) -> AppResult<ProjectSectionDto> {
        // 1. 获取写入许可
        let _permit = app_state.acquire_write_permit().await;

        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 2. 查询被拖动的章节
        let mut section = ProjectSectionRepository::find_by_id(&mut *tx, section_id)
            .await?
            .ok_or_else(|| AppError::not_found("ProjectSection", section_id.to_string()))?;

        // 3. 验证章节属于指定项目
        if section.project_id != project_id {
            return Err(AppError::not_found(
                "ProjectSection",
                section_id.to_string(),
            ));
        }

        // 4. 获取前后邻居的 sort_order
        let prev_sort_order = if let Some(prev_id) = request.prev_section_id {
            let prev_section = ProjectSectionRepository::find_by_id(&mut *tx, prev_id)
                .await?
                .ok_or_else(|| AppError::not_found("ProjectSection (prev)", prev_id.to_string()))?;

            // 验证属于同一项目
            if prev_section.project_id != project_id {
                return Err(AppError::validation_error(
                    "prev_section_id",
                    "前一个章节不属于同一项目",
                    "PREV_SECTION_NOT_IN_PROJECT",
                ));
            }

            prev_section.sort_order
        } else {
            None
        };

        let next_sort_order = if let Some(next_id) = request.next_section_id {
            let next_section = ProjectSectionRepository::find_by_id(&mut *tx, next_id)
                .await?
                .ok_or_else(|| AppError::not_found("ProjectSection (next)", next_id.to_string()))?;

            // 验证属于同一项目
            if next_section.project_id != project_id {
                return Err(AppError::validation_error(
                    "next_section_id",
                    "后一个章节不属于同一项目",
                    "NEXT_SECTION_NOT_IN_PROJECT",
                ));
            }

            next_section.sort_order
        } else {
            None
        };

        // 5. 计算新的 sort_order
        let new_sort_order = LexoRankService::generate_between(
            prev_sort_order.as_deref(),
            next_sort_order.as_deref(),
        )?;

        let now = app_state.clock().now_utc();

        // 6. 更新章节
        section.sort_order = Some(new_sort_order.clone());
        section.updated_at = now;

        ProjectSectionRepository::reorder(&mut tx, section_id, new_sort_order, now).await?;

        // 7. 组装 DTO
        let section_dto = ProjectSectionDto {
            id: section.id,
            project_id: section.project_id,
            title: section.title.clone(),
            description: section.description.clone(),
            sort_order: section.sort_order.clone(),
            created_at: section.created_at,
            updated_at: section.updated_at,
        };

        // 8. 写入 Event Outbox
        events::write_section_reordered_event(app_state, &mut tx, &section_dto, now).await?;

        // 9. 提交事务
        TransactionHelper::commit(tx).await?;

        Ok(section_dto)
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

    pub async fn write_section_reordered_event(
        app_state: &AppState,
        tx: &mut Transaction<'_, Sqlite>,
        section_dto: &ProjectSectionDto,
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());
        let payload = serde_json::to_value(section_dto)?;

        // 复用 updated 事件类型，因为本质上是更新了 sort_order
        let event = DomainEvent::new(
            "project_section.updated",
            "project_section",
            section_dto.id.to_string(),
            payload,
        )
        .with_aggregate_version(now.timestamp_millis());

        outbox_repo.append_in_tx(tx, &event).await?;

        Ok(())
    }
}
