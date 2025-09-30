/// 移动日程 API - 单文件组件
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::TaskSchedule,
    shared::{
        core::{AppError, AppResult, ValidationError},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `reschedule_task`

## API端点
PATCH /api/schedules/{id}

## 预期行为简介
将一个已存在的日程移动到新的日期。

## 输入输出规范
- **前置条件**: `id` 必须是有效的日程ID。请求体必须包含有效的 `new_scheduled_day`。
- **后置条件**: 返回 `200 OK` 和更新后的 `TaskSchedule` 对象。
- **不变量**: 任务必须未被全局完成。

## 边界情况
- 日程不存在: 返回 `404 Not Found`。
- 任务已完成: 返回 `409 Conflict`。
- 目标日期已有该任务的日程: 返回 `409 Conflict`。
- 日期格式无效: 返回 `422 Unprocessable Entity`。

## 预期副作用
- 更新 `task_schedules` 表中的1条记录。
- 删除旧的 `ordering` 记录，创建新的 `ordering` 记录。
- 所有数据库写入在单个事务中。

## 请求体
```json
{
  "new_scheduled_day": "YYYY-MM-DD"
}
```
*/

#[derive(Deserialize)]
pub struct RescheduleRequest {
    new_scheduled_day: String,
}

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(schedule_id): Path<Uuid>,
    Json(request): Json<RescheduleRequest>,
) -> Response {
    match logic::execute(&app_state, schedule_id, request).await {
        Ok(schedule) => success_response(schedule).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 (Validation Layer) ====================
mod validation {
    use super::*;

    pub fn validate_request(
        request: &RescheduleRequest,
    ) -> Result<NaiveDate, Vec<ValidationError>> {
        match NaiveDate::parse_from_str(&request.new_scheduled_day, "%Y-%m-%d") {
            Ok(day) => Ok(day),
            Err(_) => {
                let errors = vec![ValidationError::new(
                    "new_scheduled_day",
                    "日期格式无效，应为 YYYY-MM-DD",
                    "INVALID_DATE_FORMAT",
                )];
                Err(errors)
            }
        }
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        schedule_id: Uuid,
        request: RescheduleRequest,
    ) -> AppResult<TaskSchedule> {
        // 1. 验证请求
        let new_day = validation::validate_request(&request).map_err(AppError::ValidationFailed)?;

        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 2. 验证源日程存在
        let schedule = database::find_schedule_by_id(&mut tx, schedule_id)
            .await?
            .ok_or_else(|| AppError::not_found("Schedule", schedule_id.to_string()))?;

        // 3. 验证任务未完成
        let task = database::find_task_by_id(&mut tx, schedule.task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", schedule.task_id.to_string()))?;

        if task.completed_at.is_some() {
            return Err(AppError::conflict("Cannot reschedule a completed task"));
        }

        // 4. 检查目标日期冲突
        let new_day_dt = new_day.and_hms_opt(0, 0, 0).unwrap().and_utc();

        if let Some(existing) =
            database::find_schedule_by_task_and_day(&mut tx, schedule.task_id, new_day_dt).await?
        {
            if existing.id != schedule_id {
                return Err(AppError::conflict(
                    "Task already has a schedule on the target day",
                ));
            }
        }

        // 5. 获取旧日期用于删除排序
        let old_day = schedule.scheduled_day;

        // 6. 更新日程
        let updated_schedule = database::reschedule_in_tx(&mut tx, schedule_id, new_day_dt).await?;

        // 7. 更新排序：删除旧的，创建新的
        database::delete_ordering_for_schedule(&mut tx, schedule.task_id, old_day).await?;
        database::create_ordering_for_schedule(&mut tx, &updated_schedule).await?;

        // 8. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
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
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        row.map(|r| TaskSchedule::try_from(r))
            .transpose()
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }

    pub async fn find_task_by_id(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<Option<crate::entities::Task>> {
        let row = sqlx::query_as::<_, crate::entities::task::TaskRow>(
            r#"
            SELECT id, title, glance_note, detail_note, estimated_duration, 
                   subtasks, project_id, area_id, due_date, due_date_type, completed_at, 
                   created_at, updated_at, is_deleted, source_info,
                   external_source_id, external_source_provider, external_source_metadata,
                   recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            FROM tasks WHERE id = ? AND is_deleted = false
            "#,
        )
        .bind(task_id.to_string())
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        row.map(|r| crate::entities::Task::try_from(r))
            .transpose()
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }

    pub async fn find_schedule_by_task_and_day(
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
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        row.map(|r| TaskSchedule::try_from(r))
            .transpose()
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }

    pub async fn reschedule_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        schedule_id: Uuid,
        new_day: DateTime<Utc>,
    ) -> AppResult<TaskSchedule> {
        let now = Utc::now();
        let row = sqlx::query_as::<_, crate::entities::schedule::TaskScheduleRow>(
            r#"
            UPDATE task_schedules 
            SET scheduled_day = ?, updated_at = ?
            WHERE id = ?
            RETURNING id, task_id, scheduled_day, outcome, created_at, updated_at
            "#,
        )
        .bind(new_day.to_rfc3339())
        .bind(now.to_rfc3339())
        .bind(schedule_id.to_string())
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        TaskSchedule::try_from(row)
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }

    pub async fn delete_ordering_for_schedule(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        day: DateTime<Utc>,
    ) -> AppResult<()> {
        let context_id = day.timestamp_millis().to_string();

        sqlx::query(
            r#"
            DELETE FROM ordering 
            WHERE context_type = 'DAILY_KANBAN' 
            AND context_id = ? 
            AND task_id = ?
            "#,
        )
        .bind(&context_id)
        .bind(task_id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        Ok(())
    }

    pub async fn create_ordering_for_schedule(
        tx: &mut Transaction<'_, Sqlite>,
        schedule: &TaskSchedule,
    ) -> AppResult<()> {
        let context_id = schedule.scheduled_day.timestamp_millis().to_string();
        let now = Utc::now();

        // 获取当前最大排序值
        let max_sort_order: Option<String> = sqlx::query_scalar(
            "SELECT MAX(sort_order) FROM ordering WHERE context_type = ? AND context_id = ?",
        )
        .bind("DAILY_KANBAN")
        .bind(&context_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        let new_sort_order = match max_sort_order {
            Some(max) => crate::shared::core::utils::sort_order_utils::get_rank_after(&max)
                .unwrap_or_else(|_| {
                    crate::shared::core::utils::sort_order_utils::generate_initial_sort_order()
                }),
            None => crate::shared::core::utils::sort_order_utils::generate_initial_sort_order(),
        };

        sqlx::query(
            r#"
            INSERT INTO ordering (id, context_type, context_id, task_id, sort_order, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind("DAILY_KANBAN")
        .bind(&context_id)
        .bind(schedule.task_id.to_string())
        .bind(&new_sort_order)
        .bind(now.to_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        Ok(())
    }
}
