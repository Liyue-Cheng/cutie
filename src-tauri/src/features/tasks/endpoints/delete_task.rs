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
    features::tasks::shared::{
        assemblers::TimeBlockAssembler,
        repositories::{TaskRepository, TaskScheduleRepository, TaskTimeBlockLinkRepository},
        TaskAssembler,
    },
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
    use crate::features::shared::TransactionHelper;

    pub async fn execute(
        app_state: &AppState,
        task_id: Uuid,
        correlation_id: Option<String>,
    ) -> AppResult<DeleteTaskResponse> {
        // 开始事务（✅ 使用 TransactionHelper）
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 1. 查询任务的完整数据（✅ 使用共享 Repository）
        let task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        let task_card = TaskAssembler::task_to_card_basic(&task);
        let task_title = task.title.clone();

        // 3. 找到该任务链接的所有时间块（✅ 使用共享 Repository）
        let linked_blocks =
            TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx(&mut tx, task_id).await?;

        // 4. 删除任务（软删除）（✅ 使用共享 Repository）
        TaskRepository::soft_delete_in_tx(&mut tx, task_id).await?;

        // 5. 删除任务的所有链接和日程（✅ 使用共享 Repository）
        TaskTimeBlockLinkRepository::delete_all_for_task_in_tx(&mut tx, task_id).await?;
        TaskScheduleRepository::delete_all_in_tx(&mut tx, task_id).await?;

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

        // 7. 查询被删除的时间块的完整数据（✅ 使用共享装配器）
        let deleted_time_block_ids: Vec<uuid::Uuid> =
            blocks_to_delete.iter().map(|b| b.id).collect();
        let deleted_blocks =
            TimeBlockAssembler::assemble_for_event_in_tx(&mut tx, &deleted_time_block_ids).await?;

        // 8. 现在才真正删除时间块（✅ 使用共享 Repository）
        for block in blocks_to_delete {
            use crate::features::time_blocks::shared::repositories::TimeBlockRepository;
            TimeBlockRepository::soft_delete_in_tx(&mut tx, block.id).await?;
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

        // 10. 提交事务（✅ 使用 TransactionHelper）
        TransactionHelper::commit(tx).await?;

        // HTTP 响应不再包含副作用列表，副作用通过 SSE 推送
        Ok(DeleteTaskResponse { success: true })
    }

    /// 判断是否应该删除孤儿时间块
    async fn should_delete_orphan_block(
        block: &TimeBlock,
        deleted_task_title: &str,
        tx: &mut Transaction<'_, Sqlite>,
    ) -> AppResult<bool> {
        // 1. 检查时间块是否还有其他任务（✅ 使用共享 Repository）
        let remaining_tasks =
            TaskTimeBlockLinkRepository::count_remaining_tasks_in_block_in_tx(tx, block.id).await?;
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
// ✅ 已全部迁移到共享 Repository：
// - TaskRepository::find_by_id_in_tx, soft_delete_in_tx
// - TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx, delete_all_for_task_in_tx, count_remaining_tasks_in_block_in_tx
// - TaskScheduleRepository::delete_all_in_tx
// - TimeBlockRepository::soft_delete_in_tx
// - TimeBlockAssembler::assemble_for_event_in_tx
