use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    features::shared::{repositories::TemplateSortRepository, TransactionHelper},
    infra::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
        LexoRankService,
    },
    startup::AppState,
};

#[derive(Debug, Deserialize)]
pub struct BatchInitTemplateRanksRequest {
    pub template_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct TemplateRankAssignment {
    pub template_id: Uuid,
    pub new_rank: String,
}

#[derive(Debug, Serialize)]
pub struct BatchInitTemplateRanksResponse {
    pub assigned: Vec<TemplateRankAssignment>,
    pub updated_at: String,
}

pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<BatchInitTemplateRanksRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(res) => success_response(res).into_response(),
        Err(err) => err.into_response(),
    }
}

mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        request: BatchInitTemplateRanksRequest,
    ) -> AppResult<BatchInitTemplateRanksResponse> {
        validation::validate(&request)?;

        if request.template_ids.is_empty() {
            return Ok(BatchInitTemplateRanksResponse {
                assigned: vec![],
                updated_at: Utc::now().to_rfc3339(),
            });
        }

        let pool = app_state.db_pool();
        let existing_first = TemplateSortRepository::get_first_sort_rank(pool).await?;
        let assignments = generate_assignments(existing_first, &request.template_ids)?;
        let now = app_state.clock().now_utc();

        let _permit = app_state.acquire_write_permit().await;
        let mut tx = TransactionHelper::begin(pool).await?;
        let tuples = assignments_as_tuple(&assignments);
        TemplateSortRepository::batch_update_sort_ranks_in_tx(&mut tx, &tuples, now).await?;
        TransactionHelper::commit(tx).await?;

        Ok(BatchInitTemplateRanksResponse {
            assigned: assignments,
            updated_at: now.to_rfc3339(),
        })
    }

    fn assignments_as_tuple(assignments: &[TemplateRankAssignment]) -> Vec<(Uuid, String)> {
        assignments
            .iter()
            .map(|a| (a.template_id, a.new_rank.clone()))
            .collect()
    }

    fn generate_assignments(
        existing_first: Option<String>,
        template_ids: &[Uuid],
    ) -> AppResult<Vec<TemplateRankAssignment>> {
        if let Some(first_rank) = existing_first {
            let mut generated = Vec::with_capacity(template_ids.len());
            let mut anchor = first_rank;
            let mut temp = Vec::with_capacity(template_ids.len());

            for template_id in template_ids.iter().rev() {
                let new_rank = LexoRankService::generate_between(None, Some(&anchor))?;
                anchor = new_rank.clone();
                temp.push(TemplateRankAssignment {
                    template_id: *template_id,
                    new_rank,
                });
            }

            temp.reverse();
            generated.extend(temp);
            Ok(generated)
        } else {
            let mut generated = Vec::with_capacity(template_ids.len());
            let mut prev: Option<String> = None;

            for template_id in template_ids {
                let new_rank = LexoRankService::generate_between(prev.as_deref(), None)?;
                prev = Some(new_rank.clone());
                generated.push(TemplateRankAssignment {
                    template_id: *template_id,
                    new_rank,
                });
            }

            Ok(generated)
        }
    }
}

mod validation {
    use super::*;

    pub fn validate(request: &BatchInitTemplateRanksRequest) -> AppResult<()> {
        if request.template_ids.is_empty() {
            return Ok(());
        }

        Ok(())
    }
}
