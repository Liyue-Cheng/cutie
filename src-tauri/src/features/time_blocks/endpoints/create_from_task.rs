/// 从任务创建时间块 API - 单文件组件
///
/// 专门处理"拖动任务到日历"的场景
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{
        task::response_dtos::{AreaSummary, TaskCardDto},
        LinkedTaskSummary, ScheduleStatus, Task, TimeBlock, TimeBlockViewDto,
    },
    features::tasks::shared::TaskAssembler,
    shared::{
        core::{AppError, AppResult},
        http::error_handler::created_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `create_time_block_from_task`

## API端点
POST /api/time-blocks/from-task

## 预期行为简介
从拖动的任务创建时间块。这是专门为"拖动任务到日历"场景设计的端点。
会同时：
1. 创建时间块
2. 链接任务到时间块
3. 创建任务的日程记录（task_schedules）
4. 返回更新后的任务卡片

## 输入输出规范
- **前置条件**:
  - task_id 必须存在
  - start_time < end_time
  - 时间块不与现有时间块重叠
- **后置条件**:
  - 创建 time_blocks 记录
  - 创建 task_time_block_links 记录
  - 创建 task_schedules 记录
  - 返回时间块和更新后的任务

## 边界情况
- 如果任务不存在，返回 404
- 如果时间冲突，返回 409
*/

// ==================== 请求/响应结构 ====================
#[derive(Debug, Deserialize)]
pub struct CreateFromTaskRequest {
    pub task_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub title: Option<String>, // 可选，默认使用任务标题
}

#[derive(Debug, Serialize)]
pub struct CreateFromTaskResponse {
    pub time_block: TimeBlockViewDto,
    pub updated_task: TaskCardDto, // 更新后的任务（schedule_status = 'scheduled'）
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateFromTaskRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(response) => created_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate_request(request: &CreateFromTaskRequest) -> AppResult<()> {
        if request.start_time >= request.end_time {
            return Err(AppError::validation_error(
                "time_range",
                "开始时间必须早于结束时间",
                "INVALID_TIME_RANGE",
            ));
        }
        Ok(())
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        request: CreateFromTaskRequest,
    ) -> AppResult<CreateFromTaskResponse> {
        // 1. 验证
        validation::validate_request(&request)?;

        // 2. 开始事务
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 3. 检查任务是否存在
        let task = database::find_task_by_id_in_tx(&mut tx, request.task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", request.task_id.to_string()))?;

        // 4. 检查时间冲突
        let has_conflict = database::check_time_conflict_in_tx(
            &mut tx,
            &request.start_time,
            &request.end_time,
            None,
        )
        .await?;

        if has_conflict {
            return Err(AppError::conflict("该时间段与现有时间块重叠"));
        }

        // 5. 生成 UUID 和时间戳
        let block_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // 6. 创建时间块（使用任务标题或自定义标题）
        let title = request.title.or_else(|| Some(task.title.clone()));

        let time_block = TimeBlock {
            id: block_id,
            title,
            glance_note: None,
            detail_note: None,
            start_time: request.start_time,
            end_time: request.end_time,
            area_id: task.area_id, // 继承任务的 area
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

        database::insert_time_block_in_tx(&mut tx, &time_block).await?;

        // 7. 链接任务到时间块
        database::link_task_to_block_in_tx(&mut tx, request.task_id, block_id).await?;

        // 8. 创建日程记录
        let scheduled_day = request
            .start_time
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        let has_schedule =
            database::has_schedule_for_day_in_tx(&mut tx, request.task_id, scheduled_day).await?;
        if !has_schedule {
            database::create_task_schedule_in_tx(&mut tx, request.task_id, scheduled_day).await?;
        }

        // 9. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 10. 组装返回数据
        let mut time_block_view = TimeBlockViewDto {
            id: time_block.id,
            start_time: time_block.start_time,
            end_time: time_block.end_time,
            title: time_block.title,
            glance_note: time_block.glance_note,
            detail_note: time_block.detail_note,
            area: None,
            linked_tasks: vec![LinkedTaskSummary {
                id: task.id,
                title: task.title.clone(),
                is_completed: task.is_completed(),
            }],
            is_recurring: false,
        };

        // 11. 获取区域信息
        if let Some(area_id) = time_block.area_id {
            time_block_view.area = database::get_area_summary(app_state.db_pool(), area_id).await?;
        }

        // 12. 组装更新后的 TaskCard
        let mut updated_task = TaskAssembler::task_to_card_basic(&task);
        updated_task.schedule_status = ScheduleStatus::Scheduled; // 明确设置

        // 获取 area
        if let Some(area_id) = task.area_id {
            updated_task.area = database::get_area_summary(app_state.db_pool(), area_id).await?;
        }

        Ok(CreateFromTaskResponse {
            time_block: time_block_view,
            updated_task,
        })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::entities::TaskRow;

    pub async fn find_task_by_id_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
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
            .fetch_optional(&mut **tx)
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

    pub async fn check_time_conflict_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        start_time: &DateTime<Utc>,
        end_time: &DateTime<Utc>,
        exclude_id: Option<Uuid>,
    ) -> AppResult<bool> {
        let mut query = String::from(
            r#"
            SELECT COUNT(*) as count
            FROM time_blocks
            WHERE is_deleted = false
              AND start_time < ?
              AND end_time > ?
        "#,
        );

        if let Some(id) = exclude_id {
            query.push_str(&format!(" AND id != '{}'", id));
        }

        let count: i64 = sqlx::query_scalar(&query)
            .bind(end_time.to_rfc3339())
            .bind(start_time.to_rfc3339())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(count > 0)
    }

    pub async fn insert_time_block_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block: &TimeBlock,
    ) -> AppResult<()> {
        let query = r#"
            INSERT INTO time_blocks (
                id, title, glance_note, detail_note, start_time, end_time, area_id,
                created_at, updated_at, is_deleted, source_info,
                external_source_id, external_source_provider, external_source_metadata,
                recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(block.id.to_string())
            .bind(&block.title)
            .bind(&block.glance_note)
            .bind(&block.detail_note)
            .bind(block.start_time.to_rfc3339())
            .bind(block.end_time.to_rfc3339())
            .bind(block.area_id.map(|id| id.to_string()))
            .bind(block.created_at.to_rfc3339())
            .bind(block.updated_at.to_rfc3339())
            .bind(block.is_deleted)
            .bind(
                block
                    .source_info
                    .as_ref()
                    .map(|s| serde_json::to_string(s).unwrap()),
            )
            .bind(&block.external_source_id)
            .bind(&block.external_source_provider)
            .bind(
                block
                    .external_source_metadata
                    .as_ref()
                    .map(|m| serde_json::to_string(m).unwrap()),
            )
            .bind(&block.recurrence_rule)
            .bind(block.recurrence_parent_id.map(|id| id.to_string()))
            .bind(block.recurrence_original_date.map(|d| d.to_rfc3339()))
            .bind(
                block
                    .recurrence_exclusions
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

    pub async fn link_task_to_block_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        block_id: Uuid,
    ) -> AppResult<()> {
        let now = Utc::now();

        let query = r#"
            INSERT INTO task_time_block_links (task_id, time_block_id, created_at)
            VALUES (?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(task_id.to_string())
            .bind(block_id.to_string())
            .bind(now.to_rfc3339())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(())
    }

    pub async fn has_schedule_for_day_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        scheduled_day: DateTime<Utc>,
    ) -> AppResult<bool> {
        let query = r#"
            SELECT COUNT(*) as count
            FROM task_schedules
            WHERE task_id = ? AND DATE(scheduled_day) = DATE(?)
        "#;

        let count: i64 = sqlx::query_scalar(query)
            .bind(task_id.to_string())
            .bind(scheduled_day.to_rfc3339())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(count > 0)
    }

    pub async fn create_task_schedule_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        scheduled_day: DateTime<Utc>,
    ) -> AppResult<()> {
        let schedule_id = Uuid::new_v4();
        let now = Utc::now();

        let query = r#"
            INSERT INTO task_schedules (id, task_id, scheduled_day, outcome, created_at, updated_at)
            VALUES (?, ?, ?, 'PLANNED', ?, ?)
        "#;

        sqlx::query(query)
            .bind(schedule_id.to_string())
            .bind(task_id.to_string())
            .bind(scheduled_day.to_rfc3339())
            .bind(now.to_rfc3339())
            .bind(now.to_rfc3339())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(())
    }

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

        Ok(result.map(|(id, name, color)| AreaSummary {
            id: Uuid::parse_str(&id).unwrap(),
            name,
            color,
        }))
    }
}
