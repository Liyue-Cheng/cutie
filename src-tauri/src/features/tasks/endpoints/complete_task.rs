/// 完成任务 API - 单文件组件
///
/// 按照Vue单文件组件的思想，将一个API的所有逻辑聚合在一个文件中
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use chrono::Utc;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::repositories::TaskRepository;
use crate::{
    entities::Task,
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

use super::super::shared::{dtos::TaskResponse, validation::validate_task_business_rules};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `complete_task`

## API端点
POST /api/tasks/{id}/completion

## 预期行为简介
将指定的任务标记为已完成，设置completed_at时间戳，并执行相关的清理操作。

## 输入输出规范
- **前置条件**:
  - task_id必须是有效的UUID
  - 任务必须存在且未被删除
  - 任务当前状态必须是未完成
- **后置条件**:
  - 任务的completed_at字段被设置为当前时间
  - 任务的updated_at字段被更新
  - 返回更新后的任务对象

## 边界情况
- 如果任务不存在，返回404 Not Found
- 如果任务已经完成，返回409 Conflict
- 如果数据库操作失败，返回500 Internal Server Error

## 预期副作用
- 更新数据库中的任务记录
- 可能触发相关的业务逻辑（如截断时间块、清理日程等）

## 事务保证
- 所有数据库操作在单个事务中执行
- 如果任何步骤失败，整个操作回滚
*/

// ==================== 路由层 (Router Layer) ====================
/// 完成任务的HTTP处理器
pub async fn handle(State(app_state): State<AppState>, Path(task_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, app_state.task_repository(), task_id).await {
        Ok(task) => success_response(TaskResponse::from(task)).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
/// 完成任务的核心业务逻辑
pub mod logic {
    use super::*;

    /// 执行完成任务的业务逻辑
    pub async fn execute(
        app_state: &AppState,
        task_repo: &dyn TaskRepository,
        task_id: Uuid,
    ) -> AppResult<Task> {
        let now = Utc::now();

        // 开始事务
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 1. 检查任务是否存在（使用 repository 中的方法）
        let task = task_repo
            .find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 2. 检查任务是否已完成
        if task.is_completed() {
            return Err(AppError::conflict("任务已经完成"));
        }

        // 3. 验证业务规则
        validate_task_business_rules(&task)?;

        // 4. 更新任务状态（使用 repository 中的方法）
        let updated_task = task_repo.set_completed(&mut tx, task_id, now).await?;

        // 5. 执行相关清理操作（特定于完成任务的操作）
        database::truncate_time_blocks_in_tx(&mut tx, task_id, now).await?;
        database::cleanup_future_schedules_in_tx(&mut tx, task_id, now).await?;

        // 6. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(updated_task)
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
/// 完成任务功能专用的数据库操作（只包含特定操作）
pub mod database {
    use super::*;

    /// 截断任务相关的时间块（在事务中）- 只有完成任务时用到
    pub async fn truncate_time_blocks_in_tx(
        _tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        completed_at: chrono::DateTime<Utc>,
    ) -> AppResult<()> {
        // TODO: 实现时间块截断逻辑
        // 这里需要查找任务相关的进行中时间块，并将其结束时间设置为completed_at

        tracing::debug!(
            "Truncating time blocks for task {} at {}",
            task_id,
            completed_at
        );

        // 暂时只记录日志，具体实现需要时间块模块的支持
        Ok(())
    }

    /// 清理任务的未来日程（在事务中）- 只有完成任务时用到
    pub async fn cleanup_future_schedules_in_tx(
        _tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        completed_at: chrono::DateTime<Utc>,
    ) -> AppResult<()> {
        // TODO: 实现日程清理逻辑
        // 这里需要删除任务在completed_at之后的所有日程安排

        tracing::debug!(
            "Cleaning up future schedules for task {} after {}",
            task_id,
            completed_at
        );

        // 暂时只记录日志，具体实现需要日程模块的支持
        Ok(())
    }
}
