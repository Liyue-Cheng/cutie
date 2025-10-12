/// 重新打开任务 API - 单文件组件
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{Task, TaskResponse},
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `reopen_task`

## API端点
DELETE /api/tasks/{id}/completion

## 预期行为简介
将一个已完成的任务重新打开。此操作是幂等的。

## 输入输出规范
- **前置条件**: `id`必须是有效的UUID。
- **后置条件**: 成功时，返回`200 OK`状态码和更新后的`Task`对象，其`completed_at`字段为`NULL`。
- **不变量**: 只有已完成的任务才能被重新打开。

## 边界情况
- 任务不存在: 返回`404 Not Found`。
- 任务本就未完成: 幂等地返回`200 OK`和当前任务对象，不执行任何写操作。

## 预期副作用
- 更新`tasks`表中对应记录的`completed_at`为`NULL`。
- 更新`tasks`表中对应记录的`updated_at`。
- 在`task_schedules`表中，将该任务所有`COMPLETED_ON_DAY`的`outcome`重置为`PLANNED`。

## 事务保证
- 所有数据库操作在单个事务中执行。
*/

// ==================== 路由层 (Router Layer) ====================
/// 重新打开任务的HTTP处理器
pub async fn handle(State(app_state): State<AppState>, Path(task_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, task_id).await {
        Ok(task) => success_response(TaskResponse::from(task)).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 (Validation Layer) ====================
// 此端点无复杂输入验证，故验证层为空。

// ==================== 业务层 (Service/Logic Layer) ====================
/// 重新打开任务的核心业务逻辑
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, task_id: Uuid) -> AppResult<Task> {
        let now = app_state.clock().now_utc();
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
        })?;

        // 1. 获取任务
        let mut task = database::find_task_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 2. 幂等检查：如果任务未完成，则直接返回
        if !task.is_completed() {
            return Ok(task);
        }

        // 3. 核心操作：重新打开任务
        task.reopen(now);

        // 4. 持久化核心操作
        let updated_task = database::reopen_task_in_tx(&mut tx, &task).await?;

        // 5. 耦合操作：重置日程结局
        database::reset_completed_outcomes_in_tx(&mut tx, task_id).await?;

        // 6. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(updated_task)
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
/// 重新打开任务功能专用的数据库操作
mod database {
    use super::*;
    use crate::entities::TaskRow;

    /// 在事务中根据ID查找任务
    pub async fn find_task_by_id_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<Option<Task>> {
        let row = sqlx::query_as::<_, TaskRow>(
            r#"
            SELECT * FROM tasks WHERE id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(task_id.to_string())
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e)))?;

        row.map(|r| Task::try_from(r))
            .transpose()
            .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::QueryError(e)))
    }

    /// 在事务中重新打开任务
    pub async fn reopen_task_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task: &Task,
    ) -> AppResult<Task> {
        let row = sqlx::query_as::<_, TaskRow>(
            r#"
            UPDATE tasks SET completed_at = NULL, updated_at = ?
            WHERE id = ? RETURNING *
            "#,
        )
        .bind(task.updated_at.to_rfc3339())
        .bind(task.id.to_string())
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e)))?;

        Task::try_from(row)
            .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::QueryError(e)))
    }

    /// 在事务中重置已完成的日程结局
    pub async fn reset_completed_outcomes_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE task_schedules SET outcome = 'PLANNED'
            WHERE task_id = ? AND outcome = 'COMPLETED_ON_DAY'
            "#,
        )
        .bind(task_id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e)))?;

        Ok(())
    }
}
