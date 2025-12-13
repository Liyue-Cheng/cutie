/// Create shutdown ritual step - Single File Component endpoint
///
/// POST /api/shutdown-ritual/steps
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::{
    entities::{CreateShutdownRitualStepRequest, ShutdownRitualStepDto},
    features::shared::TransactionHelper,
    infra::{
        core::{AppError, AppResult, ValidationError},
        events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        },
        http::error_handler::created_response,
        LexoRankService,
    },
    startup::AppState,
};

// ==================== CABC Documentation ====================
/*
1. Endpoint Signature
   POST /api/shutdown-ritual/steps

2. High-Level Behavior
   Create a persistent shutdown ritual step template, appended to the end by LexoRank.

3. Input/Output Specification
   Body: { title: string }
   Response: ShutdownRitualStepDto

4. Validation Rules
   - title must be non-empty, <= 255

5. Business Logic Walkthrough
   - Validate input
   - Generate id, timestamps, order_rank (append to end)
   - Insert into DB in transaction
   - Write outbox event: shutdown_ritual.step.created
   - Commit and return created dto

6. Edge Cases
   - Empty table: use initial_rank

7. Expected Side Effects
   - INSERT shutdown_ritual_steps
   - INSERT event_outbox

8. Contract
   - Returned DTO equals outbox payload
*/

#[derive(Debug, Serialize)]
pub struct CreateShutdownRitualStepResponse {
    #[serde(flatten)]
    pub step: ShutdownRitualStepDto,
}

pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateShutdownRitualStepRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(dto) => created_response(CreateShutdownRitualStepResponse { step: dto }).into_response(),
        Err(err) => err.into_response(),
    }
}

mod validation {
    use super::*;

    pub fn validate_request(request: &CreateShutdownRitualStepRequest) -> AppResult<()> {
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

    async fn get_highest_rank_in_tx(tx: &mut Transaction<'_, Sqlite>) -> AppResult<Option<String>> {
        let row = sqlx::query_scalar::<_, String>(
            r#"
            SELECT order_rank
            FROM shutdown_ritual_steps
            ORDER BY order_rank DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?;
        Ok(row)
    }

    pub async fn execute(
        app_state: &AppState,
        request: CreateShutdownRitualStepRequest,
    ) -> AppResult<ShutdownRitualStepDto> {
        validation::validate_request(&request)?;

        let id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        let _permit = app_state.acquire_write_permit().await;
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        let last_rank = get_highest_rank_in_tx(&mut tx).await?;
        let new_rank = match last_rank {
            Some(r) => LexoRankService::generate_between(Some(r.as_str()), None)?,
            None => LexoRankService::initial_rank(),
        };

        sqlx::query(
            r#"
            INSERT INTO shutdown_ritual_steps (id, title, order_rank, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(request.title.trim())
        .bind(&new_rank)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?;

        let dto = ShutdownRitualStepDto {
            id,
            title: request.title.trim().to_string(),
            order_rank: new_rank,
            created_at: now,
            updated_at: now,
        };

        // Outbox event (transactional)
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());
        let payload = serde_json::to_value(&dto)?;
        let event = DomainEvent::new(
            "shutdown_ritual.step.created",
            "shutdown_ritual_step",
            id.to_string(),
            payload,
        );
        outbox_repo.append_in_tx(&mut tx, &event).await?;

        TransactionHelper::commit(tx).await?;
        Ok(dto)
    }
}
