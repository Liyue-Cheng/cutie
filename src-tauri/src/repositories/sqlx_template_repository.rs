use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use super::{TemplateCount, TemplateRepository, TemplateUsageStats, Transaction};
use crate::common::error::DbError;
use crate::core::models::{Subtask, Template};

/// TemplateRepository的SQLx实现
pub struct SqlxTemplateRepository {
    pool: SqlitePool,
}

impl SqlxTemplateRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 将数据库行转换为Template对象
    fn row_to_template(row: &sqlx::sqlite::SqliteRow) -> Result<Template, sqlx::Error> {
        let subtasks_template_json: Option<String> = row.try_get("subtasks_template")?;
        let subtasks_template = subtasks_template_json
            .and_then(|json| serde_json::from_str::<Vec<Subtask>>(&json).ok());

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

    /// 将Template对象转换为数据库参数
    fn template_to_params(
        template: &Template,
    ) -> (
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<i32>,
        Option<String>,
        Option<String>,
        String,
        String,
        bool,
    ) {
        let subtasks_template_json = template
            .subtasks_template
            .as_ref()
            .and_then(|s| serde_json::to_string(s).ok());

        (
            template.id.to_string(),
            template.name.clone(),
            template.title_template.clone(),
            template.glance_note_template.clone(),
            template.detail_note_template.clone(),
            template.estimated_duration_template,
            subtasks_template_json,
            template.area_id.map(|id| id.to_string()),
            template.created_at.to_rfc3339(),
            template.updated_at.to_rfc3339(),
            template.is_deleted,
        )
    }
}

