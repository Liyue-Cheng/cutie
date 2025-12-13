/// Toggle shutdown ritual progress for a date - Single File Component endpoint
///
/// POST /api/shutdown-ritual/progress/toggle
use axum::{
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    entities::{ShutdownRitualProgressDto, ToggleShutdownRitualRequest},
    features::shared::TransactionHelper,
    infra::{
        core::{AppError, AppResult},
        events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        },
        http::{
            error_handler::success_response,
            extractors::{extract_client_time, extract_correlation_id},
        },
    },
    startup::AppState,
};

// ==================== CABC Documentation ====================
/*
1. Endpoint Signature
   POST /api/shutdown-ritual/progress/toggle

2. High-Level Behavior
   Toggle completion state of a step for a given date. Completed state is daily and resets by date.

3. Input/Output Specification
   Body: { step_id: UUID, date: YYYY-MM-DD }
   Header: X-Client-Time (required)
   Response: { step_id, date, completed_at }

4. Validation Rules
   - date must be valid YYYY-MM-DD
   - step_id must exist
   - X-Client-Time must exist and be a valid ISO datetime with timezone

5. Business Logic Walkthrough
   - Validate date
   - Upsert progress row for (step_id, date)
   - If row exists: toggle completed_at (NULL <-> client_time)
   - If not exists: insert with completed_at = client_time
   - Emit outbox event shutdown_ritual.progress.toggled with response payload

6. Edge Cases
   - Missing row: create it

7. Expected Side Effects
   - INSERT/UPDATE shutdown_ritual_progress
   - INSERT event_outbox

8. Contract
   - Returned payload equals outbox payload
*/

#[derive(Debug, Serialize)]
pub struct ToggleShutdownRitualProgressResponse {
    #[serde(flatten)]
    pub progress: ShutdownRitualProgressDto,
}

pub async fn handle(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ToggleShutdownRitualRequest>,
) -> Response {
    let correlation_id = extract_correlation_id(&headers);
    let client_time = match extract_client_time(&headers) {
        Ok(time) => time,
        Err(err) => return err.into_response(),
    };

    match logic::execute(&app_state, request, client_time, correlation_id).await {
        Ok(dto) => success_response(ToggleShutdownRitualProgressResponse { progress: dto })
            .into_response(),
        Err(err) => err.into_response(),
    }
}

mod validation {
    use super::*;
    use crate::infra::core::utils::time_utils;

    pub fn validate_date(date: &str) -> AppResult<()> {
        time_utils::parse_date_yyyy_mm_dd(date).map(|_| ()).map_err(|_| {
            AppError::validation_error(
                "date",
                "日期格式错误，请使用 YYYY-MM-DD 格式",
                "INVALID_DATE_FORMAT",
            )
        })
    }
}

mod logic {
    use super::*;
    use sqlx::{Sqlite, Transaction};

    #[derive(Debug, sqlx::FromRow)]
    struct ExistingRow {
        id: String,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    async fn ensure_step_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        step_id: Uuid,
    ) -> AppResult<()> {
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(1) FROM shutdown_ritual_steps WHERE id = ?",
        )
        .bind(step_id.to_string())
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?;

        if exists <= 0 {
            return Err(AppError::not_found("ShutdownRitualStep", step_id.to_string()));
        }
        Ok(())
    }

    async fn find_existing_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        step_id: Uuid,
        date: &str,
    ) -> AppResult<Option<ExistingRow>> {
        let row = sqlx::query_as::<_, ExistingRow>(
            r#"
            SELECT id, completed_at
            FROM shutdown_ritual_progress
            WHERE step_id = ? AND date = ?
            "#,
        )
        .bind(step_id.to_string())
        .bind(date)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(row)
    }

    pub async fn execute(
        app_state: &AppState,
        request: ToggleShutdownRitualRequest,
        client_time: chrono::DateTime<chrono::Utc>,
        correlation_id: Option<String>,
    ) -> AppResult<ShutdownRitualProgressDto> {
        validation::validate_date(&request.date)?;

        let now = app_state.clock().now_utc();
        let progress_id = app_state.id_generator().new_uuid();

        let _permit = app_state.acquire_write_permit().await;
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        ensure_step_exists_in_tx(&mut tx, request.step_id).await?;

        let existing = find_existing_in_tx(&mut tx, request.step_id, &request.date).await?;

        let new_completed_at = match existing {
            Some(row) => {
                // toggle
                let toggled = if row.completed_at.is_some() {
                    None
                } else {
                    Some(client_time)
                };

                sqlx::query(
                    r#"
                    UPDATE shutdown_ritual_progress
                    SET completed_at = ?, updated_at = ?
                    WHERE id = ?
                    "#,
                )
                .bind(toggled)
                .bind(now)
                .bind(row.id)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::DatabaseError(e.into()))?;

                toggled
            }
            None => {
                // insert as completed
                let completed_at = Some(client_time);
                sqlx::query(
                    r#"
                    INSERT INTO shutdown_ritual_progress (id, step_id, date, completed_at, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?)
                    "#,
                )
                .bind(progress_id.to_string())
                .bind(request.step_id.to_string())
                .bind(&request.date)
                .bind(completed_at)
                .bind(now)
                .bind(now)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::DatabaseError(e.into()))?;

                completed_at
            }
        };

        let dto = ShutdownRitualProgressDto {
            step_id: request.step_id,
            date: request.date,
            completed_at: new_completed_at,
        };

        // Outbox event
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());
        let mut event = DomainEvent::new(
            "shutdown_ritual.progress.toggled",
            "shutdown_ritual_progress",
            request.step_id.to_string(),
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


