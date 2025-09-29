/// 时间块数据访问层

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::shared::{
    core::{AppResult, DbError, SourceInfo, Task, TimeBlock},
    database::{FreeTimeSlot, Repository, TimeBlockRepository, TimeBlockTaskRepository},
};

/// 时间块仓库的SQLx实现
#[derive(Clone)]
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
impl Repository<TimeBlock> for SqlxTimeBlockRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<TimeBlock>> {
        let row = sqlx::query("SELECT * FROM time_blocks WHERE id = ? AND is_deleted = FALSE")
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

    async fn create(&self, time_block: &TimeBlock) -> AppResult<TimeBlock> {
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
        .bind(time_block.source_info.as_ref().and_then(|s| serde_json::to_string(s).ok()))
        .bind(&time_block.external_source_id)
        .bind(&time_block.external_source_provider)
        .bind(time_block.external_source_metadata.as_ref().and_then(|s| serde_json::to_string(s).ok()))
        .bind(&time_block.recurrence_rule)
        .bind(time_block.recurrence_parent_id.map(|id| id.to_string()))
        .bind(time_block.recurrence_original_date.map(|dt| dt.to_rfc3339()))
        .bind(time_block.recurrence_exclusions.as_ref().and_then(|s| serde_json::to_string(s).ok()))
        .execute(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        Ok(time_block.clone())
    }

    async fn update(&self, time_block: &TimeBlock) -> AppResult<TimeBlock> {
        let result = sqlx::query(
            r#"
            UPDATE time_blocks SET
                title = ?, glance_note = ?, detail_note = ?, start_time = ?, end_time = ?,
                area_id = ?, updated_at = ?
            WHERE id = ? AND is_deleted = FALSE
            "#,
        )
        .bind(&time_block.title)
        .bind(&time_block.glance_note)
        .bind(&time_block.detail_note)
        .bind(time_block.start_time.to_rfc3339())
        .bind(time_block.end_time.to_rfc3339())
        .bind(time_block.area_id.map(|id| id.to_string()))
        .bind(time_block.updated_at.to_rfc3339())
        .bind(time_block.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::shared::core::AppError::not_found(
                "TimeBlock",
                time_block.id.to_string(),
            ));
        }

        Ok(time_block.clone())
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        let result = sqlx::query(
            "UPDATE time_blocks SET is_deleted = TRUE, updated_at = ? WHERE id = ? AND is_deleted = FALSE",
        )
        .bind(Utc::now().to_rfc3339())
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::shared::core::AppError::not_found(
                "TimeBlock",
                id.to_string(),
            ));
        }

        Ok(())
    }

    async fn find_all(&self) -> AppResult<Vec<TimeBlock>> {
        let rows = sqlx::query("SELECT * FROM time_blocks WHERE is_deleted = FALSE ORDER BY start_time ASC")
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        let time_blocks = rows
            .iter()
            .map(Self::row_to_time_block)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(time_blocks)
    }
}

#[async_trait]
impl TimeBlockRepository for SqlxTimeBlockRepository {
    async fn find_by_date(&self, date: DateTime<Utc>) -> AppResult<Vec<TimeBlock>> {
        let day_start = crate::shared::core::normalize_to_day_start(date);
        let day_end = day_start + chrono::Duration::days(1);

        let rows = sqlx::query(
            r#"
            SELECT * FROM time_blocks 
            WHERE is_deleted = FALSE 
            AND start_time >= ? AND start_time < ?
            ORDER BY start_time ASC
            "#,
        )
        .bind(day_start.to_rfc3339())
        .bind(day_end.to_rfc3339())
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let time_blocks = rows
            .iter()
            .map(Self::row_to_time_block)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(time_blocks)
    }

