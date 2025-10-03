/// 获取所有未完成任务 API - 单文件组件
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    entities::{ScheduleStatus, Task, TaskCardDto},
    features::tasks::shared::TaskAssembler,
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `get_all_incomplete`

## API端点
GET /api/views/all-incomplete

## 预期行为简介
返回所有未完成的任务，无论是否已排期。

## 输入输出规范
- **前置条件**: 无
- **后置条件**: 返回所有 is_deleted = false 且 completed_at IS NULL 的任务

## 边界情况
- 如果没有任务，返回空数组
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

        // 1. 获取所有未完成任务
        let tasks = database::find_all_incomplete_tasks(pool).await?;

        // 2. 为每个任务组装 TaskCardDto
        let mut task_cards = Vec::new();
        for task in tasks {
            let task_card = assemble_task_card(&task, pool).await?;
            task_cards.push(task_card);
        }

        // 3. 按 created_at 倒序（最新的在前）
        task_cards.sort_by(|a, b| b.id.cmp(&a.id));

        Ok(task_cards)
    }

    /// 组装单个任务的 TaskCard（✅ area_id 已由 TaskAssembler 填充）
    async fn assemble_task_card(task: &Task, pool: &sqlx::SqlitePool) -> AppResult<TaskCardDto> {
        let mut card = TaskAssembler::task_to_card_basic(task);

        // 判断 schedule_status
        let has_schedule = database::has_any_schedule(pool, task.id).await?;
        card.schedule_status = if has_schedule {
            ScheduleStatus::Scheduled
        } else {
            ScheduleStatus::Staging
        };

        Ok(card)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::entities::TaskRow;

    pub async fn find_all_incomplete_tasks(pool: &sqlx::SqlitePool) -> AppResult<Vec<Task>> {
        let query = r#"
            SELECT 
                id, title, glance_note, detail_note, estimated_duration, 
                subtasks, project_id, area_id, due_date, due_date_type, completed_at, 
                created_at, updated_at, is_deleted, source_info,
                external_source_id, external_source_provider, external_source_metadata,
                recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            FROM tasks
            WHERE is_deleted = false AND completed_at IS NULL
            ORDER BY created_at DESC
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

    pub async fn has_any_schedule(pool: &sqlx::SqlitePool, task_id: Uuid) -> AppResult<bool> {
        let query = r#"
            SELECT COUNT(*) as count
            FROM task_schedules
            WHERE task_id = ?
        "#;

        let count: i64 = sqlx::query_scalar(query)
            .bind(task_id.to_string())
            .fetch_one(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(count > 0)
    }
}
