/// 完成任务 API - 单文件组件
///
/// 按照 Cutie 的精确业务逻辑实现
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use chrono::Utc;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use serde::Serialize;

use crate::{
    entities::{TaskCardDto, TimeBlock},
    features::tasks::shared::TaskAssembler,
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

/// 完成任务的响应
#[derive(Debug, Serialize)]
pub struct CompleteTaskResponse {
    pub task: TaskCardDto,
    // 注意：副作用（deleted/truncated time blocks）已通过 SSE 推送
}

// ==================== 文档层 ====================
/*
CABC for `complete_task`

## API端点
POST /api/tasks/{id}/completion

## 预期行为简介
完成任务，并根据 Cutie 的业务规则智能处理相关的日程和时间块。

## Cutie 业务逻辑
1. 当天日程 → 设置为已完成（outcome = 'COMPLETED_ON_DAY'）
2. 未来日程 → 删除
3. 时间块（仅链接此任务 + 在过去） → 保留
4. 时间块（仅链接此任务 + 标题一致 + 正在发生） → 截断到 now
5. 时间块（仅链接此任务 + 标题一致 + 在未来） → 删除

## 输入输出规范
- **前置条件**: task_id 必须存在且未完成
- **后置条件**: 任务完成，相关数据清理

## 边界情况
- 任务不存在 → 404
- 任务已完成 → 409 Conflict
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>, Path(task_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, task_id).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, task_id: Uuid) -> AppResult<CompleteTaskResponse> {
        let now = app_state.clock().now_utc();

        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 1. 查找任务
        let task = database::find_task_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 2. 检查是否已完成
        if task.is_completed() {
            return Err(AppError::conflict("任务已经完成"));
        }

        // 3. 设置任务为已完成
        database::set_task_completed_in_tx(&mut tx, task_id, now).await?;

        // 4. 处理日程：当天设为完成，未来删除
        database::update_today_schedule_to_completed_in_tx(&mut tx, task_id, now).await?;
        database::delete_future_schedules_in_tx(&mut tx, task_id, now).await?;

        // 5. 查询所有链接的时间块
        let linked_blocks = database::find_linked_time_blocks_in_tx(&mut tx, task_id).await?;

        // 6. 第一遍：收集需要删除的时间块（✅ 在删除之前先查询完整数据）
        let mut blocks_to_delete = Vec::new();
        let mut blocks_to_truncate = Vec::new();
        let mut blocks_to_keep = Vec::new();

        for block in linked_blocks {
            let action =
                classify_time_block_action(&block, &task.title, task_id, now, &mut tx).await?;
            match action {
                TimeBlockAction::Deleted => blocks_to_delete.push(block),
                TimeBlockAction::Truncated => blocks_to_truncate.push(block),
                TimeBlockAction::None => blocks_to_keep.push(block),
            }
        }

        // 7. 查询将被删除的时间块的完整数据（✅ 在删除之前）
        let deleted_time_block_ids: Vec<Uuid> = blocks_to_delete.iter().map(|b| b.id).collect();
        let deleted_blocks = if !deleted_time_block_ids.is_empty() {
            database::find_time_blocks_for_event(&mut tx, &deleted_time_block_ids).await?
        } else {
            Vec::new()
        };

        // 8. 现在执行删除和截断操作
        for block in blocks_to_delete {
            database::delete_time_block_in_tx(&mut tx, block.id).await?;
            tracing::info!("Deleted future block {}", block.id);
        }

        let mut truncated_time_block_ids = Vec::new();
        for block in blocks_to_truncate {
            database::truncate_time_block_to_now_in_tx(&mut tx, block.id, now).await?;
            truncated_time_block_ids.push(block.id);
            tracing::info!("Truncated ongoing block {} to {}", block.id, now);
        }

        // 9. 查询被截断的时间块的完整数据（✅ 在截断之后，获取新的 end_time）
        let truncated_blocks = if !truncated_time_block_ids.is_empty() {
            database::find_time_blocks_for_event(&mut tx, &truncated_time_block_ids).await?
        } else {
            Vec::new()
        };

        // 10. 重新查询任务并组装完整 TaskCard（用于事件载荷）
        let updated_task_in_tx = database::find_task_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;
        let task_card_for_event = TaskAssembler::task_to_card_basic(&updated_task_in_tx);

        // 11. 在事务中写入领域事件到 outbox
        // ✅ 一个业务事务 = 一个领域事件（包含所有副作用的完整数据）
        use crate::shared::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        {
            let payload = serde_json::json!({
                "task": task_card_for_event,
                "side_effects": {
                    "deleted_time_blocks": deleted_blocks,     // ✅ 完整对象
                    "truncated_time_blocks": truncated_blocks, // ✅ 完整对象
                }
            });
            let event = DomainEvent::new("task.completed", "task", task_id.to_string(), payload)
                .with_aggregate_version(now.timestamp_millis());
            outbox_repo.append_in_tx(&mut tx, &event).await?;
        }

        // 12. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 13. 返回结果（复用事件中的 task_card）
        // HTTP 响应与 SSE 事件载荷保持一致
        Ok(CompleteTaskResponse {
            task: task_card_for_event,
        })
    }

    /// 时间块处理动作
    enum TimeBlockAction {
        None,      // 保留
        Truncated, // 截断
        Deleted,   // 删除
    }

    /// 分类时间块应该执行的动作（不实际执行）
    async fn classify_time_block_action(
        block: &TimeBlock,
        task_title: &str,
        task_id: Uuid,
        now: chrono::DateTime<Utc>,
        tx: &mut Transaction<'_, Sqlite>,
    ) -> AppResult<TimeBlockAction> {
        // 1. 检查是否仅链接此任务
        let is_exclusive = database::is_exclusive_link_in_tx(tx, block.id, task_id).await?;
        if !is_exclusive {
            // 多任务共享，不处理
            return Ok(TimeBlockAction::None);
        }

        // 2. 检查标题是否一致（自动创建的标志）
        let is_auto_created = block
            .title
            .as_ref()
            .map(|t| t == task_title)
            .unwrap_or(false);

        // 3. 判断时间状态
        if block.end_time < now {
            // 在过去：保留（无论是否自动创建）
            tracing::info!("Block {} in the past, keeping it", block.id);
            return Ok(TimeBlockAction::None);
        }

        if !is_auto_created {
            // 手动创建的：保留
            tracing::info!("Block {} is manually created, keeping it", block.id);
            return Ok(TimeBlockAction::None);
        }

        // 4. 自动创建的时间块：根据时间分类
        if block.start_time <= now && block.end_time > now {
            // 正在发生：需要截断
            tracing::info!("Block {} is ongoing, will truncate", block.id);
            return Ok(TimeBlockAction::Truncated);
        } else if block.start_time > now {
            // 在未来：需要删除
            tracing::info!("Block {} is in the future, will delete", block.id);
            return Ok(TimeBlockAction::Deleted);
        }

        Ok(TimeBlockAction::None)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::entities::{TaskRow, TimeBlockRow};

    pub async fn find_task_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<Option<crate::entities::Task>> {
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
                let task = crate::entities::Task::try_from(r).map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::QueryError(e))
                })?;
                Ok(Some(task))
            }
            None => Ok(None),
        }
    }

    pub async fn set_task_completed_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        completed_at: chrono::DateTime<Utc>,
    ) -> AppResult<()> {
        let query = "UPDATE tasks SET completed_at = ?, updated_at = ? WHERE id = ?";
        sqlx::query(query)
            .bind(completed_at.to_rfc3339())
            .bind(completed_at.to_rfc3339())
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(())
    }

    pub async fn update_today_schedule_to_completed_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        now: chrono::DateTime<Utc>,
    ) -> AppResult<()> {
        let today = now.date_naive();
        let query = r#"
            UPDATE task_schedules 
            SET outcome = 'COMPLETED_ON_DAY', updated_at = ?
            WHERE task_id = ? AND DATE(scheduled_day) = DATE(?)
        "#;

        sqlx::query(query)
            .bind(now.to_rfc3339())
            .bind(task_id.to_string())
            .bind(today.and_hms_opt(0, 0, 0).unwrap().and_utc().to_rfc3339())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(())
    }

    pub async fn delete_future_schedules_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        now: chrono::DateTime<Utc>,
    ) -> AppResult<()> {
        let today = now.date_naive();
        let query = r#"
            DELETE FROM task_schedules 
            WHERE task_id = ? AND DATE(scheduled_day) > DATE(?)
        "#;

        sqlx::query(query)
            .bind(task_id.to_string())
            .bind(today.and_hms_opt(0, 0, 0).unwrap().and_utc().to_rfc3339())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(())
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

    /// 截断时间块到 now
    pub async fn truncate_time_block_to_now_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
        now: chrono::DateTime<Utc>,
    ) -> AppResult<()> {
        let query = "UPDATE time_blocks SET end_time = ?, updated_at = ? WHERE id = ?";
        sqlx::query(query)
            .bind(now.to_rfc3339())
            .bind(now.to_rfc3339())
            .bind(block_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(())
    }

    /// 删除时间块（软删除）
    pub async fn delete_time_block_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
    ) -> AppResult<()> {
        let query = "UPDATE time_blocks SET is_deleted = true, updated_at = ? WHERE id = ?";
        sqlx::query(query)
            .bind(Utc::now().to_rfc3339())
            .bind(block_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        // 同时删除链接
        let delete_links = "DELETE FROM task_time_block_links WHERE time_block_id = ?";
        sqlx::query(delete_links)
            .bind(block_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
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
                    (String, String, Option<chrono::DateTime<Utc>>),
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
