/// 获取单个 Project API - 单文件组件
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    entities::ProjectDto,
    features::shared::repositories::ProjectRepository,
    infra::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `get_project`

## 1. 端点签名 (Endpoint Signature)

GET /api/projects/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要查看某个项目的详细信息，
> 以便了解项目的状态、描述和统计信息。

### 2.2. 核心业务逻辑 (Core Business Logic)

根据项目 ID 从数据库中查询项目详情，并返回 ProjectDto。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 项目ID

### 3.2. 响应 (Responses)

**200 OK:**

```json
{
  "id": "uuid",
  "name": "string",
  "description": "string | null",
  "status": "ACTIVE | COMPLETED",
  "due_date": "YYYY-MM-DD | null",
  "completed_at": "ISO8601 timestamp | null",
  "area_id": "uuid | null",
  "total_tasks": 0,
  "completed_tasks": 0,
  "created_at": "ISO8601 timestamp",
  "updated_at": "ISO8601 timestamp"
}
```

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
2. 调用 `ProjectRepository::find_by_id` 查询项目。
3. 如果项目不存在或已删除，返回 `404 Not Found`。
4. 组装 `ProjectDto` 并返回 `200 OK`。

## 6. 边界情况 (Edge Cases)

- **项目不存在:** 返回 `404` 错误。
- **项目已删除:** 返回 `404` 错误。

## 7. 预期副作用 (Expected Side Effects)

- **数据库读取:**
    - **`SELECT`:** 1次，根据 ID 查询 `projects` 表。

*（无数据库写入，无 SSE 事件，无其他已知副作用）*

## 8. 契约 (Contract)

- 返回的项目统计信息（total_tasks、completed_tasks）由后端维护，保证准确性
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>, Path(project_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, project_id).await {
        Ok(project) => success_response(project).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, project_id: Uuid) -> AppResult<ProjectDto> {
        let pool = app_state.db_pool();
        let project = ProjectRepository::find_by_id(pool, project_id)
            .await?
            .ok_or_else(|| AppError::not_found("Project", project_id.to_string()))?;

        Ok(ProjectDto {
            id: project.id,
            name: project.name,
            description: project.description,
            status: project.status,
            due_date: project.due_date,
            completed_at: project.completed_at,
            area_id: project.area_id,
            created_at: project.created_at,
            updated_at: project.updated_at,
        })
    }
}

