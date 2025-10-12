/// TaskRepository的SQLite实现
///
/// 提供Task实体的具体数据库操作实现

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::infra::core::{AppResult, DbError};
use crate::entities::{DueDateType, SourceInfo, Subtask, Task};
use crate::repositories::traits::TaskRepository;

/// 任务仓库的SQLite实现
#[derive(Clone)]
pub struct SqliteTaskRepository {
    pool: SqlitePool,
}

impl SqliteTaskRepository {
    /// 创建新的TaskRepository实例
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
}

#[async_trait]
impl TaskRepository for SqliteTaskRepository {
    // --- 写操作 ---
    async fn create(&self, tx: &mut Transaction<'_, Sqlite>, task: &Task) -> AppResult<Task> {
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
        .bind(task.id.to_string())
        .bind(&task.title)
        .bind(&task.glance_note)
        .bind(&task.detail_note)
        .bind(task.estimated_duration)
        .bind(
            task.subtasks
                .as_ref()
                .map(|s| serde_json::to_string(s).unwrap_or_default()),
        )
        .bind(task.project_id.map(|id| id.to_string()))
        .bind(task.area_id.map(|id| id.to_string()))
        .bind(task.due_date.map(|dt| dt.to_rfc3339()))
        .bind(task.due_date_type.as_ref().map(|t| match t {
            DueDateType::Soft => "SOFT",
            DueDateType::Hard => "HARD",
        }))
        .bind(task.completed_at.map(|dt| dt.to_rfc3339()))
        .bind(task.created_at.to_rfc3339())
        .bind(task.updated_at.to_rfc3339())
        .bind(task.is_deleted)
        .bind(
            task.source_info
                .as_ref()
                .map(|s| serde_json::to_string(s).unwrap_or_default()),
        )
        .bind(&task.external_source_id)
        .bind(&task.external_source_provider)
        .bind(
            task.external_source_metadata
                .as_ref()
                .map(|m| serde_json::to_string(m).unwrap_or_default()),
        )
        .bind(&task.recurrence_rule)
        .bind(task.recurrence_parent_id.map(|id| id.to_string()))
        .bind(task.recurrence_original_date.map(|dt| dt.to_rfc3339()))
        .bind(
            task.recurrence_exclusions
                .as_ref()
                .map(|e| serde_json::to_string(e).unwrap_or_default()),
        )
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        // 返回创建的任务
        self.find_by_id_in_tx(tx, task.id).await?.ok_or_else(|| {
            crate::infra::core::AppError::DatabaseError(DbError::NotFound {
                entity_type: "Task".to_string(),
                entity_id: task.id.to_string(),
            })
        })
    }

    async fn update(&self, tx: &mut Transaction<'_, Sqlite>, task: &Task) -> AppResult<Task> {
        let result = sqlx::query(
            r#"
            UPDATE tasks SET 
                title = ?, glance_note = ?, detail_note = ?, estimated_duration = ?, 
                subtasks = ?, project_id = ?, area_id = ?, due_date = ?, due_date_type = ?,
                completed_at = ?, updated_at = ?, source_info = ?, external_source_id = ?,
                external_source_provider = ?, external_source_metadata = ?, recurrence_rule = ?,
                recurrence_parent_id = ?, recurrence_original_date = ?, recurrence_exclusions = ?
            WHERE id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(&task.title)
        .bind(&task.glance_note)
        .bind(&task.detail_note)
        .bind(task.estimated_duration)
        .bind(
            task.subtasks
                .as_ref()
                .map(|s| serde_json::to_string(s).unwrap_or_default()),
        )
        .bind(task.project_id.map(|id| id.to_string()))
        .bind(task.area_id.map(|id| id.to_string()))
        .bind(task.due_date.map(|dt| dt.to_rfc3339()))
        .bind(task.due_date_type.as_ref().map(|t| match t {
            DueDateType::Soft => "SOFT",
            DueDateType::Hard => "HARD",
        }))
        .bind(task.completed_at.map(|dt| dt.to_rfc3339()))
        .bind(task.updated_at.to_rfc3339())
        .bind(
            task.source_info
                .as_ref()
                .map(|s| serde_json::to_string(s).unwrap_or_default()),
        )
        .bind(&task.external_source_id)
        .bind(&task.external_source_provider)
        .bind(
            task.external_source_metadata
                .as_ref()
                .map(|m| serde_json::to_string(m).unwrap_or_default()),
        )
        .bind(&task.recurrence_rule)
        .bind(task.recurrence_parent_id.map(|id| id.to_string()))
        .bind(task.recurrence_original_date.map(|dt| dt.to_rfc3339()))
        .bind(
            task.recurrence_exclusions
                .as_ref()
                .map(|e| serde_json::to_string(e).unwrap_or_default()),
        )
        .bind(task.id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::infra::core::AppError::not_found(
                "Task",
                task.id.to_string(),
            ));
        }

        // 返回更新后的任务
        self.find_by_id_in_tx(tx, task.id).await?.ok_or_else(|| {
            crate::infra::core::AppError::DatabaseError(DbError::NotFound {
                entity_type: "Task".to_string(),
                entity_id: task.id.to_string(),
            })
        })
    }

