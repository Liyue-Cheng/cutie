/// 创建任务 API - 单文件组件
///
/// 按照单文件组件模式实现
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};

use crate::{
    entities::{CreateTaskRequest, ScheduleStatus, Task, TaskCardDto},
    features::tasks::shared::{repositories::TaskRepository, TaskAssembler},
    shared::{
        core::{AppError, AppResult},
        http::error_handler::created_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `create_task`

## API端点
POST /api/tasks

## 预期行为简介
创建一个新任务。

## 输入输出规范
- **前置条件**:
  - title 不能为空，长度必须在 1-255 之间
  - area_id（如果提供）必须存在
- **后置条件**:
  - 在 tasks 表中创建新任务
  - 返回完整的 TaskCardDto

## 边界情况
- 如果 title 为空，返回 400 Bad Request
- 如果 title 超过 255 字符，返回 400 Bad Request
- 如果 area_id 不存在，目前正常返回

## 预期副作用
- 插入一条 tasks 记录

## 事务保证
- 所有数据库操作在单个事务中执行
- 如果任何步骤失败，整个操作回滚

## 请求/响应示例
Request:
{
  "title": "新任务",
  "glance_note": "快速笔记",
  "area_id": "..."
}

Response: 201 Created
{
  "id": "...",
  "title": "新任务",
  "schedule_status": "staging",
  ...
}
*/

// ==================== HTTP 处理器 ====================
/// 创建任务的 HTTP 处理器
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateTaskRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(task_card) => created_response(task_card).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate_create_request(request: &CreateTaskRequest) -> AppResult<()> {
        // 验证标题
        if request.title.trim().is_empty() {
            return Err(AppError::validation_error(
                "title",
                "任务标题不能为空",
                "TITLE_EMPTY",
            ));
        }

        if request.title.len() > 255 {
            return Err(AppError::validation_error(
                "title",
                "任务标题不能超过255个字符",
                "TITLE_TOO_LONG",
            ));
        }

        // 验证预估时长
        if let Some(duration) = request.estimated_duration {
            if duration < 0 {
                return Err(AppError::validation_error(
                    "estimated_duration",
                    "预估时长不能为负数",
                    "DURATION_NEGATIVE",
                ));
            }
            if duration > 24 * 60 * 7 {
                return Err(AppError::validation_error(
                    "estimated_duration",
                    "预估时长不能超过一周",
                    "DURATION_TOO_LONG",
                ));
            }
        }

        // 验证子任务数量
        if let Some(subtasks) = &request.subtasks {
            if subtasks.len() > 50 {
                return Err(AppError::validation_error(
                    "subtasks",
                    "子任务数量不能超过50个",
                    "TOO_MANY_SUBTASKS",
                ));
            }
        }

        Ok(())
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;
    use crate::features::shared::TransactionHelper;

    pub async fn execute(
        app_state: &AppState,
        request: CreateTaskRequest,
    ) -> AppResult<TaskCardDto> {
        // 1. 验证请求
        validation::validate_create_request(&request)?;

        // 2. 开始事务（✅ 使用 TransactionHelper）
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 3. 生成 UUID 和时间戳
        let task_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // 4. 创建任务实体
        let task = Task {
            id: task_id,
            title: request.title.clone(),
            glance_note: request.glance_note.clone(),
            detail_note: request.detail_note.clone(),
            estimated_duration: request.estimated_duration,
            subtasks: request.subtasks.clone(),
            project_id: None,
            area_id: request.area_id,
            due_date: request.due_date,
            due_date_type: request.due_date_type,
            completed_at: None,
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

        // 5. 插入任务到数据库（✅ 使用共享 Repository）
        TaskRepository::insert_in_tx(&mut tx, &task).await?;

        // 6. 提交事务（✅ 使用 TransactionHelper）
        TransactionHelper::commit(tx).await?;

        // 7. 组装返回的 TaskCardDto（✅ area_id 已由 TaskAssembler 填充）
        let mut task_card = TaskAssembler::task_to_card_basic(&task);
        task_card.schedule_status = ScheduleStatus::Staging;

        Ok(task_card)
    }
}

// ==================== 数据访问层 ====================
// ✅ 已迁移到共享 Repository：
// - TaskRepository::insert_in_tx
