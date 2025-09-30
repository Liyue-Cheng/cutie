/// 获取任务详情 API - 单文件组件
///
/// 按照单文件组件模式实现
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    entities::{Task, TaskCardDto},
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
        Ok(task_card) => success_response(task_card).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, task_id: Uuid) -> AppResult<TaskCardDto> {
        let pool = app_state.db_pool();

        // 1. 查询任务
        let task = database::find_task_by_id(pool, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 2. 组装 TaskCardDto
        let task_card = TaskAssembler::task_to_card_basic(&task);

        // TODO: 可以进一步组装完整信息（area, schedule_info 等）

        Ok(task_card)
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
}
