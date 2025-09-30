/// 更新任务 API - 单文件组件
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{Task, TaskResponse, UpdateTaskRequest},
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `update_task`

## !! 技术债警告 (V1.0) !!
- **描述:** 当前实现采用了“先读后改”(Read-Modify-Write)模式，在多用户并发环境下存在竞争条件风险。
- **原因:** 为了简化V1.0单人本地应用的开发。
- **偿还计划:** 在实现云同步或多用户功能之前，必须将此逻辑重构为使用动态SQL QueryBuilder的、真正的“部分更新”(PATCH)模式。

## API端点
PATCH /api/tasks/{id}

## 预期行为简介
原子性地更新一个任务的一个或多个属性。

## 输入输出规范
- **前置条件**: `id`必须有效。请求体`UpdateTaskRequest`中所有非`None`的字段都必须通过验证。
- **后置条件**: 成功时，返回`200 OK`状态码和更新后的完整`Task`对象的JSON。
- **不变量**: `id`, `created_at`, `completed_at` 字段不可通过此API修改。

## 边界情况
- 任务不存在: 返回`404 Not Found`。
- 输入数据验证失败: 返回`422 Unprocessable Entity`。
- 空请求体 (`{}`): 返回`200 OK`和当前任务对象，不执行任何写操作。

## 预期副作用
- 在数据库中更新一条`tasks`表的记录。
- `updated_at`字段被更新为当前时间。

## 事务保证
- 所有数据库操作在单个事务中执行。
*/

