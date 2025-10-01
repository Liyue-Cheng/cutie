/// 保存视图排序偏好 API - 单文件组件
/// PUT /view-preferences
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};

use crate::{
    entities::view_preference::{
        SaveViewPreferenceRequest, ViewPreference, ViewPreferenceDto, ViewPreferenceRow,
    },
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(payload): Json<SaveViewPreferenceRequest>,
) -> Response {
    match logic::execute(&app_state, payload).await {
        Ok(preference_dto) => success_response(preference_dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        payload: SaveViewPreferenceRequest,
    ) -> AppResult<ViewPreferenceDto> {
        // 1. 验证
        if payload.context_key.trim().is_empty() {
            return Err(AppError::validation_error(
                "context_key",
                "Context key 不能为空",
                "CONTEXT_KEY_EMPTY",
            ));
        }

        if payload.sorted_task_ids.is_empty() {
            return Err(AppError::validation_error(
                "sorted_task_ids",
                "任务ID列表不能为空",
                "TASK_IDS_EMPTY",
            ));
        }

        let pool = app_state.db_pool();
        let now = app_state.clock().now_utc();

        // 2. 构建实体
        let preference = ViewPreference {
            context_key: payload.context_key,
            sorted_task_ids: payload.sorted_task_ids,
            updated_at: now,
        };

        // 3. 保存到数据库
        let saved = database::upsert(pool, &preference).await?;

        // 4. 返回 DTO
        Ok(ViewPreferenceDto {
            context_key: saved.context_key,
            sorted_task_ids: saved.sorted_task_ids,
            updated_at: saved.updated_at,
        })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;

    pub async fn upsert(
        pool: &sqlx::SqlitePool,
        preference: &ViewPreference,
    ) -> AppResult<ViewPreference> {
        // 序列化任务ID数组为 JSON
        let sorted_task_ids_json = serde_json::to_string(&preference.sorted_task_ids)?;

        let updated_at = preference.updated_at.to_rfc3339();

        let query = r#"
            INSERT INTO view_preferences (context_key, sorted_task_ids, updated_at)
            VALUES (?, ?, ?)
            ON CONFLICT(context_key) DO UPDATE SET
                sorted_task_ids = excluded.sorted_task_ids,
                updated_at = excluded.updated_at
        "#;

        sqlx::query(query)
            .bind(&preference.context_key)
            .bind(&sorted_task_ids_json)
            .bind(&updated_at)
            .execute(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        // 返回更新后的数据
        find_by_context_key(pool, &preference.context_key)
            .await?
            .ok_or_else(|| AppError::not_found("ViewPreference", &preference.context_key))
    }

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
