/// TemplateRepository的SQLite实现
///
/// 提供Template实体的具体数据库操作实现

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::shared::core::{AppResult, DbError};
use crate::entities::{Template, Subtask};
use crate::repositories::traits::TemplateRepository;

/// 模板仓库的SQLite实现
#[derive(Clone)]
pub struct SqliteTemplateRepository {
    pool: SqlitePool,
}

impl SqliteTemplateRepository {
    /// 创建新的TemplateRepository实例
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 将数据库行转换为Template对象
    fn row_to_template(row: &sqlx::sqlite::SqliteRow) -> Result<Template, sqlx::Error> {
        let subtasks_json: Option<String> = row.try_get("subtasks_template")?;
        let subtasks_template =
            subtasks_json.and_then(|json| serde_json::from_str::<Vec<Subtask>>(&json).ok());

        Ok(Template {
            id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|_| {
                sqlx::Error::ColumnDecode {
                    index: "id".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid UUID",
                    )),
                }
            })?,
            name: row.try_get("name")?,
            title_template: row.try_get("title_template")?,
            glance_note_template: row.try_get("glance_note_template")?,
            detail_note_template: row.try_get("detail_note_template")?,
            estimated_duration_template: row.try_get("estimated_duration_template")?,
            subtasks_template,
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
        })
    }
}

#[async_trait]
impl TemplateRepository for SqliteTemplateRepository {
    // --- 写操作 ---
    async fn create(&self, tx: &mut Transaction<'_, Sqlite>, template: &Template) -> AppResult<Template> {
        sqlx::query(
            r#"
            INSERT INTO templates (
                id, name, title_template, glance_note_template, detail_note_template,
                estimated_duration_template, subtasks_template, area_id, created_at, updated_at, is_deleted
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(template.id.to_string())
        .bind(&template.name)
        .bind(&template.title_template)
        .bind(&template.glance_note_template)
        .bind(&template.detail_note_template)
        .bind(template.estimated_duration_template)
        .bind(
            template.subtasks_template
                .as_ref()
                .map(|s| serde_json::to_string(s).unwrap_or_default()),
        )
        .bind(template.area_id.map(|id| id.to_string()))
        .bind(template.created_at.to_rfc3339())
        .bind(template.updated_at.to_rfc3339())
        .bind(template.is_deleted)
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        Ok(template.clone())
    }

    async fn update(&self, tx: &mut Transaction<'_, Sqlite>, template: &Template) -> AppResult<Template> {
        let result = sqlx::query(
            r#"
            UPDATE templates SET 
                name = ?, title_template = ?, glance_note_template = ?, detail_note_template = ?,
                estimated_duration_template = ?, subtasks_template = ?, area_id = ?, updated_at = ?
            WHERE id = ? AND is_deleted = FALSE
            "#,
        )
        .bind(&template.name)
        .bind(&template.title_template)
        .bind(&template.glance_note_template)
        .bind(&template.detail_note_template)
        .bind(template.estimated_duration_template)
        .bind(
            template.subtasks_template
                .as_ref()
                .map(|s| serde_json::to_string(s).unwrap_or_default()),
        )
        .bind(template.area_id.map(|id| id.to_string()))
        .bind(template.updated_at.to_rfc3339())
        .bind(template.id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::shared::core::AppError::not_found(
                "Template",
                template.id.to_string(),
            ));
        }

        Ok(template.clone())
    }

    async fn delete(&self, tx: &mut Transaction<'_, Sqlite>, id: Uuid) -> AppResult<()> {
        let now = Utc::now();
        let result = sqlx::query(
            r#"
            UPDATE templates SET 
                is_deleted = TRUE, 
                updated_at = ? 
            WHERE id = ? AND is_deleted = FALSE
            "#,
        )
        .bind(now.to_rfc3339())
        .bind(id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::shared::core::AppError::not_found(
                "Template",
                id.to_string(),
            ));
        }

        Ok(())
    }

    // --- 读操作 ---
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Template>> {
        let row = sqlx::query("SELECT * FROM templates WHERE id = ? AND is_deleted = FALSE")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        match row {
            Some(row) => {
                let template = Self::row_to_template(&row).map_err(DbError::ConnectionError)?;
                Ok(Some(template))
            }
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> AppResult<Vec<Template>> {
        let rows = sqlx::query("SELECT * FROM templates WHERE is_deleted = FALSE ORDER BY name ASC")
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        let mut templates = Vec::new();
        for row in rows {
            let template = Self::row_to_template(&row).map_err(DbError::ConnectionError)?;
            templates.push(template);
        }

        Ok(templates)
    }
}
