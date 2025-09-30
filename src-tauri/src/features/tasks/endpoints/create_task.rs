/// 创建任务 API - 单文件组件
///
/// 按照Vue单文件组件的思想，将创建任务API的所有逻辑聚合在一个文件中
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    entities::Task,
    shared::{
        core::{AppError, AppResult},
        http::error_handler::created_response,
    },
    startup::AppState,
};

use super::super::shared::validation::validate_create_task_request;
use crate::entities::{CreateTaskRequest, TaskResponse};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `create_task`

## API端点
POST /api/tasks

## 预期行为简介
在指定的上下文中创建一个新任务，包括自动生成UUID、设置初始排序位置等。

## 输入输出规范
- **前置条件**:
  - 请求体必须包含有效的CreateTaskRequest
  - 标题不能为空且长度不超过255字符
  - 如果设置截止日期，必须指定日期类型
- **后置条件**:
  - 创建新的任务记录
  - 在指定上下文中创建排序记录
  - 返回创建的任务对象

## 边界情况
- 如果验证失败，返回422 Unprocessable Entity
- 如果上下文无效，返回400 Bad Request
- 如果数据库操作失败，返回500 Internal Server Error

## 预期副作用
- 在数据库中插入新的任务记录
- 在指定上下文中创建排序记录
- 可能触发相关的业务事件

## 事务保证
- 任务创建和排序记录创建在同一事务中执行
- 如果任何步骤失败，整个操作回滚
*/

// ==================== 路由层 (Router Layer) ====================
/// 创建任务的HTTP处理器
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateTaskRequest>,
) -> Response {
    // 验证请求
    if let Err(errors) = validate_create_task_request(&request) {
        let validation_error = AppError::ValidationFailed(errors);
        return validation_error.into_response();
    }

    match logic::execute(&app_state, request).await {
        Ok(task) => created_response(TaskResponse::from(task)).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
/// 创建任务的核心业务逻辑
pub mod logic {
    use super::*;

    /// 执行创建任务的业务逻辑
    pub async fn execute(app_state: &AppState, request: CreateTaskRequest) -> AppResult<Task> {
        let now = Utc::now();
        let task_id = Uuid::new_v4();

        // 开始事务
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 1. 创建任务对象
        let mut task = Task::new(task_id, request.title, now);

        // 设置可选字段
        task.glance_note = request.glance_note;
        task.detail_note = request.detail_note;
        task.estimated_duration = request.estimated_duration;
        task.subtasks = request.subtasks;
        task.area_id = request.area_id;
        task.due_date = request.due_date;
        task.due_date_type = request.due_date_type;

        // 2. 验证业务规则
        crate::features::tasks::shared::validation::validate_task_business_rules(&task)?;

        // 3. 在数据库中创建任务
        let created_task = database::create_task_in_tx(&mut tx, &task).await?;

        // 4. 在指定上下文中创建排序记录
        database::create_ordering_for_new_task_in_tx(
            &mut tx,
            &request.context.context_type.to_string(),
            &request.context.context_id,
            task_id,
            now,
        )
        .await?;

        // 5. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(created_task)
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
/// 创建任务功能专用的数据库操作
pub mod database {
    use super::*;
    use sqlx::{Row, Sqlite, Transaction};

    /// 在事务中创建任务
    pub async fn create_task_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task: &Task,
    ) -> AppResult<Task> {
        let query = r#"
            INSERT INTO tasks (
                id, title, glance_note, detail_note, estimated_duration,
                subtasks, project_id, area_id, due_date, due_date_type, completed_at,
                created_at, updated_at, is_deleted, source_info,
                external_source_id, external_source_provider, external_source_metadata,
                recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(task.id.to_string())
            .bind(&task.title)
            .bind(&task.glance_note)
            .bind(&task.detail_note)
            .bind(task.estimated_duration)
            .bind(
                task.subtasks
                    .as_ref()
                    .and_then(|s| serde_json::to_string(s).ok()),
            )
            .bind(task.project_id.map(|id| id.to_string()))
            .bind(task.area_id.map(|id| id.to_string()))
            .bind(task.due_date.map(|dt| dt.to_rfc3339()))
            .bind(
                task.due_date_type
                    .as_ref()
                    .and_then(|dt| serde_json::to_string(dt).ok()),
            )
            .bind(task.completed_at.map(|dt| dt.to_rfc3339()))
            .bind(task.created_at.to_rfc3339())
            .bind(task.updated_at.to_rfc3339())
            .bind(task.is_deleted)
            .bind(
                task.source_info
                    .as_ref()
                    .and_then(|si| serde_json::to_string(si).ok()),
            )
            .bind(&task.external_source_id)
            .bind(&task.external_source_provider)
            .bind(&task.external_source_metadata)
            .bind(&task.recurrence_rule)
            .bind(task.recurrence_parent_id.map(|id| id.to_string()))
            .bind(task.recurrence_original_date.map(|dt| dt.to_rfc3339()))
            .bind(
                task.recurrence_exclusions
                    .as_ref()
                    .and_then(|re| serde_json::to_string(re).ok()),
            )
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(task.clone())
    }

    /// 在事务中为新任务创建排序记录
    pub async fn create_ordering_for_new_task_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        context_type: &str,
        context_id: &str,
        task_id: Uuid,
        created_at: chrono::DateTime<Utc>,
    ) -> AppResult<()> {
        // 获取当前上下文中的最大排序位置
        let max_position_query = r#"
            SELECT COALESCE(MAX(position), 0) as max_position
            FROM orderings
            WHERE context_type = ? AND context_id = ? AND is_deleted = false
        "#;

        let max_position: i64 = sqlx::query(max_position_query)
            .bind(context_type)
            .bind(context_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?
            .get("max_position");

        let new_position = max_position + 1;

        // 创建排序记录
        let insert_query = r#"
            INSERT INTO orderings (
                id, context_type, context_id, task_id, position,
                created_at, updated_at, is_deleted
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        let ordering_id = Uuid::new_v4();
        sqlx::query(insert_query)
            .bind(ordering_id.to_string())
            .bind(context_type)
            .bind(context_id)
            .bind(task_id.to_string())
            .bind(new_position)
            .bind(created_at.to_rfc3339())
            .bind(created_at.to_rfc3339())
            .bind(false)
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        tracing::info!(
            "Created ordering record for task {} in context {}:{}",
            task_id,
            context_type,
            context_id
        );

        Ok(())
    }
}
