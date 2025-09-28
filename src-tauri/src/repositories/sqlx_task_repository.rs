use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use super::{TaskCountByStatus, TaskRepository, Transaction};
use crate::common::error::DbError;
use crate::core::models::{DueDateType, SourceInfo, Subtask, Task};

/// TaskRepository的SQLx实现
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
        String,
        String,
        Option<String>,
        Option<String>,
        Option<i32>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
        String,
        bool,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) {
        let subtasks_json = task
            .subtasks
            .as_ref()
            .and_then(|s| serde_json::to_string(s).ok());
        let source_info_json = task
            .source_info
            .as_ref()
            .and_then(|s| serde_json::to_string(s).ok());
        let external_source_metadata_json = task
            .external_source_metadata
            .as_ref()
            .and_then(|s| serde_json::to_string(s).ok());
        let recurrence_exclusions_json = task
            .recurrence_exclusions
            .as_ref()
            .and_then(|s| serde_json::to_string(s).ok());

        (
            task.id.to_string(),
            task.title.clone(),
            task.glance_note.clone(),
            task.detail_note.clone(),
            task.estimated_duration,
            subtasks_json,
            task.project_id.map(|id| id.to_string()),
            task.area_id.map(|id| id.to_string()),
            task.due_date.map(|dt| dt.to_rfc3339()),
            task.due_date_type.as_ref().map(|dt| match dt {
                DueDateType::Soft => "SOFT".to_string(),
                DueDateType::Hard => "HARD".to_string(),
            }),
            task.completed_at.map(|dt| dt.to_rfc3339()),
            task.created_at.to_rfc3339(),
            task.updated_at.to_rfc3339(),
            task.is_deleted,
            source_info_json,
            task.external_source_id.clone(),
            task.external_source_provider.clone(),
            external_source_metadata_json,
            task.recurrence_rule.clone(),
            task.recurrence_parent_id.map(|id| id.to_string()),
            task.recurrence_original_date.map(|dt| dt.to_rfc3339()),
            recurrence_exclusions_json,
        )
    }
}

