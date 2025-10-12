/// 完全取消日程 API - 单文件组件
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    http::StatusCode,
};
use chrono::Utc;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    shared::{
        core::{AppError, AppResult},
    },
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `unschedule_task_completely`

## API端点
DELETE /api/tasks/{id}/schedules

## 预期行为简介
将任务从所有日程中移除，使其回归Staging。幂等操作。

## 输入输出规范
- **前置条件**: `id` 必须是有效的任务ID。
- **后置条件**: 返回 `204 No Content`。
- **不变量**: 无。

## 边界情况
- 任务不存在: 返回 `404 Not Found`。
- 任务没有任何日程: 幂等地返回 `204`。

## 预期副作用
- 从 `task_schedules` 表删除所有相关记录。
- 删除所有 `DAILY_KANBAN` 上下文的排序记录。
- 在 `MISC` 或 `PROJECT_LIST` 上下文中创建排序记录。
- 所有数据库写入在单个事务中。
*/

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(State(app_state): State<AppState>, Path(task_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, task_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, task_id: Uuid) -> AppResult<()> {
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
        })?;

        // 1. 验证任务存在
        let task = database::find_task_by_id(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 2. 核心操作：删除所有日程
        database::delete_all_schedules_for_task_in_tx(&mut tx, task_id).await?;

        // 3. 排序处理：删除所有 DAILY_KANBAN 排序记录
        database::delete_all_daily_kanban_ordering(&mut tx, task_id).await?;

        // 4. 创建新的排序记录
        let context_type = if task.project_id.is_some() {
            "PROJECT_LIST"
        } else {
            "MISC"
        };

        let context_id = task.project_id.map(|id| id.to_string()).unwrap_or_default();

        database::create_ordering(&mut tx, task_id, context_type, &context_id).await?;

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

    pub async fn find_task_by_id(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<Option<crate::entities::Task>> {
        let row = sqlx::query_as::<_, crate::entities::task::TaskRow>(
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

        row.map(|r| crate::entities::Task::try_from(r))
            .transpose()
            .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::QueryError(e)))
    }

    pub async fn delete_all_schedules_for_task_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<()> {
        sqlx::query("DELETE FROM task_schedules WHERE task_id = ?")
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e)))?;

        Ok(())
    }

    pub async fn delete_all_daily_kanban_ordering(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            DELETE FROM ordering 
            WHERE context_type = 'DAILY_KANBAN' 
            AND task_id = ?
            "#,
        )
        .bind(task_id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e)))?;

        Ok(())
    }

    pub async fn create_ordering(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        context_type: &str,
        context_id: &str,
    ) -> AppResult<()> {
        let now = Utc::now();

        // 获取最大排序值
        let max_sort_order: Option<String> = sqlx::query_scalar(
            "SELECT MAX(sort_order) FROM ordering WHERE context_type = ? AND context_id = ?",
        )
        .bind(context_type)
        .bind(context_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e)))?;

        let new_sort_order = match max_sort_order {
            Some(max) => crate::infra::core::utils::sort_order_utils::get_rank_after(&max)
                .unwrap_or_else(|_| crate::infra::core::utils::sort_order_utils::generate_initial_sort_order()),
            None => crate::infra::core::utils::sort_order_utils::generate_initial_sort_order(),
        };

        sqlx::query(
            r#"
            INSERT INTO ordering (id, context_type, context_id, task_id, sort_order, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(context_type)
        .bind(context_id)
        .bind(task_id.to_string())
        .bind(&new_sort_order)
        .bind(now.to_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e)))?;

        Ok(())
    }
}

