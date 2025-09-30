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

use crate::{
    entities::Task,
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

use crate::features::tasks::shared::TaskAssembler;

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
    match logic::execute(&app_state, task_id).await {
        Ok(task) => {
            // 使用装配器将 Task 实体转换为 TaskCardDto
            let task_card = TaskAssembler::task_to_card_basic(&task);
            success_response(task_card).into_response()
        }
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 (Validation Layer) ====================
/// 完成任务功能专用的验证逻辑
pub mod validation {
    use super::*;

    /// 验证任务业务规则
    pub fn validate_task_business_rules(task: &Task) -> AppResult<()> {
        use crate::shared::core::AppError;

        // 验证标题长度
        if task.title.len() > 255 {
            return Err(AppError::validation_error(
                "title",
                "任务标题不能超过255个字符",
                "TITLE_TOO_LONG",
            ));
        }

        // 验证预估时长
        if let Some(duration) = task.estimated_duration {
            if duration < 0 {
                return Err(AppError::validation_error(
                    "estimated_duration",
                    "预估时长不能为负数",
                    "DURATION_NEGATIVE",
                ));
            }
            if duration > 24 * 60 * 7 {
                // 一周的分钟数
                return Err(AppError::validation_error(
                    "estimated_duration",
                    "预估时长不能超过一周",
                    "DURATION_TOO_LONG",
                ));
            }
        }

        // 验证截止日期
        if let Some(due_date) = task.due_date {
            if due_date < task.created_at {
                return Err(AppError::validation_error(
                    "due_date",
                    "截止日期不能早于创建时间",
                    "DUE_DATE_TOO_EARLY",
                ));
            }
        }

        // 验证子任务数量
        if let Some(subtasks) = &task.subtasks {
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

// ==================== 业务层 (Service/Logic Layer) ====================
/// 完成任务的核心业务逻辑
pub mod logic {
    use super::*;

    /// 执行完成任务的业务逻辑
    pub async fn execute(app_state: &AppState, task_id: Uuid) -> AppResult<Task> {
        let now = Utc::now();

        // 开始事务
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 1. 检查任务是否存在
        let task = database::find_task_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 2. 检查任务是否已完成
        if task.is_completed() {
            return Err(AppError::conflict("任务已经完成"));
        }

        // 3. 验证业务规则
        validation::validate_task_business_rules(&task)?;

        // 4. 更新任务状态
        let updated_task = database::set_task_completed_in_tx(&mut tx, task_id, now).await?;

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

    /// 在事务中根据ID查找任务
    pub async fn find_task_by_id_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<Option<Task>> {
        use crate::entities::TaskRow;

        let task_row = sqlx::query_as::<_, TaskRow>(
            r#"
            SELECT id, title, glance_note, detail_note, estimated_duration, 
                   subtasks, project_id, area_id, due_date, due_date_type, completed_at, 
                   created_at, updated_at, is_deleted, source_info,
                   external_source_id, external_source_provider, external_source_metadata,
                   recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            FROM tasks 
            WHERE id = ? AND is_deleted = false
            "#
        )
        .bind(task_id.to_string())
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        match task_row {
            Some(row) => {
                let task = Task::try_from(row).map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::QueryError(e))
                })?;
                Ok(Some(task))
            }
            None => Ok(None),
        }
    }

    /// 在事务中设置任务为已完成
    pub async fn set_task_completed_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        completed_at: chrono::DateTime<Utc>,
    ) -> AppResult<Task> {
        let query = r#"
            UPDATE tasks 
            SET completed_at = ?, updated_at = ?
            WHERE id = ? AND is_deleted = false
        "#;

        let rows_affected = sqlx::query(query)
            .bind(completed_at.to_rfc3339())
            .bind(completed_at.to_rfc3339())
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?
            .rows_affected();

        if rows_affected == 0 {
            return Err(AppError::not_found("Task", task_id.to_string()));
        }

        // 返回更新后的任务
        find_task_by_id_in_tx(tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))
    }

    /// 截断任务相关的时间块（在事务中）- 只有完成任务时用到
    pub async fn truncate_time_blocks_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        completed_at: chrono::DateTime<Utc>,
    ) -> AppResult<()> {
        // 通过关联表查找任务相关的进行中时间块（end_time > completed_at）
        let query = r#"
            UPDATE time_blocks 
            SET end_time = ?, updated_at = ?
            WHERE id IN (
                SELECT ttbl.time_block_id 
                FROM task_time_block_links ttbl
                WHERE ttbl.task_id = ?
            )
            AND end_time > ? 
            AND is_deleted = false
        "#;

        let rows_affected = sqlx::query(query)
            .bind(completed_at.to_rfc3339())
            .bind(completed_at.to_rfc3339())
            .bind(task_id.to_string())
            .bind(completed_at.to_rfc3339())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                crate::shared::core::AppError::DatabaseError(
                    crate::shared::core::DbError::ConnectionError(e),
                )
            })?
            .rows_affected();

        tracing::info!(
            "Truncated {} time blocks for task {} at {}",
            rows_affected,
            task_id,
            completed_at
        );

        Ok(())
    }

    /// 清理任务的未来日程（在事务中）- 只有完成任务时用到
    pub async fn cleanup_future_schedules_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        completed_at: chrono::DateTime<Utc>,
    ) -> AppResult<()> {
        // 删除任务在completed_at之后的所有日程安排
        // 只删除未来的日程，已经发生的日程保留作为历史记录
        let completed_date = completed_at.date_naive();

        let query = r#"
            DELETE FROM task_schedules 
            WHERE task_id = ? 
            AND DATE(scheduled_day) > DATE(?)
        "#;

        let rows_affected = sqlx::query(query)
            .bind(task_id.to_string())
            .bind(completed_date.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                crate::shared::core::AppError::DatabaseError(
                    crate::shared::core::DbError::ConnectionError(e),
                )
            })?
            .rows_affected();

        tracing::info!(
            "Cleaned up {} future schedules for task {} after {}",
            rows_affected,
            task_id,
            completed_at
        );

        Ok(())
    }
}
