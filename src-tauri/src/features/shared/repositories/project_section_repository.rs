/// ProjectSection 通用查询仓库
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{ProjectSection, ProjectSectionRow},
    infra::core::{AppError, AppResult, DbError},
};

pub struct ProjectSectionRepository;

impl ProjectSectionRepository {
    /// 列出项目的所有章节（按 sort_order 排序）
    pub async fn list_by_project(
        executor: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
        project_id: Uuid,
    ) -> AppResult<Vec<ProjectSection>> {
        let query = r#"
            SELECT id, project_id, title, description, sort_order,
                   created_at, updated_at, is_deleted
            FROM project_sections
            WHERE project_id = ? AND is_deleted = false
            ORDER BY sort_order ASC, created_at ASC
        "#;

        let rows = sqlx::query_as::<_, ProjectSectionRow>(query)
            .bind(project_id.to_string())
            .fetch_all(executor)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        let sections: Result<Vec<ProjectSection>, _> =
            rows.into_iter().map(ProjectSection::try_from).collect();

        sections.map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))
    }

    /// 根据 ID 查询章节
    pub async fn find_by_id(
        executor: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
        id: Uuid,
    ) -> AppResult<Option<ProjectSection>> {
        let query = r#"
            SELECT id, project_id, title, description, sort_order,
                   created_at, updated_at, is_deleted
            FROM project_sections
            WHERE id = ? AND is_deleted = false
        "#;

        let row = sqlx::query_as::<_, ProjectSectionRow>(query)
            .bind(id.to_string())
            .fetch_optional(executor)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        match row {
            Some(r) => Ok(Some(
                ProjectSection::try_from(r)
                    .map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))?,
            )),
            None => Ok(None),
        }
    }

    /// 插入章节（在事务中）
    pub async fn insert(
        tx: &mut Transaction<'_, Sqlite>,
        section: &ProjectSection,
    ) -> AppResult<()> {
        let query = r#"
            INSERT INTO project_sections (
                id, project_id, title, description, sort_order,
                created_at, updated_at, is_deleted
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(section.id.to_string())
            .bind(section.project_id.to_string())
            .bind(&section.title)
            .bind(&section.description)
            .bind(&section.sort_order)
            .bind(section.created_at.to_rfc3339())
            .bind(section.updated_at.to_rfc3339())
            .bind(section.is_deleted)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 更新章节（在事务中）
    pub async fn update(
        tx: &mut Transaction<'_, Sqlite>,
        section: &ProjectSection,
    ) -> AppResult<()> {
        let query = r#"
            UPDATE project_sections
            SET title = ?, description = ?, sort_order = ?, updated_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(&section.title)
            .bind(&section.description)
            .bind(&section.sort_order)
            .bind(section.updated_at.to_rfc3339())
            .bind(section.id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 软删除章节（在事务中）
    pub async fn soft_delete(
        tx: &mut Transaction<'_, Sqlite>,
        id: Uuid,
        now: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<()> {
        // 软删除章节
        let query = r#"
            UPDATE project_sections
            SET is_deleted = true, updated_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(now.to_rfc3339())
            .bind(id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        // 清除相关任务的 section_id（任务保留在项目中）
        let query_tasks = r#"
            UPDATE tasks
            SET section_id = NULL, updated_at = ?
            WHERE section_id = ? AND deleted_at IS NULL
        "#;

        sqlx::query(query_tasks)
            .bind(now.to_rfc3339())
            .bind(id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 更新排序（在事务中）
    pub async fn reorder(
        tx: &mut Transaction<'_, Sqlite>,
        section_id: Uuid,
        new_sort_order: String,
        now: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<()> {
        let query = r#"
            UPDATE project_sections
            SET sort_order = ?, updated_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(new_sort_order)
            .bind(now.to_rfc3339())
            .bind(section_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 检查章节标题在项目中是否已存在（在事务中）
    pub async fn check_title_exists_in_project(
        tx: &mut Transaction<'_, Sqlite>,
        project_id: Uuid,
        title: &str,
        exclude_id: Option<Uuid>,
    ) -> AppResult<bool> {
        let query = match exclude_id {
            Some(_) => {
                "SELECT COUNT(*) FROM project_sections WHERE project_id = ? AND title = ? AND id != ? AND is_deleted = false"
            }
            None => {
                "SELECT COUNT(*) FROM project_sections WHERE project_id = ? AND title = ? AND is_deleted = false"
            }
        };

        let count: i64 = match exclude_id {
            Some(id) => {
                sqlx::query_scalar(query)
                    .bind(project_id.to_string())
                    .bind(title)
                    .bind(id.to_string())
                    .fetch_one(&mut **tx)
                    .await
                    .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?
            }
            None => {
                sqlx::query_scalar(query)
                    .bind(project_id.to_string())
                    .bind(title)
                    .fetch_one(&mut **tx)
                    .await
                    .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?
            }
        };

        Ok(count > 0)
    }
}

