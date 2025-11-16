/// Task 核心 CRUD 仓库
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::{
    entities::{Task, TaskRow, UpdateTaskRequest},
    infra::core::{AppError, AppResult, DbError},
};

pub struct TaskRepository;

impl TaskRepository {
    /// 在事务中查询任务（完整字段）
    pub async fn find_by_id_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<Option<Task>> {
        let query = r#"
            SELECT id, title, glance_note, detail_note, estimated_duration,
                   subtasks, project_id, area_id, due_date, due_date_type, completed_at, archived_at,
                   created_at, updated_at, deleted_at, source_info,
                   external_source_id, external_source_provider, external_source_metadata,
                   recurrence_id, recurrence_original_date
            FROM tasks
            WHERE id = ? AND deleted_at IS NULL
        "#;

        let row = sqlx::query_as::<_, TaskRow>(query)
            .bind(task_id.to_string())
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        match row {
            Some(r) => {
                let task = Task::try_from(r)
                    .map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))?;
                Ok(Some(task))
            }
            None => Ok(None),
        }
    }

    /// 非事务查询任务
    pub async fn find_by_id(pool: &SqlitePool, task_id: Uuid) -> AppResult<Option<Task>> {
        let query = r#"
            SELECT id, title, glance_note, detail_note, estimated_duration,
                   subtasks, project_id, area_id, due_date, due_date_type, completed_at, archived_at,
                   created_at, updated_at, deleted_at, source_info,
                   external_source_id, external_source_provider, external_source_metadata,
                   recurrence_id, recurrence_original_date
            FROM tasks
            WHERE id = ? AND deleted_at IS NULL
        "#;

        let row = sqlx::query_as::<_, TaskRow>(query)
            .bind(task_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        match row {
            Some(r) => {
                let task = Task::try_from(r)
                    .map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))?;
                Ok(Some(task))
            }
            None => Ok(None),
        }
    }

    /// 插入任务
    pub async fn insert_in_tx(tx: &mut Transaction<'_, Sqlite>, task: &Task) -> AppResult<()> {
        let query = r#"
            INSERT INTO tasks (
                id, title, glance_note, detail_note, estimated_duration, subtasks,
                project_id, area_id, due_date, due_date_type, completed_at, archived_at,
                created_at, updated_at, deleted_at, source_info,
                external_source_id, external_source_provider, external_source_metadata,
                recurrence_id, recurrence_original_date
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(task.id.to_string())
            .bind(&task.title)
            .bind(&task.glance_note)
            .bind(&task.detail_note)
            .bind(task.estimated_duration)
            .bind(
                task.subtasks
                    .as_ref()
                    .map(|s| serde_json::to_string(s).unwrap()),
            )
            .bind(task.project_id.map(|id| id.to_string()))
            .bind(task.area_id.map(|id| id.to_string()))
            .bind(task.due_date.map(|d| d.to_string())) // ✅ NaiveDate 使用 to_string()
            .bind(
                task.due_date_type
                    .as_ref()
                    .map(|t| serde_json::to_string(t).unwrap()),
            )
            .bind(task.completed_at.map(|d| d.to_rfc3339()))
            .bind(task.archived_at.map(|d| d.to_rfc3339()))
            .bind(task.created_at.to_rfc3339())
            .bind(task.updated_at.to_rfc3339())
            .bind(task.deleted_at.map(|d| d.to_rfc3339()))
            .bind(
                task.source_info
                    .as_ref()
                    .map(|s| serde_json::to_string(s).unwrap()),
            )
            .bind(&task.external_source_id)
            .bind(&task.external_source_provider)
            .bind(
                task.external_source_metadata
                    .as_ref()
                    .map(|m| serde_json::to_string(m).unwrap()),
            )
            .bind(task.recurrence_id.map(|id| id.to_string()))
            .bind(&task.recurrence_original_date)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 更新任务（动态字段）
    pub async fn update_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        request: &UpdateTaskRequest,
    ) -> AppResult<()> {
        let now = Utc::now();

        // 收集需要更新的列
        let mut set_clauses: Vec<&str> = Vec::new();
        if request.title.is_some() {
            set_clauses.push("title = ?");
        }
        if request.glance_note.is_some() {
            set_clauses.push("glance_note = ?");
        }
        if request.detail_note.is_some() {
            set_clauses.push("detail_note = ?");
        }
        if request.estimated_duration.is_some() {
            set_clauses.push("estimated_duration = ?");
        }
        if request.subtasks.is_some() {
            set_clauses.push("subtasks = ?");
        }
        if request.area_id.is_some() {
            set_clauses.push("area_id = ?");
        }
        if request.due_date.is_some() {
            set_clauses.push("due_date = ?");
        }
        if request.due_date_type.is_some() {
            set_clauses.push("due_date_type = ?");
        }

        if set_clauses.is_empty() {
            return Ok(());
        }

        // 追加更新时间
        set_clauses.push("updated_at = ?");
        let update_clause = set_clauses.join(", ");
        let query = format!("UPDATE tasks SET {} WHERE id = ?", update_clause);

        let mut q = sqlx::query(&query);

        // 按顺序绑定各字段的值
        if let Some(title) = &request.title {
            q = q.bind(title.clone());
        }
        if let Some(glance_note) = &request.glance_note {
            q = q.bind(glance_note.clone());
        }
        if let Some(detail_note) = &request.detail_note {
            q = q.bind(detail_note.clone());
        }
        if let Some(estimated_duration) = &request.estimated_duration {
            q = q.bind(estimated_duration.clone());
        }
        if let Some(subtasks) = &request.subtasks {
            let value: Option<String> = match subtasks {
                Some(list) => Some(
                    serde_json::to_string(list)
                        .map_err(|e| AppError::DatabaseError(DbError::QueryError(e.to_string())))?,
                ),
                None => None,
            };
            q = q.bind(value);
        }
        if let Some(area_id) = &request.area_id {
            let bind_val: Option<String> = area_id.map(|id| id.to_string());
            q = q.bind(bind_val);
        }
        if let Some(due_date) = &request.due_date {
            let bind_val: Option<String> = due_date.as_ref().map(|d| d.to_string()); // ✅ NaiveDate 使用 to_string()
            q = q.bind(bind_val);
        }
        if let Some(due_date_type) = &request.due_date_type {
            let bind_val: Option<String> = due_date_type.as_ref().map(|t| match t {
                crate::entities::DueDateType::Soft => "SOFT".to_string(),
                crate::entities::DueDateType::Hard => "HARD".to_string(),
            });
            q = q.bind(bind_val);
        }

        // 绑定 updated_at 与 id
        q = q.bind(now.to_rfc3339());
        q = q.bind(task_id.to_string());

        q.execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }

    /// 软删除任务
    pub async fn soft_delete_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        deleted_at: DateTime<Utc>,
    ) -> AppResult<()> {
        let query = "UPDATE tasks SET deleted_at = ?, updated_at = ? WHERE id = ?";
        sqlx::query(query)
            .bind(deleted_at.to_rfc3339())
            .bind(deleted_at.to_rfc3339())
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;
        Ok(())
    }

    /// 恢复任务（从回收站）
    pub async fn restore_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        updated_at: DateTime<Utc>,
    ) -> AppResult<()> {
        let query = "UPDATE tasks SET deleted_at = NULL, updated_at = ? WHERE id = ?";
        sqlx::query(query)
            .bind(updated_at.to_rfc3339())
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;
        Ok(())
    }

    /// 物理删除任务（仅用于回收站清空）
    pub async fn permanently_delete_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<()> {
        let query = "DELETE FROM tasks WHERE id = ? AND deleted_at IS NOT NULL";
        sqlx::query(query)
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;
        Ok(())
    }

    /// 查询回收站中的任务（包含已删除的）
    pub async fn find_deleted_by_id_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<Option<Task>> {
        let query = r#"
            SELECT id, title, glance_note, detail_note, estimated_duration,
                   subtasks, project_id, area_id, due_date, due_date_type, completed_at, archived_at,
                   created_at, updated_at, deleted_at, source_info,
                   external_source_id, external_source_provider, external_source_metadata,
                   recurrence_id, recurrence_original_date
            FROM tasks
            WHERE id = ? AND deleted_at IS NOT NULL
        "#;

        let row = sqlx::query_as::<_, TaskRow>(query)
            .bind(task_id.to_string())
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        match row {
            Some(r) => {
                let task = Task::try_from(r)
                    .map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))?;
                Ok(Some(task))
            }
            None => Ok(None),
        }
    }

    /// 查询回收站任务列表
    pub async fn find_deleted_tasks(
        pool: &SqlitePool,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<Task>> {
        let query = r#"
            SELECT id, title, glance_note, detail_note, estimated_duration,
                   subtasks, project_id, area_id, due_date, due_date_type, completed_at, archived_at,
                   created_at, updated_at, deleted_at, source_info,
                   external_source_id, external_source_provider, external_source_metadata,
                   recurrence_id, recurrence_original_date
            FROM tasks
            WHERE deleted_at IS NOT NULL
            ORDER BY deleted_at DESC
            LIMIT ? OFFSET ?
        "#;

        let rows = sqlx::query_as::<_, TaskRow>(query)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        let tasks: Result<Vec<Task>, String> = rows.into_iter().map(Task::try_from).collect();
        tasks.map_err(|e| AppError::DatabaseError(DbError::QueryError(e)))
    }

    /// 设置任务为已完成
    pub async fn set_completed_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        completed_at: DateTime<Utc>,
    ) -> AppResult<()> {
        let query = "UPDATE tasks SET completed_at = ?, updated_at = ? WHERE id = ?";
        sqlx::query(query)
            .bind(completed_at.to_rfc3339())
            .bind(completed_at.to_rfc3339())
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;
        Ok(())
    }

    /// 设置任务的循环字段（recurrence_id 与 recurrence_original_date）
    pub async fn set_recurrence_fields_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        recurrence_id: Uuid,
        original_date: &str,
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        let query =
            "UPDATE tasks SET recurrence_id = ?, recurrence_original_date = ?, updated_at = ? WHERE id = ?";
        sqlx::query(query)
            .bind(recurrence_id.to_string())
            .bind(original_date)
            .bind(now.to_rfc3339())
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;
        Ok(())
    }

    /// 重新打开任务（设置 completed_at 为 NULL）
    pub async fn set_reopened_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        updated_at: DateTime<Utc>,
    ) -> AppResult<()> {
        let query = "UPDATE tasks SET completed_at = NULL, updated_at = ? WHERE id = ?";
        sqlx::query(query)
            .bind(updated_at.to_rfc3339())
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;
        Ok(())
    }

    /// 更新任务的子任务列表
    pub async fn update_subtasks_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        subtasks: Option<Vec<crate::entities::Subtask>>,
    ) -> AppResult<()> {
        let subtasks_json = subtasks
            .map(|s| serde_json::to_string(&s))
            .transpose()
            .map_err(|e| AppError::StringError(format!("Failed to serialize subtasks: {}", e)))?;

        let query = "UPDATE tasks SET subtasks = ? WHERE id = ?";
        sqlx::query(query)
            .bind(subtasks_json)
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;
        Ok(())
    }
}
