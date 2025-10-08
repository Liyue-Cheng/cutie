/// TimeBlock 核心 CRUD 仓库
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::{
    entities::{TimeBlock, TimeBlockRow, UpdateTimeBlockRequest},
    shared::core::{AppError, AppResult, DbError},
};

pub struct TimeBlockRepository;

impl TimeBlockRepository {
    /// 在事务中查询时间块
    pub async fn find_by_id_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
    ) -> AppResult<TimeBlock> {
        let query = r#"
            SELECT id, title, glance_note, detail_note, start_time, end_time, is_all_day, area_id,
                   created_at, updated_at, is_deleted, source_info,
                   external_source_id, external_source_provider, external_source_metadata,
                   recurrence_rule, recurrence_parent_id, recurrence_original_date
            FROM time_blocks
            WHERE id = ? AND is_deleted = false
        "#;

        let row = sqlx::query_as::<_, TimeBlockRow>(query)
            .bind(block_id.to_string())
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        row.ok_or_else(|| AppError::not_found("TimeBlock", block_id.to_string()))
            .and_then(|r| {
                TimeBlock::try_from(r).map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))
            })
    }

    /// 非事务查询时间块
    pub async fn find_by_id(pool: &SqlitePool, block_id: Uuid) -> AppResult<TimeBlock> {
        let query = r#"
            SELECT id, title, glance_note, detail_note, start_time, end_time, is_all_day, area_id,
                   created_at, updated_at, is_deleted, source_info,
                   external_source_id, external_source_provider, external_source_metadata,
                   recurrence_rule, recurrence_parent_id, recurrence_original_date
            FROM time_blocks
            WHERE id = ? AND is_deleted = false
        "#;

        let row = sqlx::query_as::<_, TimeBlockRow>(query)
            .bind(block_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        row.ok_or_else(|| AppError::not_found("TimeBlock", block_id.to_string()))
            .and_then(|r| {
                TimeBlock::try_from(r).map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))
            })
    }

    /// 插入时间块
    pub async fn insert_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block: &TimeBlock,
    ) -> AppResult<()> {
        let query = r#"
            INSERT INTO time_blocks (
                id, title, glance_note, detail_note, start_time, end_time, is_all_day, area_id,
                created_at, updated_at, is_deleted, source_info,
                external_source_id, external_source_provider, external_source_metadata,
                recurrence_rule, recurrence_parent_id, recurrence_original_date
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(block.id.to_string())
            .bind(&block.title)
            .bind(&block.glance_note)
            .bind(&block.detail_note)
            .bind(block.start_time.to_rfc3339())
            .bind(block.end_time.to_rfc3339())
            .bind(block.is_all_day)
            .bind(block.area_id.map(|id| id.to_string()))
            .bind(block.created_at.to_rfc3339())
            .bind(block.updated_at.to_rfc3339())
            .bind(block.is_deleted)
            .bind(
                block
                    .source_info
                    .as_ref()
                    .map(|s| serde_json::to_string(s).unwrap()),
            )
            .bind(&block.external_source_id)
            .bind(&block.external_source_provider)
            .bind(
                block
                    .external_source_metadata
                    .as_ref()
                    .map(|m| serde_json::to_string(m).unwrap()),
            )
            .bind(&block.recurrence_rule)
            .bind(block.recurrence_parent_id.map(|id| id.to_string()))
            .bind(&block.recurrence_original_date)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 更新时间块
    pub async fn update_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
        request: &UpdateTimeBlockRequest,
        updated_at: DateTime<Utc>,
    ) -> AppResult<()> {
        let mut updates = Vec::new();
        let mut bindings: Vec<String> = Vec::new();

        // 构建动态 UPDATE 语句
        if let Some(ref title_opt) = request.title {
            updates.push("title = ?");
            bindings.push(title_opt.clone().unwrap_or_default());
        }

        if let Some(ref glance_note_opt) = request.glance_note {
            updates.push("glance_note = ?");
            bindings.push(glance_note_opt.clone().unwrap_or_default());
        }

        if let Some(ref detail_note_opt) = request.detail_note {
            updates.push("detail_note = ?");
            bindings.push(detail_note_opt.clone().unwrap_or_default());
        }

        if let Some(start_time) = request.start_time {
            updates.push("start_time = ?");
            bindings.push(start_time.to_rfc3339());
        }

        if let Some(end_time) = request.end_time {
            updates.push("end_time = ?");
            bindings.push(end_time.to_rfc3339());
        }

        if let Some(is_all_day) = request.is_all_day {
            updates.push("is_all_day = ?");
            bindings.push(if is_all_day { "1" } else { "0" }.to_string());
        }

        if let Some(ref area_id_opt) = request.area_id {
            updates.push("area_id = ?");
            bindings.push(area_id_opt.map(|id| id.to_string()).unwrap_or_default());
        }

        // 如果没有任何字段要更新，直接返回
        if updates.is_empty() {
            return Ok(());
        }

        // 添加 updated_at
        updates.push("updated_at = ?");

        let query = format!("UPDATE time_blocks SET {} WHERE id = ?", updates.join(", "));

        let mut query_builder = sqlx::query(&query);

        // 绑定参数
        for binding in bindings {
            query_builder = query_builder.bind(binding);
        }

        // 绑定 updated_at 和 id
        query_builder = query_builder
            .bind(updated_at.to_rfc3339())
            .bind(block_id.to_string());

        query_builder
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 软删除时间块
    pub async fn soft_delete_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
    ) -> AppResult<()> {
        let query = "UPDATE time_blocks SET is_deleted = true, updated_at = ? WHERE id = ?";
        sqlx::query(query)
            .bind(Utc::now().to_rfc3339())
            .bind(block_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;
        Ok(())
    }

    /// 截断时间块到指定时间
    pub async fn truncate_to_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
        end_time: DateTime<Utc>,
    ) -> AppResult<()> {
        let query = "UPDATE time_blocks SET end_time = ?, updated_at = ? WHERE id = ?";
        sqlx::query(query)
            .bind(end_time.to_rfc3339())
            .bind(end_time.to_rfc3339())
            .bind(block_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;
        Ok(())
    }

    /// 查询时间范围内的时间块
    pub async fn find_in_range(
        pool: &SqlitePool,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> AppResult<Vec<TimeBlock>> {
        let mut query = String::from(
            r#"
            SELECT 
                id, title, glance_note, detail_note, start_time, end_time, is_all_day, area_id,
                created_at, updated_at, is_deleted, source_info,
                external_source_id, external_source_provider, external_source_metadata,
                recurrence_rule, recurrence_parent_id, recurrence_original_date
            FROM time_blocks
            WHERE is_deleted = false
        "#,
        );

        // 添加时间范围过滤
        if start_time.is_some() {
            query.push_str(" AND end_time >= ?");
        }
        if end_time.is_some() {
            query.push_str(" AND start_time < ?");
        }

        query.push_str(" ORDER BY start_time ASC");

        let mut query_builder = sqlx::query_as::<_, TimeBlockRow>(&query);

        if let Some(start) = start_time {
            query_builder = query_builder.bind(start.to_rfc3339());
        }
        if let Some(end) = end_time {
            query_builder = query_builder.bind(end.to_rfc3339());
        }

        let rows = query_builder
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        let blocks: Result<Vec<TimeBlock>, _> = rows.into_iter().map(TimeBlock::try_from).collect();

        blocks.map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))
    }

    /// 检查时间块是否存在
    pub async fn exists_in_tx(tx: &mut Transaction<'_, Sqlite>, block_id: Uuid) -> AppResult<bool> {
        let query = "SELECT COUNT(*) FROM time_blocks WHERE id = ?";
        let count: i64 = sqlx::query_scalar(query)
            .bind(block_id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;
        Ok(count > 0)
    }
}
