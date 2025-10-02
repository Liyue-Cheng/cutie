/// 删除任务 API - 单文件组件
///
/// 软删除任务，并根据业务规则清理孤儿时间块
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use serde::Serialize;

use crate::{
    entities::TimeBlock,
    features::tasks::shared::TaskAssembler,
    shared::{
        core::{AppError, AppResult},
        http::{error_handler::success_response, extractors::extract_correlation_id},
    },
    startup::AppState,
};

/// 删除任务的响应
#[derive(Debug, Serialize)]
pub struct DeleteTaskResponse {
    pub success: bool,
    // 注意：deleted_time_block_ids 已通过 SSE 推送
}

// ==================== 文档层 ====================
/*
CABC for `delete_task`

## API端点
DELETE /api/tasks/{id}

## 预期行为简介
软删除任务（设置 is_deleted = true）。
根据 Cutie 的业务规则，如果任务链接的时间块变成"孤儿"，也会删除该时间块。

## 输入输出规范
- **前置条件**: task_id 必须存在
- **后置条件**:
  - 任务的 is_deleted = true
  - 删除所有 task_time_block_links 记录
  - 删除所有 task_schedules 记录
  - 如果时间块变成孤儿且是自动创建的，删除该时间块

## 边界情况
- 如果任务不存在，返回 404
- 如果任务已删除，返回 204（幂等）

## 孤儿时间块定义
- 该时间块只链接了这一个任务
- 删除这个任务后，时间块没有任何关联任务
- 时间块的 title 与任务 title 相同（自动创建的标志）

## 预期副作用
- 更新 tasks 表（is_deleted = true）
- 删除 task_time_block_links 记录
- 删除 task_schedules 记录
- 可能删除孤儿时间块
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    let correlation_id = extract_correlation_id(&headers);
    match logic::execute(&app_state, task_id, correlation_id).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        task_id: Uuid,
        correlation_id: Option<String>,
    ) -> AppResult<DeleteTaskResponse> {
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 1. 查询任务的完整数据（在删除之前，用于事件载荷 ✅ 禁止片面数据）
        let task = database::find_task_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        let task_card = TaskAssembler::task_to_card_basic(&task);
        let task_title = task.title.clone();

        // 3. 找到该任务链接的所有时间块
        let linked_blocks = database::find_linked_time_blocks_in_tx(&mut tx, task_id).await?;

        // 4. 删除任务（软删除）
        database::soft_delete_task_in_tx(&mut tx, task_id).await?;

        // 5. 删除任务的所有链接和日程
        database::delete_task_links_in_tx(&mut tx, task_id).await?;
        database::delete_task_schedules_in_tx(&mut tx, task_id).await?;

        // 6. 检查并标记需要删除的孤儿时间块，但先不删除（需要先查询完整数据）
        let mut blocks_to_delete = Vec::new();
        for block in linked_blocks {
            let should_delete = should_delete_orphan_block(&block, &task_title, &mut tx).await?;
            if should_delete {
                tracing::info!(
                    "Will delete orphan time block {} after deleting task {}",
                    block.id,
                    task_id
                );
                blocks_to_delete.push(block);
            }
        }

        // 7. 查询被删除的时间块的完整数据（✅ 在删除之前查询）
        let deleted_time_block_ids: Vec<uuid::Uuid> =
            blocks_to_delete.iter().map(|b| b.id).collect();
        let deleted_blocks = if !deleted_time_block_ids.is_empty() {
            database::find_time_blocks_for_event(&mut tx, &deleted_time_block_ids).await?
        } else {
            Vec::new()
        };

        // 8. 现在才真正删除时间块
        for block in blocks_to_delete {
            database::soft_delete_time_block_in_tx(&mut tx, block.id).await?;
        }

        // 9. 在事务中写入领域事件到 outbox
        // ✅ 一个业务事务 = 一个领域事件（包含所有副作用的完整数据）
        use crate::shared::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());
        let now = app_state.clock().now_utc();

        {
            let payload = serde_json::json!({
                "task": task_card,  // ✅ 完整 TaskCard
                "deleted_at": now.to_rfc3339(),
                "side_effects": {
                    "deleted_time_blocks": deleted_blocks,  // ✅ 完整对象
                }
            });
            let mut event = DomainEvent::new("task.deleted", "task", task_id.to_string(), payload)
                .with_aggregate_version(now.timestamp_millis());
            
            // 关联 correlation_id（用于前端去重和请求追踪）
            if let Some(cid) = correlation_id {
                event = event.with_correlation_id(cid);
            }
            
            outbox_repo.append_in_tx(&mut tx, &event).await?;
        }

        // 10. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // HTTP 响应不再包含副作用列表，副作用通过 SSE 推送
        Ok(DeleteTaskResponse { success: true })
    }

    /// 判断是否应该删除孤儿时间块
    async fn should_delete_orphan_block(
        block: &TimeBlock,
        deleted_task_title: &str,
        tx: &mut Transaction<'_, Sqlite>,
    ) -> AppResult<bool> {
        // 1. 检查时间块是否还有其他任务
        let remaining_tasks = database::count_remaining_tasks_in_block_in_tx(tx, block.id).await?;
        if remaining_tasks > 0 {
            return Ok(false); // 还有其他任务，不删除
        }

        // 2. 判断是否自动创建的（title 与任务相同）
        if let Some(block_title) = &block.title {
            if block_title == deleted_task_title {
                return Ok(true); // 孤儿 + 自动创建 = 删除
            }
        }

        // 3. 用户手动创建的空时间块，保留
        Ok(false)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::entities::{TaskRow, TimeBlockRow};

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

    pub async fn soft_delete_task_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<()> {
        let query = "UPDATE tasks SET is_deleted = true, updated_at = ? WHERE id = ?";
        sqlx::query(query)
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(())
    }

    pub async fn delete_task_links_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<()> {
        let query = "DELETE FROM task_time_block_links WHERE task_id = ?";
        sqlx::query(query)
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(())
    }

    pub async fn delete_task_schedules_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
    ) -> AppResult<()> {
        let query = "DELETE FROM task_schedules WHERE task_id = ?";
        sqlx::query(query)
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(())
    }

    pub async fn count_remaining_tasks_in_block_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
    ) -> AppResult<i64> {
        let query = "SELECT COUNT(*) FROM task_time_block_links WHERE time_block_id = ?";
        sqlx::query_scalar(query)
            .bind(block_id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))
    }

    pub async fn soft_delete_time_block_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
    ) -> AppResult<()> {
        let query = "UPDATE time_blocks SET is_deleted = true, updated_at = ? WHERE id = ?";
        sqlx::query(query)
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(block_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;
        Ok(())
    }

    /// 查询任务的完整数据（用于事件载荷）
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

    /// 查询时间块的完整数据用于事件载荷（复用 complete_task.rs 的实现）
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
