/// 删除任务 API - 单文件组件
///
/// 按照Vue单文件组件的思想，将一个API的所有逻辑聚合在一个文件中
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::Task,
    crate::infra::core::{AppError, AppResult},
    startup::AppState,
};
use axum::http::StatusCode;

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `delete_task`

## API端点
DELETE /api/tasks/{id}

## 预期行为简介
软删除指定的任务（设置is_deleted标记为true），而不是物理删除。

## 输入输出规范
- **前置条件**:
  - task_id必须是有效的UUID
  - 任务必须存在且未被删除
- **后置条件**:
  - 任务的is_deleted字段被设置为true
  - 任务的updated_at字段被更新
  - 返回204 No Content

## 边界情况
- 如果任务不存在，返回404 Not Found
- 如果任务已被删除，返回409 Conflict
- 如果数据库操作失败，返回500 Internal Server Error

## 预期副作用
- 更新数据库中的任务记录
- 保留所有相关数据（日程、排序等）以便可能的恢复

## 事务保证
- 所有数据库操作在单个事务中执行
- 如果任何步骤失败，整个操作回滚
*/

// ==================== 路由层 (Router Layer) ====================
/// 删除任务的HTTP处理器
pub async fn handle(State(app_state): State<AppState>, Path(task_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, task_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, task_id: Uuid) -> AppResult<()> {
        // 开启事务
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
        })?;

        // 1. 查询任务是否存在
        let task = database::find_task_by_id_in_tx(&mut tx, task_id).await?;

        let task = match task {
            Some(t) => t,
            None => return Err(AppError::not_found("Task", task_id.to_string())),
        };

        // 2. 检查任务是否已被删除
        if task.is_deleted {
            return Err(AppError::Conflict {
                message: "Task is already deleted".to_string(),
            });
        }

        // 3. 获取当前时间
        let now = app_state.clock().now_utc();

        // 4. 软删除任务
        database::soft_delete_task_in_tx(&mut tx, task_id, now).await?;

        // 5. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(())
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
mod database {
    use super::*;
    use crate::entities::task::TaskRow;
    use chrono::{DateTime, Utc};

    pub async fn find_task_by_id_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<Option<Task>> {
        let row = sqlx::query_as::<_, TaskRow>(
            r#"
            SELECT id, title, glance_note, detail_note, estimated_duration,
                   subtasks, project_id, area_id, due_date, due_date_type, completed_at,
                   created_at, updated_at, is_deleted, source_info,
                   external_source_id, external_source_provider, external_source_metadata,
                   recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            FROM tasks WHERE id = ?
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

    pub async fn soft_delete_task_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE tasks 
            SET is_deleted = true, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(now.to_rfc3339())
        .bind(task_id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e)))?;

        Ok(())
    }
}
