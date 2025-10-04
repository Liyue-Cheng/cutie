/// 列出所有 Areas API - 单文件组件
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};

use crate::{
    entities::{Area, AreaDto},
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `list_areas`

## 1. 端点签名 (Endpoint Signature)

GET /api/areas

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要查看所有可用的领域列表，
> 以便了解我的任务分类体系并选择合适的领域来分类任务。

### 2.2. 核心业务逻辑 (Core Business Logic)

从数据库中查询所有未删除的领域（`is_deleted = false`），
按名称字母序升序排列，并将结果组装成 `AreaDto` 列表返回。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- 无

**Query Parameters:**
- 无（当前版本不支持分页、过滤、排序参数）

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `AreaDto[]`

```json
[
  {
    "id": "uuid",
    "name": "string",
    "color": "string (#RRGGBB)",
    "parent_area_id": "uuid | null",
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

1.  调用 `database::find_all_areas` 查询数据库：
    - 查询 `areas` 表
    - 过滤条件：`is_deleted = false`
    - 排序：按 `name` 字段升序
2.  将查询结果（`Vec<Area>`）映射为 `Vec<AreaDto>`。
3.  返回 `200 OK` 和领域列表。

## 6. 边界情况 (Edge Cases)

- **数据库中没有领域:** 返回空数组 `[]`（200 OK）。
- **所有领域都已删除:** 返回空数组 `[]`（200 OK）。
- **领域数量很大:** 当前无分页机制，可能返回大量数据（性能问题）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库读取:**
    - **`SELECT`:** 1次，查询 `areas` 表所有未删除的领域（带 `is_deleted = false` 过滤，按 `name` 排序）。
- **日志记录:**
    - 失败时（数据库错误），以 `ERROR` 级别记录详细错误信息。

*（无数据库写入，无 SSE 事件，无其他已知副作用）*

**性能考虑：**
1. 当前实现会一次性返回所有领域，没有分页机制。
2. 如果领域数量超过数百个，建议添加分页参数（limit/offset 或 cursor-based）。
3. 考虑添加客户端缓存或 SSE 订阅机制，减少重复查询。
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>) -> Response {
    match logic::execute(&app_state).await {
        Ok(areas) => success_response(areas).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState) -> AppResult<Vec<AreaDto>> {
        let pool = app_state.db_pool();
        let areas = database::find_all_areas(pool).await?;

        let area_dtos = areas
            .into_iter()
            .map(|area| AreaDto {
                id: area.id,
                name: area.name,
                color: area.color,
                parent_area_id: area.parent_area_id,
                created_at: area.created_at,
                updated_at: area.updated_at,
            })
            .collect();

        Ok(area_dtos)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::entities::AreaRow;

    pub async fn find_all_areas(pool: &sqlx::SqlitePool) -> AppResult<Vec<Area>> {
        let query = r#"
            SELECT id, name, color, parent_area_id, created_at, updated_at, is_deleted
            FROM areas
            WHERE is_deleted = false
            ORDER BY name ASC
        "#;

        let rows = sqlx::query_as::<_, AreaRow>(query)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        let areas: Result<Vec<Area>, _> = rows.into_iter().map(Area::try_from).collect();

        areas.map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }
}
