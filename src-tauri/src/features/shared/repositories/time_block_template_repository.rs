/// TimeBlockTemplate 核心 CRUD 仓库
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::{
    entities::{TimeBlockTemplate, TimeBlockTemplateRow, UpdateTimeBlockTemplateRequest},
    infra::core::{AppError, AppResult, DbError},
};

pub struct TimeBlockTemplateRepository;

impl TimeBlockTemplateRepository {
    /// 根据ID查询模板
    pub async fn find_by_id(
        pool: &SqlitePool,
        template_id: Uuid,
    ) -> AppResult<Option<TimeBlockTemplate>> {
        let query = r#"
            SELECT id, title, glance_note_template, detail_note_template,
                   duration_minutes, start_time_local, time_type, is_all_day,
                   area_id, created_at, updated_at, is_deleted
            FROM time_block_templates
            WHERE id = ? AND is_deleted = 0
        "#;

        let row = sqlx::query_as::<_, TimeBlockTemplateRow>(query)
            .bind(template_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        match row {
            Some(r) => {
                let template = TimeBlockTemplate::try_from(r)
                    .map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))?;
                Ok(Some(template))
            }
            None => Ok(None),
        }
    }

    /// 在事务中查询模板
    pub async fn find_by_id_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        template_id: Uuid,
    ) -> AppResult<Option<TimeBlockTemplate>> {
        let query = r#"
            SELECT id, title, glance_note_template, detail_note_template,
                   duration_minutes, start_time_local, time_type, is_all_day,
                   area_id, created_at, updated_at, is_deleted
            FROM time_block_templates
            WHERE id = ? AND is_deleted = 0
        "#;

        let row = sqlx::query_as::<_, TimeBlockTemplateRow>(query)
            .bind(template_id.to_string())
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        match row {
            Some(r) => {
                let template = TimeBlockTemplate::try_from(r)
                    .map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))?;
                Ok(Some(template))
            }
            None => Ok(None),
        }
    }

    /// 插入模板
    pub async fn insert_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        template: &TimeBlockTemplate,
    ) -> AppResult<()> {
        let time_type_str = match template.time_type {
            crate::entities::time_block::TimeType::Floating => "FLOATING",
            crate::entities::time_block::TimeType::Fixed => "FIXED",
        };

        let query = r#"
            INSERT INTO time_block_templates (
                id, title, glance_note_template, detail_note_template,
                duration_minutes, start_time_local, time_type, is_all_day,
                area_id, created_at, updated_at, is_deleted
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(template.id.to_string())
            .bind(&template.title)
            .bind(&template.glance_note_template)
            .bind(&template.detail_note_template)
            .bind(template.duration_minutes)
            .bind(&template.start_time_local)
            .bind(time_type_str)
            .bind(template.is_all_day)
            .bind(template.area_id.map(|id| id.to_string()))
            .bind(template.created_at)
            .bind(template.updated_at)
            .bind(template.is_deleted)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 更新模板
    pub async fn update_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        template_id: Uuid,
        request: &UpdateTimeBlockTemplateRequest,
        updated_at: DateTime<Utc>,
    ) -> AppResult<TimeBlockTemplate> {
        let mut set_clauses = vec![];

        if request.title.is_some() {
            set_clauses.push("title = ?");
        }
        if request.glance_note_template.is_some() {
            set_clauses.push("glance_note_template = ?");
        }
        if request.detail_note_template.is_some() {
            set_clauses.push("detail_note_template = ?");
        }
        if request.duration_minutes.is_some() {
            set_clauses.push("duration_minutes = ?");
        }
        if request.start_time_local.is_some() {
            set_clauses.push("start_time_local = ?");
        }
        if request.time_type.is_some() {
            set_clauses.push("time_type = ?");
        }
        if request.is_all_day.is_some() {
            set_clauses.push("is_all_day = ?");
        }
        if request.area_id.is_some() {
            set_clauses.push("area_id = ?");
        }
        set_clauses.push("updated_at = ?");

        let set_clause = set_clauses.join(", ");
        let query = format!(
            "UPDATE time_block_templates SET {} WHERE id = ?",
            set_clause
        );

        let mut q = sqlx::query(&query);

        if let Some(ref title_opt) = request.title {
            q = q.bind(title_opt.clone());
        }
        if let Some(ref note_opt) = request.glance_note_template {
            q = q.bind(note_opt.clone());
        }
        if let Some(ref note_opt) = request.detail_note_template {
            q = q.bind(note_opt.clone());
        }
        if let Some(duration) = request.duration_minutes {
            q = q.bind(duration);
        }
        if let Some(ref time_local) = request.start_time_local {
            q = q.bind(time_local);
        }
        if let Some(ref time_type) = request.time_type {
            let time_type_str = match time_type {
                crate::entities::time_block::TimeType::Floating => "FLOATING",
                crate::entities::time_block::TimeType::Fixed => "FIXED",
            };
            q = q.bind(time_type_str);
        }
        if let Some(is_all_day) = request.is_all_day {
            q = q.bind(is_all_day);
        }
        if let Some(ref area_id_opt) = request.area_id {
            q = q.bind(area_id_opt.map(|id| id.to_string()));
        }
        q = q.bind(updated_at);
        q = q.bind(template_id.to_string());

        q.execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Self::find_by_id_in_tx(tx, template_id)
            .await?
            .ok_or_else(|| AppError::NotFound {
                entity_type: "TimeBlockTemplate".to_string(),
                entity_id: template_id.to_string(),
            })
    }

    /// 软删除模板
    pub async fn soft_delete_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        template_id: Uuid,
        updated_at: DateTime<Utc>,
    ) -> AppResult<()> {
        let query = r#"
            UPDATE time_block_templates
            SET is_deleted = 1, updated_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(updated_at)
            .bind(template_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 查询所有未删除的模板
    pub async fn find_all(pool: &SqlitePool) -> AppResult<Vec<TimeBlockTemplate>> {
        let query = r#"
            SELECT id, title, glance_note_template, detail_note_template,
                   duration_minutes, start_time_local, time_type, is_all_day,
                   area_id, created_at, updated_at, is_deleted
            FROM time_block_templates
            WHERE is_deleted = 0
            ORDER BY created_at DESC
        "#;

        let rows = sqlx::query_as::<_, TimeBlockTemplateRow>(query)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        let templates: Result<Vec<TimeBlockTemplate>, _> =
            rows.into_iter().map(TimeBlockTemplate::try_from).collect();

        templates.map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))
    }
}