    async fn set_completed(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        completion_time: DateTime<Utc>,
    ) -> AppResult<Task> {
        let result = sqlx::query(
            r#"
            UPDATE tasks SET 
                completed_at = ?, 
                updated_at = ? 
            WHERE id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(completion_time.to_rfc3339())
        .bind(completion_time.to_rfc3339())
        .bind(task_id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::infra::core::AppError::not_found(
                "Task",
                task_id.to_string(),
            ));
        }

        // 返回更新后的任务
        self.find_by_id_in_tx(tx, task_id).await?.ok_or_else(|| {
            crate::infra::core::AppError::DatabaseError(DbError::NotFound {
                entity_type: "Task".to_string(),
                entity_id: task_id.to_string(),
            })
        })
    }

    async fn reopen(&self, tx: &mut Transaction<'_, Sqlite>, task_id: Uuid) -> AppResult<Task> {
        let now = Utc::now();
        let result = sqlx::query(
            r#"
            UPDATE tasks SET 
                completed_at = NULL, 
                updated_at = ? 
            WHERE id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(now.to_rfc3339())
        .bind(task_id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::infra::core::AppError::not_found(
                "Task",
                task_id.to_string(),
            ));
        }

        // 返回更新后的任务
        self.find_by_id_in_tx(tx, task_id).await?.ok_or_else(|| {
            crate::infra::core::AppError::DatabaseError(DbError::NotFound {
                entity_type: "Task".to_string(),
                entity_id: task_id.to_string(),
            })
        })
    }

    // --- 读操作 ---
    async fn find_by_id(&self, task_id: Uuid) -> AppResult<Option<Task>> {
        let row = sqlx::query("SELECT * FROM tasks WHERE id = ? AND deleted_at IS NULL")
            .bind(task_id.to_string())
            .fetch_optional(&self.pool)
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

    async fn find_by_id_in_tx(&self, tx: &mut Transaction<'_, Sqlite>, task_id: Uuid) -> AppResult<Option<Task>> {
        let row = sqlx::query("SELECT * FROM tasks WHERE id = ? AND deleted_at IS NULL")
            .bind(task_id.to_string())
            .fetch_optional(&mut **tx)
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

    async fn find_many_by_ids(&self, task_ids: &[Uuid]) -> AppResult<Vec<Task>> {
        if task_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = task_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT * FROM tasks WHERE id IN ({}) AND deleted_at IS NULL ORDER BY created_at DESC",
            placeholders
        );

        let mut query_builder = sqlx::query(&query);
        for task_id in task_ids {
            query_builder = query_builder.bind(task_id.to_string());
        }

        let rows = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        let mut tasks = Vec::new();
        for row in rows {
            let task = Self::row_to_task(&row).map_err(DbError::ConnectionError)?;
            tasks.push(task);
        }

        Ok(tasks)
    }

    async fn find_unscheduled(&self) -> AppResult<Vec<Task>> {
        let rows = sqlx::query(
            r#"
            SELECT t.* FROM tasks t
            LEFT JOIN task_schedule ts ON t.id = ts.task_id
            WHERE t.deleted_at IS NULL 
              AND t.completed_at IS NULL
              AND ts.task_id IS NULL
            ORDER BY t.created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let mut tasks = Vec::new();
        for row in rows {
            let task = Self::row_to_task(&row).map_err(DbError::ConnectionError)?;
            tasks.push(task);
        }

        Ok(tasks)
    }

    async fn exists(&self, task_id: Uuid) -> AppResult<bool> {
        let row = sqlx::query("SELECT 1 FROM tasks WHERE id = ? AND deleted_at IS NULL")
            .bind(task_id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        Ok(row.is_some())
    }
}
