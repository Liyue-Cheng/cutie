/// 删除时间块 API - 单文件组件
///
/// 软删除时间块，不影响任务的排期状态
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
CABC for `delete_time_block`

## API端点
DELETE /api/time-blocks/{id}

## 预期行为简介
软删除时间块（设置 is_deleted = true）。
重要：不影响任务的排期状态，任务的 task_schedules 记录保留。

## 输入输出规范
- **前置条件**: block_id 必须存在
- **后置条件**:
  - 时间块的 is_deleted = true
  - 删除 task_time_block_links 记录
  - 保留 task_schedules 记录（任务仍被"排期"）

## Cutie 业务逻辑
删除时间块 ≠ 取消任务排期
- 时间块只是"具体执行时间"
- 删除它不影响任务"是否被安排到某一天"
- 任务仍在 Planned 列，只是没有具体时间段

## 边界情况
- 如果时间块不存在，返回 404
- 如果时间块已删除，返回 204（幂等）
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>, Path(block_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, block_id).await {
        Ok(()) => no_content_response().into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, block_id: Uuid) -> AppResult<()> {
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 1. 检查时间块是否存在
        let block_exists = database::check_time_block_exists_in_tx(&mut tx, block_id).await?;
        if !block_exists {
            return Err(AppError::not_found("TimeBlock", block_id.to_string()));
        }

        // 2. 软删除时间块
        database::soft_delete_time_block_in_tx(&mut tx, block_id).await?;

        // 3. 删除任务链接（但保留 task_schedules！）
        database::delete_block_links_in_tx(&mut tx, block_id).await?;

        // 4. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(())
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;

    pub async fn check_time_block_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
    ) -> AppResult<bool> {
        let query = "SELECT COUNT(*) FROM time_blocks WHERE id = ?";
        let count: i64 = sqlx::query_scalar(query)
            .bind(block_id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(count > 0)
    }

    pub async fn soft_delete_time_block_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
    ) -> AppResult<()> {
        let query = "UPDATE time_blocks SET is_deleted = true, updated_at = ? WHERE id = ?";
        sqlx::query(query)
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(block_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(())
    }

    pub async fn delete_block_links_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
    ) -> AppResult<()> {
        let query = "DELETE FROM task_time_block_links WHERE time_block_id = ?";
        sqlx::query(query)
            .bind(block_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(())
    }
}

