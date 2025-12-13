/// Get shutdown ritual state for a date - Single File Component endpoint
///
/// GET /api/shutdown-ritual/state?date=YYYY-MM-DD
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    entities::{ShutdownRitualProgressDto, ShutdownRitualStateDto, ShutdownRitualStepDto},
    infra::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== CABC Documentation ====================
/*
1. Endpoint Signature
   GET /api/shutdown-ritual/state?date=YYYY-MM-DD

2. High-Level Behavior
   Return ordered ritual steps and per-step completion state for the given day.

3. Input/Output Specification
   Query: date (YYYY-MM-DD)
   Response: { date, steps: [...], progress: [...] }

4. Validation Rules
   - date must be valid YYYY-MM-DD.

5. Business Logic Walkthrough
   - Validate date
   - SELECT steps ORDER BY order_rank ASC
   - SELECT progress WHERE date = ?
   - Return state

6. Edge Cases
   - No steps: steps=[], progress=[]
   - Steps exist but no progress rows yet: progress=[]

7. Expected Side Effects
   - Read-only, no outbox events

8. Contract
   - Steps are always ordered by order_rank ascending
*/

#[derive(Debug, Deserialize)]
pub struct GetStateQuery {
    pub date: String,
}

pub async fn handle(State(app_state): State<AppState>, Query(query): Query<GetStateQuery>) -> Response {
    match logic::execute(&app_state, &query.date).await {
        Ok(dto) => success_response(dto).into_response(),
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

    #[derive(Debug, sqlx::FromRow)]
    struct ProgressRow {
        step_id: String,
        date: String,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    pub async fn execute(app_state: &AppState, date: &str) -> AppResult<ShutdownRitualStateDto> {
        validation::validate_date(date)?;

        let pool = app_state.db_pool();

        // steps
        let step_rows = sqlx::query_as::<_, crate::entities::shutdown_ritual::ShutdownRitualStepRow>(
            r#"
            SELECT id, title, order_rank, created_at, updated_at
            FROM shutdown_ritual_steps
            ORDER BY order_rank ASC
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?;

        let mut steps: Vec<ShutdownRitualStepDto> = Vec::with_capacity(step_rows.len());
        for row in step_rows {
            let step = crate::entities::shutdown_ritual::ShutdownRitualStep::try_from(row).map_err(
                |e| AppError::DatabaseError(crate::infra::core::DbError::QueryError(e)),
            )?;

            steps.push(ShutdownRitualStepDto {
                id: step.id,
                title: step.title,
                order_rank: step.order_rank,
                created_at: step.created_at,
                updated_at: step.updated_at,
            });
        }

        // progress for date
        let progress_rows = sqlx::query_as::<_, ProgressRow>(
            r#"
            SELECT step_id, date, completed_at
            FROM shutdown_ritual_progress
            WHERE date = ?
            "#,
        )
        .bind(date)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?;

        let mut progress: Vec<ShutdownRitualProgressDto> = Vec::with_capacity(progress_rows.len());
        for row in progress_rows {
            let step_id = Uuid::parse_str(&row.step_id).map_err(|_| {
                AppError::validation_error("step_id", "Invalid UUID", "INVALID_UUID")
            })?;
            progress.push(ShutdownRitualProgressDto {
                step_id,
                date: row.date,
                completed_at: row.completed_at,
            });
        }

        Ok(ShutdownRitualStateDto {
            date: date.to_string(),
            steps,
            progress,
        })
    }
}


