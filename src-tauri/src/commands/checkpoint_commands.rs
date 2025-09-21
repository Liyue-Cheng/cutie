use crate::core::{db::DbPool, models::Checkpoint};
use chrono::Utc;
use tauri::State;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct CreateCheckpointPayload {
    pub task_id: Uuid,
    pub title: String,
    pub sort_key: String,
}

#[tauri::command]
pub async fn create_checkpoint(
    pool: State<'_, DbPool>,
    payload: CreateCheckpointPayload,
) -> Result<Checkpoint, String> {
    let checkpoint = Checkpoint {
        id: Uuid::new_v4(),
        task_id: payload.task_id,
        title: payload.title,
        is_completed: false,
        sort_key: payload.sort_key,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        remote_updated_at: None,
    };

    sqlx::query(
        "INSERT INTO checkpoints (id, task_id, title, is_completed, sort_key, created_at, updated_at, deleted_at, remote_updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(&checkpoint.id)
    .bind(&checkpoint.task_id)
    .bind(&checkpoint.title)
    .bind(&checkpoint.is_completed)
    .bind(&checkpoint.sort_key)
    .bind(&checkpoint.created_at)
    .bind(&checkpoint.updated_at)
    .bind(&checkpoint.deleted_at)
    .bind(&checkpoint.remote_updated_at)
    .execute(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(checkpoint)
}

#[tauri::command]
pub async fn get_checkpoint(pool: State<'_, DbPool>, id: Uuid) -> Result<Checkpoint, String> {
    let checkpoint = sqlx::query_as::<_, Checkpoint>(
        "SELECT * FROM checkpoints WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_one(&*pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(checkpoint)
}

#[tauri::command]
pub async fn list_checkpoints_for_task(
    pool: State<'_, DbPool>,
    task_id: Uuid,
) -> Result<Vec<Checkpoint>, String> {
    let checkpoints = sqlx::query_as::<_, Checkpoint>(
        "SELECT * FROM checkpoints WHERE task_id = $1 AND deleted_at IS NULL ORDER BY sort_key ASC",
    )
    .bind(task_id)
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(checkpoints)
}

#[derive(serde::Deserialize)]
pub struct UpdateCheckpointPayload {
    pub title: Option<String>,
    pub is_completed: Option<bool>,
    pub sort_key: Option<String>,
}

#[tauri::command]
pub async fn update_checkpoint(
    pool: State<'_, DbPool>,
    id: Uuid,
    payload: UpdateCheckpointPayload,
) -> Result<Checkpoint, String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let mut checkpoint =
        sqlx::query_as::<_, Checkpoint>("SELECT * FROM checkpoints WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

    if let Some(title) = payload.title {
        checkpoint.title = title;
    }
    if let Some(is_completed) = payload.is_completed {
        checkpoint.is_completed = is_completed;
    }
    if let Some(sort_key) = payload.sort_key {
        checkpoint.sort_key = sort_key;
    }
    checkpoint.updated_at = Utc::now();

    sqlx::query(
        "UPDATE checkpoints SET title = $1, is_completed = $2, sort_key = $3, updated_at = $4 WHERE id = $5",
    )
    .bind(&checkpoint.title)
    .bind(&checkpoint.is_completed)
    .bind(&checkpoint.sort_key)
    .bind(&checkpoint.updated_at)
    .bind(&checkpoint.id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(checkpoint)
}

#[tauri::command]
pub async fn delete_checkpoint(pool: State<'_, DbPool>, id: Uuid) -> Result<(), String> {
    sqlx::query("UPDATE checkpoints SET deleted_at = $1 WHERE id = $2")
        .bind(Utc::now())
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
