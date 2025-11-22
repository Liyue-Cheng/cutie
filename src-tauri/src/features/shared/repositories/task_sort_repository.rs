use chrono::{DateTime, Utc};
use sqlx::{Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::infra::core::{AppError, AppResult, DbError};

pub struct TaskSortRepository;

impl TaskSortRepository {
    /// 获取指定任务在某个视图中的 rank
    pub async fn get_task_rank(
        pool: &SqlitePool,
        task_id: Uuid,
        view_context: &str,
    ) -> AppResult<Option<String>> {
        let query = r#"
            SELECT json_extract(sort_positions, ?) as rank
            FROM tasks
            WHERE id = ? AND deleted_at IS NULL
        "#;

        let json_path = format!("$.{}", view_context);
        let row: Option<(Option<String>,)> = sqlx::query_as(query)
            .bind(&json_path)
            .bind(task_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        match row {
            Some((rank,)) => Ok(rank),
            None => Err(AppError::not_found("Task", task_id.to_string())),
        }
    }

    /// 获取指定视图中最前面的 rank（按字典序）
    pub async fn get_first_rank_in_view(
        pool: &SqlitePool,
        view_context: &str,
    ) -> AppResult<Option<String>> {
        let query = r#"
            SELECT json_extract(sort_positions, ?) as rank
            FROM tasks
            WHERE deleted_at IS NULL
              AND json_extract(sort_positions, ?) IS NOT NULL
            ORDER BY json_extract(sort_positions, ?) ASC
            LIMIT 1
        "#;

        let json_path = format!("$.{}", view_context);
        let row: Option<(Option<String>,)> = sqlx::query_as(query)
            .bind(&json_path)
            .bind(&json_path)
            .bind(&json_path)
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(row.and_then(|(rank,)| rank))
    }

    /// 更新任务在指定视图中的 rank（事务内）
    pub async fn update_task_rank_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        view_context: &str,
        new_rank: &str,
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        let query = r#"
            UPDATE tasks
            SET sort_positions = json_set(COALESCE(sort_positions, '{}'), ?, ?),
                updated_at = ?
            WHERE id = ? AND deleted_at IS NULL
        "#;

        let json_path = format!("$.{}", view_context);
        let result = sqlx::query(query)
            .bind(&json_path)
            .bind(new_rank)
            .bind(now.to_rfc3339())
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("Task", task_id.to_string()));
        }

        Ok(())
    }

    /// 更新任务在指定视图中的 rank（无事务快捷方法）
    pub async fn update_task_rank(
        pool: &SqlitePool,
        task_id: Uuid,
        view_context: &str,
        new_rank: &str,
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Self::update_task_rank_in_tx(&mut tx, task_id, view_context, new_rank, now).await?;
        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }
}
