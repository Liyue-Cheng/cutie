/// 更新任务 API - 单文件组件
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{
        task::response_dtos::AreaSummary, ScheduleStatus, Task, TaskCardDto, UpdateTaskRequest,
    },
    features::tasks::shared::TaskAssembler,
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `update_task`

## API端点
PATCH /api/tasks/{id}

## 预期行为简介
更新任务的可变字段（标题、笔记、子任务等）。

## 输入输出规范
- **前置条件**: task_id 必须存在
- **后置条件**: 任务字段被更新，返回最新的 TaskCardDto

## 边界情况
- 任务不存在 → 404
- 所有字段都是 None → 422（无需更新）
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(request): Json<UpdateTaskRequest>,
) -> Response {
    match logic::execute(&app_state, task_id, request).await {
        Ok(task_card) => success_response(task_card).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate_update_request(request: &UpdateTaskRequest) -> AppResult<()> {
        // 检查是否为空更新
        if request.is_empty() {
            return Err(AppError::validation_error(
                "request",
                "至少需要更新一个字段",
                "EMPTY_UPDATE",
            ));
        }

        // 验证标题
        if let Some(title) = &request.title {
            if title.trim().is_empty() {
                return Err(AppError::validation_error(
                    "title",
                    "标题不能为空",
                    "TITLE_EMPTY",
                ));
            }
            if title.len() > 255 {
                return Err(AppError::validation_error(
                    "title",
                    "标题不能超过255个字符",
                    "TITLE_TOO_LONG",
                ));
            }
        }

        // 验证子任务数量
        if let Some(Some(subtasks)) = &request.subtasks {
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

    pub async fn execute(
        app_state: &AppState,
        task_id: Uuid,
        request: UpdateTaskRequest,
    ) -> AppResult<TaskCardDto> {
        // 1. 验证
        validation::validate_update_request(&request)?;

        // 2. 开启事务
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 3. 检查任务是否存在
        let task_exists = database::check_task_exists_in_tx(&mut tx, task_id).await?;
        if !task_exists {
            return Err(AppError::not_found("Task", task_id.to_string()));
        }

        // 4. 更新任务
        database::update_task_in_tx(&mut tx, task_id, &request).await?;

        // 5. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 6. 重新查询任务以获取最新数据
        let task = database::find_task_by_id(app_state.db_pool(), task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 7. 组装 TaskCardDto
        let mut task_card = TaskAssembler::task_to_card_basic(&task);

        // 8. 获取关联信息
        let has_schedule = database::has_any_schedule(app_state.db_pool(), task_id).await?;
        task_card.schedule_status = if has_schedule {
            ScheduleStatus::Scheduled
        } else {
            ScheduleStatus::Staging
        };

        if let Ok(sort_order) = database::get_task_sort_order(app_state.db_pool(), task_id).await {
            task_card.sort_order = sort_order;
        }

        if let Some(area_id) = task.area_id {
            task_card.area = database::get_area_summary(app_state.db_pool(), area_id).await?;
        }

        Ok(task_card)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::entities::TaskRow;

    pub async fn check_task_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<bool> {
        let query = "SELECT COUNT(*) FROM tasks WHERE id = ? AND is_deleted = false";
        let count: i64 = sqlx::query_scalar(query)
            .bind(task_id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(count > 0)
    }

    pub async fn update_task_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        request: &UpdateTaskRequest,
    ) -> AppResult<()> {
        let now = chrono::Utc::now();
        let mut updates = Vec::new();
        let mut bindings: Vec<String> = Vec::new();

        // 动态构建 UPDATE 语句
        if let Some(title) = &request.title {
            updates.push("title = ?");
            bindings.push(title.clone());
        }
        if let Some(glance_note) = &request.glance_note {
            updates.push("glance_note = ?");
            bindings.push(glance_note.clone().unwrap_or_default());
        }
        if let Some(detail_note) = &request.detail_note {
            updates.push("detail_note = ?");
            bindings.push(detail_note.clone().unwrap_or_default());
        }
        if let Some(subtasks) = &request.subtasks {
            updates.push("subtasks = ?");
            bindings.push(
                subtasks
                    .as_ref()
                    .map(|s| serde_json::to_string(s).unwrap())
                    .unwrap_or_default(),
            );
        }
        if let Some(area_id) = &request.area_id {
            updates.push("area_id = ?");
            bindings.push(area_id.map(|id| id.to_string()).unwrap_or_default());
        }

        if updates.is_empty() {
            return Ok(());
        }

        updates.push("updated_at = ?");
        let update_clause = updates.join(", ");
        let query = format!("UPDATE tasks SET {} WHERE id = ?", update_clause);

        let mut query_builder = sqlx::query(&query);
        for binding in bindings {
            query_builder = query_builder.bind(binding);
        }
        query_builder = query_builder.bind(now.to_rfc3339());
        query_builder = query_builder.bind(task_id.to_string());

        query_builder.execute(&mut **tx).await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        Ok(())
    }

    pub async fn find_task_by_id(
        pool: &sqlx::SqlitePool,
        task_id: Uuid,
    ) -> AppResult<Option<Task>> {
        let query = r#"
            SELECT id, title, glance_note, detail_note, estimated_duration, 
                   subtasks, project_id, area_id, due_date, due_date_type, completed_at, 
                   created_at, updated_at, is_deleted, source_info,
                   external_source_id, external_source_provider, external_source_metadata,
                   recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            FROM tasks 
            WHERE id = ? AND is_deleted = false
        "#;

        let row = sqlx::query_as::<_, TaskRow>(query)
            .bind(task_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        match row {
            Some(r) => {
                let task = Task::try_from(r).map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::QueryError(e))
                })?;
                Ok(Some(task))
            }
            None => Ok(None),
        }
    }

    pub async fn has_any_schedule(pool: &sqlx::SqlitePool, task_id: Uuid) -> AppResult<bool> {
        let query = "SELECT COUNT(*) FROM task_schedules WHERE task_id = ?";
        let count: i64 = sqlx::query_scalar(query)
            .bind(task_id.to_string())
            .fetch_one(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(count > 0)
    }

    pub async fn get_task_sort_order(pool: &sqlx::SqlitePool, task_id: Uuid) -> AppResult<String> {
        let query = "SELECT sort_order FROM orderings WHERE context_type = 'MISC' AND context_id = 'staging' AND task_id = ?";
        let result = sqlx::query_scalar::<_, String>(query)
            .bind(task_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(result.unwrap_or_else(|| "zzz".to_string()))
    }

    pub async fn get_area_summary(
        pool: &sqlx::SqlitePool,
        area_id: Uuid,
    ) -> AppResult<Option<AreaSummary>> {
        let query = "SELECT id, name, color FROM areas WHERE id = ? AND is_deleted = false";
        let result = sqlx::query_as::<_, (String, String, String)>(query)
            .bind(area_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(result.map(|(id, name, color)| AreaSummary {
            id: Uuid::parse_str(&id).unwrap(),
            name,
            color,
        }))
    }
}
