/// 删除领域 API - 单文件组件
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::Area,
    shared::core::{AppError, AppResult, ValidationError},
    startup::AppState,
};

pub async fn handle(State(app_state): State<AppState>, Path(area_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, area_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => err.into_response(),
    }
}

mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, area_id: Uuid) -> AppResult<()> {
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        let area = match database::find_area_by_id_in_tx(&mut tx, area_id).await? {
            Some(a) => a,
            None => {
                return Ok(());
            }
        };

        // 边界检查：确保领域未被使用
        let tasks_count = database::count_tasks_in_area(&mut tx, area_id).await?;
        if tasks_count > 0 {
            return Err(AppError::conflict(
                "无法删除尚在使用的领域（有任务关联）",
            ));
        }

        let projects_count = database::count_projects_in_area(&mut tx, area_id).await?;
        if projects_count > 0 {
            return Err(AppError::conflict(
                "无法删除尚在使用的领域（有项目关联）",
            ));
        }

        let now = app_state.clock().now_utc();
        database::delete_area_in_tx(&mut tx, area.id, now).await?;

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
    use crate::entities::area::AreaRow;
    use chrono::DateTime;

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

    pub async fn count_tasks_in_area(
        tx: &mut Transaction<'_, Sqlite>,
        area_id: Uuid,
    ) -> AppResult<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM tasks WHERE area_id = ? AND is_deleted = false",
        )
        .bind(area_id.to_string())
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        Ok(count)
    }

    pub async fn count_projects_in_area(
        tx: &mut Transaction<'_, Sqlite>,
        area_id: Uuid,
    ) -> AppResult<i64> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM projects WHERE area_id = ? AND is_deleted = false",
        )
        .bind(area_id.to_string())
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        Ok(count)
    }

    pub async fn delete_area_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        area_id: Uuid,
        updated_at: DateTime<Utc>,
    ) -> AppResult<()> {
        sqlx::query("UPDATE areas SET is_deleted = true, updated_at = ? WHERE id = ?")
            .bind(updated_at.to_rfc3339())
            .bind(area_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(())
    }
}
