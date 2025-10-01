/// 获取任务详情 API - 单文件组件
///
/// 按照单文件组件模式实现
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    entities::{DailyOutcome, ScheduleRecord, Task, TaskDetailDto},
    features::tasks::shared::TaskAssembler,
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `get_task`

## API端点
GET /api/tasks/{id}

## 预期行为简介
返回指定任务的完整卡片信息，用于调试和详情查看。

## 输入输出规范
- **前置条件**: task_id 必须是有效的 UUID
- **后置条件**: 返回任务的 TaskCardDto

## 边界情况
- 如果任务不存在，返回 404 Not Found
- 如果任务已删除，返回 404 Not Found
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>, Path(task_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, task_id).await {
        Ok(task_detail) => success_response(task_detail).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, task_id: Uuid) -> AppResult<TaskDetailDto> {
        let pool = app_state.db_pool();

        // 1. 查询任务
        let task = database::find_task_by_id(pool, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 2. 组装基础 TaskCard
        let mut task_card = TaskAssembler::task_to_card_basic(&task);

        // 3. 查询 schedules 历史
        let schedules = database::get_task_schedules(pool, task_id).await?;

        // 4. ✅ 关键：根据实际 schedules 判断 schedule_status
        // 如果有任何 schedule 记录，状态就是 scheduled
        task_card.schedule_status = if !schedules.is_empty() {
            crate::entities::ScheduleStatus::Scheduled
        } else {
            crate::entities::ScheduleStatus::Staging
        };

        // 5. 获取其他关联信息

        if let Some(area_id) = task.area_id {
            task_card.area = database::get_area_summary(pool, area_id).await?;
        }

        // 6. 组装 TaskDetailDto
        let task_detail = TaskDetailDto {
            card: task_card,
            detail_note: task.detail_note.clone(),
            schedules,
            project: None, // TODO: 查询项目信息
            created_at: task.created_at,
            updated_at: task.updated_at,
        };

        Ok(task_detail)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::entities::TaskRow;

    pub async fn find_task_by_id(
        pool: &sqlx::SqlitePool,
        task_id: Uuid,
    ) -> AppResult<Option<Task>> {
        let query = r#"
            SELECT id, title, glance_note, detail_note, estimated_duration, 
                   subtasks, project_id, area_id, due_date, due_date_type, completed_at, 
                   created_at, updated_at, is_deleted, source_info,
                   external_source_id, external_source_provider, external_source_metadata,
                   recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            FROM tasks 
            WHERE id = ? AND is_deleted = false
        "#;

        let row = sqlx::query_as::<_, TaskRow>(query)
            .bind(task_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        match row {
            Some(r) => {
                let task = Task::try_from(r).map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::QueryError(e))
                })?;
                Ok(Some(task))
            }
            None => Ok(None),
        }
    }

    pub async fn get_area_summary(
        pool: &sqlx::SqlitePool,
        area_id: Uuid,
    ) -> AppResult<Option<crate::entities::task::response_dtos::AreaSummary>> {
        let query = "SELECT id, name, color FROM areas WHERE id = ? AND is_deleted = false";
        let result = sqlx::query_as::<_, (String, String, String)>(query)
            .bind(area_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(result.map(
            |(id, name, color)| crate::entities::task::response_dtos::AreaSummary {
                id: Uuid::parse_str(&id).unwrap(),
                name,
                color,
            },
        ))
    }

    /// 获取任务的所有日程记录
    pub async fn get_task_schedules(
        pool: &sqlx::SqlitePool,
        task_id: Uuid,
    ) -> AppResult<Vec<ScheduleRecord>> {
        let query = r#"
            SELECT scheduled_day, outcome
            FROM task_schedules
            WHERE task_id = ?
            ORDER BY scheduled_day ASC
        "#;

        let rows = sqlx::query_as::<_, (String, String)>(query)
            .bind(task_id.to_string())
            .fetch_all(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        let schedules = rows
            .into_iter()
            .filter_map(|(day_str, outcome_str)| {
                let day = chrono::DateTime::parse_from_rfc3339(&day_str)
                    .ok()?
                    .with_timezone(&chrono::Utc);

                let outcome = match outcome_str.as_str() {
                    "PLANNED" => DailyOutcome::Planned,
                    "PRESENCE_LOGGED" => DailyOutcome::PresenceLogged,
                    "COMPLETED_ON_DAY" => DailyOutcome::Completed,
                    "CARRIED_OVER" => DailyOutcome::CarriedOver,
                    _ => return None,
                };

                Some(ScheduleRecord { day, outcome })
            })
            .collect();

        Ok(schedules)
    }
}
