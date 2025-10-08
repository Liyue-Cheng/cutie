use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;
use std::collections::HashMap;

use crate::common::error::DbError;
use crate::common::utils::time_utils::normalize_to_day_start;
use crate::core::models::{TimeBlock, SourceInfo};
use super::{TimeBlockRepository, Transaction, FreeTimeSlot, TimeBlockUsageStats};

/// TimeBlockRepository的SQLx实现
pub struct SqlxTimeBlockRepository {
    pool: SqlitePool,
}

impl SqlxTimeBlockRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 将数据库行转换为TimeBlock对象
    fn row_to_time_block(row: &sqlx::sqlite::SqliteRow) -> Result<TimeBlock, sqlx::Error> {
        let source_info_json: Option<String> = row.try_get("source_info")?;
        let source_info = source_info_json
            .and_then(|json| serde_json::from_str::<SourceInfo>(&json).ok());

        let external_source_metadata_json: Option<String> = row.try_get("external_source_metadata")?;
        let external_source_metadata = external_source_metadata_json
            .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok());

        let recurrence_exclusions_json: Option<String> = row.try_get("recurrence_exclusions")?;
        let recurrence_exclusions = recurrence_exclusions_json
            .and_then(|json| serde_json::from_str::<Vec<DateTime<Utc>>>(&json).ok());

        Ok(TimeBlock {
            id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|_| {
                sqlx::Error::ColumnDecode {
                    index: "id".to_string(),
                    source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UUID")),
                }
            })?,
            title: row.try_get("title")?,
            glance_note: row.try_get("glance_note")?,
            detail_note: row.try_get("detail_note")?,
            start_time: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("start_time")?)
                .map_err(|_| sqlx::Error::ColumnDecode {
                    index: "start_time".to_string(),
                    source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid datetime")),
                })?
                .with_timezone(&Utc),
            end_time: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("end_time")?)
                .map_err(|_| sqlx::Error::ColumnDecode {
                    index: "end_time".to_string(),
                    source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid datetime")),
                })?
                .with_timezone(&Utc),
            area_id: row.try_get::<Option<String>, _>("area_id")?
                .and_then(|s| Uuid::parse_str(&s).ok()),
            created_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("created_at")?)
                .map_err(|_| sqlx::Error::ColumnDecode {
                    index: "created_at".to_string(),
                    source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid datetime")),
                })?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("updated_at")?)
                .map_err(|_| sqlx::Error::ColumnDecode {
                    index: "updated_at".to_string(),
                    source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid datetime")),
                })?
                .with_timezone(&Utc),
            is_deleted: row.try_get("is_deleted")?,
            source_info,
            external_source_id: row.try_get("external_source_id")?,
            external_source_provider: row.try_get("external_source_provider")?,
            external_source_metadata,
            recurrence_rule: row.try_get("recurrence_rule")?,
            recurrence_parent_id: row.try_get::<Option<String>, _>("recurrence_parent_id")?
                .and_then(|s| Uuid::parse_str(&s).ok()),
            recurrence_original_date: row.try_get::<Option<String>, _>("recurrence_original_date")?
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            recurrence_exclusions,
        })
    }

    /// 将TimeBlock对象转换为数据库参数
    fn time_block_to_params(time_block: &TimeBlock) -> (String, Option<String>, Option<String>, Option<String>, String, String, Option<String>, String, String, bool, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>) {
        let source_info_json = time_block.source_info.as_ref()
            .and_then(|s| serde_json::to_string(s).ok());
        let external_source_metadata_json = time_block.external_source_metadata.as_ref()
            .and_then(|s| serde_json::to_string(s).ok());
        let recurrence_exclusions_json = time_block.recurrence_exclusions.as_ref()
            .and_then(|s| serde_json::to_string(s).ok());

        (
            time_block.id.to_string(),
            time_block.title.clone(),
            time_block.glance_note.clone(),
            time_block.detail_note.clone(),
            time_block.start_time.to_rfc3339(),
            time_block.end_time.to_rfc3339(),
            time_block.area_id.map(|id| id.to_string()),
            time_block.created_at.to_rfc3339(),
            time_block.updated_at.to_rfc3339(),
            time_block.is_deleted,
            source_info_json,
            time_block.external_source_id.clone(),
            time_block.external_source_provider.clone(),
            external_source_metadata_json,
            time_block.recurrence_rule.clone(),
            time_block.recurrence_parent_id.map(|id| id.to_string()),
            time_block.recurrence_original_date.map(|dt| dt.to_rfc3339()),
            recurrence_exclusions_json,
        )
    }
}

