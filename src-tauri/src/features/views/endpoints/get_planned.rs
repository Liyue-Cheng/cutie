/// 获取已排期任务 API - 单文件组件
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    entities::{task::response_dtos::AreaSummary, ScheduleStatus, Task, TaskCardDto},
    features::tasks::shared::TaskAssembler,
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `get_planned`

## API端点
GET /api/views/planned

## 预期行为简介
返回所有已排期（scheduled）的未完成任务。

## 输入输出规范
- **前置条件**: 无
- **后置条件**: 返回所有有 task_schedules 记录的未完成任务

## 边界情况
- 如果没有已排期任务，返回空数组
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>) -> Response {
    match logic::execute(&app_state).await {
        Ok(task_cards) => success_response(task_cards).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState) -> AppResult<Vec<TaskCardDto>> {
        let pool = app_state.db_pool();

        // 1. 获取所有已排期任务
        let tasks = database::find_planned_tasks(pool).await?;

        // 2. 为每个任务组装 TaskCardDto
        let mut task_cards = Vec::new();
        for task in tasks {
            let task_card = assemble_task_card(&task, pool).await?;
            task_cards.push(task_card);
        }

        // 3. 按最近的 scheduled_day 排序
        task_cards.sort_by(|a, b| a.sort_order.cmp(&b.sort_order));

        Ok(task_cards)
    }

    /// 组装单个任务的 TaskCard
    async fn assemble_task_card(task: &Task, pool: &sqlx::SqlitePool) -> AppResult<TaskCardDto> {
        let mut card = TaskAssembler::task_to_card_basic(task);

        // 明确设置为 scheduled
        card.schedule_status = ScheduleStatus::Scheduled;

        // 获取 sort_order
        let sort_order = database::get_task_sort_order(pool, task.id).await?;
        card.sort_order = sort_order;

        // 获取 area
        if let Some(area_id) = task.area_id {
            card.area = database::get_area_summary(pool, area_id).await?;
        }

        Ok(card)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::entities::TaskRow;

    pub async fn find_planned_tasks(pool: &sqlx::SqlitePool) -> AppResult<Vec<Task>> {
        let query = r#"
            SELECT DISTINCT
                t.id, t.title, t.glance_note, t.detail_note, t.estimated_duration, 
                t.subtasks, t.project_id, t.area_id, t.due_date, t.due_date_type, t.completed_at, 
                t.created_at, t.updated_at, t.is_deleted, t.source_info,
                t.external_source_id, t.external_source_provider, t.external_source_metadata,
                t.recurrence_rule, t.recurrence_parent_id, t.recurrence_original_date, t.recurrence_exclusions
            FROM tasks t
            INNER JOIN task_schedules ts ON t.id = ts.task_id
            WHERE t.is_deleted = false AND t.completed_at IS NULL
            ORDER BY ts.scheduled_day ASC, t.created_at DESC
        "#;

        let rows = sqlx::query_as::<_, TaskRow>(query)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        let tasks: Result<Vec<Task>, _> = rows.into_iter().map(Task::try_from).collect();

        tasks.map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }

    pub async fn get_task_sort_order(pool: &sqlx::SqlitePool, task_id: Uuid) -> AppResult<String> {
        let query = r#"
            SELECT sort_order 
            FROM orderings 
            WHERE context_type = 'MISC' AND context_id = 'staging' AND task_id = ?
        "#;

        let result = sqlx::query_scalar::<_, String>(query)
            .bind(task_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(result.unwrap_or_else(|| "zzz".to_string()))
    }

    pub async fn get_area_summary(
        pool: &sqlx::SqlitePool,
        area_id: Uuid,
    ) -> AppResult<Option<AreaSummary>> {
        let query = r#"
            SELECT id, name, color
            FROM areas
            WHERE id = ? AND is_deleted = false
        "#;

        let result = sqlx::query_as::<_, (String, String, String)>(query)
            .bind(area_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(result.map(|(id, name, color)| AreaSummary {
            id: Uuid::parse_str(&id).unwrap(),
            name,
            color,
        }))
    }
}