    async fn find_by_date_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> AppResult<Vec<TimeBlock>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM time_blocks 
            WHERE is_deleted = FALSE 
            AND start_time >= ? AND end_time <= ?
            ORDER BY start_time ASC
            "#,
        )
        .bind(start_date.to_rfc3339())
        .bind(end_date.to_rfc3339())
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let time_blocks = rows
            .iter()
            .map(Self::row_to_time_block)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(time_blocks)
    }

    async fn find_by_task_id(&self, task_id: Uuid) -> AppResult<Vec<TimeBlock>> {
        let rows = sqlx::query(
            r#"
            SELECT tb.* FROM time_blocks tb
            INNER JOIN time_block_tasks tbt ON tb.id = tbt.time_block_id
            WHERE tb.is_deleted = FALSE AND tbt.task_id = ?
            ORDER BY tb.start_time ASC
            "#,
        )
        .bind(task_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let time_blocks = rows
            .iter()
            .map(Self::row_to_time_block)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(time_blocks)
    }

    async fn find_by_area_id(&self, area_id: Uuid) -> AppResult<Vec<TimeBlock>> {
        let rows = sqlx::query(
            "SELECT * FROM time_blocks WHERE area_id = ? AND is_deleted = FALSE ORDER BY start_time ASC",
        )
        .bind(area_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let time_blocks = rows
            .iter()
            .map(Self::row_to_time_block)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(time_blocks)
    }

    async fn check_conflict(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        exclude_id: Option<Uuid>,
    ) -> AppResult<bool> {
        let mut query_str = r#"
            SELECT COUNT(*) as count FROM time_blocks 
            WHERE is_deleted = FALSE 
            AND NOT (end_time <= ? OR start_time >= ?)
        "#.to_string();

        let mut query = sqlx::query(&query_str)
            .bind(start_time.to_rfc3339())
            .bind(end_time.to_rfc3339());

        if let Some(exclude_id) = exclude_id {
            query_str.push_str(" AND id != ?");
            query = sqlx::query(&query_str)
                .bind(start_time.to_rfc3339())
                .bind(end_time.to_rfc3339())
                .bind(exclude_id.to_string());
        }

        let row = query.fetch_one(&self.pool).await.map_err(DbError::ConnectionError)?;
        let count: i64 = row.try_get("count").unwrap_or(0);

        Ok(count > 0)
    }

    async fn find_free_slots(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        min_duration_minutes: i32,
    ) -> AppResult<Vec<FreeTimeSlot>> {
        // 获取时间范围内的所有时间块
        let existing_blocks = self.find_by_date_range(start_time, end_time).await?;

        let mut free_slots = Vec::new();
        let mut current_time = start_time;

        // 按开始时间排序
        let mut sorted_blocks = existing_blocks;
        sorted_blocks.sort_by(|a, b| a.start_time.cmp(&b.start_time));

        for block in &sorted_blocks {
            // 检查当前时间到下一个时间块开始之间是否有足够的空闲时间
            if block.start_time > current_time {
                let gap_duration = (block.start_time - current_time).num_minutes();
                if gap_duration >= min_duration_minutes as i64 {
                    free_slots.push(FreeTimeSlot {
                        start_time: current_time,
                        end_time: block.start_time,
                        duration_minutes: gap_duration as i32,
                    });
                }
            }
            current_time = current_time.max(block.end_time);
        }

        // 检查最后一个时间块到结束时间之间的空闲时间
        if current_time < end_time {
            let gap_duration = (end_time - current_time).num_minutes();
            if gap_duration >= min_duration_minutes as i64 {
                free_slots.push(FreeTimeSlot {
                    start_time: current_time,
                    end_time,
                    duration_minutes: gap_duration as i32,
                });
            }
        }

        Ok(free_slots)
    }
}

/// 时间块任务关联仓库的SQLx实现
#[derive(Clone)]
pub struct SqlxTimeBlockTaskRepository {
    pool: SqlitePool,
}

impl SqlxTimeBlockTaskRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TimeBlockTaskRepository for SqlxTimeBlockTaskRepository {
    async fn link_task(&self, time_block_id: Uuid, task_id: Uuid) -> AppResult<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO time_block_tasks (time_block_id, task_id) VALUES (?, ?)",
        )
        .bind(time_block_id.to_string())
        .bind(task_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        Ok(())
    }

    async fn unlink_task(&self, time_block_id: Uuid, task_id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM time_block_tasks WHERE time_block_id = ? AND task_id = ?")
            .bind(time_block_id.to_string())
            .bind(task_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        Ok(())
    }

    async fn get_tasks_for_block(&self, time_block_id: Uuid) -> AppResult<Vec<Task>> {
        // 简化实现，返回空列表
        // 在实际应用中，这里需要连接tasks表
        Ok(Vec::new())
    }

    async fn get_blocks_for_task(&self, task_id: Uuid) -> AppResult<Vec<TimeBlock>> {
        let rows = sqlx::query(
            r#"
            SELECT tb.* FROM time_blocks tb
            INNER JOIN time_block_tasks tbt ON tb.id = tbt.time_block_id
            WHERE tb.is_deleted = FALSE AND tbt.task_id = ?
            ORDER BY tb.start_time ASC
            "#,
        )
        .bind(task_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let time_blocks = rows
            .iter()
            .map(SqlxTimeBlockRepository::row_to_time_block)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(time_blocks)
    }

    async fn clear_block_tasks(&self, time_block_id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM time_block_tasks WHERE time_block_id = ?")
            .bind(time_block_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        Ok(())
    }

    async fn clear_task_blocks(&self, task_id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM time_block_tasks WHERE task_id = ?")
            .bind(task_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::database::connection::create_test_database;

    #[tokio::test]
    async fn test_time_block_crud_operations() {
        let pool = create_test_database().await.unwrap();
        let repo = SqlxTimeBlockRepository::new(pool);

        // 创建测试时间块
        let now = Utc::now();
        let time_block = TimeBlock::new(
            Uuid::new_v4(),
            now + chrono::Duration::hours(1),
            now + chrono::Duration::hours(2),
            now,
        ).unwrap();

        // 测试创建
        let created_block = repo.create(&time_block).await.unwrap();
        assert_eq!(created_block.start_time, time_block.start_time);

        // 测试查找
        let found_block = repo.find_by_id(time_block.id).await.unwrap().unwrap();
        assert_eq!(found_block.id, time_block.id);

        // 测试删除
        repo.delete(time_block.id).await.unwrap();
        let deleted_block = repo.find_by_id(time_block.id).await.unwrap();
        assert!(deleted_block.is_none());
    }

    #[tokio::test]
    async fn test_time_conflict_check() {
        let pool = create_test_database().await.unwrap();
        let repo = SqlxTimeBlockRepository::new(pool);

        let now = Utc::now();
        
        // 创建时间块
        let time_block = TimeBlock::new(
            Uuid::new_v4(),
            now + chrono::Duration::hours(1),
            now + chrono::Duration::hours(2),
            now,
        ).unwrap();
        repo.create(&time_block).await.unwrap();

        // 测试冲突检查
        let has_conflict = repo
            .check_conflict(
                now + chrono::Duration::minutes(30),
                now + chrono::Duration::minutes(90),
                None,
            )
            .await
            .unwrap();

        assert!(has_conflict);

        // 测试无冲突时间
        let no_conflict = repo
            .check_conflict(
                now + chrono::Duration::hours(3),
                now + chrono::Duration::hours(4),
                None,
            )
            .await
            .unwrap();

        assert!(!no_conflict);
    }
}
