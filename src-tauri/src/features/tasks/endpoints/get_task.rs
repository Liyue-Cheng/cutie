/// 获取任务详情 API - 单文件组件
///
/// 按照单文件组件模式实现
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    entities::{Task, TaskDetailDto},
    features::{
        shared::repositories::AreaRepository,
        tasks::shared::{repositories::TaskScheduleRepository, TaskAssembler},
    },
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

        // 3. 查询 schedules 历史（✅ 使用共享 Repository）
        let schedules = TaskScheduleRepository::get_all_for_task(pool, task_id).await?;

        // 4. ✅ 关键：根据实际 schedules 判断 schedule_status
        // 如果有任何 schedule 记录，状态就是 scheduled
        task_card.schedule_status = if !schedules.is_empty() {
            crate::entities::ScheduleStatus::Scheduled
        } else {
            crate::entities::ScheduleStatus::Staging
        };

        // 5. 获取其他关联信息（✅ 使用共享 Repository）
        if let Some(area_id) = task.area_id {
            task_card.area = AreaRepository::get_summary(pool, area_id).await?;
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

    pub async fn find_task_by_id(
        pool: &sqlx::SqlitePool,
        task_id: Uuid,
    ) -> AppResult<Option<Task>> {
        use crate::features::tasks::shared::repositories::TaskRepository;
        TaskRepository::find_by_id(pool, task_id).await
    }
}

// ✅ 已迁移到共享 Repository：
// - TaskRepository::find_by_id
// - TaskScheduleRepository::get_all_for_task
// - AreaRepository::get_summary
