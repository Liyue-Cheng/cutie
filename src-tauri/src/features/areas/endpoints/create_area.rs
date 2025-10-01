/// 创建 Area API - 单文件组件
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{Area, AreaDto, CreateAreaRequest},
    shared::{
        core::{AppError, AppResult},
        http::error_handler::created_response,
    },
    startup::AppState,
};

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateAreaRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(area_dto) => created_response(area_dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, request: CreateAreaRequest) -> AppResult<AreaDto> {
        // 1. 验证
        if request.name.trim().is_empty() {
            return Err(AppError::validation_error(
                "name",
                "名称不能为空",
                "NAME_EMPTY",
            ));
        }
        if !Area::validate_color(&request.color) {
            return Err(AppError::validation_error(
                "color",
                "颜色格式无效",
                "INVALID_COLOR",
            ));
        }

        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 2. 检查名称唯一性
        let exists = database::check_name_exists_in_tx(&mut tx, &request.name).await?;
        if exists {
            return Err(AppError::conflict("Area 名称已存在"));
        }

        // 3. 生成 ID 和时间戳
        let area_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // 4. 创建 Area
        let area = Area {
            id: area_id,
            name: request.name,
            color: request.color,
            parent_area_id: request.parent_area_id,
            created_at: now,
            updated_at: now,
            is_deleted: false,
        };

        database::insert_area_in_tx(&mut tx, &area).await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

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

// ==================== 数据访问层 ====================
mod database {
    use super::*;

    pub async fn check_name_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        name: &str,
    ) -> AppResult<bool> {
        let query = "SELECT COUNT(*) FROM areas WHERE name = ? AND is_deleted = false";
        let count: i64 = sqlx::query_scalar(query)
            .bind(name)
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(count > 0)
    }

    pub async fn insert_area_in_tx(tx: &mut Transaction<'_, Sqlite>, area: &Area) -> AppResult<()> {
        let query = r#"
            INSERT INTO areas (id, name, color, parent_area_id, created_at, updated_at, is_deleted)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(area.id.to_string())
            .bind(&area.name)
            .bind(&area.color)
            .bind(area.parent_area_id.map(|id| id.to_string()))
            .bind(area.created_at.to_rfc3339())
            .bind(area.updated_at.to_rfc3339())
            .bind(area.is_deleted)
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(())
    }
}