#[async_trait]
impl TemplateRepository for SqlxTemplateRepository {
    async fn create(
        &self,
        tx: &mut Transaction<'_>,
        template: &Template,
    ) -> Result<Template, DbError> {
        let params = Self::template_to_params(template);

        let result = sqlx::query(
            r#"
            INSERT INTO templates (id, name, title_template, glance_note_template, detail_note_template, 
                                 estimated_duration_template, subtasks_template, area_id, created_at, updated_at, is_deleted)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&params.0).bind(&params.1).bind(&params.2).bind(&params.3).bind(&params.4)
        .bind(&params.5).bind(&params.6).bind(&params.7).bind(&params.8).bind(&params.9)
        .bind(&params.10)
        .execute(&mut **tx)
        .await;

        match result {
            Ok(_) => Ok(template.clone()),
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                Err(DbError::ConstraintViolation {
                    message: format!("Template with id {} already exists", template.id),
                })
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn update(
        &self,
        tx: &mut Transaction<'_>,
        template: &Template,
    ) -> Result<Template, DbError> {
        let params = Self::template_to_params(template);

        let result = sqlx::query(
            r#"
            UPDATE templates SET 
                name = ?, title_template = ?, glance_note_template = ?, detail_note_template = ?,
                estimated_duration_template = ?, subtasks_template = ?, area_id = ?, updated_at = ?
            WHERE id = ? AND is_deleted = FALSE
            "#,
        )
        .bind(&params.1)
        .bind(&params.2)
        .bind(&params.3)
        .bind(&params.4)
        .bind(&params.5)
        .bind(&params.6)
        .bind(&params.7)
        .bind(&params.9)
        .bind(&params.0)
        .execute(&mut **tx)
        .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(DbError::NotFound {
                        entity_type: "Template".to_string(),
                        entity_id: template.id.to_string(),
                    })
                } else {
                    Ok(template.clone())
                }
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_by_id(&self, template_id: Uuid) -> Result<Option<Template>, DbError> {
        let result = sqlx::query("SELECT * FROM templates WHERE id = ? AND is_deleted = FALSE")
            .bind(template_id.to_string())
            .fetch_optional(&self.pool)
            .await;

        match result {
            Ok(Some(row)) => Ok(Some(
                Self::row_to_template(&row).map_err(DbError::ConnectionError)?,
            )),
            Ok(None) => Ok(None),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_all(&self) -> Result<Vec<Template>, DbError> {
        let result =
            sqlx::query("SELECT * FROM templates WHERE is_deleted = FALSE ORDER BY name ASC")
                .fetch_all(&self.pool)
                .await;

        match result {
            Ok(rows) => {
                let templates: Result<Vec<Template>, _> =
                    rows.iter().map(|row| Self::row_to_template(row)).collect();
                templates.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn search_by_name(&self, name_query: &str) -> Result<Vec<Template>, DbError> {
        let search_pattern = format!("%{}%", name_query);

        let result = sqlx::query(
            "SELECT * FROM templates WHERE name LIKE ? AND is_deleted = FALSE ORDER BY name ASC",
        )
        .bind(&search_pattern)
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let templates: Result<Vec<Template>, _> =
                    rows.iter().map(|row| Self::row_to_template(row)).collect();
                templates.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_by_area_id(&self, area_id: Uuid) -> Result<Vec<Template>, DbError> {
        let result = sqlx::query(
            "SELECT * FROM templates WHERE area_id = ? AND is_deleted = FALSE ORDER BY name ASC",
        )
        .bind(area_id.to_string())
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let templates: Result<Vec<Template>, _> =
                    rows.iter().map(|row| Self::row_to_template(row)).collect();
                templates.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_containing_variable(
        &self,
        variable_name: &str,
    ) -> Result<Vec<Template>, DbError> {
        let variable_pattern = format!("%{{{{{}}}}}%", variable_name);

        let result = sqlx::query(
            r#"
            SELECT * FROM templates 
            WHERE (title_template LIKE ? OR glance_note_template LIKE ? OR detail_note_template LIKE ?)
            AND is_deleted = FALSE 
            ORDER BY name ASC
            "#
        )
        .bind(&variable_pattern).bind(&variable_pattern).bind(&variable_pattern)
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let templates: Result<Vec<Template>, _> =
                    rows.iter().map(|row| Self::row_to_template(row)).collect();
                templates.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn soft_delete(
        &self,
        tx: &mut Transaction<'_>,
        template_id: Uuid,
    ) -> Result<(), DbError> {
        let result =
            sqlx::query("UPDATE templates SET is_deleted = TRUE, updated_at = ? WHERE id = ?")
                .bind(Utc::now().to_rfc3339())
                .bind(template_id.to_string())
                .execute(&mut **tx)
                .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn restore(
        &self,
        tx: &mut Transaction<'_>,
        template_id: Uuid,
    ) -> Result<Template, DbError> {
        let result =
            sqlx::query("UPDATE templates SET is_deleted = FALSE, updated_at = ? WHERE id = ?")
                .bind(Utc::now().to_rfc3339())
                .bind(template_id.to_string())
                .execute(&mut **tx)
                .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(DbError::NotFound {
                        entity_type: "Template".to_string(),
                        entity_id: template_id.to_string(),
                    })
                } else {
                    // 查询恢复后的模板
                    let template_result = sqlx::query("SELECT * FROM templates WHERE id = ?")
                        .bind(template_id.to_string())
                        .fetch_one(&mut **tx)
                        .await;

                    match template_result {
                        Ok(row) => {
                            Ok(Self::row_to_template(&row).map_err(DbError::ConnectionError)?)
                        }
                        Err(e) => Err(DbError::ConnectionError(e)),
                    }
                }
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn clone_template(
        &self,
        tx: &mut Transaction<'_>,
        template_id: Uuid,
        new_name: String,
    ) -> Result<Template, DbError> {
        // 首先查找原模板
        let original = self
            .find_by_id(template_id)
            .await?
            .ok_or_else(|| DbError::NotFound {
                entity_type: "Template".to_string(),
                entity_id: template_id.to_string(),
            })?;

        // 检查新名称是否可用
        let name_available = self.is_name_available(&new_name, None).await?;
        if !name_available {
            return Err(DbError::ConstraintViolation {
                message: format!("Template name '{}' already exists", new_name),
            });
        }

        // 创建克隆模板
        let now = Utc::now();
        let cloned_template = Template {
            id: uuid::Uuid::new_v4(),
            name: new_name,
            title_template: original.title_template,
            glance_note_template: original.glance_note_template,
            detail_note_template: original.detail_note_template,
            estimated_duration_template: original.estimated_duration_template,
            subtasks_template: original.subtasks_template,
            area_id: original.area_id,
            created_at: now,
            updated_at: now,
            is_deleted: false,
        };

        self.create(tx, &cloned_template).await
    }

    async fn is_name_available(
        &self,
        name: &str,
        exclude_id: Option<Uuid>,
    ) -> Result<bool, DbError> {
        let query = if let Some(exclude_id) = exclude_id {
            sqlx::query("SELECT COUNT(*) as count FROM templates WHERE name = ? AND id != ? AND is_deleted = FALSE")
                .bind(name)
                .bind(exclude_id.to_string())
        } else {
            sqlx::query(
                "SELECT COUNT(*) as count FROM templates WHERE name = ? AND is_deleted = FALSE",
            )
            .bind(name)
        };

        let result = query.fetch_one(&self.pool).await;

        match result {
            Ok(row) => {
                let count: i64 = row.try_get("count").map_err(DbError::ConnectionError)?;
                Ok(count == 0)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn get_usage_stats(&self) -> Result<Vec<TemplateUsageStats>, DbError> {
        // 注意：这需要在tasks表中添加template_id字段来跟踪模板使用情况
        // 目前的schema中没有这个字段，所以这里返回空结果
        // 在实际实现中，需要修改数据库schema来支持模板使用统计
        Ok(Vec::new())
    }

    async fn batch_soft_delete(
        &self,
        tx: &mut Transaction<'_>,
        template_ids: &[Uuid],
    ) -> Result<(), DbError> {
        if template_ids.is_empty() {
            return Ok(());
        }

        let ids_str: Vec<String> = template_ids.iter().map(|id| id.to_string()).collect();
        let placeholders = vec!["?"; template_ids.len()].join(",");

        let query = format!(
            "UPDATE templates SET is_deleted = TRUE, updated_at = ? WHERE id IN ({})",
            placeholders
        );

        let mut query_builder = sqlx::query(&query).bind(Utc::now().to_rfc3339());
        for id_str in &ids_str {
            query_builder = query_builder.bind(id_str);
        }

        let result = query_builder.execute(&mut **tx).await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn export_templates(
        &self,
        template_ids: Option<&[Uuid]>,
    ) -> Result<Vec<Template>, DbError> {
        let result = if let Some(ids) = template_ids {
            if ids.is_empty() {
                return Ok(Vec::new());
            }

            let ids_str: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
            let placeholders = vec!["?"; ids.len()].join(",");

            let query = format!(
                "SELECT * FROM templates WHERE id IN ({}) AND is_deleted = FALSE ORDER BY name ASC",
                placeholders
            );

            let mut query_builder = sqlx::query(&query);
            for id_str in &ids_str {
                query_builder = query_builder.bind(id_str);
            }

            query_builder.fetch_all(&self.pool).await
        } else {
            sqlx::query("SELECT * FROM templates WHERE is_deleted = FALSE ORDER BY name ASC")
                .fetch_all(&self.pool)
                .await
        };

        match result {
            Ok(rows) => {
                let templates: Result<Vec<Template>, _> =
                    rows.iter().map(|row| Self::row_to_template(row)).collect();
                templates.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn count_templates(&self) -> Result<TemplateCount, DbError> {
        let result = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total,
                COUNT(CASE WHEN is_deleted = FALSE THEN 1 END) as active,
                COUNT(CASE WHEN is_deleted = TRUE THEN 1 END) as deleted
            FROM templates
            "#,
        )
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(row) => Ok(TemplateCount {
                total: row.try_get("total").map_err(DbError::ConnectionError)?,
                active: row.try_get("active").map_err(DbError::ConnectionError)?,
                deleted: row.try_get("deleted").map_err(DbError::ConnectionError)?,
            }),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }
}
