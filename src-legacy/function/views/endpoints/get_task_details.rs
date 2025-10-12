/// 获取任务详情视图 API - 单文件组件
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::Task,
    crate::infra::core::{AppError, AppResult},
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `get_task_details`

## API端点
GET /api/views/tasks/{id}

## 预期行为简介
获取指定任务的完整详情。

## 输入输出规范
- **前置条件**: `id` 必须是有效的任务ID。
- **后置条件**: 返回 `200 OK` 和任务对象。
- **不变量**: 无。

## 边界情况
- 任务不存在: 返回 `404 Not Found`。

## 预期副作用
- 无副作用（只读操作）。
*/

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(State(app_state): State<AppState>, Path(task_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, task_id).await {
        Ok(task) => Json(task).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, task_id: Uuid) -> AppResult<Task> {
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
        })?;

        let task = database::find_task_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(task)
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
mod database {
    use super::*;
    use crate::entities::task::TaskRow;

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
            FROM tasks WHERE id = ? AND deleted_at IS NULL
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
}
