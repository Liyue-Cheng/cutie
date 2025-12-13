/// Update shutdown ritual step order rank (LexoRank) - Single File Component endpoint
///
/// PATCH /api/shutdown-ritual/steps/:id/order-rank
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use crate::{
    entities::{UpdateShutdownRitualStepSortRequest, UpdateShutdownRitualStepSortResponse},
    features::shared::TransactionHelper,
    infra::{
        core::{AppError, AppResult},
        events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        },
        http::error_handler::success_response,
        LexoRankService,
    },
    startup::AppState,
};

// ==================== CABC Documentation ====================
/*
1. Endpoint Signature
   PATCH /api/shutdown-ritual/steps/:id/order-rank

2. High-Level Behavior
   Update a step's LexoRank order_rank based on neighbor steps.

3. Input/Output Specification
   Path: id (UUID)
   Body: { prev_step_id?: UUID|null, next_step_id?: UUID|null }
   Response: { step_id, new_rank }

4. Validation Rules
   - prev_step_id and next_step_id cannot be the same (when both present)
   - step must exist

5. Business Logic Walkthrough
   - Fetch neighbor ranks by ids
   - Generate new rank between(prev, next)
   - Update step.order_rank
   - Emit outbox event shutdown_ritual.step.reordered

6. Edge Cases
   - Moving to start/end: one neighbor null

7. Expected Side Effects
   - UPDATE shutdown_ritual_steps
   - INSERT event_outbox

8. Contract
   - Returned payload equals outbox payload
*/

pub async fn handle(
    State(app_state): State<AppState>,
    Path(step_id): Path<Uuid>,
    Json(request): Json<UpdateShutdownRitualStepSortRequest>,
) -> Response {
    match logic::execute(&app_state, step_id, request).await {
        Ok(res) => success_response(res).into_response(),
        Err(err) => err.into_response(),
    }
}

mod logic {
    use super::*;

    async fn get_neighbor_rank(
        pool: &sqlx::SqlitePool,
        id: Option<Uuid>,
    ) -> AppResult<Option<String>> {
        let Some(id) = id else {
            return Ok(None);
        };

        let rank = sqlx::query_scalar::<_, Option<String>>(
            "SELECT order_rank FROM shutdown_ritual_steps WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?
        .flatten();

        Ok(rank)
    }

    pub async fn execute(
        app_state: &AppState,
        step_id: Uuid,
        request: UpdateShutdownRitualStepSortRequest,
    ) -> AppResult<UpdateShutdownRitualStepSortResponse> {
        if request.prev_step_id == request.next_step_id && request.prev_step_id.is_some() {
            return Err(AppError::validation_error(
                "prev_step_id",
                "prev_step_id and next_step_id cannot be the same",
                "INVALID_NEIGHBORS",
            ));
        }

        let pool = app_state.db_pool();
        let prev_rank = get_neighbor_rank(pool, request.prev_step_id).await?;
        let next_rank = get_neighbor_rank(pool, request.next_step_id).await?;

        let new_rank =
            LexoRankService::generate_between(prev_rank.as_deref(), next_rank.as_deref())?;
        let now = app_state.clock().now_utc();

        let _permit = app_state.acquire_write_permit().await;
        let mut tx = TransactionHelper::begin(pool).await?;

        let affected = sqlx::query(
            r#"
            UPDATE shutdown_ritual_steps
            SET order_rank = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&new_rank)
        .bind(now)
        .bind(step_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::not_found("ShutdownRitualStep", step_id.to_string()));
        }

        let res = UpdateShutdownRitualStepSortResponse {
            step_id,
            new_rank: new_rank.clone(),
        };

        // Outbox event
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());
        let payload = serde_json::to_value(&res)?;
        let event = DomainEvent::new(
            "shutdown_ritual.step.reordered",
            "shutdown_ritual_step",
            step_id.to_string(),
            payload,
        );
        outbox_repo.append_in_tx(&mut tx, &event).await?;

        TransactionHelper::commit(tx).await?;
        Ok(res)
    }
}


