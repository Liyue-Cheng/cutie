/// 记录努力 API - 单文件组件
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use chrono::Utc;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{Outcome, TaskSchedule},
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `log_presence`

## API端点
POST /api/schedules/{id}/presence

## 预期行为简介
为指定的日程记录"努力已付出"。幂等操作。

## 输入输出规范
- **前置条件**: `id` 必须是有效的日程ID。
- **后置条件**: 返回 `200 OK` 和更新后的 `TaskSchedule` 对象，其 `outcome` 为 `PRESENCE_LOGGED`。
- **不变量**: 日程的 `outcome` 不能是 `COMPLETED_ON_DAY`。

## 边界情况
- 日程不存在: 返回 `404 Not Found`。
- 日程已完成（`COMPLETED_ON_DAY`）: 返回 `409 Conflict`。
- 日程已经是 `PRESENCE_LOGGED`: 幂等地返回 `200 OK`。

## 预期副作用
- 更新 `task_schedules` 表中的1条记录。
- 所有数据库写入在单个事务中。
*/

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(State(app_state): State<AppState>, Path(schedule_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, schedule_id).await {
        Ok(schedule) => success_response(schedule).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, schedule_id: Uuid) -> AppResult<TaskSchedule> {
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
        })?;

        // 1. 验证日程存在
        let schedule = database::find_schedule_by_id(&mut tx, schedule_id)
            .await?
            .ok_or_else(|| AppError::not_found("Schedule", schedule_id.to_string()))?;

        // 2. 检查日程状态
        if schedule.outcome == Outcome::CompletedOnDay {
            return Err(AppError::conflict(
                "Cannot log presence for a completed schedule",
            ));
        }

        // 3. 幂等检查
        if schedule.outcome == Outcome::PresenceLogged {
            return Ok(schedule);
        }

        // 4. 核心操作：更新 outcome
        let updated_schedule =
            database::update_outcome_in_tx(&mut tx, schedule_id, Outcome::PresenceLogged).await?;

        // 5. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(updated_schedule)
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
mod database {
    use super::*;

    pub async fn find_schedule_by_id(
        tx: &mut Transaction<'_, Sqlite>,
        schedule_id: Uuid,
    ) -> AppResult<Option<TaskSchedule>> {
        let row = sqlx::query_as::<_, crate::entities::schedule::TaskScheduleRow>(
            r#"
            SELECT id, task_id, scheduled_day, outcome, created_at, updated_at
            FROM task_schedules WHERE id = ?
            "#,
        )
        .bind(schedule_id.to_string())
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e)))?;

        row.map(|r| TaskSchedule::try_from(r))
            .transpose()
            .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::QueryError(e)))
    }

    pub async fn update_outcome_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        schedule_id: Uuid,
        outcome: Outcome,
    ) -> AppResult<TaskSchedule> {
        let now = Utc::now();
        let outcome_json =
            serde_json::to_string(&outcome).unwrap_or_else(|_| "\"PLANNED\"".to_string());

        let row = sqlx::query_as::<_, crate::entities::schedule::TaskScheduleRow>(
            r#"
            UPDATE task_schedules 
            SET outcome = ?, updated_at = ?
            WHERE id = ?
            RETURNING id, task_id, scheduled_day, outcome, created_at, updated_at
            "#,
        )
        .bind(outcome_json)
        .bind(now.to_rfc3339())
        .bind(schedule_id.to_string())
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e)))?;

        TaskSchedule::try_from(row)
            .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::QueryError(e)))
    }
}
