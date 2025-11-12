/// TaskRecurrenceLink 核心 CRUD 仓库
use chrono::DateTime;
use sqlx::{Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::{
    entities::{TaskRecurrenceLink, TaskRecurrenceLinkRow},
    infra::core::{AppError, AppResult, DbError},
};

pub struct TaskRecurrenceLinkRepository;

impl TaskRecurrenceLinkRepository {
    /// 查询某个循环规则在某天的链接（在事务中）
    pub async fn find_link_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        recurrence_id: Uuid,
        instance_date: &str,
    ) -> AppResult<Option<TaskRecurrenceLink>> {
        let query = r#"
            SELECT recurrence_id, instance_date, task_id, created_at
            FROM task_recurrence_links
            WHERE recurrence_id = ? AND instance_date = ?
        "#;

        let row = sqlx::query_as::<_, TaskRecurrenceLinkRow>(query)
            .bind(recurrence_id.to_string())
            .bind(instance_date)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        match row {
            Some(r) => {
                let link = TaskRecurrenceLink::try_from(r)
                    .map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))?;
                Ok(Some(link))
            }
            None => Ok(None),
        }
    }

    /// 查询某个循环规则在某天的链接（非事务）
    pub async fn find_link(
        pool: &SqlitePool,
        recurrence_id: Uuid,
        instance_date: &str,
    ) -> AppResult<Option<TaskRecurrenceLink>> {
        let query = r#"
            SELECT recurrence_id, instance_date, task_id, created_at
            FROM task_recurrence_links
            WHERE recurrence_id = ? AND instance_date = ?
        "#;

        let row = sqlx::query_as::<_, TaskRecurrenceLinkRow>(query)
            .bind(recurrence_id.to_string())
            .bind(instance_date)
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        match row {
            Some(r) => {
                let link = TaskRecurrenceLink::try_from(r)
                    .map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))?;
                Ok(Some(link))
            }
            None => Ok(None),
        }
    }

    /// 插入循环实例链接（在事务中）
    pub async fn insert_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        link: &TaskRecurrenceLink,
    ) -> AppResult<()> {
        let query = r#"
            INSERT INTO task_recurrence_links (
                recurrence_id, instance_date, task_id, created_at
            ) VALUES (?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(link.recurrence_id.to_string())
            .bind(&link.instance_date)
            .bind(link.task_id.to_string())
            .bind(link.created_at)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 删除某个循环规则在某天的链接（在事务中）
    pub async fn delete_link_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        recurrence_id: Uuid,
        instance_date: &str,
    ) -> AppResult<()> {
        let query = r#"
            DELETE FROM task_recurrence_links
            WHERE recurrence_id = ? AND instance_date = ?
        "#;

        sqlx::query(query)
            .bind(recurrence_id.to_string())
            .bind(instance_date)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 删除某个循环规则的所有链接（在事务中）
    pub async fn delete_all_for_recurrence_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        recurrence_id: Uuid,
    ) -> AppResult<()> {
        let query = r#"
            DELETE FROM task_recurrence_links
            WHERE recurrence_id = ?
        "#;

        sqlx::query(query)
            .bind(recurrence_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 查询某个循环规则的所有链接
    pub async fn find_all_for_recurrence(
        pool: &SqlitePool,
        recurrence_id: Uuid,
    ) -> AppResult<Vec<TaskRecurrenceLink>> {
        let query = r#"
            SELECT recurrence_id, instance_date, task_id, created_at
            FROM task_recurrence_links
            WHERE recurrence_id = ?
            ORDER BY instance_date DESC
        "#;

        let rows = sqlx::query_as::<_, TaskRecurrenceLinkRow>(query)
            .bind(recurrence_id.to_string())
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        let links: Result<Vec<TaskRecurrenceLink>, _> =
            rows.into_iter().map(TaskRecurrenceLink::try_from).collect();

        links.map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))
    }

    /// 查询某个任务的循环链接信息
    pub async fn find_by_task_id(
        pool: &SqlitePool,
        task_id: Uuid,
    ) -> AppResult<Option<TaskRecurrenceLink>> {
        let query = r#"
            SELECT recurrence_id, instance_date, task_id, created_at
            FROM task_recurrence_links
            WHERE task_id = ?
        "#;

        let row = sqlx::query_as::<_, TaskRecurrenceLinkRow>(query)
            .bind(task_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        match row {
            Some(r) => {
                let link = TaskRecurrenceLink::try_from(r)
                    .map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))?;
                Ok(Some(link))
            }
            None => Ok(None),
        }
    }
}
