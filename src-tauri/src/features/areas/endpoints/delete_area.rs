/// 删除 Area API
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    shared::{
        core::{AppError, AppResult},
        http::error_handler::no_content_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `delete_area`

## 1. 端点签名 (Endpoint Signature)

DELETE /api/areas/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要删除一个不再使用的领域，
> 以便清理我的领域列表并保持任务分类体系的整洁。

### 2.2. 核心业务逻辑 (Core Business Logic)

软删除数据库中指定 ID 的领域（设置 `is_deleted = true`，更新 `updated_at`）。
领域的删除是软删除，不会物理删除数据库记录，也不会影响已关联该领域的任务。
已关联该领域的任务仍然保留其 `area_id` 引用，但前端可以选择性地隐藏或显示已删除领域的任务。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 领域ID

### 3.2. 响应 (Responses)

**204 No Content:**

*   成功删除，无响应体。

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

1.  从路径参数中提取 `area_id`。
2.  启动数据库事务（`db_pool().begin()`）。
3.  调用 `database::check_area_exists_in_tx` 检查领域是否存在。
4.  如果领域不存在或已删除，返回 `404 Not Found` 错误并回滚事务。
5.  调用 `database::soft_delete_area_in_tx` 软删除领域：
    - 设置 `is_deleted = true`
    - 更新 `updated_at` 为当前时间
6.  提交数据库事务（`tx.commit()`）。
7.  返回 `204 No Content`。

## 6. 边界情况 (Edge Cases)

- **领域不存在:** 返回 `404` 错误。
- **领域已删除:** 幂等，返回 `404` 错误（因为 `check_area_exists_in_tx` 不返回已删除的领域）。
- **领域有子领域:** 当前实现中仍然允许删除，子领域的 `parent_area_id` 仍然指向被删除的领域。
- **领域有关联任务:** 当前实现中允许删除，任务的 `area_id` 仍然保留该引用。
- **领域有时间块关联:** 不影响时间块，时间块不直接关联领域。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次，查询 `areas` 表以检查领域存在性（带 `is_deleted = false` 过滤）。
    - **`UPDATE`:** 1条记录在 `areas` 表（设置 `is_deleted = true`, `updated_at`）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **级联影响:**
    - **已关联任务:** 任务的 `area_id` 字段保持不变，前端需要处理已删除领域的显示逻辑。
    - **子领域:** 子领域的 `parent_area_id` 保持不变，可能成为"孤儿"领域（父领域已删除）。
- **日志记录:**
    - 失败时（如领域不存在），可能以 `WARN` 级别记录详细错误信息。

*（无 SSE 事件，无写入许可，无其他已知副作用）*

**注意事项：**
1. 当前实现不会自动清理或调整关联数据（任务、子领域）。
2. 如果需要防止删除有子领域的领域，需要在业务逻辑中添加检查。
3. 如果需要级联删除子领域或任务，需要在业务逻辑中添加相应处理。
*/

pub async fn handle(State(app_state): State<AppState>, Path(area_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, area_id).await {
        Ok(()) => no_content_response().into_response(),
        Err(err) => err.into_response(),
    }
}

mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, area_id: Uuid) -> AppResult<()> {
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        let exists = database::check_area_exists_in_tx(&mut tx, area_id).await?;
        if !exists {
            return Err(AppError::not_found("Area", area_id.to_string()));
        }

        database::soft_delete_area_in_tx(&mut tx, area_id).await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(())
    }
}

mod database {
    use super::*;

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
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(count > 0)
    }

    pub async fn soft_delete_area_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        area_id: Uuid,
    ) -> AppResult<()> {
        let query = "UPDATE areas SET is_deleted = true, updated_at = ? WHERE id = ?";
        sqlx::query(query)
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(area_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(())
    }
}

