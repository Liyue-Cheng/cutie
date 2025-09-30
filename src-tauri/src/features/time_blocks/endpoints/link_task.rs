/// 链接任务到时间块 API - 单文件组件
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    shared::core::{AppError, AppResult, ValidationError},
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `link_task_to_time_block`

## API端点
POST /api/time-blocks/{id}/links

## 预期行为简介
将一个任务链接到一个时间块。幂等操作。

## 输入输出规范
- **前置条件**: `id` 必须是有效的时间块ID。请求体必须包含有效的 `task_id`。
- **后置条件**: 返回 `204 No Content`。
- **不变量**: 无。

## 边界情况
- 时间块不存在: 返回 `404 Not Found`。
- 任务不存在: 返回 `404 Not Found`。
- 链接已存在: 幂等地返回 `204`。

## 预期副作用
- 在 `task_time_block_links` 表插入1条记录（如果不存在）。
- 所有数据库写入在单个事务中。

## 请求体
```json
{
  "task_id": "uuid-string"
}
```
*/

#[derive(Deserialize)]
pub struct LinkTaskRequest {
    task_id: String,
}

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(block_id): Path<Uuid>,
    Json(request): Json<LinkTaskRequest>,
) -> Response {
    match logic::execute(&app_state, block_id, request).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 (Validation Layer) ====================
mod validation {
    use super::*;

    pub fn validate_request(request: &LinkTaskRequest) -> Result<Uuid, Vec<ValidationError>> {
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
        request: LinkTaskRequest,
    ) -> AppResult<()> {
        // 1. 验证请求
        let task_id = validation::validate_request(&request).map_err(AppError::ValidationFailed)?;

        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 2. 验证时间块存在
        let block_exists = database::time_block_exists_in_tx(&mut tx, block_id).await?;
        if !block_exists {
            return Err(AppError::not_found("TimeBlock", block_id.to_string()));
        }

        // 3. 验证任务存在
        let task_exists = database::task_exists_in_tx(&mut tx, task_id).await?;
        if !task_exists {
            return Err(AppError::not_found("Task", task_id.to_string()));
        }

        // 4. 幂等检查：链接是否已存在
        let link_exists = database::link_exists_in_tx(&mut tx, task_id, block_id).await?;
        if link_exists {
            // 幂等：链接已存在，直接返回成功
            return Ok(());
        }

        // 5. 核心操作：创建链接
        let now = app_state.clock().now_utc();
        database::link_task_to_block_in_tx(&mut tx, task_id, block_id, now).await?;

        // 6. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(())
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
mod database {
    use super::*;
    use chrono::DateTime;

    pub async fn time_block_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
    ) -> AppResult<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM time_blocks WHERE id = ? AND is_deleted = false",
        )
        .bind(block_id.to_string())
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        Ok(count > 0)
    }

    pub async fn task_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM tasks WHERE id = ? AND is_deleted = false",
        )
        .bind(task_id.to_string())
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        Ok(count > 0)
    }

    pub async fn link_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        block_id: Uuid,
    ) -> AppResult<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM task_time_block_links WHERE task_id = ? AND time_block_id = ?",
        )
        .bind(task_id.to_string())
        .bind(block_id.to_string())
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        Ok(count > 0)
    }

    pub async fn link_task_to_block_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        time_block_id: Uuid,
        created_at: DateTime<Utc>,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO task_time_block_links (task_id, time_block_id, created_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(task_id.to_string())
        .bind(time_block_id.to_string())
        .bind(created_at.to_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        Ok(())
    }
}


