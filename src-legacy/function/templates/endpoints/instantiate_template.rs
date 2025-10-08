/// 从模板创建任务 API - 单文件组件
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{task::ContextType, Task, Template},
    shared::{
        core::{AppError, AppResult, ValidationError},
        http::error_handler::created_response,
    },
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `instantiate_template`

## API端点
POST /api/templates/{id}/instantiate

## 预期行为简介
使用指定的模板，在给定的上下文中创建一个新任务。

## 输入输出规范
- **前置条件**: `id` 必须是有效的模板ID。请求体必须包含有效的 `context`。
- **后置条件**: 返回 `201 Created` 和新创建的 `Task` 对象。
- **不变量**: 无。

## 边界情况
- 模板不存在: 返回 `404 Not Found`。
- context 无效: 返回 `422 Unprocessable Entity`。

## 预期副作用
- 在 `tasks` 表插入1条记录。
- 根据上下文，在 `task_schedule` 或 `ordering` 表中插入1条记录。
- 所有数据库写入在单个事务中。

## 请求体
```json
{
  "context": {
    "type": "DAILY_KANBAN",
    "id": "1729555200"
  }
}
```
*/

#[derive(Deserialize)]
pub struct InstantiateTemplateRequest {
    context: CreationContext,
}

#[derive(Deserialize)]
pub struct CreationContext {
    #[serde(rename = "type")]
    context_type: String,
    id: String,
}

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(template_id): Path<Uuid>,
    Json(request): Json<InstantiateTemplateRequest>,
) -> Response {
    match logic::execute(&app_state, template_id, request).await {
        Ok(task) => created_response(task).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 (Validation Layer) ====================
mod validation {
    use super::*;

    pub struct ValidatedContext {
        pub context_type: ContextType,
        pub context_id: String,
    }

    pub fn validate_context(
        context: &CreationContext,
    ) -> Result<ValidatedContext, Vec<ValidationError>> {
        let mut errors = Vec::new();

        // 验证 context_type
        let context_type = match context.context_type.as_str() {
            "DAILY_KANBAN" => ContextType::DailyKanban,
            "PROJECT_LIST" => ContextType::ProjectList,
            "MISC" => ContextType::Misc,
            _ => {
                errors.push(ValidationError::new(
                    "context.type",
                    "Context type 无效",
                    "INVALID_CONTEXT_TYPE",
                ));
                ContextType::Misc
            }
        };

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(ValidatedContext {
            context_type,
            context_id: context.id.clone(),
        })
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        template_id: Uuid,
        request: InstantiateTemplateRequest,
    ) -> AppResult<Task> {
        // 1. 验证上下文
        let context =
            validation::validate_context(&request.context).map_err(AppError::ValidationFailed)?;

        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 2. 获取模板
        let template = database::find_template_by_id_in_tx(&mut tx, template_id)
            .await?
            .ok_or_else(|| AppError::not_found("Template", template_id.to_string()))?;

        // 3. 使用模板数据构建任务（简化版：不解析模板变量）
        let task_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        let mut new_task = Task::new(task_id, template.title_template.clone(), now);
        new_task.glance_note = template.glance_note_template.clone();
        new_task.detail_note = template.detail_note_template.clone();
        new_task.estimated_duration = template.estimated_duration_template;
        new_task.subtasks = template.subtasks_template.clone();
        new_task.area_id = template.area_id;

        // 4. 核心操作：创建任务
        let created_task = database::create_task_in_tx(&mut tx, &new_task).await?;

        // 5. 处理后续安排（根据上下文）
        match context.context_type {
            ContextType::DailyKanban => {
                // 解析日期
                let timestamp: i64 = context.context_id.parse().map_err(|_| {
                    AppError::ValidationFailed(vec![ValidationError::new(
                        "context.id",
                        "Invalid DAILY_KANBAN context_id",
                        "INVALID_CONTEXT_ID",
                    )])
                })?;
                let scheduled_day =
                    chrono::DateTime::from_timestamp(timestamp, 0).ok_or_else(|| {
                        AppError::ValidationFailed(vec![ValidationError::new(
                            "context.id",
                            "Invalid DAILY_KANBAN timestamp",
                            "INVALID_TIMESTAMP",
                        )])
                    })?;

                database::create_schedule_and_ordering_in_tx(
                    &mut tx,
                    &created_task,
                    scheduled_day.with_timezone(&Utc),
                    now,
                )
                .await?;
            }
            ContextType::ProjectList => {
                // 从 context_id 提取 project_id
                if !context.context_id.starts_with("project::") {
                    return Err(AppError::ValidationFailed(vec![ValidationError::new(
                        "context.id",
                        "PROJECT_LIST context_id must start with 'project::'",
                        "INVALID_CONTEXT_ID",
                    )]));
                }
                let project_id_str = &context.context_id[9..];
                let project_id = Uuid::parse_str(project_id_str).map_err(|_| {
                    AppError::ValidationFailed(vec![ValidationError::new(
                        "context.id",
                        "Invalid project UUID",
                        "INVALID_PROJECT_UUID",
                    )])
                })?;

                database::create_ordering_in_tx(
                    &mut tx,
                    &created_task,
                    context.context_type,
                    context.context_id,
                    now,
                )
                .await?;
            }
            ContextType::Misc => {
                database::create_ordering_in_tx(
                    &mut tx,
                    &created_task,
                    context.context_type,
                    context.context_id,
                    now,
                )
                .await?;
            }
            _ => {
                return Err(AppError::ValidationFailed(vec![ValidationError::new(
                    "context.type",
                    "Unsupported context type",
                    "UNSUPPORTED_CONTEXT_TYPE",
                )]));
            }
        }

        // 6. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(created_task)
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
mod database {
    use super::*;
    use crate::entities::{
        schedule::TaskSchedule,
        task::{Outcome, TaskRow},
        template::TemplateRow,
    };
    use chrono::DateTime;

    pub async fn find_template_by_id_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        template_id: Uuid,
    ) -> AppResult<Option<Template>> {
        let row = sqlx::query_as::<_, TemplateRow>(
            r#"
            SELECT id, name, title_template, glance_note_template, detail_note_template,
                   estimated_duration_template, subtasks_template, area_id,
                   created_at, updated_at, is_deleted
            FROM templates WHERE id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(template_id.to_string())
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        row.map(|r| Template::try_from(r))
            .transpose()
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }

    pub async fn create_task_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task: &Task,
    ) -> AppResult<Task> {
        let subtasks_json = task
            .subtasks
            .as_ref()
            .and_then(|s| serde_json::to_string(s).ok());

        let due_date_type_json = task
            .due_date_type
            .as_ref()
            .and_then(|dt| serde_json::to_string(dt).ok());

        let source_info_json = task
            .source_info
            .as_ref()
            .and_then(|s| serde_json::to_string(s).ok());

        let external_metadata_json = task
            .external_source_metadata
            .as_ref()
            .and_then(|m| serde_json::to_string(m).ok());

        let recurrence_exclusions_json = task
            .recurrence_exclusions
            .as_ref()
            .and_then(|e| serde_json::to_string(e).ok());

        let row = sqlx::query_as::<_, TaskRow>(
            r#"
            INSERT INTO tasks (
                id, title, glance_note, detail_note, estimated_duration, subtasks,
                project_id, area_id, due_date, due_date_type, completed_at,
                created_at, updated_at, is_deleted, source_info,
                external_source_id, external_source_provider, external_source_metadata,
                recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id, title, glance_note, detail_note, estimated_duration, subtasks,
                      project_id, area_id, due_date, due_date_type, completed_at,
                      created_at, updated_at, is_deleted, source_info,
                      external_source_id, external_source_provider, external_source_metadata,
                      recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            "#,
        )
        .bind(task.id.to_string())
        .bind(&task.title)
        .bind(&task.glance_note)
        .bind(&task.detail_note)
        .bind(task.estimated_duration)
        .bind(subtasks_json)
        .bind(task.project_id.map(|id| id.to_string()))
        .bind(task.area_id.map(|id| id.to_string()))
        .bind(task.due_date.map(|dt| dt.to_rfc3339()))
        .bind(due_date_type_json)
        .bind(task.completed_at.map(|dt| dt.to_rfc3339()))
        .bind(task.created_at.to_rfc3339())
        .bind(task.updated_at.to_rfc3339())
        .bind(task.is_deleted)
        .bind(source_info_json)
        .bind(&task.external_source_id)
        .bind(&task.external_source_provider)
        .bind(external_metadata_json)
        .bind(&task.recurrence_rule)
        .bind(task.recurrence_parent_id.map(|id| id.to_string()))
        .bind(task.recurrence_original_date.map(|dt| dt.to_rfc3339()))
        .bind(recurrence_exclusions_json)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        Task::try_from(row)
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }

    pub async fn create_schedule_and_ordering_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task: &Task,
        scheduled_day: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        let schedule_id = Uuid::new_v4();
        let schedule = TaskSchedule::new(schedule_id, task.id, scheduled_day, now);

        let outcome_str = match schedule.outcome {
            Outcome::Planned => "PLANNED",
            Outcome::PresenceLogged => "PRESENCE_LOGGED",
            Outcome::CompletedOnDay => "COMPLETED_ON_DAY",
            Outcome::CarriedOver => "CARRIED_OVER",
        };

        sqlx::query(
            r#"
            INSERT INTO task_schedules (id, task_id, scheduled_day, outcome, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(schedule.id.to_string())
        .bind(schedule.task_id.to_string())
        .bind(schedule.scheduled_day.to_rfc3339())
        .bind(outcome_str)
        .bind(schedule.created_at.to_rfc3339())
        .bind(schedule.updated_at.to_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        // 创建排序记录
        // 使用日期的 RFC3339 字符串作为 context_id，而不是时间戳
        let context_id = scheduled_day.to_rfc3339();
        let sort_order =
            crate::shared::core::utils::sort_order_utils::generate_initial_sort_order();

        sqlx::query(
            r#"
            INSERT INTO ordering (id, context_type, context_id, task_id, sort_order, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind("DAILY_KANBAN")
        .bind(&context_id)
        .bind(task.id.to_string())
        .bind(&sort_order)
        .bind(now.to_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        Ok(())
    }

    pub async fn create_ordering_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task: &Task,
        context_type: ContextType,
        context_id: String,
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        let context_type_str = match context_type {
            ContextType::DailyKanban => "DAILY_KANBAN",
            ContextType::ProjectList => "PROJECT_LIST",
            ContextType::Misc => "MISC",
            ContextType::AreaFilter => "AREA_FILTER",
        };

        let sort_order =
            crate::shared::core::utils::sort_order_utils::generate_initial_sort_order();

        sqlx::query(
            r#"
            INSERT INTO ordering (id, context_type, context_id, task_id, sort_order, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(Uuid::new_v4().to_string())
        .bind(context_type_str)
        .bind(&context_id)
        .bind(task.id.to_string())
        .bind(&sort_order)
        .bind(now.to_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        Ok(())
    }
}
