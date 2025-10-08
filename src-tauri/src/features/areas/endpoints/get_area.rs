/// 获取单个 Area API
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

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
CABC for `get_area`

## 1. 端点签名 (Endpoint Signature)

GET /api/areas/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要查询单个领域的详细信息，
> 以便查看该领域的名称、颜色、父领域关系和创建/更新时间等元数据。

### 2.2. 核心业务逻辑 (Core Business Logic)

从数据库中查询指定 ID 的领域，仅返回未删除的领域（`is_deleted = false`）。
将查询结果组装成 `AreaDto` 并返回。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 领域ID

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `AreaDto`

```json
{
  "id": "uuid",
  "name": "string",
  "color": "string (#RRGGBB)",
  "parent_area_id": "uuid | null",
  "created_at": "ISO8601 timestamp",
  "updated_at": "ISO8601 timestamp"
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Area not found: {id}"
}
```

## 4. 验证规则 (Validation Rules)

- `area_id`:
    - **必须**是有效的 UUID 格式（由 Axum 路径提取器自动验证）。
    - **必须**存在于数据库中。
    - **必须**未被删除（`is_deleted = false`）。
    - 违反时返回 `404 NOT_FOUND`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  从路径参数中提取 `area_id`（Axum 自动解析 UUID）。
2.  调用 `database::find_area_by_id` 查询数据库。
3.  如果领域不存在或已删除，返回 `404 Not Found` 错误。
4.  组装 `AreaDto` 并返回 `200 OK`。

## 6. 边界情况 (Edge Cases)

- **`area_id` 格式无效（非 UUID）:** Axum 路径提取器自动返回 `400 Bad Request`。
- **领域不存在:** 返回 `404` 错误。
- **领域已删除:** 返回 `404` 错误（查询条件中排除了已删除的领域）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库读取:**
    - **`SELECT`:** 1次，查询 `areas` 表以获取领域详情（带 `is_deleted = false` 过滤）。
- **日志记录:**
    - 失败时（如领域不存在），可能以 `WARN` 级别记录详细错误信息。

*（无数据库写入，无 SSE 事件，无其他已知副作用）*
*/

pub async fn handle(State(app_state): State<AppState>, Path(area_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, area_id).await {
        Ok(area_dto) => success_response(area_dto).into_response(),
        Err(err) => err.into_response(),
    }
}

mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, area_id: Uuid) -> AppResult<AreaDto> {
        let pool = app_state.db_pool();
        let area = database::find_area_by_id(pool, area_id)
            .await?
            .ok_or_else(|| AppError::not_found("Area", area_id.to_string()))?;

        Ok(AreaDto {
            id: area.id,
            name: area.name,
            color: area.color,
            parent_area_id: area.parent_area_id,
            created_at: area.created_at,
            updated_at: area.updated_at,
        })
    }
}

mod database {
    use super::*;
    use crate::entities::AreaRow;

    pub async fn find_area_by_id(
        pool: &sqlx::SqlitePool,
        area_id: Uuid,
    ) -> AppResult<Option<Area>> {
        let query = r#"
            SELECT id, name, color, parent_area_id, created_at, updated_at, is_deleted
            FROM areas
            WHERE id = ? AND is_deleted = false
        "#;

        let row = sqlx::query_as::<_, AreaRow>(query)
            .bind(area_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        match row {
            Some(r) => {
                let area = Area::try_from(r).map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::QueryError(e))
                })?;
                Ok(Some(area))
            }
            None => Ok(None),
        }
    }
}
