/// TimeBlockRecurrenceLink 核心 CRUD 仓库
use sqlx::{Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::{
    entities::{TimeBlockRecurrenceLink, TimeBlockRecurrenceLinkRow},
    infra::core::{AppError, AppResult, DbError},
};

pub struct TimeBlockRecurrenceLinkRepository;

impl TimeBlockRecurrenceLinkRepository {
    /// 查询链接
    pub async fn find_link(
        pool: &SqlitePool,
        recurrence_id: Uuid,
        instance_date: &str,
    ) -> AppResult<Option<TimeBlockRecurrenceLink>> {
        let query = r#"
            SELECT recurrence_id, instance_date, time_block_id, created_at
            FROM time_block_recurrence_links
            WHERE recurrence_id = ? AND instance_date = ?
        "#;

        let row = sqlx::query_as::<_, TimeBlockRecurrenceLinkRow>(query)
            .bind(recurrence_id.to_string())
            .bind(instance_date)
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        match row {
            Some(r) => {
                let link = TimeBlockRecurrenceLink::try_from(r)
                    .map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))?;
                Ok(Some(link))
            }
            None => Ok(None),
        }
    }

    /// 在事务中查询链接
    pub async fn find_link_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        recurrence_id: Uuid,
        instance_date: &str,
    ) -> AppResult<Option<TimeBlockRecurrenceLink>> {
        let query = r#"
            SELECT recurrence_id, instance_date, time_block_id, created_at
            FROM time_block_recurrence_links
            WHERE recurrence_id = ? AND instance_date = ?
        "#;

        let row = sqlx::query_as::<_, TimeBlockRecurrenceLinkRow>(query)
            .bind(recurrence_id.to_string())
            .bind(instance_date)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        match row {
            Some(r) => {
                let link = TimeBlockRecurrenceLink::try_from(r)
                    .map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))?;
                Ok(Some(link))
            }
            None => Ok(None),
        }
    }

    /// 根据时间块ID查询链接
    pub async fn find_by_time_block_id(
        pool: &SqlitePool,
        time_block_id: Uuid,
    ) -> AppResult<Option<TimeBlockRecurrenceLink>> {
        let query = r#"
            SELECT recurrence_id, instance_date, time_block_id, created_at
            FROM time_block_recurrence_links
            WHERE time_block_id = ?
        "#;

        let row = sqlx::query_as::<_, TimeBlockRecurrenceLinkRow>(query)
            .bind(time_block_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        match row {
            Some(r) => {
                let link = TimeBlockRecurrenceLink::try_from(r)
                    .map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))?;
                Ok(Some(link))
            }
            None => Ok(None),
        }
    }

    /// 插入链接
    pub async fn insert_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        link: &TimeBlockRecurrenceLink,
    ) -> AppResult<()> {
        let query = r#"
            INSERT INTO time_block_recurrence_links (
                recurrence_id, instance_date, time_block_id, created_at
            ) VALUES (?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(link.recurrence_id.to_string())
            .bind(&link.instance_date)
            .bind(link.time_block_id.to_string())
            .bind(link.created_at)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 删除某个循环规则的所有链接
    pub async fn delete_by_recurrence_id_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        recurrence_id: Uuid,
    ) -> AppResult<()> {
        let query = r#"
            DELETE FROM time_block_recurrence_links WHERE recurrence_id = ?
        "#;

        sqlx::query(query)
            .bind(recurrence_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 查询某个循环规则的所有链接
    pub async fn find_by_recurrence_id(
        pool: &SqlitePool,
        recurrence_id: Uuid,
    ) -> AppResult<Vec<TimeBlockRecurrenceLink>> {
        let query = r#"
            SELECT recurrence_id, instance_date, time_block_id, created_at
            FROM time_block_recurrence_links
            WHERE recurrence_id = ?
            ORDER BY instance_date DESC
        "#;

        let rows = sqlx::query_as::<_, TimeBlockRecurrenceLinkRow>(query)
            .bind(recurrence_id.to_string())
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        let links: Result<Vec<TimeBlockRecurrenceLink>, _> = rows
            .into_iter()
            .map(TimeBlockRecurrenceLink::try_from)
            .collect();

        links.map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))
    }

    /// 查询某天的所有链接
    pub async fn find_by_date(
        pool: &SqlitePool,
        instance_date: &str,
    ) -> AppResult<Vec<TimeBlockRecurrenceLink>> {
        let query = r#"
            SELECT recurrence_id, instance_date, time_block_id, created_at
            FROM time_block_recurrence_links
            WHERE instance_date = ?
        "#;

        let rows = sqlx::query_as::<_, TimeBlockRecurrenceLinkRow>(query)
            .bind(instance_date)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        let links: Result<Vec<TimeBlockRecurrenceLink>, _> = rows
            .into_iter()
            .map(TimeBlockRecurrenceLink::try_from)
            .collect();

        links.map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))
    }
}
