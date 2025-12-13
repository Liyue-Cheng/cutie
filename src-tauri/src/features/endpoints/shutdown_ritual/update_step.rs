/// Update shutdown ritual step - Single File Component endpoint
///
/// PATCH /api/shutdown-ritual/steps/:id
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    entities::{ShutdownRitualStepDto, UpdateShutdownRitualStepRequest},
    features::shared::TransactionHelper,
    infra::{
        core::{AppError, AppResult, ValidationError},
        events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        },
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== CABC Documentation ====================
/*
1. Endpoint Signature
   PATCH /api/shutdown-ritual/steps/:id

2. High-Level Behavior
   Update a ritual step title.

3. Input/Output Specification
   Path: id (UUID)
   Body: { title: string }
   Response: ShutdownRitualStepDto

4. Validation Rules
   - title non-empty, <=255
   - step must exist

5. Business Logic Walkthrough
   - Validate input
   - Update title + updated_at
   - Read back row and return dto
   - Emit outbox event shutdown_ritual.step.updated with dto payload

6. Edge Cases
   - Not found -> 404

7. Expected Side Effects
   - UPDATE shutdown_ritual_steps
   - INSERT event_outbox

8. Contract
   - Returned DTO equals outbox payload
*/

#[derive(Debug, Serialize)]
pub struct UpdateShutdownRitualStepResponse {
    #[serde(flatten)]
    pub step: ShutdownRitualStepDto,
}

pub async fn handle(
    State(app_state): State<AppState>,
    Path(step_id): Path<Uuid>,
    Json(request): Json<UpdateShutdownRitualStepRequest>,
) -> Response {
    match logic::execute(&app_state, step_id, request).await {
        Ok(dto) => success_response(UpdateShutdownRitualStepResponse { step: dto }).into_response(),
        Err(err) => err.into_response(),
    }
}

mod validation {
    use super::*;

    pub fn validate_request(request: &UpdateShutdownRitualStepRequest) -> AppResult<()> {
        let mut errors = Vec::new();
        if request.title.trim().is_empty() {
            errors.push(ValidationError {
                field: "title".to_string(),
                code: "TITLE_EMPTY".to_string(),
                message: "title cannot be empty".to_string(),
            });
        }
        if request.title.len() > 255 {
            errors.push(ValidationError {
                field: "title".to_string(),
                code: "TITLE_TOO_LONG".to_string(),
                message: "title too long (max 255 characters)".to_string(),
            });
        }
        if !errors.is_empty() {
            return Err(AppError::ValidationFailed(errors));
        }
        Ok(())
    }
}

mod logic {
    use super::*;
    use sqlx::{Sqlite, Transaction};

    async fn find_step_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        step_id: Uuid,
    ) -> AppResult<ShutdownRitualStepDto> {
        let row = sqlx::query_as::<_, crate::entities::shutdown_ritual::ShutdownRitualStepRow>(
            r#"
            SELECT id, title, order_rank, created_at, updated_at
            FROM shutdown_ritual_steps
            WHERE id = ?
            "#,
        )
        .bind(step_id.to_string())
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?
        .ok_or_else(|| AppError::not_found("ShutdownRitualStep", step_id.to_string()))?;

        let step = crate::entities::shutdown_ritual::ShutdownRitualStep::try_from(row).map_err(
            |e| AppError::DatabaseError(crate::infra::core::DbError::QueryError(e)),
        )?;

        Ok(ShutdownRitualStepDto {
            id: step.id,
            title: step.title,
            order_rank: step.order_rank,
            created_at: step.created_at,
            updated_at: step.updated_at,
        })
    }

    pub async fn execute(
        app_state: &AppState,
        step_id: Uuid,
        request: UpdateShutdownRitualStepRequest,
    ) -> AppResult<ShutdownRitualStepDto> {
        validation::validate_request(&request)?;

        let now = app_state.clock().now_utc();

        let _permit = app_state.acquire_write_permit().await;
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        let affected = sqlx::query(
            r#"
            UPDATE shutdown_ritual_steps
            SET title = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(request.title.trim())
        .bind(now)
        .bind(step_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::not_found("ShutdownRitualStep", step_id.to_string()));
        }

        let dto = find_step_in_tx(&mut tx, step_id).await?;

        // Outbox event (transactional)
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());
        let payload = serde_json::to_value(&dto)?;
        let event = DomainEvent::new(
            "shutdown_ritual.step.updated",
            "shutdown_ritual_step",
            step_id.to_string(),
            payload,
        );
        outbox_repo.append_in_tx(&mut tx, &event).await?;

        TransactionHelper::commit(tx).await?;
        Ok(dto)
    }
}


