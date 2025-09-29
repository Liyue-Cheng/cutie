/// 任务数据访问层 - 共享仓库
///
/// 提供所有任务API端点共享的数据库操作
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::shared::{
    core::{AppResult, DbError, DueDateType, SourceInfo, Subtask, Task},
    database::{Repository, TaskRepository, TaskStats},
};

/// 任务仓库的SQLx实现
#[derive(Clone)]
pub struct TaskRepo {
    pub pool: SqlitePool,
}

impl TaskRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 开始数据库事务
    pub async fn begin_transaction(&self) -> AppResult<Transaction<'_, Sqlite>> {
        self.pool
            .begin()
            .await
            .map_err(DbError::ConnectionError)
            .map_err(Into::into)
    }

    /// 将数据库行转换为Task对象
    pub fn row_to_task(row: &sqlx::sqlite::SqliteRow) -> Result<Task, sqlx::Error> {
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

    /// 在事务中设置任务完成状态
    pub async fn set_completed_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        completed_at: DateTime<Utc>,
    ) -> AppResult<Task> {
        let result = sqlx::query(
            r#"
            UPDATE tasks SET 
                completed_at = ?, 
                updated_at = ? 
            WHERE id = ? AND is_deleted = FALSE
            "#,
        )
        .bind(completed_at.to_rfc3339())
        .bind(completed_at.to_rfc3339())
        .bind(task_id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::shared::core::AppError::not_found(
                "Task",
                task_id.to_string(),
            ));
        }

        // 获取更新后的任务
        let row = sqlx::query("SELECT * FROM tasks WHERE id = ?")
            .bind(task_id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(DbError::ConnectionError)?;

        Self::row_to_task(&row).map_err(|e| DbError::ConnectionError(e).into())
    }

    /// 查找任务（不在事务中）
    pub async fn find_by_id(pool: &SqlitePool, task_id: Uuid) -> AppResult<Option<Task>> {
        let row = sqlx::query("SELECT * FROM tasks WHERE id = ? AND is_deleted = FALSE")
            .bind(task_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(DbError::ConnectionError)?;

        match row {
            Some(row) => {
                let task = Self::row_to_task(&row).map_err(DbError::ConnectionError)?;
                Ok(Some(task))
            }
            None => Ok(None),
        }
    }
}

// 保持与原有TaskRepository trait的兼容性
#[async_trait]
impl Repository<Task> for TaskRepo {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Task>> {
        crate::features::tasks::endpoints::get_task::database::find_by_id(&self.pool, id).await
    }

    async fn create(&self, task: &Task) -> AppResult<Task> {
        // 实现创建逻辑...
        todo!("Implement create")
    }

    async fn update(&self, task: &Task) -> AppResult<Task> {
        // 实现更新逻辑...
        todo!("Implement update")
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        // 实现删除逻辑...
        todo!("Implement delete")
    }

    async fn find_all(&self) -> AppResult<Vec<Task>> {
        // 实现查找所有逻辑...
        todo!("Implement find_all")
    }
}

#[async_trait]
impl TaskRepository for TaskRepo {
    async fn search(&self, query: &str, limit: Option<usize>) -> AppResult<Vec<Task>> {
        todo!("Implement search")
    }

    async fn find_unscheduled(&self) -> AppResult<Vec<Task>> {
        todo!("Implement find_unscheduled")
    }

    async fn find_by_project_id(&self, project_id: Uuid) -> AppResult<Vec<Task>> {
        todo!("Implement find_by_project_id")
    }

    async fn find_by_area_id(&self, area_id: Uuid) -> AppResult<Vec<Task>> {
        todo!("Implement find_by_area_id")
    }

    async fn find_completed(&self) -> AppResult<Vec<Task>> {
        todo!("Implement find_completed")
    }

    async fn get_stats(&self) -> AppResult<TaskStats> {
        todo!("Implement get_stats")
    }
}
