/// 模板数据访问层

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::shared::{
    core::{AppResult, DbError, Subtask, Template},
    database::{Repository, TemplateRepository},
};

/// 模板仓库的SQLx实现
#[derive(Clone)]
pub struct SqlxTemplateRepository {
    pool: SqlitePool,
}

impl SqlxTemplateRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 将数据库行转换为Template对象
    fn row_to_template(row: &sqlx::sqlite::SqliteRow) -> Result<Template, sqlx::Error> {
        let subtasks_json: Option<String> = row.try_get("subtasks_template")?;
        let subtasks_template = subtasks_json
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
    fn template_to_params(template: &Template) -> (
        String,         // id
        String,         // name
        String,         // title_template
        Option<String>, // glance_note_template
        Option<String>, // detail_note_template
        Option<i32>,    // estimated_duration_template
        Option<String>, // subtasks_template
        Option<String>, // area_id
        String,         // created_at
        String,         // updated_at
        bool,           // is_deleted
    ) {
        (
            template.id.to_string(),
            template.name.clone(),
            template.title_template.clone(),
            template.glance_note_template.clone(),
            template.detail_note_template.clone(),
            template.estimated_duration_template,
            template
                .subtasks_template
                .as_ref()
                .and_then(|s| serde_json::to_string(s).ok()),
            template.area_id.map(|id| id.to_string()),
            template.created_at.to_rfc3339(),
            template.updated_at.to_rfc3339(),
            template.is_deleted,
        )
    }
}

#[async_trait]
impl Repository<Template> for SqlxTemplateRepository {
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

    async fn create(&self, template: &Template) -> AppResult<Template> {
        let params = Self::template_to_params(template);

        sqlx::query(
            r#"
            INSERT INTO templates (
                id, name, title_template, glance_note_template, detail_note_template,
                estimated_duration_template, subtasks_template, area_id,
                created_at, updated_at, is_deleted
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&params.0)  // id
        .bind(&params.1)  // name
        .bind(&params.2)  // title_template
        .bind(&params.3)  // glance_note_template
        .bind(&params.4)  // detail_note_template
        .bind(&params.5)  // estimated_duration_template
        .bind(&params.6)  // subtasks_template
        .bind(&params.7)  // area_id
        .bind(&params.8)  // created_at
        .bind(&params.9)  // updated_at
        .bind(&params.10) // is_deleted
        .execute(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        Ok(template.clone())
    }

    async fn update(&self, template: &Template) -> AppResult<Template> {
        let params = Self::template_to_params(template);

        let result = sqlx::query(
            r#"
            UPDATE templates SET
                name = ?, title_template = ?, glance_note_template = ?,
                detail_note_template = ?, estimated_duration_template = ?,
                subtasks_template = ?, area_id = ?, updated_at = ?
            WHERE id = ? AND is_deleted = FALSE
            "#,
        )
        .bind(&params.1) // name
        .bind(&params.2) // title_template
        .bind(&params.3) // glance_note_template
        .bind(&params.4) // detail_note_template
        .bind(&params.5) // estimated_duration_template
        .bind(&params.6) // subtasks_template
        .bind(&params.7) // area_id
        .bind(&params.9) // updated_at
        .bind(&params.0) // id
        .execute(&self.pool)
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

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        let result = sqlx::query(
            "UPDATE templates SET is_deleted = TRUE, updated_at = ? WHERE id = ? AND is_deleted = FALSE",
        )
        .bind(Utc::now().to_rfc3339())
        .bind(id.to_string())
        .execute(&self.pool)
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

    async fn find_all(&self) -> AppResult<Vec<Template>> {
        let rows = sqlx::query("SELECT * FROM templates WHERE is_deleted = FALSE ORDER BY name ASC")
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        let templates = rows
            .iter()
            .map(Self::row_to_template)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(templates)
    }
}

#[async_trait]
impl TemplateRepository for SqlxTemplateRepository {
    async fn search_by_name(&self, query: &str) -> AppResult<Vec<Template>> {
        let search_pattern = format!("%{}%", query);

        let rows = sqlx::query(
            r#"
            SELECT * FROM templates 
            WHERE is_deleted = FALSE 
            AND (name LIKE ? OR title_template LIKE ?)
            ORDER BY 
                CASE WHEN name LIKE ? THEN 1 ELSE 2 END,
                name ASC
            "#,
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let templates = rows
            .iter()
            .map(Self::row_to_template)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(templates)
    }

    async fn find_by_area_id(&self, area_id: Uuid) -> AppResult<Vec<Template>> {
        let rows = sqlx::query(
            "SELECT * FROM templates WHERE area_id = ? AND is_deleted = FALSE ORDER BY name ASC",
        )
        .bind(area_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let templates = rows
            .iter()
            .map(Self::row_to_template)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(templates)
    }

    async fn find_with_variable(&self, variable_name: &str) -> AppResult<Vec<Template>> {
        let variable_pattern = format!("%{{{{{}}}}}%", variable_name);

        let rows = sqlx::query(
            r#"
            SELECT * FROM templates 
            WHERE is_deleted = FALSE 
            AND (
                title_template LIKE ? OR 
                glance_note_template LIKE ? OR 
                detail_note_template LIKE ?
            )
            ORDER BY name ASC
            "#,
        )
        .bind(&variable_pattern)
        .bind(&variable_pattern)
        .bind(&variable_pattern)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let templates = rows
            .iter()
            .map(Self::row_to_template)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(templates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::database::connection::create_test_database;

    #[tokio::test]
    async fn test_template_crud_operations() {
        let pool = create_test_database().await.unwrap();
        let repo = SqlxTemplateRepository::new(pool);

        // 创建测试模板
        let template = Template::new(
            Uuid::new_v4(),
            "Test Template".to_string(),
            "Task for {{date}}".to_string(),
            Utc::now(),
        );

        // 测试创建
        let created_template = repo.create(&template).await.unwrap();
        assert_eq!(created_template.name, template.name);

        // 测试查找
        let found_template = repo.find_by_id(template.id).await.unwrap().unwrap();
        assert_eq!(found_template.id, template.id);

        // 测试更新
        let mut updated_template = found_template.clone();
        updated_template.name = "Updated Template".to_string();
        updated_template.updated_at = Utc::now();

        let updated = repo.update(&updated_template).await.unwrap();
        assert_eq!(updated.name, "Updated Template");

        // 测试删除
        repo.delete(template.id).await.unwrap();
        let deleted_template = repo.find_by_id(template.id).await.unwrap();
        assert!(deleted_template.is_none());
    }

    #[tokio::test]
    async fn test_template_search() {
        let pool = create_test_database().await.unwrap();
        let repo = SqlxTemplateRepository::new(pool);

        // 创建测试模板
        let template1 = Template::new(
            Uuid::new_v4(),
            "Daily Standup".to_string(),
            "Standup for {{date}}".to_string(),
            Utc::now(),
        );
        let template2 = Template::new(
            Uuid::new_v4(),
            "Weekly Review".to_string(),
            "Review for week {{week}}".to_string(),
            Utc::now(),
        );

        repo.create(&template1).await.unwrap();
        repo.create(&template2).await.unwrap();

        // 测试按名称搜索
        let results = repo.search_by_name("Daily").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Daily Standup");

        // 测试按变量搜索
        let date_templates = repo.find_with_variable("date").await.unwrap();
        assert_eq!(date_templates.len(), 1);
        assert_eq!(date_templates[0].name, "Daily Standup");
    }
}
