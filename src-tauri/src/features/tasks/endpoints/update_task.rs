/// 更新任务 API - 单文件组件
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{
        task::response_dtos::AreaSummary, ScheduleStatus, Task, TaskCardDto, UpdateTaskRequest,
    },
    features::tasks::shared::TaskAssembler,
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

        // 2. 开启事务
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 3. 查询旧任务数据（用于比较变更）
        let old_task = database::find_task_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 4. 更新任务
        database::update_task_in_tx(&mut tx, task_id, &request).await?;

        // 5. 检查标题或 area 是否有变更
        let title_changed =
            request.title.is_some() && request.title.as_ref() != Some(&old_task.title);
        let area_changed = request.area_id.is_some() && request.area_id != Some(old_task.area_id);

        // 6. 如果标题或 area 有变更，更新唯一关联的时间块
        let mut updated_time_block_ids = Vec::new();
        if title_changed || area_changed {
            let linked_blocks = database::find_linked_time_blocks_in_tx(&mut tx, task_id).await?;

            for block in linked_blocks {
                // 检查是否是唯一关联
                let is_exclusive =
                    database::is_exclusive_link_in_tx(&mut tx, block.id, task_id).await?;
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

                // 更新时间块的标题和 area
                let new_title = request.title.clone();
                let new_area_id = request.area_id.clone(); // 保留三态：不更新/置空/设置值

                database::update_time_block_in_tx(
                    &mut tx,
                    block.id,
                    new_title.as_deref(),
                    new_area_id,
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

        // 7. 查询更新后的完整时间块数据（用于事件）
        let updated_blocks = if !updated_time_block_ids.is_empty() {
            database::find_time_blocks_for_event(&mut tx, &updated_time_block_ids).await?
        } else {
            Vec::new()
        };

        // 8. 重新查询任务以获取最新数据
        let task = database::find_task_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 9. 组装 TaskCardDto（用于事件载荷）
        let mut task_card_for_event = TaskAssembler::task_to_card_basic(&task);

        // 9.1. 在事务内查询关联信息，确保 SSE 事件中的任务数据是完整的
        let has_schedule = database::has_any_schedule_in_tx(&mut tx, task_id).await?;
        task_card_for_event.schedule_status = if has_schedule {
            ScheduleStatus::Scheduled
        } else {
            ScheduleStatus::Staging
        };

        if let Some(area_id) = task.area_id {
            task_card_for_event.area = database::get_area_summary_in_tx(&mut tx, area_id).await?;
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

        // 11. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

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
    use crate::entities::{TaskRow, TimeBlock, TimeBlockRow};

    pub async fn find_task_in_tx(
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

    pub async fn update_task_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        request: &UpdateTaskRequest,
    ) -> AppResult<()> {
        let now = chrono::Utc::now();

        // tracing::info!("📝 update_task_in_tx: request = {:?}", request);

        // 收集需要更新的列
        let mut set_clauses: Vec<&str> = Vec::new();
        if request.title.is_some() {
            set_clauses.push("title = ?");
        }
        if request.glance_note.is_some() {
            set_clauses.push("glance_note = ?");
            // tracing::info!("  glance_note will be set to: {:?}", request.glance_note);
        }
        if request.detail_note.is_some() {
            set_clauses.push("detail_note = ?");
            // tracing::info!("  detail_note will be set to: {:?}", request.detail_note);
        }
        if request.subtasks.is_some() {
            set_clauses.push("subtasks = ?");
        }
        if request.area_id.is_some() {
            set_clauses.push("area_id = ?");
            // tracing::info!("  area_id will be set to: {:?}", request.area_id);
        }

        if set_clauses.is_empty() {
            return Ok(());
        }

        // 追加更新时间
        set_clauses.push("updated_at = ?");
        let update_clause = set_clauses.join(", ");
        let query = format!("UPDATE tasks SET {} WHERE id = ?", update_clause);

        let mut q = sqlx::query(&query);

        // 按顺序绑定各字段的值（正确处理 NULL）
        if let Some(title) = &request.title {
            q = q.bind(title.clone());
        }
        if let Some(glance_note) = &request.glance_note {
            // Option<Option<String>>: None = 不更新, Some(None) = 设为 NULL, Some(Some(v)) = 设为 v
            q = q.bind(glance_note.clone());
        }
        if let Some(detail_note) = &request.detail_note {
            q = q.bind(detail_note.clone());
        }
        if let Some(subtasks) = &request.subtasks {
            // 将 Vec<Subtask> 序列化为 JSON 字符串；None 表示置 NULL
            let value: Option<String> = match subtasks {
                Some(list) => Some(serde_json::to_string(list).map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::QueryError(e.to_string()))
                })?),
                None => None,
            };
            q = q.bind(value);
        }
        if let Some(area_id) = &request.area_id {
            // None 表示置 NULL；Some(uuid) 表示设置；转换为 Option<String>
            let bind_val: Option<String> = area_id.map(|id| id.to_string());
            q = q.bind(bind_val);
        }

        // 绑定 updated_at 与 id
        q = q.bind(now.to_rfc3339());
        q = q.bind(task_id.to_string());

        let result = q.execute(&mut **tx).await.map_err(|e| {
            tracing::error!("❌ SQL execution error: {:?}", e);
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        tracing::info!(
            "✅ Task updated, rows_affected = {}",
            result.rows_affected()
        );

        Ok(())
    }

    pub async fn has_any_schedule_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<bool> {
        let query = "SELECT COUNT(*) FROM task_schedules WHERE task_id = ?";
        let count: i64 = sqlx::query_scalar(query)
            .bind(task_id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(count > 0)
    }

    pub async fn get_area_summary_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        area_id: Uuid,
    ) -> AppResult<Option<AreaSummary>> {
        let query = "SELECT id, name, color FROM areas WHERE id = ? AND is_deleted = false";
        let result = sqlx::query_as::<_, (String, String, String)>(query)
            .bind(area_id.to_string())
            .fetch_optional(&mut **tx)
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

    pub async fn find_linked_time_blocks_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<Vec<TimeBlock>> {
        let query = r#"
            SELECT DISTINCT
                tb.id, tb.title, tb.glance_note, tb.detail_note, tb.start_time, tb.end_time, 
                tb.area_id, tb.created_at, tb.updated_at, tb.is_deleted, tb.source_info,
                tb.external_source_id, tb.external_source_provider, tb.external_source_metadata,
                tb.recurrence_rule, tb.recurrence_parent_id, tb.recurrence_original_date, 
                tb.recurrence_exclusions
            FROM time_blocks tb
            INNER JOIN task_time_block_links ttbl ON tb.id = ttbl.time_block_id
            WHERE ttbl.task_id = ? AND tb.is_deleted = false
        "#;

        let rows = sqlx::query_as::<_, TimeBlockRow>(query)
            .bind(task_id.to_string())
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        let blocks: Result<Vec<TimeBlock>, _> = rows.into_iter().map(TimeBlock::try_from).collect();

        blocks.map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }

    /// 检查时间块是否仅链接此任务
    pub async fn is_exclusive_link_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
        _task_id: Uuid, // 用于未来验证，当前只检查总数
    ) -> AppResult<bool> {
        let query = r#"
            SELECT COUNT(*) as count
            FROM task_time_block_links
            WHERE time_block_id = ?
        "#;

        let total_count: i64 = sqlx::query_scalar(query)
            .bind(block_id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        // 如果只有1个链接，且是这个任务，则为独占
        Ok(total_count == 1)
    }

    /// 更新时间块的标题和 area
    pub async fn update_time_block_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
        new_title: Option<&str>,
        new_area_id: Option<Option<Uuid>>, // None: 不更新; Some(None): 置 NULL; Some(Some(id)): 设置
        now: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<()> {
        let mut set_clauses: Vec<&str> = Vec::new();
        if new_title.is_some() {
            set_clauses.push("title = ?");
        }
        if new_area_id.is_some() {
            set_clauses.push("area_id = ?");
        }

        if set_clauses.is_empty() {
            return Ok(());
        }

        set_clauses.push("updated_at = ?");
        let update_clause = set_clauses.join(", ");
        let query = format!("UPDATE time_blocks SET {} WHERE id = ?", update_clause);

        let mut q = sqlx::query(&query);
        if let Some(title) = new_title {
            q = q.bind(title.to_string());
        }
        if let Some(area_opt) = new_area_id {
            // 正确处理 Option<Uuid>: None = NULL, Some(id) = 值
            let bind_val: Option<String> = area_opt.map(|id| id.to_string());
            q = q.bind(bind_val);
        }
        q = q.bind(now.to_rfc3339());
        q = q.bind(block_id.to_string());

        q.execute(&mut **tx).await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        Ok(())
    }

    /// 查询时间块的完整数据用于事件载荷
    /// ✅ 禁止片面数据：返回完整的 TimeBlockViewDto
    pub async fn find_time_blocks_for_event(
        tx: &mut Transaction<'_, Sqlite>,
        time_block_ids: &[Uuid],
    ) -> AppResult<Vec<crate::entities::TimeBlockViewDto>> {
        use crate::entities::{
            task::response_dtos::AreaSummary, LinkedTaskSummary, TimeBlockViewDto,
        };

        if time_block_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut result = Vec::new();

        for block_id in time_block_ids {
            // 1. 查询时间块（✅ 完整字段列表）
            let query = r#"
                SELECT id, title, glance_note, detail_note, start_time, end_time, area_id,
                       created_at, updated_at, is_deleted, source_info,
                       external_source_id, external_source_provider, external_source_metadata,
                       recurrence_rule, recurrence_parent_id, recurrence_original_date, recurrence_exclusions
                FROM time_blocks
                WHERE id = ? AND is_deleted = false
            "#;

            let block_row = sqlx::query_as::<_, TimeBlockRow>(query)
                .bind(block_id.to_string())
                .fetch_optional(&mut **tx)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
                })?;

            if let Some(row) = block_row {
                let block = TimeBlock::try_from(row).map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::QueryError(e))
                })?;

                // 2. 查询关联的任务
                let links_query = r#"
                    SELECT t.id, t.title, t.completed_at
                    FROM tasks t
                    INNER JOIN task_time_block_links l ON t.id = l.task_id
                    WHERE l.time_block_id = ? AND t.is_deleted = false
                "#;

                let linked_tasks_rows = sqlx::query_as::<
                    _,
                    (String, String, Option<chrono::DateTime<chrono::Utc>>),
                >(links_query)
                .bind(block_id.to_string())
                .fetch_all(&mut **tx)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
                })?;

                let linked_tasks: Vec<LinkedTaskSummary> = linked_tasks_rows
                    .into_iter()
                    .map(|(id, title, completed_at)| LinkedTaskSummary {
                        id: Uuid::parse_str(&id).unwrap(),
                        title,
                        is_completed: completed_at.is_some(),
                    })
                    .collect();

                // 3. 查询 Area 信息（如果有）
                let area = if let Some(area_id) = block.area_id {
                    let area_query = "SELECT id, name, color FROM areas WHERE id = ?";
                    sqlx::query_as::<_, (String, String, String)>(area_query)
                        .bind(area_id.to_string())
                        .fetch_optional(&mut **tx)
                        .await
                        .map_err(|e| {
                            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(
                                e,
                            ))
                        })?
                        .map(|(id, name, color)| AreaSummary {
                            id: Uuid::parse_str(&id).unwrap(),
                            name,
                            color,
                        })
                } else {
                    None
                };

                // 4. 组装 TimeBlockViewDto
                let view = TimeBlockViewDto {
                    id: block.id,
                    start_time: block.start_time,
                    end_time: block.end_time,
                    title: block.title,
                    glance_note: block.glance_note,
                    detail_note: block.detail_note,
                    area,
                    linked_tasks,
                    is_recurring: block.recurrence_rule.is_some(),
                };

                result.push(view);
            }
        }

        Ok(result)
    }
}
