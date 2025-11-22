/// 更新 ProjectSection API - 单文件组件
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{ProjectSectionDto, UpdateProjectSectionRequest},
    features::shared::{repositories::ProjectSectionRepository, TransactionHelper},
    infra::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `update_section`

## 1. 端点签名 (Endpoint Signature)

PATCH /api/projects/{project_id}/sections/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要更新章节的信息，
> 包括标题、描述和排序。

### 2.2. 核心业务逻辑 (Core Business Logic)

根据章节 ID 更新章节信息。支持部分更新（PATCH），只更新提供的字段。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `project_id` (UUID, required): 项目ID
- `id` (UUID, required): 章节ID

**请求体 (Request Body):** `application/json`

```json
{
  "title": "string (optional, 1-200字符)",
  "description": "string | null (optional)",
  "sort_order": "string (optional)"
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
  "sort_order": "string | null",
  "created_at": "ISO8601 timestamp",
  "updated_at": "ISO8601 timestamp"
}
```

**404 Not Found:** 章节不存在
**422 Unprocessable Entity:** 验证失败
**409 Conflict:** 标题在项目中已存在

## 4. 验证规则 (Validation Rules)

- `title`: 如果提供，长度在 1-200 字符之间，且不与项目中其他章节重复
- `description`: 如果提供，长度不超过 2000 字符

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1. 验证输入。
2. 获取写入许可。
3. 开启事务。
4. 查询章节是否存在，并验证属于指定项目。
5. 如果提供 title，检查唯一性（排除当前章节）。
6. 更新章节字段。
7. 更新数据库。
8. 写入 Event Outbox：发送 `project_section.updated` 事件。
9. 提交事务。
10. 返回更新后的 ProjectSectionDto。

## 6. 边界情况 (Edge Cases)

- **章节不存在:** 返回 `404`。
- **章节不属于指定项目:** 返回 `404`。
- **标题已存在:** 返回 `409`。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 查询章节和检查标题唯一性。
    - **`UPDATE`:** 1条记录到 `project_sections` 表。
    - **`INSERT`:** 1条记录到 `event_outbox` 表。
- **SSE 事件:**
    - **事件类型:** `project_section.updated`
    - **载荷:** 完整的 `ProjectSectionDto` 对象

## 8. 契约 (Contract)

- HTTP 响应与 SSE 事件载荷完全一致
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path((project_id, section_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateProjectSectionRequest>,
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
        request: UpdateProjectSectionRequest,
    ) -> AppResult<ProjectSectionDto> {
        // 1. 验证
        validation::validate_update_request(&request)?;

        // 2. 获取写入许可
        let _permit = app_state.acquire_write_permit().await;

        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 3. 查询章节
        let mut section = ProjectSectionRepository::find_by_id(&mut *tx, section_id)
            .await?
            .ok_or_else(|| AppError::not_found("ProjectSection", section_id.to_string()))?;

        // 4. 验证章节属于指定项目
        if section.project_id != project_id {
            return Err(AppError::not_found(
                "ProjectSection",
                section_id.to_string(),
            ));
        }

        // 5. 如果更新标题，检查唯一性
        if let Some(ref new_title) = request.title {
            let exists = ProjectSectionRepository::check_title_exists_in_project(
                &mut tx,
                project_id,
                new_title,
                Some(section_id),
            )
            .await?;
            if exists {
                return Err(AppError::conflict("章节标题在项目中已存在"));
            }
        }

        let now = app_state.clock().now_utc();

        // 6. 更新字段
        if let Some(title) = request.title {
            section.title = title;
        }
        if let Some(description) = request.description {
            section.description = description;
        }
        if let Some(sort_order) = request.sort_order {
            section.sort_order = Some(sort_order);
        }

        section.updated_at = now;

        // 7. 更新数据库
        ProjectSectionRepository::update(&mut tx, &section).await?;

        // 8. 组装 DTO
        let section_dto = ProjectSectionDto {
            id: section.id,
            project_id: section.project_id,
            title: section.title.clone(),
            description: section.description.clone(),
            sort_order: section.sort_order.clone(),
            created_at: section.created_at,
            updated_at: section.updated_at,
        };

        // 9. 写入 Event Outbox
        events::write_section_updated_event(app_state, &mut tx, &section_dto, now).await?;

        // 10. 提交事务
        TransactionHelper::commit(tx).await?;

        Ok(section_dto)
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate_update_request(request: &UpdateProjectSectionRequest) -> AppResult<()> {
        // 验证 title
        if let Some(ref title) = request.title {
            let title = title.trim();
            if title.is_empty() {
                return Err(AppError::validation_error(
                    "title",
                    "标题不能为空",
                    "TITLE_EMPTY",
                ));
            }
            if title.len() > 200 {
                return Err(AppError::validation_error(
                    "title",
                    "标题长度不能超过200字符",
                    "TITLE_TOO_LONG",
                ));
            }
        }

        // 验证 description
        if let Some(Some(ref desc)) = request.description {
            if desc.len() > 2000 {
                return Err(AppError::validation_error(
                    "description",
                    "描述长度不能超过2000字符",
                    "DESCRIPTION_TOO_LONG",
                ));
            }
        }

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

    pub async fn write_section_updated_event(
        app_state: &AppState,
        tx: &mut Transaction<'_, Sqlite>,
        section_dto: &ProjectSectionDto,
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());
        let payload = serde_json::to_value(section_dto)?;

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
