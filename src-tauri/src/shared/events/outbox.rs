/// 事件发件箱仓储实现（Transactional Outbox Pattern）
use sqlx::{Sqlite, SqlitePool, Transaction};

use super::models::{DomainEvent, EventOutboxRow};
use crate::shared::core::{AppError, AppResult};

/// 事件发件箱仓储接口
#[async_trait::async_trait]
pub trait EventOutboxRepository: Send + Sync {
    /// 在事务中写入事件到 outbox
    async fn append_in_tx(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        event: &DomainEvent,
    ) -> AppResult<i64>;

    /// 批量查询未分发的事件（用于 dispatcher）
    async fn fetch_undispatched(&self, limit: i64) -> AppResult<Vec<(i64, DomainEvent)>>;

    /// 标记事件为已分发
    async fn mark_dispatched(&self, outbox_id: i64) -> AppResult<()>;
}

/// Sqlx 实现
pub struct SqlxEventOutboxRepository {
    pool: SqlitePool,
}

impl SqlxEventOutboxRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl EventOutboxRepository for SqlxEventOutboxRepository {
    async fn append_in_tx(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        event: &DomainEvent,
    ) -> AppResult<i64> {
        let payload_json = serde_json::to_string(&event.payload).map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::QueryError(format!(
                "Failed to serialize payload: {}",
                e
            )))
        })?;

        let result = sqlx::query(
            r#"
            INSERT INTO event_outbox (
                event_id, event_type, version,
                aggregate_type, aggregate_id, aggregate_version,
                correlation_id, occurred_at, payload, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(event.event_id.to_string())
        .bind(&event.event_type)
        .bind(event.version)
        .bind(&event.aggregate_type)
        .bind(&event.aggregate_id)
        .bind(event.aggregate_version)
        .bind(&event.correlation_id)
        .bind(event.occurred_at.to_rfc3339())
        .bind(&payload_json)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(result.last_insert_rowid())
    }

    async fn fetch_undispatched(&self, limit: i64) -> AppResult<Vec<(i64, DomainEvent)>> {
        let rows = sqlx::query_as::<_, EventOutboxRow>(
            r#"
            SELECT * FROM event_outbox
            WHERE dispatched_at IS NULL
            ORDER BY id ASC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?;

        let mut events = Vec::new();
        for row in rows {
            if let Ok(event) = row.to_domain_event() {
                events.push((row.id, event));
            }
        }

        Ok(events)
    }

    async fn mark_dispatched(&self, outbox_id: i64) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE event_outbox
            SET dispatched_at = ?
            WHERE id = ?
            "#,
        )
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(outbox_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(())
    }
}
