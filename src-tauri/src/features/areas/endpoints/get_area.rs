/// 获取单个 Area API
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    entities::{Area, AreaDto},
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

pub async fn handle(State(app_state): State<AppState>, Path(area_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, area_id).await {
        Ok(area_dto) => success_response(area_dto).into_response(),
        Err(err) => err.into_response(),
    }
}

mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, area_id: Uuid) -> AppResult<AreaDto> {
        let pool = app_state.db_pool();
        let area = database::find_area_by_id(pool, area_id)
            .await?
            .ok_or_else(|| AppError::not_found("Area", area_id.to_string()))?;

        Ok(AreaDto {
            id: area.id,
            name: area.name,
            color: area.color,
            parent_area_id: area.parent_area_id,
            created_at: area.created_at,
            updated_at: area.updated_at,
        })
    }
}

mod database {
    use super::*;
    use crate::entities::AreaRow;

    pub async fn find_area_by_id(
        pool: &sqlx::SqlitePool,
        area_id: Uuid,
    ) -> AppResult<Option<Area>> {
        let query = r#"
            SELECT id, name, color, parent_area_id, created_at, updated_at, is_deleted
            FROM areas
            WHERE id = ? AND is_deleted = false
        "#;

        let row = sqlx::query_as::<_, AreaRow>(query)
            .bind(area_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        match row {
            Some(r) => {
                let area = Area::try_from(r)
                    .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))?;
                Ok(Some(area))
            }
            None => Ok(None),
        }
    }
}

