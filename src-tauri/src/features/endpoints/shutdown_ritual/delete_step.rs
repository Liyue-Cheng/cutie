/// Delete shutdown ritual step - Single File Component endpoint
///
/// DELETE /api/shutdown-ritual/steps/:id
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    features::shared::TransactionHelper,
    infra::{
        core::{AppError, AppResult},
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
   DELETE /api/shutdown-ritual/steps/:id

2. High-Level Behavior
   Permanently delete a ritual step template. Progress rows cascade delete.

3. Input/Output Specification
   Path: id (UUID)
   Response: { id }

4. Validation Rules
   - step must exist

5. Business Logic Walkthrough
   - Delete step row
   - Write outbox event shutdown_ritual.step.deleted with {id}
   - Commit

6. Edge Cases
   - Not found -> 404

7. Expected Side Effects
   - DELETE shutdown_ritual_steps (and cascade progress)
   - INSERT event_outbox

8. Contract
   - Returned payload equals outbox payload
*/

#[derive(Debug, Serialize)]
pub struct DeleteShutdownRitualStepResponse {
    pub id: Uuid,
}

pub async fn handle(State(app_state): State<AppState>, Path(step_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, step_id).await {
        Ok(dto) => success_response(dto).into_response(),
        Err(err) => err.into_response(),
    }
}

mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        step_id: Uuid,
    ) -> AppResult<DeleteShutdownRitualStepResponse> {
        let _permit = app_state.acquire_write_permit().await;
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        let affected = sqlx::query("DELETE FROM shutdown_ritual_steps WHERE id = ?")
            .bind(step_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::not_found("ShutdownRitualStep", step_id.to_string()));
        }

        let payload = serde_json::json!({ "id": step_id });
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());
        let event = DomainEvent::new(
            "shutdown_ritual.step.deleted",
            "shutdown_ritual_step",
            step_id.to_string(),
            payload,
        );
        outbox_repo.append_in_tx(&mut tx, &event).await?;

        TransactionHelper::commit(tx).await?;

        Ok(DeleteShutdownRitualStepResponse { id: step_id })
    }
}


