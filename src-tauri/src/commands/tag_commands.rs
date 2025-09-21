use crate::core::{db::DbPool, models::Tag};
use chrono::Utc;
use tauri::State;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct CreateTagPayload {
    pub title: String,
    pub color: Option<String>,
    pub sort_key: Option<String>,
}

#[tauri::command]
pub async fn create_tag(pool: State<'_, DbPool>, payload: CreateTagPayload) -> Result<Tag, String> {
    let tag = Tag {
        id: Uuid::new_v4(),
        title: payload.title,
        color: payload.color,
        sort_key: payload.sort_key,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        remote_updated_at: None,
    };

    sqlx::query(
        "INSERT INTO tags (id, title, color, sort_key, created_at, updated_at, deleted_at, remote_updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(&tag.id)
    .bind(&tag.title)
    .bind(&tag.color)
    .bind(&tag.sort_key)
    .bind(&tag.created_at)
    .bind(&tag.updated_at)
    .bind(&tag.deleted_at)
    .bind(&tag.remote_updated_at)
    .execute(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(tag)
}

#[tauri::command]
pub async fn get_tag(pool: State<'_, DbPool>, id: Uuid) -> Result<Tag, String> {
    let tag = sqlx::query_as::<_, Tag>("SELECT * FROM tags WHERE id = $1 AND deleted_at IS NULL")
        .bind(id)
        .fetch_one(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(tag)
}

#[tauri::command]
pub async fn list_tags(pool: State<'_, DbPool>) -> Result<Vec<Tag>, String> {
    let tags = sqlx::query_as::<_, Tag>(
        "SELECT * FROM tags WHERE deleted_at IS NULL ORDER BY sort_key ASC",
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(tags)
}

#[derive(serde::Deserialize)]
pub struct UpdateTagPayload {
    pub title: Option<String>,
    pub color: Option<String>,
    pub sort_key: Option<String>,
}

#[tauri::command]
pub async fn update_tag(
    pool: State<'_, DbPool>,
    id: Uuid,
    payload: UpdateTagPayload,
) -> Result<Tag, String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let mut tag = sqlx::query_as::<_, Tag>("SELECT * FROM tags WHERE id = $1 FOR UPDATE")
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(title) = payload.title {
        tag.title = title;
    }
    if let Some(color) = payload.color {
        tag.color = Some(color);
    }
    if let Some(sort_key) = payload.sort_key {
        tag.sort_key = Some(sort_key);
    }
    tag.updated_at = Utc::now();

    sqlx::query(
        "UPDATE tags SET title = $1, color = $2, sort_key = $3, updated_at = $4 WHERE id = $5",
    )
    .bind(&tag.title)
    .bind(&tag.color)
    .bind(&tag.sort_key)
    .bind(&tag.updated_at)
    .bind(&tag.id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(tag)
}

#[tauri::command]
pub async fn delete_tag(pool: State<'_, DbPool>, id: Uuid) -> Result<(), String> {
    sqlx::query("UPDATE tags SET deleted_at = $1 WHERE id = $2")
        .bind(Utc::now())
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
