/// 创建任务 API - 单文件组件
///
/// 按照架构纲领V1.0设计，严格遵循依赖注入和职责分离原则
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{CreateTaskRequest, Task, TaskResponse},
    shared::{
        core::{AppError, AppResult, ValidationError},
        http::error_handler::created_response,
        utils::sort_order_utils,
    },
    startup::AppState,
};

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
  - context_type和context_id必须符合规范
- **后置条件**:
  - 创建新的任务记录
  - 在指定上下文中创建排序记录（使用LexoRank算法）
  - 返回201 Created和创建的任务对象

## 边界情况
- 如果验证失败，返回422 Unprocessable Entity
- 如果上下文ID格式无效，返回422 Unprocessable Entity
- 如果数据库操作失败，返回500 Internal Server Error

## 预期副作用
- 在数据库中插入新的任务记录
- 在指定上下文中创建排序记录
- 使用注入的Clock和IdGenerator保证可测试性

## 事务保证
- 任务创建和排序记录创建在同一事务中执行
- 如果任何步骤失败，整个操作回滚
*/

// ==================== 路由层 (Router Layer) ====================
/// 创建任务的HTTP处理器
///
/// **职责:** 仅负责HTTP层面的解析和调度，不包含任何业务逻辑或验证
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateTaskRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(task) => created_response(TaskResponse::from(task)).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 (Validation Layer) ====================
/// 创建任务功能专用的验证逻辑
mod validation {
    use super::*;

    /// 验证创建任务请求（整合所有验证逻辑）
    ///
    /// **职责:** 验证所有输入数据的合法性和业务规则
    pub fn validate_create_task_request(
        request: &CreateTaskRequest,
    ) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // 1. 验证标题
        let title = request.title.trim();
        if title.is_empty() {
            errors.push(ValidationError::new(
                "title",
                "任务标题不能为空",
                "TITLE_EMPTY",
            ));
        }

        if request.title.len() > 255 {
            errors.push(ValidationError::new(
                "title",
                "任务标题不能超过255个字符",
                "TITLE_TOO_LONG",
            ));
        }

        // 2. 验证预估时长
        if let Some(duration) = request.estimated_duration {
            if duration < 0 {
                errors.push(ValidationError::new(
                    "estimated_duration",
                    "预估时长不能为负数",
                    "DURATION_NEGATIVE",
                ));
            }
            if duration > 24 * 60 * 7 {
                // 一周的分钟数
                errors.push(ValidationError::new(
                    "estimated_duration",
                    "预估时长不能超过一周",
                    "DURATION_TOO_LONG",
                ));
            }
        }

        // 3. 验证截止日期
        if request.due_date.is_some() && request.due_date_type.is_none() {
            errors.push(ValidationError::new(
                "due_date_type",
                "设置截止日期时必须指定日期类型（SOFT或HARD）",
                "DUE_DATE_TYPE_REQUIRED",
            ));
        }

        if request.due_date.is_none() && request.due_date_type.is_some() {
            errors.push(ValidationError::new(
                "due_date",
                "指定日期类型时必须设置截止日期",
                "DUE_DATE_REQUIRED",
            ));
        }

        // 4. 验证子任务数量
        if let Some(subtasks) = &request.subtasks {
            if subtasks.len() > 50 {
                errors.push(ValidationError::new(
                    "subtasks",
                    "子任务数量不能超过50个",
                    "TOO_MANY_SUBTASKS",
                ));
            }
        }

