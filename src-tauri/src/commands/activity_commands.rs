use crate::core::{db::DbPool, models::Activity};
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use tauri::State;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct CreateActivityPayload {
    pub title: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub timezone: Option<String>,
    pub is_all_day: Option<bool>,
    pub color: Option<String>,
    pub metadata: Option<JsonValue>,
}

#[tauri::command]
pub async fn create_activity(
    pool: State<'_, DbPool>,
    payload: CreateActivityPayload,
) -> Result<Activity, String> {
    let activity = Activity {
        id: Uuid::new_v4(),
        title: payload.title,
        start_time: payload.start_time,
        end_time: payload.end_time,
        timezone: payload.timezone,
        is_all_day: payload.is_all_day.unwrap_or(false),
        color: payload.color,
        metadata: payload.metadata,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
        remote_updated_at: None,
        origin_id: None,
        connector_id: None,
    };

    sqlx::query(
        "INSERT INTO activities (id, title, start_time, end_time, timezone, is_all_day, color, metadata, created_at, updated_at, deleted_at, remote_updated_at, origin_id, connector_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
    )
    .bind(&activity.id)
    .bind(&activity.title)
    .bind(&activity.start_time)
    .bind(&activity.end_time)
    .bind(&activity.timezone)
    .bind(&activity.is_all_day)
    .bind(&activity.color)
    .bind(&activity.metadata)
    .bind(&activity.created_at)
    .bind(&activity.updated_at)
    .bind(&activity.deleted_at)
    .bind(&activity.remote_updated_at)
    .bind(&activity.origin_id)
    .bind(&activity.connector_id)
    .execute(&**pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(activity)
}

#[tauri::command]
pub async fn get_activity(pool: State<'_, DbPool>, id: Uuid) -> Result<Activity, String> {
    let activity = sqlx::query_as::<_, Activity>("SELECT * FROM activities WHERE id = $1 AND deleted_at IS NULL")
        .bind(id)
        .fetch_one(&**pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(activity)
}

#[tauri::command]
pub async fn list_activities(pool: State<'_, DbPool>) -> Result<Vec<Activity>, String> {
    let activities = sqlx::query_as::<_, Activity>("SELECT * FROM activities WHERE deleted_at IS NULL ORDER BY start_time ASC")
        .fetch_all(&**pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(activities)
}

#[derive(serde::Deserialize)]
pub struct UpdateActivityPayload {
    pub title: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub timezone: Option<String>,
    pub is_all_day: Option<bool>,
    pub color: Option<String>,
    pub metadata: Option<JsonValue>,
}

#[tauri::command]
pub async fn update_activity(
    pool: State<'_, DbPool>,
    id: Uuid,
    payload: UpdateActivityPayload,
) -> Result<Activity, String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let mut activity = sqlx::query_as::<_, Activity>("SELECT * FROM activities WHERE id = $1 FOR UPDATE")
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    
    if let Some(title) = payload.title {
        activity.title = Some(title);
    }
    if let Some(start_time) = payload.start_time {
        activity.start_time = start_time;
    }
    if let Some(end_time) = payload.end_time {
        activity.end_time = end_time;
    }
    if let Some(timezone) = payload.timezone {
        activity.timezone = Some(timezone);
    }
    if let Some(is_all_day) = payload.is_all_day {
        activity.is_all_day = is_all_day;
    }
    if let Some(color) = payload.color {
        activity.color = Some(color);
    }
    if let Some(metadata) = payload.metadata {
        activity.metadata = Some(metadata);
    }
    activity.updated_at = Utc::now();

    sqlx::query(
        "UPDATE activities SET title = $1, start_time = $2, end_time = $3, timezone = $4, is_all_day = $5, color = $6, metadata = $7, updated_at = $8 WHERE id = $9",
    )
    .bind(&activity.title)
    .bind(&activity.start_time)
    .bind(&activity.end_time)
    .bind(&activity.timezone)
    .bind(&activity.is_all_day)
    .bind(&activity.color)
    .bind(&activity.metadata)
    .bind(&activity.updated_at)
    .bind(&activity.id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(activity)
}

#[tauri::command]
pub async fn delete_activity(pool: State<'_, DbPool>, id: Uuid) -> Result<(), String> {
    sqlx::query("UPDATE activities SET deleted_at = $1 WHERE id = $2")
        .bind(Utc::now())
        .bind(id)
        .execute(&**pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}