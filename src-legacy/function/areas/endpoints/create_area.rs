/// 创建领域 API - 单文件组件
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::Area,
    shared::{
        core::{AppError, AppResult, ValidationError},
        http::error_handler::created_response,
    },
    startup::AppState,
};

#[derive(Deserialize)]
pub struct CreateAreaRequest {
    name: String,
    color: String,
    parent_area_id: Option<String>,
}

pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateAreaRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(area) => created_response(area).into_response(),
        Err(err) => err.into_response(),
    }
}

mod validation {
    use super::*;

    pub struct ValidatedAreaData {
        pub name: String,
        pub color: String,
        pub parent_area_id: Option<Uuid>,
    }

    pub fn validate_request(
        request: &CreateAreaRequest,
    ) -> Result<ValidatedAreaData, Vec<ValidationError>> {
        let mut errors = Vec::new();

        if request.name.trim().is_empty() {
            errors.push(ValidationError::new("name", "领域名称不能为空", "NAME_REQUIRED"));
        }

        if !Area::validate_color(&request.color) {
            errors.push(ValidationError::new(
                "color",
                "颜色格式无效，应为 #RRGGBB 格式",
                "INVALID_COLOR",
            ));
        }

        let parent_area_id = if let Some(ref parent_id_str) = request.parent_area_id {
            match Uuid::parse_str(parent_id_str) {
                Ok(id) => Some(id),
                Err(_) => {
                    errors.push(ValidationError::new(
                        "parent_area_id",
                        "父领域 ID 格式无效",
                        "INVALID_PARENT_AREA_ID",
                    ));
                    None
                }
            }
        } else {
            None
        };

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(ValidatedAreaData {
            name: request.name.trim().to_string(),
            color: request.color.clone(),
            parent_area_id,
        })
    }
}

mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, request: CreateAreaRequest) -> AppResult<Area> {
        let validated =
            validation::validate_request(&request).map_err(AppError::ValidationFailed)?;

        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        if let Some(parent_id) = validated.parent_area_id {
            let parent_exists = database::area_exists_in_tx(&mut tx, parent_id).await?;
            if !parent_exists {
                return Err(AppError::not_found("Area", parent_id.to_string()));
            }
        }

        let new_area_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        let mut new_area = Area::new(new_area_id, validated.name, validated.color, now);
        new_area.parent_area_id = validated.parent_area_id;

        let created_area = database::create_area_in_tx(&mut tx, &new_area).await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(created_area)
    }
}

mod database {
    use super::*;
    use crate::entities::area::AreaRow;

    pub async fn area_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        area_id: Uuid,
    ) -> AppResult<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM areas WHERE id = ? AND deleted_at IS NULL")
                .bind(area_id.to_string())
                .fetch_one(&mut **tx)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
                })?;

        Ok(count > 0)
    }

    pub async fn create_area_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        area: &Area,
    ) -> AppResult<Area> {
        let row = sqlx::query_as::<_, AreaRow>(
            r#"
            INSERT INTO areas (id, name, color, parent_area_id, created_at, updated_at, is_deleted)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING id, name, color, parent_area_id, created_at, updated_at, is_deleted
            "#,
        )
        .bind(area.id.to_string())
        .bind(&area.name)
        .bind(&area.color)
        .bind(area.parent_area_id.map(|id| id.to_string()))
        .bind(area.created_at.to_rfc3339())
        .bind(area.updated_at.to_rfc3339())
        .bind(area.is_deleted)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        Area::try_from(row)
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }
}
