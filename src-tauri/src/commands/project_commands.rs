use crate::core::{db::DbPool, models::Project};
use chrono::Utc;
use serde_json::Value as JsonValue;
use tauri::State;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct CreateProjectPayload {
  pub title: String,
  pub description: Option<String>,
  pub icon: Option<String>,
  pub color: Option<String>,
  pub status: Option<String>,
  pub metadata: Option<JsonValue>,
}

#[tauri::command]
pub async fn create_project(
  pool: State<'_, DbPool>,
  payload: CreateProjectPayload,
) -> Result<Project, String> {
  let project = Project {
    id: Uuid::new_v4(),
    title: payload.title,
    description: payload.description,
    icon: payload.icon,
    color: payload.color,
    status: payload.status.unwrap_or_else(|| "active".to_string()),
    metadata: payload.metadata,
    created_at: Utc::now(),
    updated_at: Utc::now(),
    deleted_at: None,
    remote_updated_at: None,
  };

  sqlx::query(
    "INSERT INTO projects (id, title, description, icon, color, status, metadata, created_at, updated_at, deleted_at, remote_updated_at)
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
  )
  .bind(&project.id)
  .bind(&project.title)
  .bind(&project.description)
  .bind(&project.icon)
  .bind(&project.color)
  .bind(&project.status)
  .bind(&project.metadata)
  .bind(&project.created_at)
  .bind(&project.updated_at)
  .bind(&project.deleted_at)
  .bind(&project.remote_updated_at)
  .execute(&**pool)
  .await
  .map_err(|e| e.to_string())?;

  Ok(project)
}

#[tauri::command]
pub async fn get_project(pool: State<'_, DbPool>, id: Uuid) -> Result<Project, String> {
  let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1 AND deleted_at IS NULL")
    .bind(id)
    .fetch_one(&**pool)
    .await
    .map_err(|e| e.to_string())?;
  Ok(project)
}

#[tauri::command]
pub async fn list_projects(pool: State<'_, DbPool>) -> Result<Vec<Project>, String> {
  let projects = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE deleted_at IS NULL ORDER BY created_at DESC")
    .fetch_all(&**pool)
    .await
    .map_err(|e| e.to_string())?;
  Ok(projects)
}

#[derive(serde::Deserialize)]
pub struct UpdateProjectPayload {
  pub title: Option<String>,
  pub description: Option<String>,
  pub icon: Option<String>,
  pub color: Option<String>,
  pub status: Option<String>,
  pub metadata: Option<JsonValue>,
}

#[tauri::command]
pub async fn update_project(
  pool: State<'_, DbPool>,
  id: Uuid,
  payload: UpdateProjectPayload,
) -> Result<Project, String> {
  let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

  let mut project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1 FOR UPDATE")
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

  if let Some(title) = payload.title {
    project.title = title;
  }
  if let Some(description) = payload.description {
    project.description = Some(description);
  }
  if let Some(icon) = payload.icon {
    project.icon = Some(icon);
  }
  if let Some(color) = payload.color {
    project.color = Some(color);
  }
  if let Some(status) = payload.status {
    project.status = status;
  }
  if let Some(metadata) = payload.metadata {
    project.metadata = Some(metadata);
  }
  project.updated_at = Utc::now();

  sqlx::query(
    "UPDATE projects SET title = $1, description = $2, icon = $3, color = $4, status = $5, metadata = $6, updated_at = $7 WHERE id = $8",
  )
  .bind(&project.title)
  .bind(&project.description)
  .bind(&project.icon)
  .bind(&project.color)
  .bind(&project.status)
  .bind(&project.metadata)
  .bind(&project.updated_at)
  .bind(&project.id)
  .execute(&mut *tx)
  .await
  .map_err(|e| e.to_string())?;

  tx.commit().await.map_err(|e| e.to_string())?;

  Ok(project)
}

#[tauri::command]
pub async fn delete_project(pool: State<'_, DbPool>, id: Uuid) -> Result<(), String> {
    sqlx::query("UPDATE projects SET deleted_at = $1 WHERE id = $2")
        .bind(Utc::now())
        .bind(id)
        .execute(&**pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}