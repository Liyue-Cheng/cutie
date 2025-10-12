/// 彻底删除任务 API - 单文件组件
///
/// 从回收站彻底删除任务（物理删除）
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::TimeBlock,
    features::shared::repositories::{
        TaskRepository, TaskScheduleRepository, TaskTimeBlockLinkRepository,
    },
    infra::{
        core::{AppError, AppResult},
        http::{error_handler::success_response, extractors::extract_correlation_id},
    },
    startup::AppState,
};

/// 彻底删除任务的响应
#[derive(Debug, Serialize)]
pub struct PermanentlyDeleteTaskResponse {
    pub success: bool,
}

// ==================== 文档层 ====================
/*
CABC for `permanently_delete_task`

## 1. 端点签名 (Endpoint Signature)

DELETE /api/tasks/{id}/permanently

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我从回收站彻底删除一个任务时，我希望系统能够：
> 1. 物理删除任务记录（不可恢复）
> 2. 清理所有关联数据（schedules、links）
> 3. 智能处理关联的时间块（孤儿时间块也删除）

### 2.2. 核心业务逻辑 (Core Business Logic)

物理删除任务（DELETE FROM tasks）：
1. 验证任务在回收站中（deleted_at IS NOT NULL）
2. 查询关联的时间块
3. 删除任务的所有链接和日程
4. 物理删除任务记录
5. 检查并删除孤儿时间块

**安全约束：** 只能删除已在回收站中的任务（deleted_at IS NOT NULL）

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

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Task not found in trash: {id}"
}
```

**409 Conflict:**

```json
{
  "error_code": "CONFLICT",
  "message": "Task is not in trash, cannot permanently delete"
}
```

## 4. 验证规则 (Validation Rules)

- `task_id`:
    - **必须**是有效的 UUID 格式。
    - **必须**存在于回收站中（deleted_at IS NOT NULL）。
    - 违反时返回 `404 NOT_FOUND` 或 `409 CONFLICT`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  获取写入许可（`app_state.acquire_write_permit()`）。
2.  启动数据库事务（`TransactionHelper::begin`）。
3.  查询回收站中的任务（`TaskRepository::find_deleted_by_id_in_tx`）。
4.  如果任务不存在或不在回收站中，返回错误。
5.  查询任务链接的所有时间块（用于后续孤儿检查）。
6.  删除任务的所有链接记录（`TaskTimeBlockLinkRepository::delete_all_for_task_in_tx`）。
7.  删除任务的所有日程记录（`TaskScheduleRepository::delete_all_in_tx`）。
8.  物理删除任务（`TaskRepository::permanently_delete_in_tx`）。
9.  检查并删除孤儿时间块。
10. 写入领域事件到 outbox（`task.permanently_deleted`）。
11. 提交事务。
12. 返回成功响应。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **任务不在回收站中（deleted_at IS NULL）:** 返回 `409` 错误，拒绝删除。
- **时间块还有其他任务:** 不删除时间块（避免影响其他任务）。
- **时间块是孤儿且自动创建:** 删除时间块。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 查询回收站中的任务、链接的时间块。
    - **`DELETE`:** 1条记录从 `tasks` 表（物理删除）。
    - **`DELETE`:** 0-N 条记录从 `task_time_block_links` 表（级联删除）。
    - **`DELETE`:** 0-N 条记录从 `task_schedules` 表（级联删除）。
    - **`UPDATE`:** 0-N 条记录在 `time_blocks` 表（软删除孤儿时间块）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `task.permanently_deleted` 事件，包含：
        - 任务ID

## 8. 契约 (Contract)

### Pre-conditions:
- 任务存在于回收站中（deleted_at IS NOT NULL）

### Post-conditions:
- 任务记录被物理删除
- 所有关联数据被清理
- 孤儿时间块被删除

### Invariants:
- 不能删除不在回收站中的任务
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
    ) -> AppResult<PermanentlyDeleteTaskResponse> {
        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 开始事务（✅ 使用 TransactionHelper）
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 1. 查询回收站中的任务（✅ 使用共享 Repository）
        let task = TaskRepository::find_deleted_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task in trash", task_id.to_string()))?;

        // 2. 验证任务确实在回收站中
        if task.deleted_at.is_none() {
            return Err(AppError::conflict(
                "Task is not in trash, cannot permanently delete",
            ));
        }

        // 3. 查询任务链接的所有时间块（用于后续孤儿检查）
        let linked_blocks =
            TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx(&mut tx, task_id).await?;

        // 4. 删除任务的所有链接和日程
        TaskTimeBlockLinkRepository::delete_all_for_task_in_tx(&mut tx, task_id).await?;
        TaskScheduleRepository::delete_all_in_tx(&mut tx, task_id).await?;

        // 5. 物理删除任务
        TaskRepository::permanently_delete_in_tx(&mut tx, task_id).await?;

        // 6. 检查并删除孤儿时间块
        for block in linked_blocks {
            let should_delete = should_delete_orphan_block(&block, &mut tx).await?;
            if should_delete {
                tracing::info!(
                    "Will delete orphan time block {} after permanently deleting task {}",
                    block.id,
                    task_id
                );
                use crate::features::shared::repositories::TimeBlockRepository;
                TimeBlockRepository::soft_delete_in_tx(&mut tx, block.id).await?;
            }
        }

        let now = app_state.clock().now_utc();

        // 7. 写入领域事件到 outbox
        use crate::infra::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        {
            let payload = serde_json::json!({
                "task_id": task_id.to_string(),
            });
            let mut event = DomainEvent::new(
                "task.permanently_deleted",
                "task",
                task_id.to_string(),
                payload,
            )
            .with_aggregate_version(now.timestamp_millis());

            // 关联 correlation_id（用于前端去重和请求追踪）
            if let Some(cid) = correlation_id {
                event = event.with_correlation_id(cid);
            }

            outbox_repo.append_in_tx(&mut tx, &event).await?;
        }

        // 8. 提交事务
        TransactionHelper::commit(tx).await?;

        Ok(PermanentlyDeleteTaskResponse { success: true })
    }

    /// 判断是否应该删除孤儿时间块
    ///
    /// 删除规则：
    /// 1. 时间块没有其他任务链接（孤儿）
    /// 2. 时间块的 source_type == "native::from_task"（从任务拖拽创建）
    async fn should_delete_orphan_block(
        block: &TimeBlock,
        tx: &mut Transaction<'_, Sqlite>,
    ) -> AppResult<bool> {
        // 1. 检查时间块是否还有其他任务
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
// - TaskRepository::find_deleted_by_id_in_tx, permanently_delete_in_tx
// - TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx, delete_all_for_task_in_tx, count_remaining_tasks_in_block_in_tx
// - TaskScheduleRepository::delete_all_in_tx
// - TimeBlockRepository::soft_delete_in_tx
