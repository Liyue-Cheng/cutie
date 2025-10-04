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
    features::time_blocks::shared::{repositories::TimeBlockRepository, TimeBlockConflictChecker},
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
创建一个纯时间块（会议、约会、独立事件）。
🔧 职责分离：此端点不关联任务，任务关联使用 POST /time-blocks/from-task

## 输入输出规范
- **前置条件**:
  - start_time < end_time
  - 时间块不与现有时间块重叠（关键约束）
- **后置条件**:
  - 在 time_blocks 表中创建新时间块
  - 返回完整的 TimeBlockViewDto

## 边界情况
- 如果时间范围无效，返回 400 Bad Request
- 如果与现有时间块重叠，返回 409 Conflict

## 预期副作用
- 插入一条 time_blocks 记录

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

        // 7. 提交事务
        // 🔧 REMOVED: 任务关联逻辑已移除，职责分离
        // 任务关联应使用 POST /time-blocks/from-task 端点
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 8. 组装返回的 TimeBlockViewDto（✅ area_id 已直接从 time_block 获取）
        let time_block_view = TimeBlockViewDto {
            id: time_block.id,
            start_time: time_block.start_time,
            end_time: time_block.end_time,
            title: time_block.title,
            glance_note: time_block.glance_note,
            detail_note: time_block.detail_note,
            area_id: time_block.area_id,
            linked_tasks: Vec::new(), // 🔧 纯时间块不关联任务
            is_recurring: time_block.recurrence_rule.is_some(),
        };

        Ok(time_block_view)
    }
}

// ==================== 数据访问层 ====================
// ✅ 已全部迁移到共享 Repository：
// - TimeBlockConflictChecker::check_in_tx
// - TimeBlockRepository::insert_in_tx
//
// 🔧 职责分离说明：
// 此端点仅创建纯时间块，不涉及任务关联
// 任务关联使用专门的 POST /time-blocks/from-task 端点
