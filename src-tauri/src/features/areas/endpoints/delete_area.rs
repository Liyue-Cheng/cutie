/// 删除 Area API
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    shared::{
        core::{AppError, AppResult},
        http::error_handler::no_content_response,
    },
    startup::AppState,
};

pub async fn handle(State(app_state): State<AppState>, Path(area_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, area_id).await {
        Ok(()) => no_content_response().into_response(),
        Err(err) => err.into_response(),
    }
}

mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, area_id: Uuid) -> AppResult<()> {
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        let exists = database::check_area_exists_in_tx(&mut tx, area_id).await?;
        if !exists {
            return Err(AppError::not_found("Area", area_id.to_string()));
        }

        database::soft_delete_area_in_tx(&mut tx, area_id).await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(())
    }
}

mod database {
    use super::*;

    pub async fn check_area_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        area_id: Uuid,
    ) -> AppResult<bool> {
        let query = "SELECT COUNT(*) FROM areas WHERE id = ? AND is_deleted = false";
        let count: i64 = sqlx::query_scalar(query)
            .bind(area_id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(count > 0)
    }

    pub async fn soft_delete_area_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        area_id: Uuid,
    ) -> AppResult<()> {
        let query = "UPDATE areas SET is_deleted = true, updated_at = ? WHERE id = ?";
        sqlx::query(query)
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(area_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(())
    }
}

