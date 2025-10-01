/// 列出所有 Areas API - 单文件组件
use axum::{extract::State, response::{IntoResponse, Response}};
use uuid::Uuid;

use crate::{
    entities::{Area, AreaDto},
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>) -> Response {
    match logic::execute(&app_state).await {
        Ok(areas) => success_response(areas).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState) -> AppResult<Vec<AreaDto>> {
        let pool = app_state.db_pool();
        let areas = database::find_all_areas(pool).await?;

        let area_dtos = areas
            .into_iter()
            .map(|area| AreaDto {
                id: area.id,
                name: area.name,
                color: area.color,
                parent_area_id: area.parent_area_id,
                created_at: area.created_at,
                updated_at: area.updated_at,
            })
            .collect();

        Ok(area_dtos)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::entities::AreaRow;

    pub async fn find_all_areas(pool: &sqlx::SqlitePool) -> AppResult<Vec<Area>> {
        let query = r#"
            SELECT id, name, color, parent_area_id, created_at, updated_at, is_deleted
            FROM areas
            WHERE is_deleted = false
            ORDER BY name ASC
        "#;

        let rows = sqlx::query_as::<_, AreaRow>(query)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        let areas: Result<Vec<Area>, _> = rows.into_iter().map(Area::try_from).collect();

        areas.map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }
}

