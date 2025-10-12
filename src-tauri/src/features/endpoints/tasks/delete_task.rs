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
    features::shared::{
        assemblers::TimeBlockAssembler,
        repositories::{TaskRepository, TaskScheduleRepository, TaskTimeBlockLinkRepository},
        TaskAssembler,
    },
    infra::{
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

## 1. 端点签名 (Endpoint Signature)

DELETE /api/tasks/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我删除一个任务时，我希望系统能够：
> 1. 软删除任务（标记为已删除，而非物理删除）
> 2. 清理所有相关的日程和链接记录
> 3. 智能清理"孤儿"时间块（只关联这一个任务且是自动创建的）

### 2.2. 核心业务逻辑 (Core Business Logic)

软删除任务（设置 `deleted_at = now`），并清理相关数据：
1. 删除所有 `task_time_block_links` 记录
2. 删除所有 `task_schedules` 记录
3. 检查时间块是否变成"孤儿"，如果是且为自动创建的，则删除该时间块

**孤儿时间块定义：**
- 该时间块只链接了这一个任务
- 删除这个任务后，时间块没有任何关联任务
- 时间块的 `source_type == "native::from_task"`（自动创建的标志）

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 任务ID

**请求头 (Request Headers):**
- `X-Correlation-ID` (optional): 用于前端去重和请求追踪

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`

```json
{
  "success": true
}
```

**注意：** 副作用（删除的时间块）通过 SSE 事件推送。

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Task not found: {id}"
}
```

## 4. 验证规则 (Validation Rules)

- `task_id`:
    - **必须**是有效的 UUID 格式。
    - **必须**存在于数据库中。
    - 违反时返回 `404 NOT_FOUND`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  获取写入许可（`app_state.acquire_write_permit()`）。
2.  启动数据库事务（`TransactionHelper::begin`）。
3.  查询任务的完整数据（`TaskRepository::find_by_id_in_tx`）。
4.  如果任务不存在，返回 404 错误。
5.  组装基础 `TaskCardDto`（用于事件载荷）。
6.  查询任务链接的所有时间块（`TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx`）。
7.  软删除任务（`TaskRepository::soft_delete_in_tx`，设置 `deleted_at = now`）。
8.  删除任务的所有链接记录（`TaskTimeBlockLinkRepository::delete_all_for_task_in_tx`）。
9.  删除任务的所有日程记录（`TaskScheduleRepository::delete_all_in_tx`）。
10. 对每个链接的时间块，调用 `should_delete_orphan_block` 判断是否应该删除：
    - 检查时间块是否还有其他任务（`count_remaining_tasks_in_block_in_tx`）
    - 检查时间块是否是自动创建的（标题与任务标题一致）
11. 在执行删除之前，先查询被删除的时间块的完整数据（用于 SSE 事件）。
12. 删除孤儿时间块（`TimeBlockRepository::soft_delete_in_tx`）。
13. 写入领域事件到 outbox（包含删除的任务和副作用的时间块）。
14. 提交事务（`TransactionHelper::commit`）。
15. 返回成功响应。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **任务已删除:** 幂等，返回 `404` 错误（因为 find_by_id_in_tx 不返回已删除的任务）。
- **时间块还有其他任务:** 不删除时间块（避免影响其他任务）。
- **时间块是手动创建的（标题与任务不一致）:** 不删除时间块（保留用户的手动数据）。
- **时间块是孤儿且自动创建:** 删除时间块。
- **无关联时间块和日程的任务:** 只软删除任务本身。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 查询任务、链接的时间块、剩余任务数量。
    - **`UPDATE`:** 1条记录在 `tasks` 表（设置 `deleted_at = now`）。
    - **`DELETE`:** 0-N 条记录在 `task_time_block_links` 表。
    - **`DELETE`:** 0-N 条记录在 `task_schedules` 表。
    - **`UPDATE`:** 0-N 条记录在 `time_blocks` 表（软删除孤儿时间块）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `task.trashed` 事件，包含：
        - 删除的任务（`TaskCardDto`）
        - 删除时间（`deleted_at`）
        - 副作用：删除的时间块列表（`TimeBlockViewDto[]`）
- **日志记录:**
    - 记录被删除的孤儿时间块 ID。
    - 失败时，记录详细错误信息。

*（无其他已知副作用）*
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
        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 开始事务（✅ 使用 TransactionHelper）
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 1. 查询任务的完整数据（✅ 使用共享 Repository）
        let task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 3. 找到该任务链接的所有时间块（✅ 使用共享 Repository）
        let linked_blocks =
            TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx(&mut tx, task_id).await?;

        let now = app_state.clock().now_utc();

        // 4. 删除任务（软删除）（✅ 使用共享 Repository）
        TaskRepository::soft_delete_in_tx(&mut tx, task_id, now).await?;

        // 4.5 组装 task_card（在软删除之后，包含 deleted_at）
        let mut task_card = TaskAssembler::task_to_card_basic(&task);
        task_card.deleted_at = Some(now); // 手动设置 deleted_at
        task_card.is_deleted = true;

        // 5. 删除任务的所有链接和日程（✅ 使用共享 Repository）
        TaskTimeBlockLinkRepository::delete_all_for_task_in_tx(&mut tx, task_id).await?;
        TaskScheduleRepository::delete_all_in_tx(&mut tx, task_id).await?;

        // 6. 检查并标记需要删除的孤儿时间块，但先不删除（需要先查询完整数据）
        let mut blocks_to_delete = Vec::new();
        for block in linked_blocks {
            let should_delete = should_delete_orphan_block(&block, &mut tx).await?;
            if should_delete {
                tracing::info!(
                    "Will delete orphan time block {} (source_type={:?}) after deleting task {}",
                    block.id,
                    block.source_info.as_ref().map(|s| &s.source_type),
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
            use crate::features::shared::repositories::TimeBlockRepository;
            TimeBlockRepository::soft_delete_in_tx(&mut tx, block.id).await?;
        }

        // 9. 在事务中写入领域事件到 outbox
        // ✅ 一个业务事务 = 一个领域事件（包含所有副作用的完整数据）
        use crate::infra::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        {
            let payload = serde_json::json!({
                "task": task_card,  // ✅ 完整 TaskCard
                "deleted_at": now.to_rfc3339(),
                "side_effects": {
                    "deleted_time_blocks": deleted_blocks,  // ✅ 完整对象
                }
            });
            let mut event = DomainEvent::new("task.trashed", "task", task_id.to_string(), payload)
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
    ///
    /// 删除规则：
    /// 1. 时间块没有其他任务链接（孤儿）
    /// 2. 时间块的 source_type == "native::from_task"（从任务拖拽创建）
    ///
    /// 保留规则：
    /// - native::manual：手动创建的时间块
    /// - external::*：外部导入的时间块
    /// - 无 source_info：旧数据（向后兼容，默认保留）
    async fn should_delete_orphan_block(
        block: &TimeBlock,
        tx: &mut Transaction<'_, Sqlite>,
    ) -> AppResult<bool> {
        // 1. 检查时间块是否还有其他任务（✅ 使用共享 Repository）
        let remaining_tasks =
            TaskTimeBlockLinkRepository::count_remaining_tasks_in_block_in_tx(tx, block.id).await?;
        if remaining_tasks > 0 {
            return Ok(false); // 还有其他任务，不删除
        }

        // 2. 基于 source_info 判断是否应删除
        if let Some(source_info) = &block.source_info {
            if source_info.source_type == "native::from_task" {
                return Ok(true); // 孤儿 + 从任务创建 = 删除
            }
        }

        // 3. 默认保留（手动创建、外部导入、或无来源信息的旧数据）
        Ok(false)
    }
}

// ==================== 数据访问层 ====================
// ✅ 已全部迁移到共享 Repository：
// - TaskRepository::find_by_id_in_tx, soft_delete_in_tx(task_id, deleted_at)
// - TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx, delete_all_for_task_in_tx, count_remaining_tasks_in_block_in_tx
// - TaskScheduleRepository::delete_all_in_tx
// - TimeBlockRepository::soft_delete_in_tx
// - TimeBlockAssembler::assemble_for_event_in_tx
