/// 获取所有领域 API - 单文件组件
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::{Sqlite, Transaction};

use crate::{
    entities::Area,
    shared::core::{AppError, AppResult},
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `list_areas`

## API端点
GET /api/areas

## 预期行为简介
获取所有未删除的领域列表。

## 输入输出规范
- **前置条件**: 无。
- **后置条件**: 返回 `200 OK` 和领域数组。
- **不变量**: 无。

## 边界情况
- 没有领域: 返回空数组。

## 预期副作用
- 无副作用（只读操作）。
*/

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(State(app_state): State<AppState>) -> Response {
    match logic::execute(&app_state).await {
        Ok(areas) => Json(areas).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState) -> AppResult<Vec<Area>> {
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        let areas = database::find_all_areas_in_tx(&mut tx).await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(areas)
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
mod database {
    use super::*;
    use crate::entities::area::AreaRow;

    pub async fn find_all_areas_in_tx(tx: &mut Transaction<'_, Sqlite>) -> AppResult<Vec<Area>> {
        let rows = sqlx::query_as::<_, AreaRow>(
            r#"
            SELECT id, name, color, parent_area_id, created_at, updated_at, is_deleted
            FROM areas
            WHERE deleted_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        rows.into_iter()
            .map(|r| {
                Area::try_from(r).map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::QueryError(e))
                })
            })
            .collect()
    }
}
