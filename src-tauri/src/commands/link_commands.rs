use crate::core::db::DbPool;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn link_task_to_activity(
    pool: State<'_, DbPool>,
    task_id: Uuid,
    activity_id: Uuid,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO task_activity_links (id, task_id, activity_id) VALUES ($1, $2, $3)",
    )
    .bind(Uuid::new_v4())
    .bind(task_id)
    .bind(activity_id)
    .execute(&**pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn unlink_task_from_activity(
    pool: State<'_, DbPool>,
    task_id: Uuid,
    activity_id: Uuid,
) -> Result<(), String> {
    sqlx::query("DELETE FROM task_activity_links WHERE task_id = $1 AND activity_id = $2")
        .bind(task_id)
        .bind(activity_id)
        .execute(&**pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn add_tag_to_project(
    pool: State<'_, DbPool>,
    project_id: Uuid,
    tag_id: Uuid,
) -> Result<(), String> {
    sqlx::query("INSERT INTO project_tags (project_id, tag_id) VALUES ($1, $2)")
        .bind(project_id)
        .bind(tag_id)
        .execute(&**pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn remove_tag_from_project(
    pool: State<'_, DbPool>,
    project_id: Uuid,
    tag_id: Uuid,
) -> Result<(), String> {
    sqlx::query("DELETE FROM project_tags WHERE project_id = $1 AND tag_id = $2")
        .bind(project_id)
        .bind(tag_id)
        .execute(&**pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn add_tag_to_task(
    pool: State<'_, DbPool>,
    task_id: Uuid,
    tag_id: Uuid,
) -> Result<(), String> {
    sqlx::query("INSERT INTO task_tags (task_id, tag_id) VALUES ($1, $2)")
        .bind(task_id)
        .bind(tag_id)
        .execute(&**pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn remove_tag_from_task(
    pool: State<'_, DbPool>,
    task_id: Uuid,
    tag_id: Uuid,
) -> Result<(), String> {
    sqlx::query("DELETE FROM task_tags WHERE task_id = $1 AND tag_id = $2")
        .bind(task_id)
        .bind(tag_id)
        .execute(&**pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}