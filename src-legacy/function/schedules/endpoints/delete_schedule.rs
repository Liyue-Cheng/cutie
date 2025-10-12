/// 删除单日日程 API - 单文件组件
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::TaskSchedule,
    crate::infra::core::{AppError, AppResult},
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `delete_schedule`

## API端点
DELETE /api/schedules/{id}

## 预期行为简介
删除一个具体的、单一的日程安排。幂等操作。

## 输入输出规范
- **前置条件**: `id` 必须是有效的日程ID。
- **后置条件**: 返回 `204 No Content`。
- **不变量**: 无。

## 边界情况
- 日程不存在: 幂等地返回 `204`。

## 预期副作用
- 从 `task_schedules` 表删除1条记录。
- 删除对应的 `ordering` 记录（`DAILY_KANBAN` 上下文）。
- 如果任务没有其他日程，在 `MISC` 上下文中创建排序记录（回归Staging）。
- 所有数据库写入在单个事务中。
*/

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(State(app_state): State<AppState>, Path(schedule_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, schedule_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, schedule_id: Uuid) -> AppResult<()> {
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
        })?;

        // 1. 验证日程存在（幂等）
        let schedule = match database::find_schedule_by_id(&mut tx, schedule_id).await? {
            Some(s) => s,
            None => {
                // 幂等：日程不存在也返回成功
                return Ok(());
            }
        };

        // 2. 核心操作：删除日程
        database::delete_schedule_in_tx(&mut tx, schedule_id).await?;

        // 3. 排序处理：删除对应的 DAILY_KANBAN 排序记录
        database::delete_ordering_for_schedule(&mut tx, schedule.task_id, schedule.scheduled_day)
            .await?;

        // 4. 回归Staging检查
        let has_other_schedules =
            database::task_has_other_schedules(&mut tx, schedule.task_id).await?;

        if !has_other_schedules {
            // 任务没有其他日程了，在MISC上下文中创建排序记录
            database::create_misc_ordering(&mut tx, schedule.task_id).await?;
        }

        // 5. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(())
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
mod database {
    use super::*;
    use chrono::DateTime;

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

    pub async fn delete_schedule_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        schedule_id: Uuid,
    ) -> AppResult<()> {
        sqlx::query("DELETE FROM task_schedules WHERE id = ?")
            .bind(schedule_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
            })?;

        Ok(())
    }

    pub async fn delete_ordering_for_schedule(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        day: DateTime<chrono::Utc>,
    ) -> AppResult<()> {
        // 使用日期的 RFC3339 字符串作为 context_id，而不是时间戳
        let context_id = day.to_rfc3339();

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
        .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e)))?;

        Ok(())
    }

    pub async fn task_has_other_schedules(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM task_schedules WHERE task_id = ?")
                .bind(task_id.to_string())
                .fetch_one(&mut **tx)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
                })?;

        Ok(count > 0)
    }

    pub async fn create_misc_ordering(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<()> {
        let now = Utc::now();

        // 获取MISC上下文的最大排序值
        let max_sort_order: Option<String> = sqlx::query_scalar(
            "SELECT MAX(sort_order) FROM ordering WHERE context_type = 'MISC' AND context_id = ''",
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e)))?;

        let new_sort_order = match max_sort_order {
            Some(max) => crate::infra::core::utils::sort_order_utils::get_rank_after(&max)
                .unwrap_or_else(|_| {
                    crate::infra::core::utils::sort_order_utils::generate_initial_sort_order()
                }),
            None => crate::infra::core::utils::sort_order_utils::generate_initial_sort_order(),
        };

        sqlx::query(
            r#"
            INSERT INTO ordering (id, context_type, context_id, task_id, sort_order, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind("MISC")
        .bind("")
        .bind(task_id.to_string())
        .bind(&new_sort_order)
        .bind(now.to_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e)))?;

        Ok(())
    }
}
