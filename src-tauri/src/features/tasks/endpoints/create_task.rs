/// 创建任务 API - 单文件组件
///
/// 按照单文件组件模式实现
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{CreateTaskRequest, ScheduleStatus, Task, TaskCardDto},
    features::tasks::shared::TaskAssembler,
    shared::{
        core::{AppError, AppResult},
        http::error_handler::created_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `create_task`

## API端点
POST /api/tasks

## 预期行为简介
创建一个新任务。

## 输入输出规范
- **前置条件**:
  - title 不能为空，长度必须在 1-255 之间
  - area_id（如果提供）必须存在
- **后置条件**:
  - 在 tasks 表中创建新任务
  - 返回完整的 TaskCardDto

## 边界情况
- 如果 title 为空，返回 400 Bad Request
- 如果 title 超过 255 字符，返回 400 Bad Request
- 如果 area_id 不存在，返回 404 Not Found

## 预期副作用
- 插入一条 tasks 记录

## 事务保证
- 所有数据库操作在单个事务中执行
- 如果任何步骤失败，整个操作回滚

## 请求/响应示例
Request:
{
  "title": "新任务",
  "glance_note": "快速笔记",
  "area_id": "..."
}

Response: 201 Created
{
  "id": "...",
  "title": "新任务",
  "schedule_status": "staging",
  ...
}
*/

// ==================== HTTP 处理器 ====================
/// 创建任务的 HTTP 处理器
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateTaskRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(task_card) => created_response(task_card).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate_create_request(request: &CreateTaskRequest) -> AppResult<()> {
        // 验证标题
        if request.title.trim().is_empty() {
            return Err(AppError::validation_error(
                "title",
                "任务标题不能为空",
                "TITLE_EMPTY",
            ));
        }

        if request.title.len() > 255 {
            return Err(AppError::validation_error(
                "title",
                "任务标题不能超过255个字符",
                "TITLE_TOO_LONG",
            ));
        }

        // 验证预估时长
        if let Some(duration) = request.estimated_duration {
            if duration < 0 {
                return Err(AppError::validation_error(
                    "estimated_duration",
                    "预估时长不能为负数",
                    "DURATION_NEGATIVE",
                ));
            }
            if duration > 24 * 60 * 7 {
                return Err(AppError::validation_error(
                    "estimated_duration",
                    "预估时长不能超过一周",
                    "DURATION_TOO_LONG",
                ));
            }
        }

        // 验证子任务数量
        if let Some(subtasks) = &request.subtasks {
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
        request: CreateTaskRequest,
    ) -> AppResult<TaskCardDto> {
        // 1. 验证请求
        validation::validate_create_request(&request)?;

        // 2. 开始事务
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 3. 生成 UUID 和时间戳
        let task_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // 4. 创建任务实体
        let task = Task {
            id: task_id,
            title: request.title.clone(),
            glance_note: request.glance_note.clone(),
            detail_note: request.detail_note.clone(),
            estimated_duration: request.estimated_duration,
            subtasks: request.subtasks.clone(),
            project_id: None,
            area_id: request.area_id,
            due_date: request.due_date,
            due_date_type: request.due_date_type,
            completed_at: None,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            source_info: None,
            external_source_id: None,
            external_source_provider: None,
            external_source_metadata: None,
            recurrence_rule: None,
            recurrence_parent_id: None,
            recurrence_original_date: None,
            recurrence_exclusions: None,
        };

        // 5. 插入任务到数据库
        database::insert_task_in_tx(&mut tx, &task).await?;

        // 6. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 7. 组装返回的 TaskCardDto
        let mut task_card = TaskAssembler::task_to_card_basic(&task);
        task_card.schedule_status = ScheduleStatus::Staging;

        // 获取 area 信息（如果有）
        if let Some(area_id) = task.area_id {
            // 在事务外查询，因为已经提交了
            task_card.area = database::get_area_summary(app_state.db_pool(), area_id).await?;
        }

        Ok(task_card)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::entities::AreaSummary;

    /// 在事务中插入任务
    pub async fn insert_task_in_tx(tx: &mut Transaction<'_, Sqlite>, task: &Task) -> AppResult<()> {
        let query = r#"
            INSERT INTO tasks (
                id, title, glance_note, detail_note, estimated_duration, subtasks,
                project_id, area_id, due_date, due_date_type, completed_at,
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
                    .map(|s| serde_json::to_string(s).unwrap()),
            )
            .bind(task.project_id.map(|id| id.to_string()))
            .bind(task.area_id.map(|id| id.to_string()))
            .bind(task.due_date.map(|d| d.to_rfc3339()))
            .bind(
                task.due_date_type
                    .as_ref()
                    .map(|t| serde_json::to_string(t).unwrap()),
            )
            .bind(task.completed_at.map(|d| d.to_rfc3339()))
            .bind(task.created_at.to_rfc3339())
            .bind(task.updated_at.to_rfc3339())
            .bind(task.is_deleted)
            .bind(
                task.source_info
                    .as_ref()
                    .map(|s| serde_json::to_string(s).unwrap()),
            )
            .bind(&task.external_source_id)
            .bind(&task.external_source_provider)
            .bind(
                task.external_source_metadata
                    .as_ref()
                    .map(|m| serde_json::to_string(m).unwrap()),
            )
            .bind(&task.recurrence_rule)
            .bind(task.recurrence_parent_id.map(|id| id.to_string()))
            .bind(task.recurrence_original_date.map(|d| d.to_rfc3339()))
            .bind(
                task.recurrence_exclusions
                    .as_ref()
                    .map(|e| serde_json::to_string(e).unwrap()),
            )
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(())
    }

    /// 获取区域摘要信息
    pub async fn get_area_summary(
        pool: &sqlx::SqlitePool,
        area_id: Uuid,
    ) -> AppResult<Option<AreaSummary>> {
        let query = r#"
            SELECT id, name, color
            FROM areas
            WHERE id = ? AND is_deleted = false
        "#;

        let result = sqlx::query_as::<_, (String, String, String)>(query)
            .bind(area_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(
            result.map(|(id, name, color)| crate::entities::AreaSummary {
                id: Uuid::parse_str(&id).unwrap(),
                name,
                color,
            }),
        )
    }
}
