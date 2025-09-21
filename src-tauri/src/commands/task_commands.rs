use crate::core::{db::DbPool, models::Task};
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::Error as SqlxError;
use tauri::State;
use uuid::Uuid;

// --- Payloads ---

#[derive(serde::Deserialize, Default)]
pub struct CreateTaskPayload {
    pub project_id: Option<Uuid>,
    pub title: String,
    pub status: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub sort_key: String,
    pub metadata: Option<JsonValue>,
}

#[derive(serde::Deserialize, Default)]
pub struct UpdateTaskPayload {
    pub project_id: Option<Uuid>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub sort_key: Option<String>,
    pub metadata: Option<JsonValue>,
}

// --- Core Logic Functions (Testable) ---

pub async fn create_task_core(
    pool: &DbPool,
    payload: CreateTaskPayload,
) -> Result<Task, SqlxError> {
    let task = Task {
        id: Uuid::new_v4(),
        project_id: payload.project_id,
        title: payload.title,
        status: payload.status.unwrap_or_else(|| "todo".to_string()),
        due_date: payload.due_date,
        completed_at: None,
        sort_key: payload.sort_key,
        metadata: payload.metadata,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        remote_updated_at: None,
    };
    sqlx::query("INSERT INTO tasks (id, project_id, title, status, due_date, completed_at, sort_key, metadata, created_at, updated_at, deleted_at, remote_updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)")
        .bind(task.id).bind(task.project_id).bind(&task.title).bind(&task.status).bind(task.due_date).bind(task.completed_at).bind(&task.sort_key).bind(&task.metadata).bind(task.created_at).bind(task.updated_at).bind(task.deleted_at).bind(task.remote_updated_at)
        .execute(pool).await?;
    Ok(task)
}

pub async fn get_task_core(pool: &DbPool, id: Uuid) -> Result<Task, SqlxError> {
    sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1 AND deleted_at IS NULL")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn list_tasks_core(pool: &DbPool) -> Result<Vec<Task>, SqlxError> {
    sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE deleted_at IS NULL ORDER BY sort_key ASC")
        .fetch_all(pool)
        .await
}

pub async fn list_inbox_tasks_core(pool: &DbPool) -> Result<Vec<Task>, SqlxError> {
    sqlx::query_as::<_, Task>(
        "SELECT t.* FROM tasks t LEFT JOIN task_activity_links tal ON t.id = tal.task_id WHERE t.project_id IS NULL AND tal.task_id IS NULL AND t.deleted_at IS NULL ORDER BY t.sort_key ASC"
    )
    .fetch_all(pool)
    .await
}

pub async fn update_task_core(
    pool: &DbPool,
    id: Uuid,
    payload: UpdateTaskPayload,
) -> Result<Task, SqlxError> {
    let mut tx = pool.begin().await?;
    let mut task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

    if let Some(project_id) = payload.project_id {
        task.project_id = Some(project_id);
    }
    if let Some(title) = payload.title {
        task.title = title;
    }
    if let Some(status) = payload.status {
        task.status = status;
    }
    if let Some(due_date) = payload.due_date {
        task.due_date = Some(due_date);
    }
    if let Some(completed_at) = payload.completed_at {
        task.completed_at = Some(completed_at);
    }
    if let Some(sort_key) = payload.sort_key {
        task.sort_key = sort_key;
    }
    if let Some(metadata) = payload.metadata {
        task.metadata = Some(metadata);
    }
    task.updated_at = Utc::now();

    sqlx::query("UPDATE tasks SET project_id = $1, title = $2, status = $3, due_date = $4, completed_at = $5, sort_key = $6, metadata = $7, updated_at = $8 WHERE id = $9")
        .bind(task.project_id).bind(&task.title).bind(&task.status).bind(task.due_date).bind(task.completed_at).bind(&task.sort_key).bind(&task.metadata).bind(task.updated_at).bind(task.id)
        .execute(&mut *tx).await?;

    tx.commit().await?;
    Ok(task)
}

pub async fn delete_task_core(pool: &DbPool, id: Uuid) -> Result<(), SqlxError> {
    sqlx::query("UPDATE tasks SET deleted_at = $1 WHERE id = $2")
        .bind(Utc::now())
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// --- Tauri Commands (Wrappers) ---

#[tauri::command]
pub async fn create_task(
    pool: State<'_, DbPool>,
    payload: CreateTaskPayload,
) -> Result<Task, String> {
    create_task_core(&pool, payload)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_task(pool: State<'_, DbPool>, id: Uuid) -> Result<Task, String> {
    get_task_core(&pool, id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_tasks(pool: State<'_, DbPool>) -> Result<Vec<Task>, String> {
    list_tasks_core(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_inbox_tasks(pool: State<'_, DbPool>) -> Result<Vec<Task>, String> {
    list_inbox_tasks_core(&pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_task(
    pool: State<'_, DbPool>,
    id: Uuid,
    payload: UpdateTaskPayload,
) -> Result<Task, String> {
    update_task_core(&pool, id, payload)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_task(pool: State<'_, DbPool>, id: Uuid) -> Result<(), String> {
    delete_task_core(&pool, id).await.map_err(|e| e.to_string())
}
