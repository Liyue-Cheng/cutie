/// 更新 Area API
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{Area, AreaDto, UpdateAreaRequest},
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

pub async fn handle(
    State(app_state): State<AppState>,
    Path(area_id): Path<Uuid>,
    Json(request): Json<UpdateAreaRequest>,
) -> Response {
    match logic::execute(&app_state, area_id, request).await {
        Ok(area_dto) => success_response(area_dto).into_response(),
        Err(err) => err.into_response(),
    }
}

mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        area_id: Uuid,
        request: UpdateAreaRequest,
    ) -> AppResult<AreaDto> {
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 1. 检查存在
        let exists = database::check_area_exists_in_tx(&mut tx, area_id).await?;
        if !exists {
            return Err(AppError::not_found("Area", area_id.to_string()));
        }

        // 2. 验证
        if let Some(name) = &request.name {
            if name.trim().is_empty() {
                return Err(AppError::validation_error("name", "名称不能为空", "NAME_EMPTY"));
            }
        }
        if let Some(color) = &request.color {
            if !Area::validate_color(color) {
                return Err(AppError::validation_error(
                    "color",
                    "颜色格式无效",
                    "INVALID_COLOR",
                ));
            }
        }

        // 3. 更新
        database::update_area_in_tx(&mut tx, area_id, &request).await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 4. 重新查询
        let area = database::find_area_by_id(app_state.db_pool(), area_id)
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

    pub async fn update_area_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        area_id: Uuid,
        request: &UpdateAreaRequest,
    ) -> AppResult<()> {
        let now = chrono::Utc::now();
        let mut updates = Vec::new();
        let mut bindings: Vec<String> = Vec::new();

        if let Some(name) = &request.name {
            updates.push("name = ?");
            bindings.push(name.clone());
        }
        if let Some(color) = &request.color {
            updates.push("color = ?");
            bindings.push(color.clone());
        }
        if let Some(parent_id) = &request.parent_area_id {
            updates.push("parent_area_id = ?");
            bindings.push(parent_id.map(|id| id.to_string()).unwrap_or_default());
        }

        if updates.is_empty() {
            return Ok(());
        }

        updates.push("updated_at = ?");
        let update_clause = updates.join(", ");
        let query = format!("UPDATE areas SET {} WHERE id = ?", update_clause);

        let mut query_builder = sqlx::query(&query);
        for binding in bindings {
            query_builder = query_builder.bind(binding);
        }
        query_builder = query_builder.bind(now.to_rfc3339());
        query_builder = query_builder.bind(area_id.to_string());

        query_builder.execute(&mut **tx).await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        Ok(())
    }

    pub async fn find_area_by_id(pool: &sqlx::SqlitePool, area_id: Uuid) -> AppResult<Option<Area>> {
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