#[async_trait]
impl TaskRepository for SqlxTaskRepository {
    async fn begin_transaction(&self) -> Result<Transaction<'_>, DbError> {
        self.pool.begin().await.map_err(DbError::ConnectionError)
    }

    async fn create(&self, tx: &mut Transaction<'_>, task: &Task) -> Result<Task, DbError> {
        let params = Self::task_to_params(task);

        let result = sqlx::query(
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
        .bind(&params.0)
        .bind(&params.1)
        .bind(&params.2)
        .bind(&params.3)
        .bind(&params.4)
        .bind(&params.5)
        .bind(&params.6)
        .bind(&params.7)
        .bind(&params.8)
        .bind(&params.9)
        .bind(&params.10)
        .bind(&params.11)
        .bind(&params.12)
        .bind(&params.13)
        .bind(&params.14)
        .bind(&params.15)
        .bind(&params.16)
        .bind(&params.17)
        .bind(&params.18)
        .bind(&params.19)
        .bind(&params.20)
        .bind(&params.21)
        .execute(&mut **tx)
        .await;

        match result {
            Ok(_) => Ok(task.clone()),
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                Err(DbError::ConstraintViolation {
                    message: format!("Task with id {} already exists", task.id),
                })
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn update(&self, tx: &mut Transaction<'_>, task: &Task) -> Result<Task, DbError> {
        let params = Self::task_to_params(task);

        let result = sqlx::query(
            r#"
            UPDATE tasks SET 
                title = ?, glance_note = ?, detail_note = ?, estimated_duration = ?, subtasks = ?,
                project_id = ?, area_id = ?, due_date = ?, due_date_type = ?, completed_at = ?,
                updated_at = ?, source_info = ?, external_source_id = ?, external_source_provider = ?,
                external_source_metadata = ?, recurrence_rule = ?, recurrence_parent_id = ?,
                recurrence_original_date = ?, recurrence_exclusions = ?
            WHERE id = ? AND is_deleted = FALSE
            "#
        )
        .bind(&params.1).bind(&params.2).bind(&params.3).bind(&params.4).bind(&params.5)
        .bind(&params.6).bind(&params.7).bind(&params.8).bind(&params.9).bind(&params.10)
        .bind(&params.12).bind(&params.14).bind(&params.15).bind(&params.16).bind(&params.17)
        .bind(&params.18).bind(&params.19).bind(&params.20).bind(&params.21)
        .bind(&params.0)
        .execute(&mut **tx)
        .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(DbError::NotFound {
                        entity_type: "Task".to_string(),
                        entity_id: task.id.to_string(),
                    })
                } else {
                    Ok(task.clone())
                }
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn set_completed(
        &self,
        tx: &mut Transaction<'_>,
        task_id: Uuid,
        completion_time: DateTime<Utc>,
    ) -> Result<Task, DbError> {
        let result = sqlx::query(
            "UPDATE tasks SET completed_at = ?, updated_at = ? WHERE id = ? AND is_deleted = FALSE",
        )
        .bind(completion_time.to_rfc3339())
        .bind(Utc::now().to_rfc3339())
        .bind(task_id.to_string())
        .execute(&mut **tx)
        .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(DbError::NotFound {
                        entity_type: "Task".to_string(),
                        entity_id: task_id.to_string(),
                    })
                } else {
                    // 重新查询更新后的任务
                    self.find_by_id(task_id)
                        .await?
                        .ok_or_else(|| DbError::NotFound {
                            entity_type: "Task".to_string(),
                            entity_id: task_id.to_string(),
                        })
                }
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn reopen(&self, tx: &mut Transaction<'_>, task_id: Uuid) -> Result<Task, DbError> {
        let result = sqlx::query(
            "UPDATE tasks SET completed_at = NULL, updated_at = ? WHERE id = ? AND is_deleted = FALSE"
        )
        .bind(Utc::now().to_rfc3339())
        .bind(task_id.to_string())
        .execute(&mut **tx)
        .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(DbError::NotFound {
                        entity_type: "Task".to_string(),
                        entity_id: task_id.to_string(),
                    })
                } else {
                    // 重新查询更新后的任务
                    self.find_by_id(task_id)
                        .await?
                        .ok_or_else(|| DbError::NotFound {
                            entity_type: "Task".to_string(),
                            entity_id: task_id.to_string(),
                        })
                }
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_by_id(&self, task_id: Uuid) -> Result<Option<Task>, DbError> {
        let result = sqlx::query("SELECT * FROM tasks WHERE id = ? AND is_deleted = FALSE")
            .bind(task_id.to_string())
            .fetch_optional(&self.pool)
            .await;

        match result {
            Ok(Some(row)) => Ok(Some(
                Self::row_to_task(&row).map_err(DbError::ConnectionError)?,
            )),
            Ok(None) => Ok(None),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_many_by_ids(&self, task_ids: &[Uuid]) -> Result<Vec<Task>, DbError> {
        if task_ids.is_empty() {
            return Ok(Vec::new());
        }

        let ids_str: Vec<String> = task_ids.iter().map(|id| id.to_string()).collect();
        let placeholders = vec!["?"; task_ids.len()].join(",");

        let query = format!(
            "SELECT * FROM tasks WHERE id IN ({}) AND is_deleted = FALSE ORDER BY updated_at DESC",
            placeholders
        );

        let mut query_builder = sqlx::query(&query);
        for id_str in &ids_str {
            query_builder = query_builder.bind(id_str);
        }

        let result = query_builder.fetch_all(&self.pool).await;

        match result {
            Ok(rows) => {
                let tasks: Result<Vec<Task>, _> =
                    rows.iter().map(|row| Self::row_to_task(row)).collect();
                tasks.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_unscheduled(&self) -> Result<Vec<Task>, DbError> {
        let result = sqlx::query(
            r#"
            SELECT t.* FROM tasks t
            LEFT JOIN task_schedules ts ON t.id = ts.task_id
            WHERE t.is_deleted = FALSE 
            AND ts.task_id IS NULL
            ORDER BY t.updated_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let tasks: Result<Vec<Task>, _> =
                    rows.iter().map(|row| Self::row_to_task(row)).collect();
                tasks.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_by_project_id(&self, project_id: Uuid) -> Result<Vec<Task>, DbError> {
        let result = sqlx::query(
            "SELECT * FROM tasks WHERE project_id = ? AND is_deleted = FALSE ORDER BY updated_at DESC"
        )
        .bind(project_id.to_string())
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let tasks: Result<Vec<Task>, _> =
                    rows.iter().map(|row| Self::row_to_task(row)).collect();
                tasks.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_by_area_id(&self, area_id: Uuid) -> Result<Vec<Task>, DbError> {
        let result = sqlx::query(
            "SELECT * FROM tasks WHERE area_id = ? AND is_deleted = FALSE ORDER BY updated_at DESC",
        )
        .bind(area_id.to_string())
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let tasks: Result<Vec<Task>, _> =
                    rows.iter().map(|row| Self::row_to_task(row)).collect();
                tasks.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_completed(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Task>, DbError> {
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let result = sqlx::query(
            "SELECT * FROM tasks WHERE completed_at IS NOT NULL AND is_deleted = FALSE ORDER BY completed_at DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let tasks: Result<Vec<Task>, _> =
                    rows.iter().map(|row| Self::row_to_task(row)).collect();
                tasks.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn soft_delete(&self, tx: &mut Transaction<'_>, task_id: Uuid) -> Result<(), DbError> {
        let result = sqlx::query("UPDATE tasks SET is_deleted = TRUE, updated_at = ? WHERE id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn restore(&self, tx: &mut Transaction<'_>, task_id: Uuid) -> Result<Task, DbError> {
        let result =
            sqlx::query("UPDATE tasks SET is_deleted = FALSE, updated_at = ? WHERE id = ?")
                .bind(Utc::now().to_rfc3339())
                .bind(task_id.to_string())
                .execute(&mut **tx)
                .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(DbError::NotFound {
                        entity_type: "Task".to_string(),
                        entity_id: task_id.to_string(),
                    })
                } else {
                    // 查询恢复后的任务（包括已删除的）
                    let task_result = sqlx::query("SELECT * FROM tasks WHERE id = ?")
                        .bind(task_id.to_string())
                        .fetch_optional(&mut **tx)
                        .await;

                    match task_result {
                        Ok(Some(row)) => {
                            Ok(Self::row_to_task(&row).map_err(DbError::ConnectionError)?)
                        }
                        Ok(None) => Err(DbError::NotFound {
                            entity_type: "Task".to_string(),
                            entity_id: task_id.to_string(),
                        }),
                        Err(e) => Err(DbError::ConnectionError(e)),
                    }
                }
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn search(&self, query: &str, limit: Option<i64>) -> Result<Vec<Task>, DbError> {
        let limit = limit.unwrap_or(50);
        let search_pattern = format!("%{}%", query);

        let result = sqlx::query(
            r#"
            SELECT * FROM tasks 
            WHERE (title LIKE ? OR glance_note LIKE ? OR detail_note LIKE ?) 
            AND is_deleted = FALSE 
            ORDER BY 
                CASE 
                    WHEN title LIKE ? THEN 1
                    WHEN glance_note LIKE ? THEN 2
                    ELSE 3
                END,
                updated_at DESC
            LIMIT ?
            "#,
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let tasks: Result<Vec<Task>, _> =
                    rows.iter().map(|row| Self::row_to_task(row)).collect();
                tasks.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn count_by_status(&self) -> Result<TaskCountByStatus, DbError> {
        let result = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total,
                COUNT(CASE WHEN completed_at IS NOT NULL THEN 1 END) as completed,
                COUNT(CASE WHEN completed_at IS NULL THEN 1 END) as pending,
                COUNT(CASE WHEN completed_at IS NULL AND EXISTS(SELECT 1 FROM task_schedules ts WHERE ts.task_id = tasks.id) THEN 1 END) as scheduled,
                COUNT(CASE WHEN completed_at IS NULL AND NOT EXISTS(SELECT 1 FROM task_schedules ts WHERE ts.task_id = tasks.id) THEN 1 END) as unscheduled
            FROM tasks 
            WHERE is_deleted = FALSE
            "#
        )
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(row) => Ok(TaskCountByStatus {
                total: row.try_get("total").map_err(DbError::ConnectionError)?,
                completed: row.try_get("completed").map_err(DbError::ConnectionError)?,
                pending: row.try_get("pending").map_err(DbError::ConnectionError)?,
                scheduled: row.try_get("scheduled").map_err(DbError::ConnectionError)?,
                unscheduled: row
                    .try_get("unscheduled")
                    .map_err(DbError::ConnectionError)?,
            }),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }
}
