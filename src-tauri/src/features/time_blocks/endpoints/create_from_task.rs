/// 从任务创建时间块 API - 单文件组件
///
/// 专门处理"拖动任务到日历"的场景
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::{LinkedTaskSummary, ScheduleStatus, TaskCardDto, TimeBlock, TimeBlockViewDto},
    features::{
        tasks::shared::{
            repositories::{TaskRepository, TaskScheduleRepository, TaskTimeBlockLinkRepository},
            TaskAssembler,
        },
        time_blocks::shared::{repositories::TimeBlockRepository, TimeBlockConflictChecker},
    },
    shared::{
        core::{AppError, AppResult},
        http::error_handler::created_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `create_time_block_from_task`

## API端点
POST /api/time-blocks/from-task

## 预期行为简介
从拖动的任务创建时间块。这是专门为"拖动任务到日历"场景设计的端点。
会同时：
1. 创建时间块
2. 链接任务到时间块
3. 创建任务的日程记录（task_schedules）
4. 返回更新后的任务卡片

## 输入输出规范
- **前置条件**:
  - task_id 必须存在
  - start_time < end_time
  - 时间块不与现有时间块重叠
- **后置条件**:
  - 创建 time_blocks 记录
  - 创建 task_time_block_links 记录
  - 创建 task_schedules 记录
  - 返回时间块和更新后的任务

## 边界情况
- 如果任务不存在，返回 404
- 如果时间冲突，返回 409
*/

// ==================== 请求/响应结构 ====================
#[derive(Debug, Deserialize)]
pub struct CreateFromTaskRequest {
    pub task_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub title: Option<String>, // 可选，默认使用任务标题
}

#[derive(Debug, Serialize)]
pub struct CreateFromTaskResponse {
    pub time_block: TimeBlockViewDto,
    pub updated_task: TaskCardDto, // 更新后的任务（schedule_status = 'scheduled'）
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateFromTaskRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(response) => created_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate_request(request: &CreateFromTaskRequest) -> AppResult<()> {
        if request.start_time >= request.end_time {
            return Err(AppError::validation_error(
                "time_range",
                "开始时间必须早于结束时间",
                "INVALID_TIME_RANGE",
            ));
        }
        Ok(())
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        request: CreateFromTaskRequest,
    ) -> AppResult<CreateFromTaskResponse> {
        // 1. 验证
        validation::validate_request(&request)?;

        // 2. 开始事务
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 3. 检查任务是否存在（✅ 使用共享 Repository）
        let task = TaskRepository::find_by_id_in_tx(&mut tx, request.task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", request.task_id.to_string()))?;

        // 4. 检查时间冲突（✅ 使用共享 ConflictChecker）
        let has_conflict = TimeBlockConflictChecker::check_in_tx(
            &mut tx,
            &request.start_time,
            &request.end_time,
            None,
        )
        .await?;

        if has_conflict {
            return Err(AppError::conflict("该时间段与现有时间块重叠"));
        }

        // 5. 生成 UUID 和时间戳
        let block_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // 6. 创建时间块（使用任务标题或自定义标题）
        let title = request.title.or_else(|| Some(task.title.clone()));

        let time_block = TimeBlock {
            id: block_id,
            title,
            glance_note: None,
            detail_note: None,
            start_time: request.start_time,
            end_time: request.end_time,
            area_id: task.area_id, // 继承任务的 area
            created_at: now,
            updated_at: now,
            is_deleted: false,
            source_info: None,
            external_source_id: None,
            external_source_provider: None,
            external_source_metadata: None,
            recurrence_rule: None,
            recurrence_parent_id: None,
            recurrence_original_date: None,
            recurrence_exclusions: None,
        };

        TimeBlockRepository::insert_in_tx(&mut tx, &time_block).await?;

        // 7. 链接任务到时间块（✅ 使用共享 Repository）
        TaskTimeBlockLinkRepository::link_in_tx(&mut tx, request.task_id, block_id).await?;

        // 8. 创建日程记录（✅ 使用共享 Repository）
        let scheduled_day = request
            .start_time
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        let has_schedule = TaskScheduleRepository::has_schedule_for_day_in_tx(
            &mut tx,
            request.task_id,
            scheduled_day,
        )
        .await?;
        if !has_schedule {
            TaskScheduleRepository::create_in_tx(&mut tx, request.task_id, scheduled_day).await?;
        }

        // 9. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 10. 组装返回数据（✅ area_id 已直接从 time_block 获取）
        let time_block_view = TimeBlockViewDto {
            id: time_block.id,
            start_time: time_block.start_time,
            end_time: time_block.end_time,
            title: time_block.title,
            glance_note: time_block.glance_note,
            detail_note: time_block.detail_note,
            area_id: time_block.area_id,
            linked_tasks: vec![LinkedTaskSummary {
                id: task.id,
                title: task.title.clone(),
                is_completed: task.is_completed(),
            }],
            is_recurring: false,
        };

        // 11. 组装更新后的 TaskCard（✅ area_id 已由 TaskAssembler 填充）
        let mut updated_task = TaskAssembler::task_to_card_basic(&task);
        updated_task.schedule_status = ScheduleStatus::Scheduled; // 明确设置

        Ok(CreateFromTaskResponse {
            time_block: time_block_view,
            updated_task,
        })
    }
}

// ==================== 数据访问层 ====================
// ✅ 已全部迁移到共享 Repository：
// - TaskRepository::find_by_id_in_tx
// - TimeBlockConflictChecker::check_in_tx
// - TimeBlockRepository::insert_in_tx
// - TaskTimeBlockLinkRepository::link_in_tx
// - TaskScheduleRepository::has_schedule_for_day_in_tx, create_in_tx
