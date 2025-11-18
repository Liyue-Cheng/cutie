/// 创建 ProjectSection API - 单文件组件
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{CreateProjectSectionRequest, ProjectSection, ProjectSectionDto},
    features::shared::{repositories::{ProjectRepository, ProjectSectionRepository}, TransactionHelper},
    infra::{
        core::{AppError, AppResult},
        http::error_handler::created_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `create_section`

## 1. 端点签名 (Endpoint Signature)

POST /api/projects/{project_id}/sections

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要在项目下创建一个新的章节，
> 以便更好地组织项目中的任务。

### 2.2. 核心业务逻辑 (Core Business Logic)

在数据库中创建一个新的 `ProjectSection` 实体。系统将验证标题唯一性（在同一项目中），
并验证项目是否存在。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `project_id` (UUID, required): 项目ID

**请求体 (Request Body):** `application/json`

```json
{
  "title": "string (required, 1-200字符)",
  "description": "string | null (optional, 0-2000字符)",
  "sort_order": "string | null (optional)"
}
```

### 3.2. 响应 (Responses)

**201 Created:**

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

**404 Not Found:** 项目不存在
**422 Unprocessable Entity:** 验证失败
**409 Conflict:** 标题在项目中已存在

## 4. 验证规则 (Validation Rules)

- `title`:
    - **必须**存在。
    - **必须**为非空字符串 (trim后)。
    - **必须**长度在 1-200 字符之间。
    - 在同一项目中**必须**唯一。
- `description`:
    - 可选，如果提供则长度不超过 2000 字符。

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1. 验证输入。
2. 获取写入许可。
3. 开启事务。
4. 检查项目是否存在。
5. 检查标题在项目中是否已存在。
6. 生成 section ID 和时间戳。
7. 创建 ProjectSection 实体。
8. 插入数据库。
9. 写入 Event Outbox：发送 `project_section.created` 事件。
10. 提交事务。
11. 返回 `201 Created` 和 ProjectSectionDto。

## 6. 边界情况 (Edge Cases)

- **项目不存在:** 返回 `404`。
- **标题已存在:** 返回 `409`。
- **标题为空:** 返回 `422`。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 检查项目存在性和标题唯一性。
    - **`INSERT`:** 1条记录到 `project_sections` 表。
    - **`INSERT`:** 1条记录到 `event_outbox` 表。
- **SSE 事件:**
    - **事件类型:** `project_section.created`
    - **载荷:** 完整的 `ProjectSectionDto` 对象

## 8. 契约 (Contract)

- HTTP 响应与 SSE 事件载荷完全一致
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(request): Json<CreateProjectSectionRequest>,
) -> Response {
    match logic::execute(&app_state, project_id, request).await {
        Ok(section_dto) => created_response(section_dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        project_id: Uuid,
        request: CreateProjectSectionRequest,
    ) -> AppResult<ProjectSectionDto> {
        // 1. 验证
        validation::validate_create_request(&request)?;

        // 2. 获取写入许可
        let _permit = app_state.acquire_write_permit().await;

        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 3. 检查项目是否存在
        let project_exists = ProjectRepository::find_by_id(&mut *tx, project_id).await?;
        if project_exists.is_none() {
            return Err(AppError::not_found("Project", project_id.to_string()));
        }

        // 4. 检查标题唯一性
        let title_exists = ProjectSectionRepository::check_title_exists_in_project(
            &mut tx,
            project_id,
            &request.title,
            None,
        )
        .await?;
        if title_exists {
            return Err(AppError::conflict("章节标题在项目中已存在"));
        }

        // 5. 生成 ID 和时间戳
        let section_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // 6. 创建 ProjectSection
        let mut section = ProjectSection::new(section_id, project_id, request.title, now);
        section.description = request.description;
        section.sort_order = request.sort_order;

        // 7. 插入数据库
        ProjectSectionRepository::insert(&mut tx, &section).await?;

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
        events::write_section_created_event(app_state, &mut tx, &section_dto, now).await?;

        // 10. 提交事务
        TransactionHelper::commit(tx).await?;

        Ok(section_dto)
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate_create_request(request: &CreateProjectSectionRequest) -> AppResult<()> {
        // 验证 title
        let title = request.title.trim();
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

        // 验证 description
        if let Some(desc) = &request.description {
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

    pub async fn write_section_created_event(
        app_state: &AppState,
        tx: &mut Transaction<'_, Sqlite>,
        section_dto: &ProjectSectionDto,
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());
        let payload = serde_json::to_value(section_dto)?;

        let event = DomainEvent::new(
            "project_section.created",
            "project_section",
            section_dto.id.to_string(),
            payload,
        )
        .with_aggregate_version(now.timestamp_millis());

        outbox_repo.append_in_tx(tx, &event).await?;

        Ok(())
    }
}

