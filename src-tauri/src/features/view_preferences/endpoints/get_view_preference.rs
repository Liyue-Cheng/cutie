/// 获取视图排序偏好 API - 单文件组件
/// GET /view-preferences/:context_key
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};

use crate::{
    entities::view_preference::{ViewPreference, ViewPreferenceDto, ViewPreferenceRow},
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(context_key): Path<String>,
) -> Response {
    match logic::execute(&app_state, &context_key).await {
        Ok(preference_dto) => success_response(preference_dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, context_key: &str) -> AppResult<ViewPreferenceDto> {
        let pool = app_state.db_pool();

        // 查询视图偏好
        let preference = database::find_by_context_key(pool, context_key)
            .await?
            .ok_or_else(|| AppError::not_found("ViewPreference", context_key))?;

        // 转换为 DTO
        Ok(ViewPreferenceDto {
            context_key: preference.context_key,
            sorted_task_ids: preference.sorted_task_ids,
            updated_at: preference.updated_at,
        })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;

    pub async fn find_by_context_key(
        pool: &sqlx::SqlitePool,
        context_key: &str,
    ) -> AppResult<Option<ViewPreference>> {
        let query = r#"
            SELECT context_key, sorted_task_ids, updated_at
            FROM view_preferences
            WHERE context_key = ?
        "#;

        let row = sqlx::query_as::<_, ViewPreferenceRow>(query)
            .bind(context_key)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        match row {
            Some(row) => {
                let pref = ViewPreference::try_from(row)?;
                Ok(Some(pref))
            }
            None => Ok(None),
        }
    }
}
