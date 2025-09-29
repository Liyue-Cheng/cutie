/// 任务数据访问层
///
/// 实现任务的数据库操作
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::shared::{
    core::{AppResult, DbError, DueDateType, SourceInfo, Subtask, Task},
    database::{Repository, TaskRepository, TaskStats},
};

/// 任务仓库的SQLx实现
#[derive(Clone)]
pub struct SqlxTaskRepository {
    pool: SqlitePool,
}

impl SqlxTaskRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 将数据库行转换为Task对象
    fn row_to_task(row: &sqlx::sqlite::SqliteRow) -> Result<Task, sqlx::Error> {
        let subtasks_json: Option<String> = row.try_get("subtasks")?;
        let subtasks =
            subtasks_json.and_then(|json| serde_json::from_str::<Vec<Subtask>>(&json).ok());

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

        let due_date_type_str: Option<String> = row.try_get("due_date_type")?;
        let due_date_type = due_date_type_str.and_then(|s| match s.as_str() {
            "SOFT" => Some(DueDateType::Soft),
            "HARD" => Some(DueDateType::Hard),
            _ => None,
        });

        Ok(Task {
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
            estimated_duration: row.try_get("estimated_duration")?,
            subtasks,
            project_id: row
                .try_get::<Option<String>, _>("project_id")?
                .and_then(|s| Uuid::parse_str(&s).ok()),
            area_id: row
                .try_get::<Option<String>, _>("area_id")?
                .and_then(|s| Uuid::parse_str(&s).ok()),
            due_date: row
                .try_get::<Option<String>, _>("due_date")?
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            due_date_type,
            completed_at: row
                .try_get::<Option<String>, _>("completed_at")?
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
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

    /// 将Task对象转换为数据库参数
    fn task_to_params(
        task: &Task,
    ) -> (
        String,         // id
        String,         // title
        Option<String>, // glance_note
        Option<String>, // detail_note
        Option<i32>,    // estimated_duration
        Option<String>, // subtasks
        Option<String>, // project_id
        Option<String>, // area_id
        Option<String>, // due_date
        Option<String>, // due_date_type
        Option<String>, // completed_at
        String,         // created_at
        String,         // updated_at
        bool,           // is_deleted
        Option<String>, // source_info
        Option<String>, // external_source_id
        Option<String>, // external_source_provider
        Option<String>, // external_source_metadata
        Option<String>, // recurrence_rule
        Option<String>, // recurrence_parent_id
        Option<String>, // recurrence_original_date
        Option<String>, // recurrence_exclusions
    ) {
        (
            task.id.to_string(),
            task.title.clone(),
            task.glance_note.clone(),
            task.detail_note.clone(),
            task.estimated_duration,
            task.subtasks
                .as_ref()
                .and_then(|s| serde_json::to_string(s).ok()),
            task.project_id.map(|id| id.to_string()),
            task.area_id.map(|id| id.to_string()),
            task.due_date.map(|dt| dt.to_rfc3339()),
            task.due_date_type.as_ref().map(|t| match t {
                DueDateType::Soft => "SOFT".to_string(),
                DueDateType::Hard => "HARD".to_string(),
            }),
            task.completed_at.map(|dt| dt.to_rfc3339()),
            task.created_at.to_rfc3339(),
            task.updated_at.to_rfc3339(),
            task.is_deleted,
            task.source_info
                .as_ref()
                .and_then(|s| serde_json::to_string(s).ok()),
            task.external_source_id.clone(),
            task.external_source_provider.clone(),
            task.external_source_metadata
                .as_ref()
                .and_then(|s| serde_json::to_string(s).ok()),
            task.recurrence_rule.clone(),
            task.recurrence_parent_id.map(|id| id.to_string()),
            task.recurrence_original_date.map(|dt| dt.to_rfc3339()),
            task.recurrence_exclusions
                .as_ref()
                .and_then(|s| serde_json::to_string(s).ok()),
        )
    }
}

#[async_trait]
impl Repository<Task> for SqlxTaskRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Task>> {
        let row = sqlx::query(
            r#"
            SELECT * FROM tasks 
            WHERE id = ? AND is_deleted = FALSE
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        match row {
            Some(row) => {
                let task = Self::row_to_task(&row).map_err(|e| DbError::ConnectionError(e))?;
                Ok(Some(task))
            }
            None => Ok(None),
        }
    }

    async fn create(&self, task: &Task) -> AppResult<Task> {
        let params = Self::task_to_params(task);

        sqlx::query(
            r#"
            INSERT INTO tasks (
                id, title, glance_note, detail_note, estimated_duration, subtasks,
                project_id, area_id, due_date, due_date_type, completed_at,
                created_at, updated_at, is_deleted, source_info, external_source_id,
                external_source_provider, external_source_metadata, recurrence_rule,
                recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&params.0) // id
        .bind(&params.1) // title
        .bind(&params.2) // glance_note
        .bind(&params.3) // detail_note
        .bind(&params.4) // estimated_duration
        .bind(&params.5) // subtasks
        .bind(&params.6) // project_id
        .bind(&params.7) // area_id
        .bind(&params.8) // due_date
        .bind(&params.9) // due_date_type
        .bind(&params.10) // completed_at
        .bind(&params.11) // created_at
        .bind(&params.12) // updated_at
        .bind(&params.13) // is_deleted
        .bind(&params.14) // source_info
        .bind(&params.15) // external_source_id
        .bind(&params.16) // external_source_provider
        .bind(&params.17) // external_source_metadata
        .bind(&params.18) // recurrence_rule
        .bind(&params.19) // recurrence_parent_id
        .bind(&params.20) // recurrence_original_date
        .bind(&params.21) // recurrence_exclusions
        .execute(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        Ok(task.clone())
    }

    async fn update(&self, task: &Task) -> AppResult<Task> {
        let params = Self::task_to_params(task);

        let result = sqlx::query(
            r#"
            UPDATE tasks SET
                title = ?, glance_note = ?, detail_note = ?, estimated_duration = ?,
                subtasks = ?, project_id = ?, area_id = ?, due_date = ?,
                due_date_type = ?, completed_at = ?, updated_at = ?,
                source_info = ?, external_source_id = ?, external_source_provider = ?,
                external_source_metadata = ?, recurrence_rule = ?, recurrence_parent_id = ?,
                recurrence_original_date = ?, recurrence_exclusions = ?
            WHERE id = ? AND is_deleted = FALSE
            "#,
        )
        .bind(&params.1) // title
        .bind(&params.2) // glance_note
        .bind(&params.3) // detail_note
        .bind(&params.4) // estimated_duration
        .bind(&params.5) // subtasks
        .bind(&params.6) // project_id
        .bind(&params.7) // area_id
        .bind(&params.8) // due_date
        .bind(&params.9) // due_date_type
        .bind(&params.10) // completed_at
        .bind(&params.12) // updated_at
        .bind(&params.14) // source_info
        .bind(&params.15) // external_source_id
        .bind(&params.16) // external_source_provider
        .bind(&params.17) // external_source_metadata
        .bind(&params.18) // recurrence_rule
        .bind(&params.19) // recurrence_parent_id
        .bind(&params.20) // recurrence_original_date
        .bind(&params.21) // recurrence_exclusions
        .bind(&params.0) // id
        .execute(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::shared::core::AppError::not_found(
                "Task",
                task.id.to_string(),
            ));
        }

        Ok(task.clone())
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        let result = sqlx::query(
            "UPDATE tasks SET is_deleted = TRUE, updated_at = ? WHERE id = ? AND is_deleted = FALSE"
        )
        .bind(Utc::now().to_rfc3339())
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::shared::core::AppError::not_found(
                "Task",
                id.to_string(),
            ));
        }

        Ok(())
    }

    async fn find_all(&self) -> AppResult<Vec<Task>> {
        let rows =
            sqlx::query("SELECT * FROM tasks WHERE is_deleted = FALSE ORDER BY created_at DESC")
                .fetch_all(&self.pool)
                .await
                .map_err(DbError::ConnectionError)?;

        let tasks = rows
            .iter()
            .map(Self::row_to_task)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(tasks)
    }
}

#[async_trait]
impl TaskRepository for SqlxTaskRepository {
    async fn search(&self, query: &str, limit: Option<usize>) -> AppResult<Vec<Task>> {
        let limit_clause = limit.map_or("".to_string(), |l| format!("LIMIT {}", l));

        let sql = format!(
            r#"
            SELECT * FROM tasks 
            WHERE is_deleted = FALSE 
            AND (title LIKE ? OR glance_note LIKE ? OR detail_note LIKE ?)
            ORDER BY 
                CASE WHEN title LIKE ? THEN 1 ELSE 2 END,
                created_at DESC
            {}
            "#,
            limit_clause
        );

        let search_pattern = format!("%{}%", query);
        let title_pattern = format!("%{}%", query);

        let rows = sqlx::query(&sql)
            .bind(&search_pattern)
            .bind(&search_pattern)
            .bind(&search_pattern)
            .bind(&title_pattern)
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        let tasks = rows
            .iter()
            .map(Self::row_to_task)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(tasks)
    }

    async fn find_unscheduled(&self) -> AppResult<Vec<Task>> {
        let rows = sqlx::query(
            r#"
            SELECT t.* FROM tasks t
            LEFT JOIN task_schedules ts ON t.id = ts.task_id
            WHERE t.is_deleted = FALSE 
            AND t.completed_at IS NULL
            AND ts.id IS NULL
            ORDER BY t.created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let tasks = rows
            .iter()
            .map(Self::row_to_task)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(tasks)
    }

    async fn find_by_project_id(&self, project_id: Uuid) -> AppResult<Vec<Task>> {
        let rows = sqlx::query(
            "SELECT * FROM tasks WHERE project_id = ? AND is_deleted = FALSE ORDER BY created_at DESC"
        )
        .bind(project_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let tasks = rows
            .iter()
            .map(Self::row_to_task)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(tasks)
    }

    async fn find_by_area_id(&self, area_id: Uuid) -> AppResult<Vec<Task>> {
        let rows = sqlx::query(
            "SELECT * FROM tasks WHERE area_id = ? AND is_deleted = FALSE ORDER BY created_at DESC",
        )
        .bind(area_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let tasks = rows
            .iter()
            .map(Self::row_to_task)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(tasks)
    }

    async fn find_completed(&self) -> AppResult<Vec<Task>> {
        let rows = sqlx::query(
            "SELECT * FROM tasks WHERE completed_at IS NOT NULL AND is_deleted = FALSE ORDER BY completed_at DESC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let tasks = rows
            .iter()
            .map(Self::row_to_task)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(tasks)
    }

    async fn get_stats(&self) -> AppResult<TaskStats> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_count,
                COUNT(CASE WHEN completed_at IS NOT NULL THEN 1 END) as completed_count,
                COUNT(CASE WHEN completed_at IS NULL THEN 1 END) as pending_count,
                COUNT(CASE WHEN due_date < datetime('now') AND completed_at IS NULL THEN 1 END) as overdue_count
            FROM tasks 
            WHERE is_deleted = FALSE
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        Ok(TaskStats {
            total_count: row.try_get("total_count").unwrap_or(0),
            completed_count: row.try_get("completed_count").unwrap_or(0),
            pending_count: row.try_get("pending_count").unwrap_or(0),
            overdue_count: row.try_get("overdue_count").unwrap_or(0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::database::connection::create_test_database;

    #[tokio::test]
    async fn test_task_crud_operations() {
        let pool = create_test_database().await.unwrap();
        let repo = SqlxTaskRepository::new(pool);

        // 创建测试任务
        let task = Task::new(Uuid::new_v4(), "Test Task".to_string(), Utc::now());

        // 测试创建
        let created_task = repo.create(&task).await.unwrap();
        assert_eq!(created_task.title, task.title);

        // 测试查找
        let found_task = repo.find_by_id(task.id).await.unwrap().unwrap();
        assert_eq!(found_task.id, task.id);

        // 测试更新
        let mut updated_task = found_task.clone();
        updated_task.title = "Updated Task".to_string();
        updated_task.updated_at = Utc::now();

        let updated = repo.update(&updated_task).await.unwrap();
        assert_eq!(updated.title, "Updated Task");

        // 测试删除
        repo.delete(task.id).await.unwrap();
        let deleted_task = repo.find_by_id(task.id).await.unwrap();
        assert!(deleted_task.is_none());
    }

    #[tokio::test]
    async fn test_task_search() {
        let pool = create_test_database().await.unwrap();
        let repo = SqlxTaskRepository::new(pool);

        // 创建测试任务
        let task1 = Task::new(Uuid::new_v4(), "Search Test Task".to_string(), Utc::now());
        let task2 = Task::new(Uuid::new_v4(), "Another Task".to_string(), Utc::now());

        repo.create(&task1).await.unwrap();
        repo.create(&task2).await.unwrap();

        // 测试搜索
        let results = repo.search("Search", Some(10)).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Search Test Task");
    }
}
