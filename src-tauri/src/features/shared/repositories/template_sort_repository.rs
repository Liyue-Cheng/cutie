use chrono::{DateTime, Utc};
use sqlx::{Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::infra::core::{AppError, AppResult, DbError};

pub struct TemplateSortRepository;

impl TemplateSortRepository {
    pub async fn get_sort_rank(pool: &SqlitePool, template_id: Uuid) -> AppResult<Option<String>> {
        let row: Option<(Option<String>,)> = sqlx::query_as(
            r#"
                SELECT sort_rank
                FROM templates
                WHERE id = ? AND is_deleted = FALSE
            "#,
        )
        .bind(template_id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        match row {
            Some((rank,)) => Ok(rank),
            None => Err(AppError::not_found("Template", template_id.to_string())),
        }
    }

    pub async fn get_neighbor_rank(
        pool: &SqlitePool,
        template_id: Option<Uuid>,
    ) -> AppResult<Option<String>> {
        if let Some(id) = template_id {
            Self::get_sort_rank(pool, id).await
        } else {
            Ok(None)
        }
    }

    pub async fn get_first_sort_rank(pool: &SqlitePool) -> AppResult<Option<String>> {
        let row: Option<(Option<String>,)> = sqlx::query_as(
            r#"
                SELECT sort_rank
                FROM templates
                WHERE sort_rank IS NOT NULL AND is_deleted = FALSE
                ORDER BY sort_rank ASC
                LIMIT 1
            "#,
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(row.and_then(|(rank,)| rank))
    }

    pub async fn get_highest_sort_rank_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
    ) -> AppResult<Option<String>> {
        let row: Option<(Option<String>,)> = sqlx::query_as(
            r#"
                SELECT sort_rank
                FROM templates
                WHERE sort_rank IS NOT NULL AND is_deleted = FALSE
                ORDER BY sort_rank DESC
                LIMIT 1
            "#,
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(row.and_then(|(rank,)| rank))
    }

    pub async fn update_sort_rank_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        template_id: Uuid,
        new_rank: &str,
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        let result = sqlx::query(
            r#"
                UPDATE templates
                SET sort_rank = ?, updated_at = ?
                WHERE id = ? AND is_deleted = FALSE
            "#,
        )
        .bind(new_rank)
        .bind(now)
        .bind(template_id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("Template", template_id.to_string()));
        }

        Ok(())
    }

    pub async fn batch_update_sort_ranks_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        ranks: &[(Uuid, String)],
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        for (template_id, rank) in ranks {
            Self::update_sort_rank_in_tx(tx, *template_id, rank, now).await?;
        }

        Ok(())
    }

    pub fn generate_sequential_ranks(
        start_rank: Option<String>,
        count: usize,
    ) -> AppResult<Vec<String>> {
        let mut ranks = Vec::with_capacity(count);
        let mut prev = start_rank;
        for _ in 0..count {
            let next = crate::infra::LexoRankService::generate_between(prev.as_deref(), None)?;
            prev = Some(next.clone());
            ranks.push(next);
        }
        Ok(ranks)
    }
}
