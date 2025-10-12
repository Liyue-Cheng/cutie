/// 恢复任务 API - 单文件组件
///
/// 从回收站恢复已删除的任务
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    entities::task::response_dtos::TaskCardDto,
    features::shared::repositories::TaskRepository,
    features::shared::ViewTaskCardAssembler,
    infra::{
        core::{AppError, AppResult},
        http::{error_handler::success_response, extractors::extract_correlation_id},
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `restore_task`

## 1. 端点签名 (Endpoint Signature)

PATCH /api/tasks/{id}/restore

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我从回收站恢复一个任务时，我希望系统能够：
> 1. 将任务从回收站移出（设置 deleted_at = NULL）
> 2. 恢复任务的所有关联数据（schedules、time_blocks 链接等）
> 3. 任务重新出现在原来的视图中

### 2.2. 核心业务逻辑 (Core Business Logic)

恢复任务（设置 `deleted_at = NULL`）：
1. 查询回收站中的任务
2. 验证任务确实在回收站中
3. 设置 deleted_at = NULL
4. 发送 SSE 事件通知前端

**注意：** 任务的关联数据（schedules、time_blocks）在软删除时并未物理删除，
因此恢复后这些关联会自动生效。

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
  "id": "uuid",
  "title": "string",
  "glance_note": "string | null",
  ...
}
```

返回完整的 TaskCardDto

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
  "message": "Task is not in trash"
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
5.  恢复任务（`TaskRepository::restore_in_tx`，设置 `deleted_at = NULL`）。
6.  提交事务。
7.  装配完整的 TaskCardDto（包含 schedules、area 等信息）。
8.  写入领域事件到 outbox（`task.restored`）。
9.  返回完整的任务数据。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **任务不在回收站中（deleted_at IS NULL）:** 返回 `409` 错误。
- **任务已恢复:** 幂等，返回 `409` 错误。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 查询回收站中的任务。
    - **`UPDATE`:** 1条记录在 `tasks` 表（设置 `deleted_at = NULL`）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `task.restored` 事件，包含：
        - 恢复的任务（完整 TaskCardDto）

## 8. 契约 (Contract)

### Pre-conditions:
- 任务存在于回收站中（deleted_at IS NOT NULL）

### Post-conditions:
- 任务的 deleted_at = NULL
- 任务重新出现在正常视图中
- 返回完整 TaskCardDto

### Invariants:
- id 和 created_at 永远不变
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    let correlation_id = extract_correlation_id(&headers);
    match logic::execute(&app_state, task_id, correlation_id).await {
        Ok(task_card) => success_response(task_card).into_response(),
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
    ) -> AppResult<TaskCardDto> {
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
            return Err(AppError::conflict("Task is not in trash"));
        }

        let now = app_state.clock().now_utc();

        // 3. 恢复任务（✅ 使用共享 Repository）
        TaskRepository::restore_in_tx(&mut tx, task_id, now).await?;

        // 4. 提交事务
        TransactionHelper::commit(tx).await?;

        // 5. 装配完整的 TaskCardDto（✅ 使用共享装配器）
        let task_card = ViewTaskCardAssembler::assemble_full(&task, app_state.db_pool()).await?;

        // 6. 写入领域事件到 outbox
        use crate::infra::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        let mut outbox_tx = TransactionHelper::begin(app_state.db_pool()).await?;
        {
            let payload = serde_json::json!({
                "task": task_card,  // ✅ 完整 TaskCard
            });
            let mut event = DomainEvent::new("task.restored", "task", task_id.to_string(), payload)
                .with_aggregate_version(now.timestamp_millis());

            // 关联 correlation_id（用于前端去重和请求追踪）
            if let Some(cid) = correlation_id {
                event = event.with_correlation_id(cid);
            }

            outbox_repo.append_in_tx(&mut outbox_tx, &event).await?;
        }
        TransactionHelper::commit(outbox_tx).await?;

        Ok(task_card)
    }
}

// ==================== 数据访问层 ====================
// ✅ 已全部迁移到共享 Repository：
// - TaskRepository::find_deleted_by_id_in_tx, restore_in_tx
// - ViewTaskCardAssembler::assemble_full
