/// 从时间块解绑任务 API - 单文件组件
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    crate::infra::core::{AppError, AppResult, ValidationError},
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `unlink_task_from_time_block`

## API端点
DELETE /api/time-blocks/{id}/links

## 预期行为简介
从时间块中解绑一个任务。幂等操作。

## 输入输出规范
- **前置条件**: `id` 必须是有效的时间块ID。请求体必须包含有效的 `task_id`。
- **后置条件**: 返回 `204 No Content`。
- **不变量**: 无。

## 边界情况
- 链接不存在: 幂等地返回 `204`。

## 预期副作用
- 从 `task_time_block_links` 表删除1条记录（如果存在）。
- 所有数据库写入在单个事务中。

## 请求体
```json
{
  "task_id": "uuid-string"
}
```
*/

#[derive(Deserialize)]
pub struct UnlinkTaskRequest {
    task_id: String,
}

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(block_id): Path<Uuid>,
    Json(request): Json<UnlinkTaskRequest>,
) -> Response {
    match logic::execute(&app_state, block_id, request).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 (Validation Layer) ====================
mod validation {
    use super::*;

    pub fn validate_request(request: &UnlinkTaskRequest) -> Result<Uuid, Vec<ValidationError>> {
        match Uuid::parse_str(&request.task_id) {
            Ok(id) => Ok(id),
            Err(_) => Err(vec![ValidationError::new(
                "task_id",
                "Task ID 格式无效",
                "INVALID_TASK_ID",
            )]),
        }
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        block_id: Uuid,
        request: UnlinkTaskRequest,
    ) -> AppResult<()> {
        // 1. 验证请求
        let task_id =
            validation::validate_request(&request).map_err(AppError::ValidationFailed)?;

        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
        })?;

        // 2. 核心操作：删除链接（幂等）
        database::unlink_task_from_block_in_tx(&mut tx, task_id, block_id).await?;

        // 3. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(())
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
mod database {
    use super::*;

    pub async fn unlink_task_from_block_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        time_block_id: Uuid,
    ) -> AppResult<()> {
        sqlx::query(
            "DELETE FROM task_time_block_links WHERE task_id = ? AND time_block_id = ?",
        )
        .bind(task_id.to_string())
        .bind(time_block_id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e)))?;

        Ok(())
    }
}


