/// 更新任务 API - 单文件组件
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    entities::{ScheduleStatus, TaskCardDto, UpdateTaskRequest},
    features::{
        shared::repositories::AreaRepository,
        tasks::shared::{
            assemblers::TimeBlockAssembler,
            repositories::{TaskRepository, TaskScheduleRepository, TaskTimeBlockLinkRepository},
            TaskAssembler,
        },
    },
    shared::{
        core::{AppError, AppResult},
        http::{error_handler::success_response, extractors::extract_correlation_id},
    },
    startup::AppState,
};

/// 更新任务的响应
#[derive(Debug, Serialize)]
pub struct UpdateTaskResponse {
    pub task: TaskCardDto,
    // 注意：副作用（updated time blocks）已通过 SSE 推送
}

// ==================== 文档层 ====================
/*
CABC for `update_task`

## API端点
PATCH /api/tasks/{id}

## 预期行为简介
更新任务的可变字段（标题、笔记、子任务等）。
当标题或 area 变更时，自动更新所有唯一关联的时间块。

## 输入输出规范
- **前置条件**: task_id 必须存在
- **后置条件**: 任务字段被更新，返回最新的 TaskCardDto

## Cutie 业务逻辑
1. 更新任务字段
2. 如果标题或 area 有变更，查询所有唯一关联的时间块
3. 更新这些时间块的标题和 area（与任务保持一致）
4. 通过 SSE 推送更新事件

## 边界情况
- 任务不存在 → 404
- 所有字段都是 None → 422（无需更新）
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<UpdateTaskRequest>,
) -> Response {
    let correlation_id = extract_correlation_id(&headers);
    match logic::execute(&app_state, task_id, request, correlation_id).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate_update_request(request: &UpdateTaskRequest) -> AppResult<()> {
        tracing::trace!("Entering validation::validate_update_request");
        println!("Entering validation::validate_update_request");
        // 检查是否为空更新
        // if request.is_empty() {
        //     return Err(AppError::validation_error(
        //         "request",
        //         "至少需要更新一个字段",
        //         "EMPTY_UPDATE",
        //     ));
        // }

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
    use crate::features::shared::TransactionHelper;

    pub async fn execute(
        app_state: &AppState,
        task_id: Uuid,
        request: UpdateTaskRequest,
        correlation_id: Option<String>,
    ) -> AppResult<UpdateTaskResponse> {
        // 1. 验证
        validation::validate_update_request(&request)?;
        println!("Exiting validation::validate_update_request");

        let now = app_state.clock().now_utc();

        // 2. 开启事务（✅ 使用 TransactionHelper）
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 3. 查询旧任务数据（✅ 使用共享 Repository）
        let old_task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 4. 更新任务（✅ 使用共享 Repository）
        TaskRepository::update_in_tx(&mut tx, task_id, &request).await?;

        // 5. 检查标题或 area 是否有变更
        let title_changed =
            request.title.is_some() && request.title.as_ref() != Some(&old_task.title);
        let area_changed = request.area_id.is_some() && request.area_id != Some(old_task.area_id);

        // 6. 如果标题或 area 有变更，更新唯一关联的时间块（✅ 使用共享 Repository）
        let mut updated_time_block_ids = Vec::new();
        if title_changed || area_changed {
            let linked_blocks =
                TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx(&mut tx, task_id)
                    .await?;

            for block in linked_blocks {
                // 检查是否是唯一关联（✅ 使用共享 Repository）
                let is_exclusive = TaskTimeBlockLinkRepository::is_exclusive_link_in_tx(
                    &mut tx, block.id, task_id,
                )
                .await?;
                if !is_exclusive {
                    continue;
                }

                // 检查标题是否一致（自动创建的时间块）
                let is_auto_created = block
                    .title
                    .as_ref()
                    .map(|t| t == &old_task.title)
                    .unwrap_or(false);

                if !is_auto_created {
                    // 手动创建的时间块，不自动更新
                    continue;
                }

                // 更新时间块的标题和 area（✅ 调用数据访问层）
                database::update_time_block_title_and_area_in_tx(
                    &mut tx,
                    block.id,
                    request.title.as_deref(),
                    request.area_id,
                    now,
                )
                .await?;

                updated_time_block_ids.push(block.id);
                tracing::info!(
                    "Updated exclusive time block {} for task {}",
                    block.id,
                    task_id
                );
            }
        }

        // 7. 查询更新后的完整时间块数据（✅ 使用共享装配器）
        let updated_blocks =
            TimeBlockAssembler::assemble_for_event_in_tx(&mut tx, &updated_time_block_ids).await?;

        // 8. 重新查询任务以获取最新数据（✅ 使用共享 Repository）
        let task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 9. 组装 TaskCardDto（用于事件载荷）
        let mut task_card_for_event = TaskAssembler::task_to_card_basic(&task);

        // 9.1. 在事务内查询关联信息（✅ 使用共享 Repository）
        let has_schedule = TaskScheduleRepository::has_any_schedule(&mut *tx, task_id).await?;
        task_card_for_event.schedule_status = if has_schedule {
            ScheduleStatus::Scheduled
        } else {
            ScheduleStatus::Staging
        };

        if let Some(area_id) = task.area_id {
            task_card_for_event.area = AreaRepository::get_summary(&mut *tx, area_id).await?;
        }

        // 10. 在事务中写入领域事件到 outbox
        use crate::shared::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        {
            let payload = serde_json::json!({
                "task": task_card_for_event,
                "side_effects": {
                    "updated_time_blocks": updated_blocks,
                }
            });
            let mut event = DomainEvent::new("task.updated", "task", task_id.to_string(), payload)
                .with_aggregate_version(now.timestamp_millis());

            // 关联 correlation_id（用于前端去重和请求追踪）
            if let Some(cid) = correlation_id {
                event = event.with_correlation_id(cid);
            }

            outbox_repo.append_in_tx(&mut tx, &event).await?;
        }

        // 11. 提交事务（✅ 使用 TransactionHelper）
        TransactionHelper::commit(tx).await?;

        // 12. 返回结果（复用事件中的 task_card）
        // HTTP 响应与 SSE 事件载荷保持一致
        Ok(UpdateTaskResponse {
            task: task_card_for_event,
        })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use chrono::{DateTime, Utc};
    use sqlx::{Sqlite, Transaction};

    /// 更新时间块的标题和 area（仅用于任务更新时的联动更新）
    pub async fn update_time_block_title_and_area_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
        new_title: Option<&str>,
        new_area_id: Option<Option<Uuid>>, // None: 不更新; Some(None): 置 NULL; Some(Some(id)): 设置
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        let mut set_clauses = Vec::new();
        let mut binds: Vec<String> = Vec::new();

        if let Some(title) = new_title {
            set_clauses.push("title = ?");
            binds.push(title.to_string());
        }

        if let Some(area_opt) = new_area_id {
            set_clauses.push("area_id = ?");
            binds.push(area_opt.map(|id| id.to_string()).unwrap_or_default());
        }

        if set_clauses.is_empty() {
            return Ok(()); // 没有需要更新的字段
        }

        set_clauses.push("updated_at = ?");
        let update_clause = set_clauses.join(", ");
        let query = format!("UPDATE time_blocks SET {} WHERE id = ?", update_clause);

        let mut q = sqlx::query(&query);
        for bind in binds {
            q = q.bind(bind);
        }
        q = q.bind(now.to_rfc3339());
        q = q.bind(block_id.to_string());

        q.execute(&mut **tx).await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        Ok(())
    }
}

// ✅ 已迁移到共享 Repository：
// - TaskRepository::find_by_id_in_tx, update_in_tx
// - TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx, is_exclusive_link_in_tx
// - TaskScheduleRepository::has_any_schedule
// - AreaRepository::get_summary
// - TimeBlockAssembler::assemble_for_event_in_tx
