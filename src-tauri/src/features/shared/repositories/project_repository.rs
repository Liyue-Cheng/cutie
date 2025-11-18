/// Project 通用查询仓库
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{Project, ProjectRow},
    infra::core::{AppError, AppResult, DbError},
};

pub struct ProjectRepository;

impl ProjectRepository {
    /// 列出所有未删除的项目
    pub async fn list_all(pool: &sqlx::SqlitePool) -> AppResult<Vec<Project>> {
        let query = r#"
            SELECT id, name, description, status, due_date, completed_at,
                   area_id, created_at, updated_at, is_deleted
            FROM projects
            WHERE is_deleted = false
            ORDER BY updated_at DESC
        "#;

        let rows = sqlx::query_as::<_, ProjectRow>(query)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        let projects: Result<Vec<Project>, _> = rows.into_iter().map(Project::try_from).collect();

        projects.map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))
    }

    /// 根据 ID 查询项目
    pub async fn find_by_id(
        executor: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
        id: Uuid,
    ) -> AppResult<Option<Project>> {
        let query = r#"
            SELECT id, name, description, status, due_date, completed_at,
                   area_id, created_at, updated_at, is_deleted
            FROM projects
            WHERE id = ? AND is_deleted = false
        "#;

        let row = sqlx::query_as::<_, ProjectRow>(query)
            .bind(id.to_string())
            .fetch_optional(executor)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        match row {
            Some(r) => {
                Ok(Some(Project::try_from(r).map_err(|e| {
                    AppError::DatabaseError(DbError::QueryError(e))
                })?))
            }
            None => Ok(None),
        }
    }

    /// 根据 area_id 查询项目
    pub async fn find_by_area(pool: &sqlx::SqlitePool, area_id: Uuid) -> AppResult<Vec<Project>> {
        let query = r#"
            SELECT id, name, description, status, due_date, completed_at,
                   area_id, created_at, updated_at, is_deleted
            FROM projects
            WHERE area_id = ? AND is_deleted = false
            ORDER BY updated_at DESC
        "#;

        let rows = sqlx::query_as::<_, ProjectRow>(query)
            .bind(area_id.to_string())
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        let projects: Result<Vec<Project>, _> = rows.into_iter().map(Project::try_from).collect();

        projects.map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))
    }

    /// 插入项目（在事务中）
    pub async fn insert(tx: &mut Transaction<'_, Sqlite>, project: &Project) -> AppResult<()> {
        let query = r#"
            INSERT INTO projects (
                id, name, description, status, due_date, completed_at,
                area_id, created_at, updated_at, is_deleted
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(project.id.to_string())
            .bind(&project.name)
            .bind(&project.description)
            .bind(project.status.to_str())
            .bind(project.due_date.as_ref().map(|d| d.to_string()))
            .bind(project.completed_at.as_ref().map(|t| t.to_rfc3339()))
            .bind(project.area_id.map(|id| id.to_string()))
            .bind(project.created_at.to_rfc3339())
            .bind(project.updated_at.to_rfc3339())
            .bind(project.is_deleted)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 更新项目（在事务中）
    pub async fn update(tx: &mut Transaction<'_, Sqlite>, project: &Project) -> AppResult<()> {
        let query = r#"
            UPDATE projects
            SET name = ?, description = ?, status = ?, due_date = ?,
                completed_at = ?, area_id = ?, updated_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(&project.name)
            .bind(&project.description)
            .bind(project.status.to_str())
            .bind(project.due_date.as_ref().map(|d| d.to_string()))
            .bind(project.completed_at.as_ref().map(|t| t.to_rfc3339()))
            .bind(project.area_id.map(|id| id.to_string()))
            .bind(project.updated_at.to_rfc3339())
            .bind(project.id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 软删除项目及其关联数据（在事务中）
    pub async fn soft_delete(
        tx: &mut Transaction<'_, Sqlite>,
        id: Uuid,
        now: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<()> {
        // 软删除项目
        let query_project = r#"
            UPDATE projects
            SET is_deleted = true, updated_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query_project)
            .bind(now.to_rfc3339())
            .bind(id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        // 软删除所有关联的 sections
        let query_sections = r#"
            UPDATE project_sections
            SET is_deleted = true, updated_at = ?
            WHERE project_id = ?
        "#;

        sqlx::query(query_sections)
            .bind(now.to_rfc3339())
            .bind(id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        // 软删除所有关联的 tasks
        let query_tasks = r#"
            UPDATE tasks
            SET deleted_at = ?, updated_at = ?
            WHERE project_id = ? AND deleted_at IS NULL
        "#;

        sqlx::query(query_tasks)
            .bind(now.to_rfc3339())
            .bind(now.to_rfc3339())
            .bind(id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 检查项目名称是否已存在（在事务中）
    pub async fn check_name_exists(
        tx: &mut Transaction<'_, Sqlite>,
        name: &str,
        exclude_id: Option<Uuid>,
    ) -> AppResult<bool> {
        let query = match exclude_id {
            Some(_) => {
                "SELECT COUNT(*) FROM projects WHERE name = ? AND id != ? AND is_deleted = false"
            }
            None => "SELECT COUNT(*) FROM projects WHERE name = ? AND is_deleted = false",
        };

        let count: i64 = match exclude_id {
            Some(id) => sqlx::query_scalar(query)
                .bind(name)
                .bind(id.to_string())
                .fetch_one(&mut **tx)
                .await
                .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?,
            None => sqlx::query_scalar(query)
                .bind(name)
                .fetch_one(&mut **tx)
                .await
                .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?,
        };

        Ok(count > 0)
    }
}
