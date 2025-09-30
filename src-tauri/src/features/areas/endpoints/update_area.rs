/// 更新领域 API - 单文件组件
use axum::{
    extract::{Path, State},
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
        http::error_handler::success_response,
    },
    startup::AppState,
};

#[derive(Deserialize)]
pub struct UpdateAreaRequest {
    name: Option<String>,
    color: Option<String>,
    parent_area_id: Option<Option<String>>,
}

pub async fn handle(
    State(app_state): State<AppState>,
    Path(area_id): Path<Uuid>,
    Json(request): Json<UpdateAreaRequest>,
) -> Response {
    match logic::execute(&app_state, area_id, request).await {
        Ok(area) => success_response(area).into_response(),
        Err(err) => err.into_response(),
    }
}

mod validation {
    use super::*;

    pub struct ValidatedUpdates {
        pub name: Option<String>,
        pub color: Option<String>,
        pub parent_area_id: Option<Option<Uuid>>,
    }

    pub fn validate_request(
        request: &UpdateAreaRequest,
    ) -> Result<ValidatedUpdates, Vec<ValidationError>> {
        let mut errors = Vec::new();

        let name = if let Some(ref n) = request.name {
            if n.trim().is_empty() {
                errors.push(ValidationError::new(
                    "name",
                    "领域名称不能为空",
                    "NAME_REQUIRED",
                ));
                None
            } else {
                Some(n.trim().to_string())
            }
        } else {
            None
        };

        let color = if let Some(ref c) = request.color {
            if !Area::validate_color(c) {
                errors.push(ValidationError::new(
                    "color",
                    "颜色格式无效，应为 #RRGGBB 格式",
                    "INVALID_COLOR",
                ));
                None
            } else {
                Some(c.clone())
            }
        } else {
            None
        };

        let parent_area_id = if let Some(ref maybe_parent) = request.parent_area_id {
            Some(if let Some(ref parent_id_str) = maybe_parent {
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
            })
        } else {
            None
        };

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(ValidatedUpdates {
            name,
            color,
            parent_area_id,
        })
    }
}

mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        area_id: Uuid,
        request: UpdateAreaRequest,
    ) -> AppResult<Area> {
        let validated =
            validation::validate_request(&request).map_err(AppError::ValidationFailed)?;

        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        let mut area = database::find_area_by_id_in_tx(&mut tx, area_id)
            .await?
            .ok_or_else(|| AppError::not_found("Area", area_id.to_string()))?;

        if let Some(Some(parent_id)) = validated.parent_area_id {
            if parent_id == area_id {
                return Err(AppError::ValidationFailed(vec![ValidationError::new(
                    "parent_area_id",
                    "领域不能将自己设为父领域",
                    "CIRCULAR_DEPENDENCY",
                )]));
            }

            let parent_exists = database::area_exists_in_tx(&mut tx, parent_id).await?;
            if !parent_exists {
                return Err(AppError::not_found("Area", parent_id.to_string()));
            }
        }

        let now = app_state.clock().now_utc();

        if let Some(name) = validated.name {
            area.name = name;
        }
        if let Some(color) = validated.color {
            area.color = color;
        }
        if let Some(parent_area_id) = validated.parent_area_id {
            area.parent_area_id = parent_area_id;
        }

        area.updated_at = now;

        let updated_area = database::update_area_in_tx(&mut tx, &area).await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(updated_area)
    }
}

mod database {
    use super::*;
    use crate::entities::area::AreaRow;

    pub async fn find_area_by_id_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        area_id: Uuid,
    ) -> AppResult<Option<Area>> {
        let row = sqlx::query_as::<_, AreaRow>(
            r#"
            SELECT id, name, color, parent_area_id, created_at, updated_at, is_deleted
            FROM areas WHERE id = ? AND is_deleted = false
            "#,
        )
        .bind(area_id.to_string())
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        row.map(|r| Area::try_from(r))
            .transpose()
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }

    pub async fn area_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        area_id: Uuid,
    ) -> AppResult<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM areas WHERE id = ? AND is_deleted = false")
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
        area: &Area,
    ) -> AppResult<Area> {
        let row = sqlx::query_as::<_, AreaRow>(
            r#"
            UPDATE areas SET
                name = ?, color = ?, parent_area_id = ?, updated_at = ?
            WHERE id = ? AND is_deleted = false
            RETURNING id, name, color, parent_area_id, created_at, updated_at, is_deleted
            "#,
        )
        .bind(&area.name)
        .bind(&area.color)
        .bind(area.parent_area_id.map(|id| id.to_string()))
        .bind(area.updated_at.to_rfc3339())
        .bind(area.id.to_string())
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        Area::try_from(row)
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }
}
