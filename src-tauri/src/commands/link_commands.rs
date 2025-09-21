use crate::core::db::DbPool;
use sqlx::Error as SqlxError;
use tauri::State;
use uuid::Uuid;

// --- Core Logic Functions (Testable) ---

pub async fn link_task_to_activity_core(
    pool: &DbPool,
    task_id: Uuid,
    activity_id: Uuid,
) -> Result<(), SqlxError> {
    sqlx::query("INSERT INTO task_activity_links (id, task_id, activity_id) VALUES ($1, $2, $3)")
        .bind(Uuid::new_v4())
        .bind(task_id)
        .bind(activity_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn unlink_task_from_activity_core(
    pool: &DbPool,
    task_id: Uuid,
    activity_id: Uuid,
) -> Result<(), SqlxError> {
    sqlx::query("DELETE FROM task_activity_links WHERE task_id = $1 AND activity_id = $2")
        .bind(task_id)
        .bind(activity_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_tag_to_project_core(
    pool: &DbPool,
    project_id: Uuid,
    tag_id: Uuid,
) -> Result<(), SqlxError> {
    sqlx::query("INSERT INTO project_tags (project_id, tag_id) VALUES ($1, $2)")
        .bind(project_id)
        .bind(tag_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn remove_tag_from_project_core(
    pool: &DbPool,
    project_id: Uuid,
    tag_id: Uuid,
) -> Result<(), SqlxError> {
    sqlx::query("DELETE FROM project_tags WHERE project_id = $1 AND tag_id = $2")
        .bind(project_id)
        .bind(tag_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_tag_to_task_core(
    pool: &DbPool,
    task_id: Uuid,
    tag_id: Uuid,
) -> Result<(), SqlxError> {
    sqlx::query("INSERT INTO task_tags (task_id, tag_id) VALUES ($1, $2)")
        .bind(task_id)
        .bind(tag_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn remove_tag_from_task_core(
    pool: &DbPool,
    task_id: Uuid,
    tag_id: Uuid,
) -> Result<(), SqlxError> {
    sqlx::query("DELETE FROM task_tags WHERE task_id = $1 AND tag_id = $2")
        .bind(task_id)
        .bind(tag_id)
        .execute(pool)
        .await?;
    Ok(())
}

// --- Tauri Commands (Wrappers) ---

#[tauri::command]
pub async fn link_task_to_activity(
    pool: State<'_, DbPool>,
    task_id: Uuid,
    activity_id: Uuid,
) -> Result<(), String> {
    link_task_to_activity_core(&pool, task_id, activity_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn unlink_task_from_activity(
    pool: State<'_, DbPool>,
    task_id: Uuid,
    activity_id: Uuid,
) -> Result<(), String> {
    unlink_task_from_activity_core(&pool, task_id, activity_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_tag_to_project(
    pool: State<'_, DbPool>,
    project_id: Uuid,
    tag_id: Uuid,
) -> Result<(), String> {
    add_tag_to_project_core(&pool, project_id, tag_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_tag_from_project(
    pool: State<'_, DbPool>,
    project_id: Uuid,
    tag_id: Uuid,
) -> Result<(), String> {
    remove_tag_from_project_core(&pool, project_id, tag_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_tag_to_task(
    pool: State<'_, DbPool>,
    task_id: Uuid,
    tag_id: Uuid,
) -> Result<(), String> {
    add_tag_to_task_core(&pool, task_id, tag_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_tag_from_task(
    pool: State<'_, DbPool>,
    task_id: Uuid,
    tag_id: Uuid,
) -> Result<(), String> {
    remove_tag_from_task_core(&pool, task_id, tag_id)
        .await
        .map_err(|e| e.to_string())
}
