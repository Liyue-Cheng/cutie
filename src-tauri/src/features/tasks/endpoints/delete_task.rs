/// 删除任务 API - 单文件组件
///
/// 软删除任务，并根据业务规则清理孤儿时间块
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use serde::Serialize;

use crate::{
    entities::TimeBlock,
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

/// 删除任务的响应
#[derive(Debug, Serialize)]
pub struct DeleteTaskResponse {
    pub success: bool,
    // 注意：deleted_time_block_ids 已通过 SSE 推送
}

// ==================== 文档层 ====================
/*
CABC for `delete_task`

## API端点
DELETE /api/tasks/{id}

## 预期行为简介
软删除任务（设置 is_deleted = true）。
根据 Cutie 的业务规则，如果任务链接的时间块变成"孤儿"，也会删除该时间块。

## 输入输出规范
- **前置条件**: task_id 必须存在
- **后置条件**:
  - 任务的 is_deleted = true
  - 删除所有 task_time_block_links 记录
  - 删除所有 task_schedules 记录
  - 如果时间块变成孤儿且是自动创建的，删除该时间块

## 边界情况
- 如果任务不存在，返回 404
- 如果任务已删除，返回 204（幂等）

## 孤儿时间块定义
- 该时间块只链接了这一个任务
- 删除这个任务后，时间块没有任何关联任务
- 时间块的 title 与任务 title 相同（自动创建的标志）

## 预期副作用
- 更新 tasks 表（is_deleted = true）
- 删除 task_time_block_links 记录
- 删除 task_schedules 记录
- 可能删除孤儿时间块
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>, Path(task_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, task_id).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, task_id: Uuid) -> AppResult<DeleteTaskResponse> {
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 1. 检查任务是否存在
        let task_exists = database::check_task_exists_in_tx(&mut tx, task_id).await?;
        if !task_exists {
            return Err(AppError::not_found("Task", task_id.to_string()));
        }

        // 2. 获取任务信息（用于判断孤儿时间块）
        let task_title = database::get_task_title_in_tx(&mut tx, task_id).await?;

        // 3. 找到该任务链接的所有时间块
        let linked_blocks = database::find_linked_time_blocks_in_tx(&mut tx, task_id).await?;

        // 4. 删除任务（软删除）
        database::soft_delete_task_in_tx(&mut tx, task_id).await?;

        // 5. 删除任务的所有链接和日程
        database::delete_task_links_in_tx(&mut tx, task_id).await?;
        database::delete_task_schedules_in_tx(&mut tx, task_id).await?;

        // 6. 检查并删除孤儿时间块
        let mut deleted_time_block_ids = Vec::new();
        for block in linked_blocks {
            let should_delete = should_delete_orphan_block(&block, &task_title, &mut tx).await?;
            if should_delete {
                database::soft_delete_time_block_in_tx(&mut tx, block.id).await?;
                deleted_time_block_ids.push(block.id);
                tracing::info!(
                    "Deleted orphan time block {} after deleting task {}",
                    block.id,
                    task_id
                );
            }
        }

        // 7. 在事务中写入领域事件到 outbox
        use crate::shared::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());
        let now = app_state.clock().now_utc();

        // 7.1 发布任务删除事件
        {
            let payload = serde_json::json!({
                "task_id": task_id.to_string(),
                "deleted_at": now.to_rfc3339(),
            });
            let event = DomainEvent::new("task.deleted", "task", task_id.to_string(), payload)
                .with_aggregate_version(now.timestamp_millis());
            outbox_repo.append_in_tx(&mut tx, &event).await?;
        }

        // 7.2 发布时间块删除事件（如果有孤儿时间块被删除）
        if !deleted_time_block_ids.is_empty() {
            let payload = serde_json::json!({
                "time_block_ids": deleted_time_block_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
                "deleted_at": now.to_rfc3339(),
                "reason": "orphan_after_task_deletion",
            });
            let event = DomainEvent::new("time_blocks.deleted", "time_block", "batch", payload)
                .with_aggregate_version(now.timestamp_millis());
            outbox_repo.append_in_tx(&mut tx, &event).await?;
        }

        // 8. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // HTTP 响应不再包含副作用列表，副作用通过 SSE 推送
        Ok(DeleteTaskResponse { success: true })
    }

    /// 判断是否应该删除孤儿时间块
    async fn should_delete_orphan_block(
        block: &TimeBlock,
        deleted_task_title: &str,
        tx: &mut Transaction<'_, Sqlite>,
    ) -> AppResult<bool> {
        // 1. 检查时间块是否还有其他任务
        let remaining_tasks = database::count_remaining_tasks_in_block_in_tx(tx, block.id).await?;
        if remaining_tasks > 0 {
            return Ok(false); // 还有其他任务，不删除
        }

        // 2. 判断是否自动创建的（title 与任务相同）
        if let Some(block_title) = &block.title {
            if block_title == deleted_task_title {
                return Ok(true); // 孤儿 + 自动创建 = 删除
            }
        }

        // 3. 用户手动创建的空时间块，保留
        Ok(false)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::entities::TimeBlockRow;

    pub async fn check_task_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<bool> {
        let query = "SELECT COUNT(*) FROM tasks WHERE id = ?";
        let count: i64 = sqlx::query_scalar(query)
            .bind(task_id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(count > 0)
    }

    pub async fn get_task_title_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<String> {
        let query = "SELECT title FROM tasks WHERE id = ?";
        sqlx::query_scalar(query)
            .bind(task_id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))
    }

    pub async fn find_linked_time_blocks_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<Vec<TimeBlock>> {
        let query = r#"
            SELECT DISTINCT
                tb.id, tb.title, tb.glance_note, tb.detail_note, tb.start_time, tb.end_time, 
                tb.area_id, tb.created_at, tb.updated_at, tb.is_deleted, tb.source_info,
                tb.external_source_id, tb.external_source_provider, tb.external_source_metadata,
                tb.recurrence_rule, tb.recurrence_parent_id, tb.recurrence_original_date, 
                tb.recurrence_exclusions
            FROM time_blocks tb
            INNER JOIN task_time_block_links ttbl ON tb.id = ttbl.time_block_id
            WHERE ttbl.task_id = ? AND tb.is_deleted = false
        "#;

        let rows = sqlx::query_as::<_, TimeBlockRow>(query)
            .bind(task_id.to_string())
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        let blocks: Result<Vec<TimeBlock>, _> = rows.into_iter().map(TimeBlock::try_from).collect();

        blocks.map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }

    pub async fn soft_delete_task_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<()> {
        let query = "UPDATE tasks SET is_deleted = true, updated_at = ? WHERE id = ?";
        sqlx::query(query)
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(())
    }

    pub async fn delete_task_links_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<()> {
        let query = "DELETE FROM task_time_block_links WHERE task_id = ?";
        sqlx::query(query)
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(())
    }

    pub async fn delete_task_schedules_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<()> {
        let query = "DELETE FROM task_schedules WHERE task_id = ?";
        sqlx::query(query)
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(())
    }

    pub async fn count_remaining_tasks_in_block_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
    ) -> AppResult<i64> {
        let query = "SELECT COUNT(*) FROM task_time_block_links WHERE time_block_id = ?";
        sqlx::query_scalar(query)
            .bind(block_id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))
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
}