#[async_trait]
impl TimeBlockRepository for SqlxTimeBlockRepository {
    async fn create(&self, tx: &mut Transaction<'_>, time_block: &TimeBlock) -> Result<TimeBlock, DbError> {
        let params = Self::time_block_to_params(time_block);

        let result = sqlx::query(
            r#"
            INSERT INTO time_blocks (id, title, glance_note, detail_note, start_time, end_time, area_id,
                                   created_at, updated_at, is_deleted, source_info, external_source_id,
                                   external_source_provider, external_source_metadata, recurrence_rule,
                                   recurrence_parent_id, recurrence_original_date, recurrence_exclusions)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&params.0).bind(&params.1).bind(&params.2).bind(&params.3).bind(&params.4)
        .bind(&params.5).bind(&params.6).bind(&params.7).bind(&params.8).bind(&params.9)
        .bind(&params.10).bind(&params.11).bind(&params.12).bind(&params.13).bind(&params.14)
        .bind(&params.15).bind(&params.16).bind(&params.17)
        .execute(&mut **tx)
        .await;

        match result {
            Ok(_) => Ok(time_block.clone()),
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                Err(DbError::ConstraintViolation {
                    message: format!("TimeBlock with id {} already exists", time_block.id),
                })
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn update(&self, tx: &mut Transaction<'_>, time_block: &TimeBlock) -> Result<TimeBlock, DbError> {
        let params = Self::time_block_to_params(time_block);

        let result = sqlx::query(
            r#"
            UPDATE time_blocks SET 
                title = ?, glance_note = ?, detail_note = ?, start_time = ?, end_time = ?, area_id = ?,
                updated_at = ?, source_info = ?, external_source_id = ?, external_source_provider = ?,
                external_source_metadata = ?, recurrence_rule = ?, recurrence_parent_id = ?,
                recurrence_original_date = ?, recurrence_exclusions = ?
            WHERE id = ? AND deleted_at IS NULL
            "#
        )
        .bind(&params.1).bind(&params.2).bind(&params.3).bind(&params.4).bind(&params.5)
        .bind(&params.6).bind(&params.8).bind(&params.10).bind(&params.11).bind(&params.12)
        .bind(&params.13).bind(&params.14).bind(&params.15).bind(&params.16).bind(&params.17)
        .bind(&params.0)
        .execute(&mut **tx)
        .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(DbError::NotFound {
                        entity_type: "TimeBlock".to_string(),
                        entity_id: time_block.id.to_string(),
                    })
                } else {
                    Ok(time_block.clone())
                }
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_by_id(&self, time_block_id: Uuid) -> Result<Option<TimeBlock>, DbError> {
        let result = sqlx::query("SELECT * FROM time_blocks WHERE id = ? AND deleted_at IS NULL")
            .bind(time_block_id.to_string())
            .fetch_optional(&self.pool)
            .await;

        match result {
            Ok(Some(row)) => Ok(Some(Self::row_to_time_block(&row).map_err(DbError::ConnectionError)?)),
            Ok(None) => Ok(None),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_overlapping(&self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<Vec<TimeBlock>, DbError> {
        let result = sqlx::query(
            r#"
            SELECT * FROM time_blocks 
            WHERE deleted_at IS NULL 
            AND start_time < ? AND end_time > ?
            ORDER BY start_time ASC
            "#
        )
        .bind(end_time.to_rfc3339())
        .bind(start_time.to_rfc3339())
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let time_blocks: Result<Vec<TimeBlock>, _> = rows.iter()
                    .map(|row| Self::row_to_time_block(row))
                    .collect();
                time_blocks.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_by_date(&self, date: DateTime<Utc>) -> Result<Vec<TimeBlock>, DbError> {
        let day_start = normalize_to_day_start(date);
        let day_end = day_start + chrono::Duration::days(1);

        let result = sqlx::query(
            r#"
            SELECT * FROM time_blocks 
            WHERE deleted_at IS NULL 
            AND start_time >= ? AND start_time < ?
            ORDER BY start_time ASC
            "#
        )
        .bind(day_start.to_rfc3339())
        .bind(day_end.to_rfc3339())
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let time_blocks: Result<Vec<TimeBlock>, _> = rows.iter()
                    .map(|row| Self::row_to_time_block(row))
                    .collect();
                time_blocks.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_by_date_range(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<Vec<TimeBlock>, DbError> {
        let result = sqlx::query(
            r#"
            SELECT * FROM time_blocks 
            WHERE deleted_at IS NULL 
            AND start_time >= ? AND start_time <= ?
            ORDER BY start_time ASC
            "#
        )
        .bind(start_date.to_rfc3339())
        .bind(end_date.to_rfc3339())
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let time_blocks: Result<Vec<TimeBlock>, _> = rows.iter()
                    .map(|row| Self::row_to_time_block(row))
                    .collect();
                time_blocks.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_by_area_id(&self, area_id: Uuid) -> Result<Vec<TimeBlock>, DbError> {
        let result = sqlx::query("SELECT * FROM time_blocks WHERE area_id = ? AND deleted_at IS NULL ORDER BY start_time DESC")
            .bind(area_id.to_string())
            .fetch_all(&self.pool)
            .await;

        match result {
            Ok(rows) => {
                let time_blocks: Result<Vec<TimeBlock>, _> = rows.iter()
                    .map(|row| Self::row_to_time_block(row))
                    .collect();
                time_blocks.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_by_task_id(&self, task_id: Uuid) -> Result<Vec<TimeBlock>, DbError> {
        let result = sqlx::query(
            r#"
            SELECT tb.* FROM time_blocks tb
            INNER JOIN task_time_block_links ttbl ON tb.id = ttbl.time_block_id
            WHERE ttbl.task_id = ? AND tb.deleted_at IS NULL
            ORDER BY tb.start_time ASC
            "#
        )
        .bind(task_id.to_string())
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let time_blocks: Result<Vec<TimeBlock>, _> = rows.iter()
                    .map(|row| Self::row_to_time_block(row))
                    .collect();
                time_blocks.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn soft_delete(&self, tx: &mut Transaction<'_>, time_block_id: Uuid) -> Result<(), DbError> {
        let result = sqlx::query("UPDATE time_blocks SET is_deleted = TRUE, updated_at = ? WHERE id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(time_block_id.to_string())
            .execute(&mut **tx)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn restore(&self, tx: &mut Transaction<'_>, time_block_id: Uuid) -> Result<TimeBlock, DbError> {
        let result = sqlx::query("UPDATE time_blocks SET deleted_at IS NULL, updated_at = ? WHERE id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(time_block_id.to_string())
            .execute(&mut **tx)
            .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(DbError::NotFound {
                        entity_type: "TimeBlock".to_string(),
                        entity_id: time_block_id.to_string(),
                    })
                } else {
                    // 查询恢复后的时间块
                    let time_block_result = sqlx::query("SELECT * FROM time_blocks WHERE id = ?")
                        .bind(time_block_id.to_string())
                        .fetch_one(&mut **tx)
                        .await;

                    match time_block_result {
                        Ok(row) => Ok(Self::row_to_time_block(&row).map_err(DbError::ConnectionError)?),
                        Err(e) => Err(DbError::ConnectionError(e)),
                    }
                }
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn link_task(&self, tx: &mut Transaction<'_>, time_block_id: Uuid, task_id: Uuid) -> Result<(), DbError> {
        let result = sqlx::query(
            "INSERT OR IGNORE INTO task_time_block_links (task_id, time_block_id, created_at) VALUES (?, ?, ?)"
        )
        .bind(task_id.to_string())
        .bind(time_block_id.to_string())
        .bind(Utc::now().to_rfc3339())
        .execute(&mut **tx)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn unlink_task(&self, tx: &mut Transaction<'_>, time_block_id: Uuid, task_id: Uuid) -> Result<(), DbError> {
        let result = sqlx::query("DELETE FROM task_time_block_links WHERE task_id = ? AND time_block_id = ?")
            .bind(task_id.to_string())
            .bind(time_block_id.to_string())
            .execute(&mut **tx)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn unlink_all_tasks(&self, tx: &mut Transaction<'_>, time_block_id: Uuid) -> Result<(), DbError> {
        let result = sqlx::query("DELETE FROM task_time_block_links WHERE time_block_id = ?")
            .bind(time_block_id.to_string())
            .execute(&mut **tx)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn has_time_conflict(&self, start_time: DateTime<Utc>, end_time: DateTime<Utc>, exclude_id: Option<Uuid>) -> Result<bool, DbError> {
        let query = if let Some(exclude_id) = exclude_id {
            sqlx::query(
                r#"
                SELECT COUNT(*) as count FROM time_blocks 
                WHERE deleted_at IS NULL 
                AND id != ?
                AND start_time < ? AND end_time > ?
                "#
            )
            .bind(exclude_id.to_string())
            .bind(end_time.to_rfc3339())
            .bind(start_time.to_rfc3339())
        } else {
            sqlx::query(
                r#"
                SELECT COUNT(*) as count FROM time_blocks 
                WHERE deleted_at IS NULL 
                AND start_time < ? AND end_time > ?
                "#
            )
            .bind(end_time.to_rfc3339())
            .bind(start_time.to_rfc3339())
        };

        let result = query.fetch_one(&self.pool).await;

        match result {
            Ok(row) => {
                let count: i64 = row.try_get("count").map_err(DbError::ConnectionError)?;
                Ok(count > 0)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_free_time_slots(&self, start_time: DateTime<Utc>, end_time: DateTime<Utc>, min_duration_minutes: i32) -> Result<Vec<FreeTimeSlot>, DbError> {
        // 获取时间范围内的所有时间块
        let time_blocks = self.find_overlapping(start_time, end_time).await?;

        let mut free_slots = Vec::new();
        let mut current_time = start_time;

        // 按开始时间排序
        let mut sorted_blocks = time_blocks;
        sorted_blocks.sort_by(|a, b| a.start_time.cmp(&b.start_time));

        for time_block in sorted_blocks {
            // 如果当前时间在时间块开始之前，有空闲时间段
            if current_time < time_block.start_time {
                let duration_minutes = (time_block.start_time - current_time).num_minutes();
                if duration_minutes >= min_duration_minutes as i64 {
                    free_slots.push(FreeTimeSlot {
                        start_time: current_time,
                        end_time: time_block.start_time,
                        duration_minutes,
                    });
                }
            }
            
            // 更新当前时间到时间块结束时间
            current_time = current_time.max(time_block.end_time);
        }

        // 检查最后一个时间段
        if current_time < end_time {
            let duration_minutes = (end_time - current_time).num_minutes();
            if duration_minutes >= min_duration_minutes as i64 {
                free_slots.push(FreeTimeSlot {
                    start_time: current_time,
                    end_time,
                    duration_minutes,
                });
            }
        }

        Ok(free_slots)
    }

    async fn get_usage_statistics(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<TimeBlockUsageStats, DbError> {
        let result = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_blocks,
                SUM((julianday(end_time) - julianday(start_time)) * 24 * 60) as total_duration_minutes,
                AVG((julianday(end_time) - julianday(start_time)) * 24 * 60) as average_duration_minutes
            FROM time_blocks 
            WHERE deleted_at IS NULL 
            AND start_time >= ? AND start_time <= ?
            "#
        )
        .bind(start_date.to_rfc3339())
        .bind(end_date.to_rfc3339())
        .fetch_one(&self.pool)
        .await;

        let (total_blocks, total_duration_minutes, average_duration_minutes) = match result {
            Ok(row) => {
                let total_blocks: i64 = row.try_get("total_blocks").map_err(DbError::ConnectionError)?;
                let total_duration: Option<f64> = row.try_get("total_duration_minutes").map_err(DbError::ConnectionError)?;
                let average_duration: Option<f64> = row.try_get("average_duration_minutes").map_err(DbError::ConnectionError)?;
                
                (total_blocks, total_duration.unwrap_or(0.0) as i64, average_duration.unwrap_or(0.0))
            }
            Err(e) => return Err(DbError::ConnectionError(e)),
        };

        // 获取按领域分组的统计
        let area_result = sqlx::query(
            r#"
            SELECT area_id, COUNT(*) as count
            FROM time_blocks 
            WHERE deleted_at IS NULL 
            AND start_time >= ? AND start_time <= ?
            AND area_id IS NOT NULL
            GROUP BY area_id
            "#
        )
        .bind(start_date.to_rfc3339())
        .bind(end_date.to_rfc3339())
        .fetch_all(&self.pool)
        .await;

        let blocks_by_area = match area_result {
            Ok(rows) => {
                let mut map = HashMap::new();
                for row in rows {
                    let area_id_str: String = row.try_get("area_id").map_err(DbError::ConnectionError)?;
                    if let Ok(area_id) = Uuid::parse_str(&area_id_str) {
                        let count: i64 = row.try_get("count").map_err(DbError::ConnectionError)?;
                        map.insert(area_id, count);
                    }
                }
                map
            }
            Err(e) => return Err(DbError::ConnectionError(e)),
        };

        Ok(TimeBlockUsageStats {
            total_blocks,
            total_duration_minutes,
            average_duration_minutes,
            blocks_by_area,
            busiest_hour: None, // 简化实现，不计算最繁忙小时
            utilization_by_day: Vec::new(), // 简化实现，不计算每日利用率
        })
    }

    async fn find_recurring_blocks(&self, parent_id: Uuid) -> Result<Vec<TimeBlock>, DbError> {
        let result = sqlx::query("SELECT * FROM time_blocks WHERE recurrence_parent_id = ? AND deleted_at IS NULL ORDER BY start_time ASC")
            .bind(parent_id.to_string())
            .fetch_all(&self.pool)
            .await;

        match result {
            Ok(rows) => {
                let time_blocks: Result<Vec<TimeBlock>, _> = rows.iter()
                    .map(|row| Self::row_to_time_block(row))
                    .collect();
                time_blocks.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn truncate_at(&self, tx: &mut Transaction<'_>, time_block_id: Uuid, truncate_at: DateTime<Utc>) -> Result<TimeBlock, DbError> {
        let result = sqlx::query("UPDATE time_blocks SET end_time = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL")
            .bind(truncate_at.to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .bind(time_block_id.to_string())
            .execute(&mut **tx)
            .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(DbError::NotFound {
                        entity_type: "TimeBlock".to_string(),
                        entity_id: time_block_id.to_string(),
                    })
                } else {
                    // 查询更新后的时间块
                    let time_block_result = sqlx::query("SELECT * FROM time_blocks WHERE id = ? AND deleted_at IS NULL")
                        .bind(time_block_id.to_string())
                        .fetch_one(&mut **tx)
                        .await;

                    match time_block_result {
                        Ok(row) => Ok(Self::row_to_time_block(&row).map_err(DbError::ConnectionError)?),
                        Err(e) => Err(DbError::ConnectionError(e)),
                    }
                }
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn extend_to(&self, tx: &mut Transaction<'_>, time_block_id: Uuid, new_end_time: DateTime<Utc>) -> Result<TimeBlock, DbError> {
        let result = sqlx::query("UPDATE time_blocks SET end_time = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL")
            .bind(new_end_time.to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .bind(time_block_id.to_string())
            .execute(&mut **tx)
            .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(DbError::NotFound {
                        entity_type: "TimeBlock".to_string(),
                        entity_id: time_block_id.to_string(),
                    })
                } else {
                    // 查询更新后的时间块
                    let time_block_result = sqlx::query("SELECT * FROM time_blocks WHERE id = ? AND deleted_at IS NULL")
                        .bind(time_block_id.to_string())
                        .fetch_one(&mut **tx)
                        .await;

                    match time_block_result {
                        Ok(row) => Ok(Self::row_to_time_block(&row).map_err(DbError::ConnectionError)?),
                        Err(e) => Err(DbError::ConnectionError(e)),
                    }
                }
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn split_at(&self, tx: &mut Transaction<'_>, time_block_id: Uuid, split_at: DateTime<Utc>) -> Result<(TimeBlock, TimeBlock), DbError> {
        // 首先获取原始时间块
        let original = self.find_by_id(time_block_id).await?
            .ok_or_else(|| DbError::NotFound {
                entity_type: "TimeBlock".to_string(),
                entity_id: time_block_id.to_string(),
            })?;

        // 验证分割点在时间块范围内
        if split_at <= original.start_time || split_at >= original.end_time {
            return Err(DbError::ConstraintViolation {
                message: "Split point must be within the time block range".to_string(),
            });
        }

        // 更新原始时间块的结束时间
        let first_block = self.truncate_at(tx, time_block_id, split_at).await?;

        // 创建新的时间块
        let now = Utc::now();
        let second_block = TimeBlock {
            id: uuid::Uuid::new_v4(),
            title: original.title,
            glance_note: original.glance_note,
            detail_note: original.detail_note,
            start_time: split_at,
            end_time: original.end_time,
            area_id: original.area_id,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            source_info: original.source_info,
            external_source_id: original.external_source_id,
            external_source_provider: original.external_source_provider,
            external_source_metadata: original.external_source_metadata,
            recurrence_rule: None, // 分割后的时间块不继承重复规则
            recurrence_parent_id: None,
            recurrence_original_date: None,
            recurrence_exclusions: None,
        };

        let created_second_block = self.create(tx, &second_block).await?;

        Ok((first_block, created_second_block))
    }
}
