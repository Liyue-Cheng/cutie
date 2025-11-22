use axum::{
    extract::{Path, State},
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
pub struct UpdateTemplateSortRequest {
    pub prev_template_id: Option<Uuid>,
    pub next_template_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct UpdateTemplateSortResponse {
    pub template_id: Uuid,
    pub new_rank: String,
}

pub async fn handle(
    State(app_state): State<AppState>,
    Path(template_id): Path<Uuid>,
    Json(request): Json<UpdateTemplateSortRequest>,
) -> Response {
    match logic::execute(&app_state, template_id, request).await {
        Ok(res) => success_response(res).into_response(),
        Err(err) => err.into_response(),
    }
}

mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        template_id: Uuid,
        request: UpdateTemplateSortRequest,
    ) -> AppResult<UpdateTemplateSortResponse> {
        // 基本校验
        if request.prev_template_id == request.next_template_id
            && request.prev_template_id.is_some()
        {
            return Err(AppError::validation_error(
                "prev_template_id",
                "prev_template_id and next_template_id cannot be the same",
                "INVALID_NEIGHBORS",
            ));
        }

        let pool = app_state.db_pool();
        let prev_rank =
            TemplateSortRepository::get_neighbor_rank(pool, request.prev_template_id).await?;
        let next_rank =
            TemplateSortRepository::get_neighbor_rank(pool, request.next_template_id).await?;

        let new_rank =
            LexoRankService::generate_between(prev_rank.as_deref(), next_rank.as_deref())?;
        let now = app_state.clock().now_utc();

        let _permit = app_state.acquire_write_permit().await;
        let mut tx = TransactionHelper::begin(pool).await?;
        TemplateSortRepository::update_sort_rank_in_tx(&mut tx, template_id, &new_rank, now)
            .await?;
        TransactionHelper::commit(tx).await?;

        Ok(UpdateTemplateSortResponse {
            template_id,
            new_rank,
        })
    }
}