        // 5. 验证上下文ID格式（基础验证，详细验证在Ordering模型中）
        if request.context.context_id.trim().is_empty() {
            errors.push(ValidationError::new(
                "context.context_id",
                "上下文ID不能为空",
                "CONTEXT_ID_EMPTY",
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
/// 创建任务的核心业务逻辑
mod logic {
    use super::*;

    /// 执行创建任务的业务逻辑
    ///
    /// **职责:**
    /// 1. 验证所有输入
    /// 2. 使用注入的依赖（Clock和IdGenerator）生成数据
    /// 3. 协调数据层操作
    /// 4. 确保事务一致性
    pub async fn execute(app_state: &AppState, request: CreateTaskRequest) -> AppResult<Task> {
        // 1. 首先验证请求（业务层的第一步，不是路由层的职责！）
        validation::validate_create_task_request(&request)
            .map_err(|errors| AppError::ValidationFailed(errors))?;

        // 2. 使用注入的抽象获取时间和ID（保证可测试性）
        let now = app_state.clock().now_utc();
        let task_id = app_state.id_generator().new_uuid();

        // 3. 开始事务
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 4. 构建任务对象
        let mut task = Task::new(task_id, request.title, now);
        task.glance_note = request.glance_note;
        task.detail_note = request.detail_note;
        task.estimated_duration = request.estimated_duration;
        task.subtasks = request.subtasks;
        task.area_id = request.area_id;
        task.due_date = request.due_date;
        task.due_date_type = request.due_date_type;

        // 5. 在数据库中创建任务（使用RETURNING *获取实际创建的记录）
        let created_task = database::create_task_in_tx(&mut tx, &task).await?;

        // 6. 在指定上下文中创建排序记录（使用正确的LexoRank算法）
        database::create_ordering_for_new_task_in_tx(
            &mut tx,
            &request.context.context_type.to_string(),
            &request.context.context_id,
            task_id,
            now,
        )
        .await?;

        // 7. 提交事务
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
mod database {
    use super::*;
    use crate::entities::TaskRow;

    /// 在事务中创建任务（使用RETURNING *获取实际创建的记录）
    ///
    /// **修正:** 使用query_as和RETURNING *，而不是手动clone
    pub async fn create_task_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task: &Task,
    ) -> AppResult<Task> {
        let task_row = sqlx::query_as::<_, TaskRow>(
            r#"
            INSERT INTO tasks (
                id, title, glance_note, detail_note, estimated_duration,
                subtasks, project_id, area_id, due_date, due_date_type, completed_at,
                created_at, updated_at, is_deleted, source_info,
                external_source_id, external_source_provider, external_source_metadata,
                recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#
        )
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
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to insert task: {}", e);
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 转换TaskRow到Task
        Task::try_from(task_row)
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }

    /// 在事务中为新任务创建排序记录
    ///
    /// **修正:** 使用正确的LexoRank字符串排序算法，而不是整数+1
    pub async fn create_ordering_for_new_task_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        context_type: &str,
        context_id: &str,
        task_id: Uuid,
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<()> {
        // 1. 获取当前上下文中的最大sort_order字符串
        let max_sort_order: Option<String> = sqlx::query_scalar(
            r#"
            SELECT MAX(sort_order) as max_order
            FROM ordering
            WHERE context_type = ? AND context_id = ?
            "#,
        )
        .bind(context_type)
        .bind(context_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to query max sort_order: {}", e);
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?
        .flatten(); // Option<Option<String>> -> Option<String>

        // 2. 使用LexoRank算法生成新的sort_order
        let new_sort_order = match max_sort_order {
            Some(max_order) => {
                // 在最大值之后生成新的排序字符串
                sort_order_utils::get_rank_after(&max_order).map_err(|e| {
                    tracing::error!("Failed to generate rank after {}: {:?}", max_order, e);
                    AppError::validation_error(
                        "sort_order",
                        &format!("无法生成排序值: {}", e),
                        "SORT_ORDER_GENERATION_FAILED",
                    )
                })?
            }
            None => {
                // 这是第一个任务，生成初始排序字符串
                sort_order_utils::generate_initial_sort_order()
            }
        };

        // 3. 创建排序记录
        let ordering_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO ordering (
                id, context_type, context_id, task_id, sort_order, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(ordering_id.to_string())
        .bind(context_type)
        .bind(context_id)
        .bind(task_id.to_string())
        .bind(&new_sort_order)
        .bind(created_at.to_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to insert ordering: {}", e);
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        tracing::info!(
            "Created ordering record for task {} in context {}:{} with sort_order: {}",
            task_id,
            context_type,
            context_id,
            new_sort_order
        );

        Ok(())
    }
}
