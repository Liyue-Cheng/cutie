/// 列出项目的所有 Sections API - 单文件组件
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    entities::ProjectSectionDto,
    features::shared::repositories::ProjectSectionRepository,
    infra::{core::AppResult, http::error_handler::success_response},
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `list_sections`

## 1. 端点签名 (Endpoint Signature)

GET /api/projects/{project_id}/sections

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要查看项目下的所有章节列表，
> 以便了解项目的组织结构并管理任务分类。

### 2.2. 核心业务逻辑 (Core Business Logic)

从数据库中查询指定项目的所有未删除章节（`is_deleted = false`），
按 `sort_order` 升序排列，并将结果组装成 `ProjectSectionDto` 列表返回。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `project_id` (UUID, required): 项目ID

### 3.2. 响应 (Responses)

**200 OK:**

```json
[
  {
    "id": "uuid",
    "project_id": "uuid",
    "title": "string",
    "description": "string | null",
    "sort_order": "string | null",
    "created_at": "ISO8601 timestamp",
    "updated_at": "ISO8601 timestamp"
  },
  ...
]
```

**注意：** 空列表返回 `[]`，而不是错误。

## 4. 验证规则 (Validation Rules)

- `project_id`: **必须**是有效的 UUID 格式（由 Axum 路径提取器自动验证）。

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1. 从路径参数中提取 `project_id`。
2. 调用 `ProjectSectionRepository::list_by_project` 查询数据库：
    - 查询 `project_sections` 表
    - 过滤条件：`project_id = ?` AND `is_deleted = false`
    - 排序：按 `sort_order` 字段升序，然后按 `created_at` 升序
3. 将查询结果（`Vec<ProjectSection>`）映射为 `Vec<ProjectSectionDto>`。
4. 返回 `200 OK` 和章节列表。

## 6. 边界情况 (Edge Cases)

- **项目没有章节:** 返回空数组 `[]`（200 OK）。
- **所有章节都已删除:** 返回空数组 `[]`（200 OK）。
- **项目不存在:** 仍然返回空数组 `[]`（当前实现）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库读取:**
    - **`SELECT`:** 1次，查询 `project_sections` 表指定项目的所有未删除章节（带 `is_deleted = false` 过滤，按 `sort_order` 和 `created_at` 排序）。

*（无数据库写入，无 SSE 事件，无其他已知副作用）*

## 8. 契约 (Contract)

- 章节按 sort_order 排序，使用 Lexorank 算法保证性能
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>, Path(project_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, project_id).await {
        Ok(sections) => success_response(sections).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        project_id: Uuid,
    ) -> AppResult<Vec<ProjectSectionDto>> {
        let pool = app_state.db_pool();
        let sections = ProjectSectionRepository::list_by_project(pool, project_id).await?;

        let section_dtos = sections
            .into_iter()
            .map(|section| ProjectSectionDto {
                id: section.id,
                project_id: section.project_id,
                title: section.title,
                description: section.description,
                sort_order: section.sort_order,
                created_at: section.created_at,
                updated_at: section.updated_at,
            })
            .collect();

        Ok(section_dtos)
    }
}