// ==================== 路由层 (Router Layer) ====================
/// 更新任务的HTTP处理器
pub async fn handle(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(request): Json<UpdateTaskRequest>,
) -> Response {
    match logic::execute(&app_state, task_id, request).await {
        Ok(task) => success_response(TaskResponse::from(task)).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 (Validation Layer) ====================
/// 更新任务功能专用的验证逻辑
mod validation {
    use crate::entities::UpdateTaskRequest;
    use crate::shared::core::ValidationError;

    pub fn validate_update_request(
        request: &UpdateTaskRequest,
    ) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // 1. 验证标题: 只有在明确传入非空值时才验证
        if let Some(title) = &request.title {
            if title.trim().is_empty() {
                errors.push(ValidationError::new(
                    "title",
                    "任务标题不能为空",
                    "TITLE_EMPTY",
                ));
            }
            if title.len() > 255 {
                errors.push(ValidationError::new(
                    "title",
                    "任务标题不能超过255个字符",
                    "TITLE_TOO_LONG",
                ));
            }
        }

        // 2. 验证预估时长
        if let Some(Some(duration)) = request.estimated_duration {
            if duration < 0 {
                errors.push(ValidationError::new(
                    "estimated_duration",
                    "预估时长不能为负数",
                    "DURATION_NEGATIVE",
                ));
            }
        }

        // 3. 验证截止日期和类型的配对关系
        match (&request.due_date, &request.due_date_type) {
            (Some(Some(_)), Some(None)) => {
                // 设了日期但清空了类型 -> 错误
                errors.push(ValidationError::new(
                    "due_date_type",
                    "设置截止日期时必须指定日期类型",
                    "DUE_DATE_TYPE_REQUIRED",
                ));
            }
            (Some(None), Some(Some(_))) => {
                // 清空了日期但设了类型 -> 错误
                errors.push(ValidationError::new(
                    "due_date",
                    "指定日期类型时必须设置截止日期",
                    "DUE_DATE_REQUIRED",
                ));
            }
            _ => {
                // 其他情况都OK:
                // - (None, None): 都不修改
                // - (Some(Some(_)), Some(Some(_))): 都设置了值
                // - (Some(None), Some(None)): 都清空
                // - (Some(Some(_)), None): 只改日期，不改类型
                // - (None, Some(Some(_))): 只改类型，不改日期
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
/// 更新任务的核心业务逻辑
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        task_id: Uuid,
        request: UpdateTaskRequest,
    ) -> AppResult<Task> {
        // 0. 检查是否为空更新
        if request.is_empty() {
            let mut tx = app_state.db_pool().begin().await.map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
            return database::find_task_by_id_in_tx(&mut tx, task_id)
                .await?
                .ok_or_else(|| AppError::not_found("Task", task_id.to_string()));
        }

        // 1. 验证请求
        validation::validate_update_request(&request).map_err(AppError::ValidationFailed)?;

        let now = app_state.clock().now_utc();
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 2. 获取并合并更新
        let mut task = database::find_task_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 使用更健壮的合并逻辑
        if let Some(title) = request.title {
            task.title = title;
        }
        if let Some(glance_note) = request.glance_note {
            task.glance_note = glance_note;
        }
        if let Some(detail_note) = request.detail_note {
            task.detail_note = detail_note;
        }
        if let Some(estimated_duration) = request.estimated_duration {
            task.estimated_duration = estimated_duration;
        }
        if let Some(subtasks) = request.subtasks {
            task.subtasks = subtasks;
        }
        if let Some(project_id) = request.project_id {
            task.project_id = project_id;
        }
        if let Some(area_id) = request.area_id {
            task.area_id = area_id;
        }
        if let Some(due_date) = request.due_date {
            task.due_date = due_date;
        }
        if let Some(due_date_type) = request.due_date_type {
            task.due_date_type = due_date_type;
        }

        task.updated_at = now;

        // 3. 持久化 (全量更新)
        let updated_task = database::update_task_in_tx(&mut tx, &task).await?;

        // 4. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(updated_task)
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
/// 更新任务功能专用的数据库操作
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
            SELECT id, title, glance_note, detail_note, estimated_duration, 
                   subtasks, project_id, area_id, due_date, due_date_type, completed_at, 
                   created_at, updated_at, is_deleted, source_info,
                   external_source_id, external_source_provider, external_source_metadata,
                   recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            FROM tasks WHERE id = ? AND is_deleted = false
            "#
        )
        .bind(task_id.to_string())
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        row.map(|r| Task::try_from(r))
            .transpose()
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }

    /// 在事务中更新任务
    pub async fn update_task_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task: &Task,
    ) -> AppResult<Task> {
        // 将 JSON 字段转换为 Value，比字符串序列化更健壮
        let subtasks_json = task
            .subtasks
            .as_ref()
            .map(|s| serde_json::to_value(s).unwrap_or(serde_json::Value::Null))
            .and_then(|v| serde_json::to_string(&v).ok());

        let due_date_type_json = task
            .due_date_type
            .as_ref()
            .map(|dt| serde_json::to_value(dt).unwrap_or(serde_json::Value::Null))
            .and_then(|v| serde_json::to_string(&v).ok());

        let row = sqlx::query_as::<_, TaskRow>(
            r#"
            UPDATE tasks SET
                title = ?,
                glance_note = ?,
                detail_note = ?,
                estimated_duration = ?,
                subtasks = ?,
                project_id = ?,
                area_id = ?,
                due_date = ?,
                due_date_type = ?,
                updated_at = ?
            WHERE id = ? AND is_deleted = false
            RETURNING id, title, glance_note, detail_note, estimated_duration, 
                      subtasks, project_id, area_id, due_date, due_date_type, completed_at, 
                      created_at, updated_at, is_deleted, source_info,
                      external_source_id, external_source_provider, external_source_metadata,
                      recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            "#
        )
        .bind(&task.title)
        .bind(&task.glance_note)
        .bind(&task.detail_note)
        .bind(task.estimated_duration)
        .bind(subtasks_json)
        .bind(task.project_id.map(|id| id.to_string()))
        .bind(task.area_id.map(|id| id.to_string()))
        .bind(task.due_date.map(|dt| dt.to_rfc3339()))
        .bind(due_date_type_json)
        .bind(task.updated_at.to_rfc3339())
        .bind(task.id.to_string())
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        Task::try_from(row)
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }
}
