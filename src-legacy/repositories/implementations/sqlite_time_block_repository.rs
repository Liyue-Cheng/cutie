/// TimeBlockRepository的SQLite实现
///
/// 提供TimeBlock实体的具体数据库操作实现
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::entities::{SourceInfo, TimeBlock};
use crate::repositories::traits::TimeBlockRepository;
use crate::infra::core::{AppResult, DbError};

/// 时间块仓库的SQLite实现
#[derive(Clone)]
pub struct SqliteTimeBlockRepository {
    pool: SqlitePool,
}

impl SqliteTimeBlockRepository {
    /// 创建新的TimeBlockRepository实例
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 将数据库行转换为TimeBlock对象
    fn row_to_time_block(row: &sqlx::sqlite::SqliteRow) -> Result<TimeBlock, sqlx::Error> {
        let source_info_json: Option<String> = row.try_get("source_info")?;
        let source_info =
            source_info_json.and_then(|json| serde_json::from_str::<SourceInfo>(&json).ok());

        let external_source_metadata_json: Option<String> =
            row.try_get("external_source_metadata")?;
        let external_source_metadata = external_source_metadata_json
            .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok());

        let recurrence_exclusions_json: Option<String> = row.try_get("recurrence_exclusions")?;
        let recurrence_exclusions = recurrence_exclusions_json
            .and_then(|json| serde_json::from_str::<Vec<DateTime<Utc>>>(&json).ok());

        Ok(TimeBlock {
            id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|_| {
                sqlx::Error::ColumnDecode {
                    index: "id".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid UUID",
                    )),
                }
            })?,
            title: row.try_get("title")?,
            glance_note: row.try_get("glance_note")?,
            detail_note: row.try_get("detail_note")?,
            start_time: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("start_time")?)
                .map_err(|_| sqlx::Error::ColumnDecode {
                    index: "start_time".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid datetime",
                    )),
                })?
                .with_timezone(&Utc),
            end_time: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("end_time")?)
                .map_err(|_| sqlx::Error::ColumnDecode {
                    index: "end_time".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid datetime",
                    )),
                })?
                .with_timezone(&Utc),
            area_id: row
                .try_get::<Option<String>, _>("area_id")?
                .and_then(|s| Uuid::parse_str(&s).ok()),
            created_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("created_at")?)
                .map_err(|_| sqlx::Error::ColumnDecode {
                    index: "created_at".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid datetime",
                    )),
                })?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("updated_at")?)
                .map_err(|_| sqlx::Error::ColumnDecode {
                    index: "updated_at".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid datetime",
                    )),
                })?
                .with_timezone(&Utc),
            is_deleted: row.try_get("is_deleted")?,
            source_info,
            external_source_id: row.try_get("external_source_id")?,
            external_source_provider: row.try_get("external_source_provider")?,
            external_source_metadata,
            recurrence_rule: row.try_get("recurrence_rule")?,
            recurrence_parent_id: row
                .try_get::<Option<String>, _>("recurrence_parent_id")?
                .and_then(|s| Uuid::parse_str(&s).ok()),
            recurrence_original_date: row
                .try_get::<Option<String>, _>("recurrence_original_date")?
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            recurrence_exclusions,
        })
    }
}

