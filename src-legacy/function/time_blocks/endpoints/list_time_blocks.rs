/// 查询时间块列表 API - 单文件组件
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use sqlx::{Sqlite, Transaction};

use crate::{
    entities::TimeBlock,
    crate::infra::core::{AppError, AppResult, ValidationError},
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `list_time_blocks`

## API端点
GET /api/time-blocks?date=2024-01-15

## 预期行为简介
查询指定日期的所有时间块。

## 输入输出规范
- **前置条件**: 查询参数 `date` 必须是 YYYY-MM-DD 格式的有效日期。
- **后置条件**: 返回 `200 OK` 和时间块数组（按开始时间排序）。
- **不变量**: 无。

## 边界情况
- 没有时间块: 返回空数组。
- date 格式无效: 返回 `422 Unprocessable Entity`。

## 预期副作用
- 无副作用（只读操作）。
*/

#[derive(Deserialize)]
pub struct ListTimeBlocksQuery {
    pub date: String,
}

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Query(query): Query<ListTimeBlocksQuery>,
) -> Response {
    match logic::execute(&app_state, query).await {
        Ok(blocks) => Json(blocks).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 (Validation Layer) ====================
mod validation {
    use super::*;

    pub fn validate_query(query: &ListTimeBlocksQuery) -> Result<NaiveDate, Vec<ValidationError>> {
        let date = NaiveDate::parse_from_str(&query.date, "%Y-%m-%d").map_err(|_| {
            vec![ValidationError::new(
                "date",
                "Invalid date format, expected YYYY-MM-DD",
                "INVALID_DATE_FORMAT",
            )]
        })?;

        Ok(date)
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        query: ListTimeBlocksQuery,
    ) -> AppResult<Vec<TimeBlock>> {
        let date = validation::validate_query(&query).map_err(AppError::ValidationFailed)?;

        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
        })?;

        let blocks = database::find_time_blocks_for_day_in_tx(&mut tx, date).await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(blocks)
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
mod database {
    use super::*;
    use crate::entities::time_block::TimeBlockRow;

    pub async fn find_time_blocks_for_day_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        date: NaiveDate,
    ) -> AppResult<Vec<TimeBlock>> {
        // 计算当天的时间范围
        let day_start = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let day_end = day_start + chrono::Duration::days(1);

        let rows = sqlx::query_as::<_, TimeBlockRow>(
            r#"
            SELECT id, title, glance_note, detail_note, start_time, end_time,
                   area_id, created_at, updated_at, is_deleted, source_info,
                   external_source_id, external_source_provider, external_source_metadata,
                   recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            FROM time_blocks
            WHERE start_time >= ? AND start_time < ?
            AND deleted_at IS NULL
            ORDER BY start_time ASC
            "#,
        )
        .bind(day_start.to_rfc3339())
        .bind(day_end.to_rfc3339())
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e)))?;

        rows.into_iter()
            .map(|r| {
                TimeBlock::try_from(r).map_err(|e| {
                    AppError::DatabaseError(crate::infra::core::DbError::QueryError(e))
                })
            })
            .collect()
    }
}
