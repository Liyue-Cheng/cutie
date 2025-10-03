/// 创建时间块 API - 单文件组件
///
/// 按照单文件组件模式实现
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};

use crate::{
    entities::{CreateTimeBlockRequest, TimeBlock, TimeBlockViewDto},
    features::{
        tasks::shared::{
            assemblers::LinkedTaskAssembler,
            repositories::{TaskScheduleRepository, TaskTimeBlockLinkRepository},
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
CABC for `create_time_block`

## API端点
POST /api/time-blocks

## 预期行为简介
创建一个新的时间块，并可选地将其与一个或多个任务关联。
支持 Cutie 的核心特性：任务与时间块多对多连接。

## 输入输出规范
- **前置条件**:
  - start_time < end_time
  - 时间块不与现有时间块重叠（关键约束）
  - linked_task_ids 中的任务必须存在
- **后置条件**:
  - 在 time_blocks 表中创建新时间块
  - 在 task_time_block_links 表中创建关联记录
  - 返回完整的 TimeBlockViewDto

## 边界情况
- 如果时间范围无效，返回 400 Bad Request
- 如果与现有时间块重叠，返回 409 Conflict
- 如果关联的任务不存在，返回 404 Not Found

## 预期副作用
- 插入一条 time_blocks 记录
- 插入 N 条 task_time_block_links 记录

## 事务保证
- 所有数据库操作在单个事务中执行
- 如果任何步骤失败，整个操作回滚
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateTimeBlockRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(time_block_view) => created_response(time_block_view).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate_create_request(request: &CreateTimeBlockRequest) -> AppResult<()> {
        // 验证时间范围
        if request.start_time >= request.end_time {
            return Err(AppError::validation_error(
                "time_range",
                "开始时间必须早于结束时间",
                "INVALID_TIME_RANGE",
            ));
        }

        // 验证时间不在过去太远（可选，根据需求）
        // 验证标题长度（如果有）
        if let Some(title) = &request.title {
            if title.len() > 255 {
                return Err(AppError::validation_error(
                    "title",
                    "标题不能超过255个字符",
                    "TITLE_TOO_LONG",
                ));
            }
        }

        Ok(())
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        request: CreateTimeBlockRequest,
    ) -> AppResult<TimeBlockViewDto> {
        // 1. 验证请求
        validation::validate_create_request(&request)?;

        // 2. 开始事务
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 3. 检查时间冲突（✅ 使用共享 ConflictChecker）
        let has_conflict = TimeBlockConflictChecker::check_in_tx(
            &mut tx,
            &request.start_time,
            &request.end_time,
            None, // 新建时没有要排除的ID
        )
        .await?;

        if has_conflict {
            return Err(AppError::conflict(
                "该时间段与现有时间块重叠，时间块不允许重叠",
            ));
        }

        // 4. 生成 UUID 和时间戳
        let block_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // 5. 创建时间块实体
        let time_block = TimeBlock {
            id: block_id,
            title: request.title.clone(),
            glance_note: request.glance_note.clone(),
            detail_note: request.detail_note.clone(),
            start_time: request.start_time,
            end_time: request.end_time,
            area_id: request.area_id,
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

        // 6. 插入时间块到数据库（✅ 使用共享 Repository）
        TimeBlockRepository::insert_in_tx(&mut tx, &time_block).await?;

        // 7. 创建任务链接
        if let Some(task_ids) = &request.linked_task_ids {
            // 从 start_time 提取日期（UTC 零点）
            let scheduled_day = request
                .start_time
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();

            for task_id in task_ids {
                // 7.1. 创建任务与时间块的链接（✅ 使用共享 Repository）
                TaskTimeBlockLinkRepository::link_in_tx(&mut tx, *task_id, block_id).await?;

                // 7.2. 创建任务的日程记录（如果还没有）（✅ 使用共享 Repository）
                // 这样任务就从 staging 移到 scheduled 状态
                let has_schedule = TaskScheduleRepository::has_schedule_for_day_in_tx(
                    &mut tx,
                    *task_id,
                    scheduled_day,
                )
                .await?;
                if !has_schedule {
                    TaskScheduleRepository::create_in_tx(&mut tx, *task_id, scheduled_day).await?;
                }
            }
        }

        // 8. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 9. 组装返回的 TimeBlockViewDto（✅ area_id 已直接从 time_block 获取）
        let mut time_block_view = TimeBlockViewDto {
            id: time_block.id,
            start_time: time_block.start_time,
            end_time: time_block.end_time,
            title: time_block.title,
            glance_note: time_block.glance_note,
            detail_note: time_block.detail_note,
            area_id: time_block.area_id,
            linked_tasks: Vec::new(),
            is_recurring: time_block.recurrence_rule.is_some(),
        };

        // 10. 获取关联的任务摘要（✅ 使用共享 Assembler）
        if let Some(task_ids) = request.linked_task_ids {
            time_block_view.linked_tasks =
                LinkedTaskAssembler::get_summaries_batch(app_state.db_pool(), &task_ids).await?;
        }

        Ok(time_block_view)
    }
}

// ==================== 数据访问层 ====================
// ✅ 已全部迁移到共享 Repository：
// - TimeBlockConflictChecker::check_in_tx
// - TimeBlockRepository::insert_in_tx
// - TaskTimeBlockLinkRepository::link_in_tx
// - TaskScheduleRepository::has_schedule_for_day_in_tx, create_in_tx
// - LinkedTaskAssembler::get_summaries_batch
