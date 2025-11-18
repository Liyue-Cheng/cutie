/// 创建 Project API - 单文件组件
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::{Sqlite, Transaction};

use crate::{
    entities::{CreateProjectRequest, Project, ProjectDto},
    features::shared::{repositories::ProjectRepository, TransactionHelper},
    infra::{
        core::{AppError, AppResult},
        http::error_handler::created_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `create_project`

## 1. 端点签名 (Endpoint Signature)

POST /api/projects

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要创建一个新的项目，
> 以便组织和管理相关的任务集合。

### 2.2. 核心业务逻辑 (Core Business Logic)

在数据库中创建一个新的 `Project` 实体。系统将验证名称唯一性（在未删除的项目中），
新项目默认状态为 ACTIVE，统计信息（total_tasks、completed_tasks）初始化为 0。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**请求体 (Request Body):** `application/json`

```json
{
  "name": "string (required, 1-200字符)",
  "description": "string | null (optional, 0-2000字符)",
  "due_date": "YYYY-MM-DD | null (optional, 必须 >= 今天)",
  "area_id": "uuid | null (optional)"
}
```

### 3.2. 响应 (Responses)

**201 Created:**

*   **Content-Type:** `application/json`
*   **Schema:** `ProjectDto`

```json
{
  "id": "uuid",
  "name": "string",
  "description": "string | null",
  "status": "ACTIVE",
  "due_date": "YYYY-MM-DD | null",
  "completed_at": null,
  "area_id": "uuid | null",
  "total_tasks": 0,
  "completed_tasks": 0,
  "created_at": "ISO8601 timestamp",
  "updated_at": "ISO8601 timestamp"
}
```

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "name", "code": "NAME_EMPTY", "message": "名称不能为空" }
  ]
}
```

**409 Conflict:**

```json
{
  "error_code": "CONFLICT",
  "message": "项目名称已存在"
}
```

## 4. 验证规则 (Validation Rules)

- `name`:
    - **必须**存在。
    - **必须**为非空字符串 (trim后)。
    - **必须**长度在 1-200 字符之间。
    - 在未删除的项目中**必须**唯一。
    - 违反时返回错误码：`NAME_EMPTY` 或 `NAME_TOO_LONG` 或 `CONFLICT`
- `description`:
    - 可选，如果提供则长度不超过 2000 字符。
    - 违反时返回错误码：`DESCRIPTION_TOO_LONG`
- `due_date`:
    - 可选，如果提供则必须是有效的日期格式 (YYYY-MM-DD)。
    - 可选，如果提供则必须 >= 今天。
    - 违反时返回错误码：`INVALID_DUE_DATE` 或 `DUE_DATE_IN_PAST`
- `area_id`:
    - 如果提供，**必须**是有效的 UUID 格式。
    - 如果提供，**应该**指向一个存在的 Area（当前未强制验证）。

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  验证 `name` 非空且长度符合要求。
2.  验证 `description` 长度符合要求（如果提供）。
3.  验证 `due_date` 格式和有效性（如果提供）。
4.  获取写入许可（`acquire_write_permit`）。
5.  启动数据库事务（`db_pool().begin()`）。
6.  调用 `ProjectRepository::check_name_exists` 检查名称是否已存在。
7.  如果名称已存在，返回 `409 Conflict` 错误并回滚事务。
8.  通过 `IdGenerator` 生成新的 `project_id`（UUID）。
9.  通过 `Clock` 服务获取当前时间 `now`。
10. 构造 `Project` 实体对象。
11. 调用 `ProjectRepository::insert` 持久化项目到 `projects` 表。
12. 写入 Event Outbox（事务内）：发送 `project.created` 事件。
13. 提交数据库事务（`tx.commit()`）。
14. 组装 `ProjectDto` 并返回 `201 Created`。

## 6. 边界情况 (Edge Cases)

- **`name` 为空或全空格:** 返回 `422` 错误，错误码 `NAME_EMPTY`。
- **`name` 已存在（未删除的项目中）:** 返回 `409` 错误，消息 "项目名称已存在"。
- **`name` 超过 200 字符:** 返回 `422` 错误，错误码 `NAME_TOO_LONG`。
- **`description` 超过 2000 字符:** 返回 `422` 错误，错误码 `DESCRIPTION_TOO_LONG`。
- **`due_date` 在过去:** 返回 `422` 错误，错误码 `DUE_DATE_IN_PAST`。
- **`area_id` 无效（不存在或格式错误）:** 当前实现中正常返回，未验证 area 存在性。
- **并发创建相同名称:** 可能导致两个检查都通过，但由于唯一约束会导致数据库错误。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次，查询 `projects` 表以检查名称唯一性。
    - **`INSERT`:** 1条记录到 `projects` 表。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（SSE 事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **SSE 事件:**
    - **事件类型:** `project.created`
    - **聚合类型:** `project`
    - **聚合ID:** 新项目的 UUID
    - **载荷:** 完整的 `ProjectDto` 对象
- **日志记录:**
    - 成功时，可能以 `INFO` 级别记录 "Project created successfully"。
    - 失败时（验证失败或数据库错误），以 `WARN` 或 `ERROR` 级别记录详细错误信息。

## 8. 契约 (Contract)

- 新创建的项目默认状态为 ACTIVE
- 统计信息（total_tasks、completed_tasks）初始化为 0
- HTTP 响应与 SSE 事件载荷完全一致
- 项目颜色从 area 继承，不存储在 projects 表中
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateProjectRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(project_dto) => created_response(project_dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        request: CreateProjectRequest,
    ) -> AppResult<ProjectDto> {
        // 1. 验证
        validation::validate_create_request(&request)?;

        // 2. 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 3. 检查名称唯一性
        let exists = ProjectRepository::check_name_exists(&mut tx, &request.name, None).await?;
        if exists {
            return Err(AppError::conflict("项目名称已存在"));
        }

        // 4. 生成 ID 和时间戳
        let project_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // 5. 创建 Project
        let mut project = Project::new(project_id, request.name, now);
        project.description = request.description;
        project.due_date = request.due_date;
        project.area_id = request.area_id;

        // 6. 插入数据库
        ProjectRepository::insert(&mut tx, &project).await?;

        // 7. 组装 DTO
        let project_dto = ProjectDto {
            id: project.id,
            name: project.name.clone(),
            description: project.description.clone(),
            status: project.status.clone(),
            due_date: project.due_date,
            completed_at: project.completed_at,
            area_id: project.area_id,
            created_at: project.created_at,
            updated_at: project.updated_at,
        };

        // 8. 写入 Event Outbox
        events::write_project_created_event(app_state, &mut tx, &project_dto, now).await?;

        // 9. 提交事务
        TransactionHelper::commit(tx).await?;

        Ok(project_dto)
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate_create_request(request: &CreateProjectRequest) -> AppResult<()> {
        // 验证 name
        let name = request.name.trim();
        if name.is_empty() {
            return Err(AppError::validation_error(
                "name",
                "名称不能为空",
                "NAME_EMPTY",
            ));
        }
        if name.len() > 200 {
            return Err(AppError::validation_error(
                "name",
                "名称长度不能超过200字符",
                "NAME_TOO_LONG",
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

        // 验证 due_date
        if let Some(due_date) = request.due_date {
            let today = chrono::Utc::now().date_naive();
            if due_date < today {
                return Err(AppError::validation_error(
                    "due_date",
                    "截止日期不能早于今天",
                    "DUE_DATE_IN_PAST",
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

    pub async fn write_project_created_event(
        app_state: &AppState,
        tx: &mut Transaction<'_, Sqlite>,
        project_dto: &ProjectDto,
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());
        let payload = serde_json::to_value(project_dto)?;

        let event = DomainEvent::new(
            "project.created",
            "project",
            project_dto.id.to_string(),
            payload,
        )
        .with_aggregate_version(now.timestamp_millis());

        outbox_repo.append_in_tx(tx, &event).await?;

        Ok(())
    }
}

