/// 链接日程到新的一天 API - 单文件组件
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{Task, TaskSchedule},
    shared::{
        core::{AppError, AppResult, ValidationError},
        http::error_handler::{created_response, success_response},
    },
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `link_schedule`

## API端点
POST /api/schedules

## 预期行为简介
为一个任务在新的日期上创建额外的日程。此操作是幂等的。

## 输入输出规范
- **前置条件**: 请求体 `LinkScheduleRequest` 必须包含有效的 `task_id` 和 `scheduled_day`。
- **后置条件**:
  - 如果日程是新创建的，返回 `201 Created` 和新的 `TaskSchedule` 对象。
  - 如果日程已存在，幂等地返回 `200 OK` 和已存在的 `TaskSchedule` 对象。
- **不变量**: 任务必须未被全局完成。

## 边界情况
- 任务不存在: 返回 `404 Not Found`。
- 任务已完成: 返回 `409 Conflict`。
- 日期格式无效: 返回 `422 Unprocessable Entity`。

## 预期副作用
- 可能在 `task_schedules` 表插入1条记录。
- 可能在 `ordering` 表插入1条记录。
- 所有数据库写入在单个事务中。

## 请求体
```json
{
  "task_id": "...",
  "scheduled_day": "YYYY-MM-DD"
}
```
*/

#[derive(Deserialize)]
pub struct LinkScheduleRequest {
    task_id: Uuid,
    scheduled_day: chrono::NaiveDate, // Serde自动处理YYYY-MM-DD格式
}

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<LinkScheduleRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok((schedule, created)) => {
            if created {
                created_response(schedule).into_response()
            } else {
                success_response(schedule).into_response()
            }
        }
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 (Validation Layer) ====================
mod validation {
    use super::*;
    use chrono::NaiveDate;

    pub fn validate_request(
        request: &LinkScheduleRequest,
    ) -> Result<NaiveDate, Vec<ValidationError>> {
        // Serde已经处理了日期格式验证，直接使用即可
        Ok(request.scheduled_day)
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
mod logic {
    use super::*;
    use crate::shared::core::time_utils;

    pub async fn execute(
        app_state: &AppState,
        request: LinkScheduleRequest,
    ) -> AppResult<(TaskSchedule, bool)> {
        let day = validation::validate_request(&request).map_err(AppError::ValidationFailed)?;
        let scheduled_day_utc = time_utils::normalize_to_day_start(
            day.and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap(),
        );

        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 1. 验证任务
        let task = database::find_task_by_id_in_tx(&mut tx, request.task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", request.task_id.to_string()))?;
        if task.is_completed() {
            return Err(AppError::conflict("无法为已完成的任务添加日程"));
        }

        // 2. 幂等检查
        if let Some(existing_schedule) = database::find_schedule_by_task_and_day_in_tx(
            &mut tx,
            request.task_id,
            scheduled_day_utc,
        )
        .await?
        {
            return Ok((existing_schedule, false));
        }

        // 3. 核心操作：创建日程
        let new_schedule_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();
        let new_schedule =
            TaskSchedule::new(new_schedule_id, request.task_id, scheduled_day_utc, now);
        let created_schedule = database::create_schedule_in_tx(&mut tx, &new_schedule).await?;

        // 4. 排序处理
        database::create_ordering_for_daily_kanban_in_tx(&mut tx, &created_schedule).await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok((created_schedule, true))
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
mod database {
    use super::*;
    use crate::{
        entities::{ordering::Ordering, ContextType, Outcome, TaskRow, TaskSchedule},
        shared::core::utils::sort_order_utils,
    };
    use chrono::{DateTime, Utc};

    pub async fn find_task_by_id_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<Option<Task>> {
        let row = sqlx::query_as::<_, TaskRow>(
            r#"
            SELECT * FROM tasks WHERE id = ? AND is_deleted = false
            "#,
        )
        .bind(task_id.to_string())
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        row.map(|r| Task::try_from(r))
            .transpose()
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }

    pub async fn find_schedule_by_task_and_day_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        day: DateTime<Utc>,
    ) -> AppResult<Option<TaskSchedule>> {
        // 使用日期范围比较：当天00:00:00 <= scheduled_day < 下一天00:00:00
        let day_start = day.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        let day_end = day_start + chrono::Duration::days(1);

        let day_start_str = day_start.to_rfc3339();
        let day_end_str = day_end.to_rfc3339();
        let task_id_str = task_id.to_string();

        let row = sqlx::query_as::<_, crate::entities::schedule::TaskScheduleRow>(
            r#"
            SELECT id, task_id, scheduled_day, outcome, created_at, updated_at
            FROM task_schedules 
            WHERE task_id = ? 
            AND scheduled_day >= ? 
            AND scheduled_day < ?
            "#,
        )
        .bind(task_id_str)
        .bind(day_start_str)
        .bind(day_end_str)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?;

        row.map(|r| TaskSchedule::try_from(r))
            .transpose()
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }

    pub async fn create_schedule_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        schedule: &TaskSchedule,
    ) -> AppResult<TaskSchedule> {
        // 将 Outcome 转换为数据库字符串格式（不带JSON引号）
        let outcome_str = match schedule.outcome {
            Outcome::Planned => "PLANNED",
            Outcome::PresenceLogged => "PRESENCE_LOGGED",
            Outcome::CompletedOnDay => "COMPLETED_ON_DAY",
            Outcome::CarriedOver => "CARRIED_OVER",
        };

        let row = sqlx::query_as::<_, crate::entities::schedule::TaskScheduleRow>(
            r#"
            INSERT INTO task_schedules (id, task_id, scheduled_day, outcome, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(schedule.id.to_string())
        .bind(schedule.task_id.to_string())
        .bind(schedule.scheduled_day.to_rfc3339())
        .bind(outcome_str)
        .bind(schedule.created_at.to_rfc3339())
        .bind(schedule.updated_at.to_rfc3339())
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?;

        TaskSchedule::try_from(row)
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }

    pub async fn create_ordering_for_daily_kanban_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        schedule: &TaskSchedule,
    ) -> AppResult<()> {
        // 使用日期的 RFC3339 字符串作为 context_id，而不是时间戳
        let context_id = schedule.scheduled_day.to_rfc3339();

        let max_sort_order: Option<String> = sqlx::query_scalar(
            "SELECT MAX(sort_order) FROM ordering WHERE context_type = ? AND context_id = ?",
        )
        .bind(ContextType::DailyKanban.to_string())
        .bind(&context_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?
        .flatten();

        let new_sort_order = match max_sort_order {
            Some(max_order) => sort_order_utils::get_rank_after(&max_order)?,
            None => sort_order_utils::generate_initial_sort_order(),
        };

        let ordering = Ordering::new(
            Uuid::new_v4(),
            ContextType::DailyKanban,
            context_id,
            schedule.task_id,
            new_sort_order,
            schedule.created_at,
        )?;

        sqlx::query(
            "INSERT INTO ordering (id, context_type, context_id, task_id, sort_order, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(ordering.id.to_string())
        .bind(ordering.context_type.to_string())
        .bind(ordering.context_id)
        .bind(ordering.task_id.to_string())
        .bind(ordering.sort_order)
        .bind(ordering.updated_at.to_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(())
    }
}
