/// Task ↔ TimeBlock 链接表操作仓库
use chrono::Utc;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{TimeBlock, TimeBlockRow},
    shared::core::{AppError, AppResult, DbError},
};

pub struct TaskTimeBlockLinkRepository;

impl TaskTimeBlockLinkRepository {
    /// 创建任务到时间块的链接
    pub async fn link_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        block_id: Uuid,
    ) -> AppResult<()> {
        let now = Utc::now();

        let query = r#"
            INSERT INTO task_time_block_links (task_id, time_block_id, created_at)
            VALUES (?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(task_id.to_string())
            .bind(block_id.to_string())
            .bind(now.to_rfc3339())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 删除任务的所有链接
    pub async fn delete_all_for_task_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<()> {
        let query = "DELETE FROM task_time_block_links WHERE task_id = ?";
        sqlx::query(query)
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;
        Ok(())
    }

    /// 删除时间块的所有链接
    pub async fn delete_all_for_block_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
    ) -> AppResult<()> {
        let query = "DELETE FROM task_time_block_links WHERE time_block_id = ?";
        sqlx::query(query)
            .bind(block_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;
        Ok(())
    }

    /// 查询任务链接的所有时间块
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
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        let blocks: Result<Vec<TimeBlock>, _> = rows.into_iter().map(TimeBlock::try_from).collect();

        blocks.map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))
    }

    /// 检查时间块是否独占链接某任务（只链接了这一个任务）
    pub async fn is_exclusive_link_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
        _task_id: Uuid,
    ) -> AppResult<bool> {
        let query = r#"
            SELECT COUNT(*) as count
            FROM task_time_block_links
            WHERE time_block_id = ?
        "#;

        let total_count: i64 = sqlx::query_scalar(query)
            .bind(block_id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(total_count == 1)
    }

    /// 统计时间块剩余链接任务数
    pub async fn count_remaining_tasks_in_block_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
    ) -> AppResult<i64> {
        let query = "SELECT COUNT(*) FROM task_time_block_links WHERE time_block_id = ?";
        sqlx::query_scalar(query)
            .bind(block_id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))
    }
}
