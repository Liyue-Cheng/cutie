/// Update shutdown ritual settings (singleton) - Single File Component endpoint
///
/// PATCH /api/shutdown-ritual/settings
use axum::{
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::{
    entities::{ShutdownRitualSettingsDto, UpdateShutdownRitualSettingsRequest},
    features::shared::TransactionHelper,
    infra::{
        core::{AppError, AppResult},
        events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        },
        http::{
            error_handler::success_response,
            extractors::extract_correlation_id,
        },
    },
    startup::AppState,
};

// ==================== CABC Documentation ====================
/*
1. Endpoint Signature
   PATCH /api/shutdown-ritual/settings

2. High-Level Behavior
   Update shutdown ritual settings (singleton), currently supports customizing the ritual title.

3. Input/Output Specification
   Body: { title?: string | null }
   Response: { title: string | null, updated_at: datetime }

4. Validation Rules
   - title when provided must be non-empty after trim and <= 64 characters

5. Business Logic Walkthrough
   - Validate input
   - Ensure singleton row exists (id='default')
   - UPDATE settings in transaction
   - Read back the settings dto
   - Emit outbox event shutdown_ritual.settings.updated with response payload
   - Commit and return

6. Edge Cases
   - title = null: clear title and fallback to frontend default
   - Missing singleton row: inserted automatically

7. Expected Side Effects
   - INSERT shutdown_ritual_settings (if missing)
   - UPDATE shutdown_ritual_settings
   - INSERT event_outbox

8. Contract
   - Returned payload equals outbox payload
*/

#[derive(Debug, Serialize)]
pub struct UpdateShutdownRitualSettingsResponse {
    #[serde(flatten)]
    pub settings: ShutdownRitualSettingsDto,
}

pub async fn handle(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<UpdateShutdownRitualSettingsRequest>,
) -> Response {
    let correlation_id = extract_correlation_id(&headers);
    match logic::execute(&app_state, request, correlation_id).await {
        Ok(dto) => success_response(UpdateShutdownRitualSettingsResponse { settings: dto }).into_response(),
        Err(err) => err.into_response(),
    }
}

mod validation {
    use super::*;

    pub fn validate_request(req: &UpdateShutdownRitualSettingsRequest) -> AppResult<()> {
        if let Some(title) = &req.title {
            let t = title.trim();
            if t.is_empty() {
                return Err(AppError::validation_error("title", "标题不能为空", "TITLE_EMPTY"));
            }
            if t.chars().count() > 64 {
                return Err(AppError::validation_error(
                    "title",
                    "标题过长（最多 64 个字符）",
                    "TITLE_TOO_LONG",
                ));
            }
        }
        Ok(())
    }
}

mod logic {
    use super::*;
    use sqlx::{Sqlite, Transaction};

    #[derive(Debug, sqlx::FromRow)]
    struct SettingsRow {
        title: Option<String>,
        updated_at: chrono::DateTime<chrono::Utc>,
    }

    async fn ensure_singleton_exists_in_tx(tx: &mut Transaction<'_, Sqlite>, now: chrono::DateTime<chrono::Utc>) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO shutdown_ritual_settings (id, title, created_at, updated_at)
            VALUES ('default', NULL, ?, ?)
            ON CONFLICT(id) DO NOTHING
            "#,
        )
        .bind(now)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?;
        Ok(())
    }

    async fn find_settings_in_tx(tx: &mut Transaction<'_, Sqlite>) -> AppResult<SettingsRow> {
        let row = sqlx::query_as::<_, SettingsRow>(
            r#"
            SELECT title, updated_at
            FROM shutdown_ritual_settings
            WHERE id = 'default'
            "#,
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?;
        Ok(row)
    }

    pub async fn execute(
        app_state: &AppState,
        request: UpdateShutdownRitualSettingsRequest,
        correlation_id: Option<String>,
    ) -> AppResult<ShutdownRitualSettingsDto> {
        validation::validate_request(&request)?;

        let now = app_state.clock().now_utc();

        let _permit = app_state.acquire_write_permit().await;
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        ensure_singleton_exists_in_tx(&mut tx, now).await?;

        let normalized_title: Option<String> = request.title.map(|t| t.trim().to_string());

        sqlx::query(
            r#"
            UPDATE shutdown_ritual_settings
            SET title = ?, updated_at = ?
            WHERE id = 'default'
            "#,
        )
        .bind(normalized_title.clone())
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?;

        let row = find_settings_in_tx(&mut tx).await?;
        let dto = ShutdownRitualSettingsDto {
            title: row.title,
            updated_at: row.updated_at,
        };

        // Outbox event (transactional)
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());
        let mut event = DomainEvent::new(
            "shutdown_ritual.settings.updated",
            "shutdown_ritual_settings",
            "default",
            serde_json::to_value(&dto)?,
        );
        if let Some(cid) = correlation_id {
            event = event.with_correlation_id(cid);
        }
        outbox_repo.append_in_tx(&mut tx, &event).await?;

        TransactionHelper::commit(tx).await?;
        Ok(dto)
    }
}


