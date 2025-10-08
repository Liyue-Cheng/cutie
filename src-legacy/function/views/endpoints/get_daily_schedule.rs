/// 获取每日日程视图 API - 单文件组件
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use sqlx::{Sqlite, Transaction};

use crate::{
    entities::Task,
    shared::core::{AppError, AppResult, ValidationError},
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `get_daily_schedule`

## API端点
GET /api/views/daily-schedule?day=YYYY-MM-DD

## 预期行为简介
获取指定日期的所有日程任务，按排序顺序返回。

## 输入输出规范
- **前置条件**: 查询参数 `day` 必须是有效的日期格式 (YYYY-MM-DD)。
- **后置条件**: 返回 `200 OK` 和任务数组。
- **不变量**: 无。

## 边界情况
- 没有日程: 返回空数组。
- 日期格式无效: 返回 `422 Unprocessable Entity`。

## 预期副作用
- 无副作用（只读操作）。
*/

#[derive(Deserialize)]
pub struct DailyScheduleQuery {
    day: String,
}

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Query(query): Query<DailyScheduleQuery>,
) -> Response {
    match logic::execute(&app_state, query).await {
        Ok(tasks) => Json(tasks).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 (Validation Layer) ====================
mod validation {
    use super::*;

    pub fn validate_query(query: &DailyScheduleQuery) -> Result<NaiveDate, Vec<ValidationError>> {
        match NaiveDate::parse_from_str(&query.day, "%Y-%m-%d") {
            Ok(date) => Ok(date),
            Err(_) => Err(vec![ValidationError::new(
                "day",
                "日期格式无效，应为 YYYY-MM-DD",
                "INVALID_DATE_FORMAT",
            )]),
        }
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, query: DailyScheduleQuery) -> AppResult<Vec<Task>> {
        let date = validation::validate_query(&query).map_err(AppError::ValidationFailed)?;

        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        let tasks = database::find_tasks_for_day_in_tx(&mut tx, date).await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(tasks)
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
mod database {
    use super::*;
    use crate::entities::task::TaskRow;
    use chrono::DateTime;

    pub async fn find_tasks_for_day_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        date: NaiveDate,
    ) -> AppResult<Vec<Task>> {
        let day_start = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let day_end = day_start + chrono::Duration::days(1);
        // 使用日期的 RFC3339 字符串作为 context_id，而不是时间戳
        let context_id = day_start.to_rfc3339();

        let rows = sqlx::query_as::<_, TaskRow>(
            r#"
            SELECT DISTINCT t.id, t.title, t.glance_note, t.detail_note, t.estimated_duration,
                   t.subtasks, t.project_id, t.area_id, t.due_date, t.due_date_type, t.completed_at,
                   t.created_at, t.updated_at, t.is_deleted, t.source_info,
                   t.external_source_id, t.external_source_provider, t.external_source_metadata,
                   t.recurrence_rule, t.recurrence_parent_id, t.recurrence_original_date, t.recurrence_exclusions
            FROM tasks t
            INNER JOIN task_schedules ts ON t.id = ts.task_id
            LEFT JOIN ordering o ON t.id = o.task_id AND o.context_type = 'DAILY_KANBAN' AND o.context_id = ?
            WHERE ts.scheduled_day >= ? AND ts.scheduled_day < ?
            AND t.deleted_at IS NULL
            ORDER BY COALESCE(o.sort_order, 'z')
            "#,
        )
        .bind(&context_id)
        .bind(day_start) // SQLx自动转换 DateTime<Utc> -> TEXT
        .bind(day_end) // SQLx自动转换 DateTime<Utc> -> TEXT
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        rows.into_iter()
            .map(|r| {
                Task::try_from(r).map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::QueryError(e))
                })
            })
            .collect()
    }
}
