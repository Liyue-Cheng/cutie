/// 更新 Area API
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{Area, AreaDto, UpdateAreaRequest},
    infra::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `update_area`

## 1. 端点签名 (Endpoint Signature)

PUT /api/areas/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要更新一个领域的名称、颜色或父领域关系，
> 以便调整我的任务分类体系和视觉标记。

### 2.2. 核心业务逻辑 (Core Business Logic)

更新数据库中指定 ID 的领域的可选字段（`name`、`color`、`parent_area_id`）。
系统将验证新值的有效性，更新 `updated_at` 时间戳，并返回更新后的完整领域信息。
所有字段都是可选的，只更新请求中提供的字段。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 领域ID

**请求体 (Request Body):** `application/json`

```json
{
  "name": "string | null (optional)",
  "color": "string (#RRGGBB) | null (optional)",
  "parent_area_id": "uuid | null | null (optional, 双重 Option)"
}
```

**注意：** `parent_area_id` 使用 `Option<Option<Uuid>>` 结构：
- 未提供字段：不更新父领域
- 提供 `null`：清除父领域（设为根领域）
- 提供 UUID：设置新的父领域

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
  "updated_at": "ISO8601 timestamp (更新后的时间)"
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Area not found: {id}"
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

## 4. 验证规则 (Validation Rules)

- `area_id`:
    - **必须**是有效的 UUID 格式（由 Axum 路径提取器自动验证）。
    - **必须**存在于数据库中。
    - **必须**未被删除（`is_deleted = false`）。
    - 违反时返回 `404 NOT_FOUND`
- `name`:
    - 如果提供，**必须**为非空字符串 (trim后)。
    - 违反时返回错误码：`NAME_EMPTY`
- `color`:
    - 如果提供，**必须**符合格式 `#RRGGBB`。
    - 违反时返回错误码：`INVALID_COLOR`
- `parent_area_id`:
    - 如果提供 UUID，**应该**指向一个存在的 Area（当前未强制验证）。

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  从路径参数中提取 `area_id`。
2.  启动数据库事务（`db_pool().begin()`）。
3.  调用 `database::check_area_exists_in_tx` 检查领域是否存在。
4.  如果领域不存在，返回 `404 Not Found` 错误并回滚事务。
5.  验证请求字段：
    - 如果提供 `name`，验证非空。
    - 如果提供 `color`，调用 `Area::validate_color` 验证格式。
6.  调用 `database::update_area_in_tx` 更新领域：
    - 动态构建 SQL UPDATE 语句（仅更新提供的字段）
    - 自动更新 `updated_at` 为当前时间
7.  提交数据库事务（`tx.commit()`）。
8.  重新查询更新后的领域（`database::find_area_by_id`）。
9.  组装 `AreaDto` 并返回 `200 OK`。

## 6. 边界情况 (Edge Cases)

- **领域不存在:** 返回 `404` 错误。
- **领域已删除:** 返回 `404` 错误（`check_area_exists_in_tx` 排除已删除的领域）。
- **`name` 为空或全空格:** 返回 `422` 错误，错误码 `NAME_EMPTY`。
- **`color` 格式无效:** 返回 `422` 错误，错误码 `INVALID_COLOR`。
- **请求体所有字段都为 null/未提供:** 仍然成功返回，但只更新 `updated_at` 时间戳。
- **`parent_area_id` 设为自己或循环引用:** 当前实现中未验证，可能导致循环依赖。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 2次（检查存在性 + 重新查询更新后的领域）。
    - **`UPDATE`:** 1条记录在 `areas` 表（更新提供的字段 + `updated_at`）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **日志记录:**
    - 失败时（验证失败或数据库错误），以 `WARN` 或 `ERROR` 级别记录详细错误信息。

*（无 SSE 事件，无写入许可，无其他已知副作用）*
*/

pub async fn handle(
    State(app_state): State<AppState>,
    Path(area_id): Path<Uuid>,
    Json(request): Json<UpdateAreaRequest>,
) -> Response {
    match logic::execute(&app_state, area_id, request).await {
        Ok(area_dto) => success_response(area_dto).into_response(),
        Err(err) => err.into_response(),
    }
}

mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        area_id: Uuid,
        request: UpdateAreaRequest,
    ) -> AppResult<AreaDto> {
        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
        })?;

        // 1. 检查存在
        let exists = database::check_area_exists_in_tx(&mut tx, area_id).await?;
        if !exists {
            return Err(AppError::not_found("Area", area_id.to_string()));
        }

        // 2. 验证
        if let Some(name) = &request.name {
            if name.trim().is_empty() {
                return Err(AppError::validation_error(
                    "name",
                    "名称不能为空",
                    "NAME_EMPTY",
                ));
            }
        }
        if let Some(color) = &request.color {
            if !Area::validate_color(color) {
                return Err(AppError::validation_error(
                    "color",
                    "颜色格式无效",
                    "INVALID_COLOR",
                ));
            }
        }

        // 3. 更新
        database::update_area_in_tx(&mut tx, area_id, &request).await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 4. 重新查询
        let area = database::find_area_by_id(app_state.db_pool(), area_id)
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

    pub async fn check_area_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        area_id: Uuid,
    ) -> AppResult<bool> {
        let query = "SELECT COUNT(*) FROM areas WHERE id = ? AND is_deleted = false";
        let count: i64 = sqlx::query_scalar(query)
            .bind(area_id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
            })?;
        Ok(count > 0)
    }

    pub async fn update_area_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        area_id: Uuid,
        request: &UpdateAreaRequest,
    ) -> AppResult<()> {
        let now = chrono::Utc::now();
        let mut updates = Vec::new();
        let mut bindings: Vec<String> = Vec::new();

        if let Some(name) = &request.name {
            updates.push("name = ?");
            bindings.push(name.clone());
        }
        if let Some(color) = &request.color {
            updates.push("color = ?");
            bindings.push(color.clone());
        }
        if let Some(parent_id) = &request.parent_area_id {
            updates.push("parent_area_id = ?");
            bindings.push(parent_id.map(|id| id.to_string()).unwrap_or_default());
        }

        if updates.is_empty() {
            return Ok(());
        }

        updates.push("updated_at = ?");
        let update_clause = updates.join(", ");
        let query = format!("UPDATE areas SET {} WHERE id = ?", update_clause);

        let mut query_builder = sqlx::query(&query);
        for binding in bindings {
            query_builder = query_builder.bind(binding);
        }
        query_builder = query_builder.bind(now.to_rfc3339());
        query_builder = query_builder.bind(area_id.to_string());

        query_builder.execute(&mut **tx).await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
        })?;

        Ok(())
    }

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
                AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
            })?;

        match row {
            Some(r) => {
                let area = Area::try_from(r).map_err(|e| {
                    AppError::DatabaseError(crate::infra::core::DbError::QueryError(e))
                })?;
                Ok(Some(area))
            }
            None => Ok(None),
        }
    }
}