#[async_trait]
impl TimeBlockRepository for SqliteTimeBlockRepository {
    // --- 写操作 ---
    async fn create(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        time_block: &TimeBlock,
    ) -> AppResult<TimeBlock> {
        sqlx::query(
            r#"
            INSERT INTO time_blocks (
                id, title, glance_note, detail_note, start_time, end_time, area_id,
                created_at, updated_at, is_deleted, source_info, external_source_id,
                external_source_provider, external_source_metadata, recurrence_rule,
                recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(time_block.id.to_string())
        .bind(&time_block.title)
        .bind(&time_block.glance_note)
        .bind(&time_block.detail_note)
        .bind(time_block.start_time.to_rfc3339())
        .bind(time_block.end_time.to_rfc3339())
        .bind(time_block.area_id.map(|id| id.to_string()))
        .bind(time_block.created_at.to_rfc3339())
        .bind(time_block.updated_at.to_rfc3339())
        .bind(time_block.is_deleted)
        .bind(
            time_block
                .source_info
                .as_ref()
                .map(|s| serde_json::to_string(s).unwrap_or_default()),
        )
        .bind(&time_block.external_source_id)
        .bind(&time_block.external_source_provider)
        .bind(
            time_block
                .external_source_metadata
                .as_ref()
                .map(|m| serde_json::to_string(m).unwrap_or_default()),
        )
        .bind(&time_block.recurrence_rule)
        .bind(time_block.recurrence_parent_id.map(|id| id.to_string()))
        .bind(
            time_block
                .recurrence_original_date
                .map(|dt| dt.to_rfc3339()),
        )
        .bind(
            time_block
                .recurrence_exclusions
                .as_ref()
                .map(|e| serde_json::to_string(e).unwrap_or_default()),
        )
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        Ok(time_block.clone())
    }

    async fn update(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        time_block: &TimeBlock,
    ) -> AppResult<TimeBlock> {
        let result = sqlx::query(
            r#"
            UPDATE time_blocks SET 
                title = ?, glance_note = ?, detail_note = ?, start_time = ?, end_time = ?,
                area_id = ?, updated_at = ?, source_info = ?, external_source_id = ?,
                external_source_provider = ?, external_source_metadata = ?, recurrence_rule = ?,
                recurrence_parent_id = ?, recurrence_original_date = ?, recurrence_exclusions = ?
            WHERE id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(&time_block.title)
        .bind(&time_block.glance_note)
        .bind(&time_block.detail_note)
        .bind(time_block.start_time.to_rfc3339())
        .bind(time_block.end_time.to_rfc3339())
        .bind(time_block.area_id.map(|id| id.to_string()))
        .bind(time_block.updated_at.to_rfc3339())
        .bind(
            time_block
                .source_info
                .as_ref()
                .map(|s| serde_json::to_string(s).unwrap_or_default()),
        )
        .bind(&time_block.external_source_id)
        .bind(&time_block.external_source_provider)
        .bind(
            time_block
                .external_source_metadata
                .as_ref()
                .map(|m| serde_json::to_string(m).unwrap_or_default()),
        )
        .bind(&time_block.recurrence_rule)
        .bind(time_block.recurrence_parent_id.map(|id| id.to_string()))
        .bind(
            time_block
                .recurrence_original_date
                .map(|dt| dt.to_rfc3339()),
        )
        .bind(
            time_block
                .recurrence_exclusions
                .as_ref()
                .map(|e| serde_json::to_string(e).unwrap_or_default()),
        )
        .bind(time_block.id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::infra::core::AppError::not_found(
                "TimeBlock",
                time_block.id.to_string(),
            ));
        }

        Ok(time_block.clone())
    }

    async fn delete(&self, tx: &mut Transaction<'_, Sqlite>, id: Uuid) -> AppResult<()> {
        let now = Utc::now();
        let result = sqlx::query(
            r#"
            UPDATE time_blocks SET 
                is_deleted = TRUE, 
                updated_at = ? 
            WHERE id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(now.to_rfc3339())
        .bind(id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::infra::core::AppError::not_found(
                "TimeBlock",
                id.to_string(),
            ));
        }

        Ok(())
    }

    // --- 读操作 ---
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<TimeBlock>> {
        let row = sqlx::query("SELECT * FROM time_blocks WHERE id = ? AND deleted_at IS NULL")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        match row {
            Some(row) => {
                let time_block = Self::row_to_time_block(&row).map_err(DbError::ConnectionError)?;
                Ok(Some(time_block))
            }
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> AppResult<Vec<TimeBlock>> {
        let rows = sqlx::query(
            "SELECT * FROM time_blocks WHERE deleted_at IS NULL ORDER BY start_time ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let mut time_blocks = Vec::new();
        for row in rows {
            let time_block = Self::row_to_time_block(&row).map_err(DbError::ConnectionError)?;
            time_blocks.push(time_block);
        }

        Ok(time_blocks)
    }
}
