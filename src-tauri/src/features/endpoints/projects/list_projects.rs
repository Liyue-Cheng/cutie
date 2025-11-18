/// 列出所有 Projects API - 单文件组件
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};

use crate::{
    entities::ProjectDto,
    features::shared::repositories::ProjectRepository,
    infra::{core::AppResult, http::error_handler::success_response},
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `list_projects`

## 1. 端点签名 (Endpoint Signature)

GET /api/projects

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要查看所有活跃的项目列表，
> 以便了解我当前正在进行的项目并管理项目中的任务。

### 2.2. 核心业务逻辑 (Core Business Logic)

从数据库中查询所有未删除的项目（`is_deleted = false`），
按 `updated_at` 降序排列，并将结果组装成 `ProjectDto` 列表返回。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- 无

**Query Parameters:**
- 无

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `ProjectDto[]`

```json
[
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
  },
  ...
]
```

**注意：** 空列表返回 `[]`，而不是错误。

## 4. 验证规则 (Validation Rules)

- 无输入参数，无需验证。

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `ProjectRepository::list_all` 查询数据库：
    - 查询 `projects` 表
    - 过滤条件：`is_deleted = false`
    - 排序：按 `updated_at` 字段降序
2.  将查询结果（`Vec<Project>`）映射为 `Vec<ProjectDto>`。
3.  返回 `200 OK` 和项目列表。

## 6. 边界情况 (Edge Cases)

- **数据库中没有项目:** 返回空数组 `[]`（200 OK）。
- **所有项目都已删除:** 返回空数组 `[]`（200 OK）。
- **项目数量很大:** 当前无分页机制，可能返回大量数据（性能问题）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库读取:**
    - **`SELECT`:** 1次，查询 `projects` 表所有未删除的项目（带 `is_deleted = false` 过滤，按 `updated_at` 降序排序）。
- **日志记录:**
    - 失败时（数据库错误），以 `ERROR` 级别记录详细错误信息。

*（无数据库写入，无 SSE 事件，无其他已知副作用）*

## 8. 契约 (Contract)

- 所有项目的 `total_tasks` 和 `completed_tasks` 字段由后端维护，保证准确性
- 项目颜色不在此 API 返回，需要前端通过 `area_id` 查询 Area 获取颜色
- 项目列表按最近更新时间降序排列
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>) -> Response {
    match logic::execute(&app_state).await {
        Ok(projects) => success_response(projects).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState) -> AppResult<Vec<ProjectDto>> {
        let pool = app_state.db_pool();
        let projects = ProjectRepository::list_all(pool).await?;

        let project_dtos = projects
            .into_iter()
            .map(|project| ProjectDto {
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
            .collect();

        Ok(project_dtos)
    }
}
