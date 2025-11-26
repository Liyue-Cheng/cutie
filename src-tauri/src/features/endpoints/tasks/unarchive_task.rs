/// 取消归档任务 API - 单文件组件
///
/// 取消归档任务，使其重新在常规视图中显示
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use serde::Serialize;

use crate::{
    entities::{SideEffects, TaskTransactionResult},
    features::shared::{repositories::TaskRepository, TaskAssembler},
    infra::{
        core::{AppError, AppResult},
        http::{error_handler::success_response, extractors::extract_correlation_id},
    },
    startup::AppState,
};

/// 取消归档任务的响应
/// ✅ HTTP 响应和 SSE 事件使用相同的数据结构
#[derive(Debug, Serialize)]
pub struct UnarchiveTaskResponse {
    #[serde(flatten)]
    pub result: TaskTransactionResult,
}

// ==================== 文档层 ====================
/*
CABC for `unarchive_task`

## 1. 端点签名 (Endpoint Signature)

POST /api/tasks/{id}/unarchive

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我取消归档一个任务时，我希望系统能够：
> 1. 移除任务的归档状态
> 2. 该任务重新在常规视图中显示
> 3. 恢复所有任务数据（日程、时间块等）的可见性

### 2.2. 核心业务逻辑 (Core Business Logic)

取消归档任务，将 `archived_at` 设置为 NULL。取消归档的任务：
- 重新出现在常规看板视图中（staging、daily、calendar等）
- 所有关联数据（日程、时间块）保持不变
- 可以再次归档

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
  "task": {
    "id": "uuid",
    "title": "string",
    "is_archived": false,
    "archived_at": null,
    ...
  }
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Task not found: {id}"
}
```

**409 Conflict:**

```json
{
  "error_code": "CONFLICT",
  "message": "任务未归档"
}
```

## 4. 验证规则 (Validation Rules)

- `task_id`:
    - **必须**是有效的 UUID 格式。
    - **必须**存在于数据库中。
    - 违反时返回 `404 NOT_FOUND`
- **业务规则验证:**
    - 任务**必须**已归档（`archived_at IS NOT NULL`）。
    - 违反时返回 `409 CONFLICT`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1. 获取当前时间 `now`。
2. 获取写入许可（`app_state.acquire_write_permit()`）。
3. 启动数据库事务（`TransactionHelper::begin`）。
4. 查询任务（`TaskRepository::find_by_id_in_tx`）。
5. 如果任务不存在，返回 404 错误。
6. 检查任务是否未归档，如果是，返回 409 冲突。
7. 更新任务的 `archived_at` 字段为 NULL。
8. 重新查询任务并组装 `TaskCardDto`。
9. 填充 `schedules` 字段。
10. 根据 schedules 设置正确的 `schedule_status`。
11. 写入领域事件到 outbox。
12. 提交事务。
13. 返回取消归档后的任务。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **任务未归档:** 返回 `409` 冲突（幂等性保护）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 查询任务。
    - **`UPDATE`:** 1条记录在 `tasks` 表（设置 `archived_at = NULL`）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `task.unarchived` 事件，包含取消归档的任务（`TaskCardDto`）

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
    ) -> AppResult<UnarchiveTaskResponse> {
        let now = app_state.clock().now_utc();

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 开始事务（✅ 使用 TransactionHelper）
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 1. 查找任务（✅ 使用共享 Repository）
        let task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 2. 检查是否未归档
        if task.archived_at.is_none() {
            return Err(AppError::conflict("任务未归档"));
        }

        // 3. 更新任务的 archived_at 字段为 NULL
        database::set_unarchived_in_tx(&mut tx, task_id, now).await?;

        // 4. 重新查询任务并组装完整 TaskCard
        let updated_task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;
        let mut task_card = TaskAssembler::task_to_card_basic(&updated_task);

        // 5. ✅ 在事务内填充 schedules 字段
        // ⚠️ 必须在写入 SSE 之前填充，确保 SSE 和 HTTP 返回的数据一致！
        task_card.schedules = TaskAssembler::assemble_schedules_in_tx(&mut tx, task_id).await?;
        // schedule_status 已删除 - 前端根据 schedules 字段实时计算

        // 7. 构建统一的事务结果
        // ✅ HTTP 响应和 SSE 事件使用相同的数据结构
        let transaction_result = TaskTransactionResult {
            task: task_card,
            side_effects: SideEffects::empty(),
        };

        // 8. 在事务中写入领域事件到 outbox
        use crate::infra::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        {
            // ✅ 使用统一的事务结果作为事件载荷
            let payload = serde_json::to_value(&transaction_result)?;

            let mut event =
                DomainEvent::new("task.unarchived", "task", task_id.to_string(), payload)
                    .with_aggregate_version(now.timestamp_millis());

            if let Some(cid) = correlation_id {
                event = event.with_correlation_id(cid);
            }

            outbox_repo.append_in_tx(&mut tx, &event).await?;
        }

        // 9. 提交事务（✅ 使用 TransactionHelper）
        TransactionHelper::commit(tx).await?;

        // 10. 返回结果
        // ✅ HTTP 响应与 SSE 事件载荷完全一致
        Ok(UnarchiveTaskResponse {
            result: transaction_result,
        })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use chrono::{DateTime, Utc};
    use sqlx::{Sqlite, Transaction};

    /// 取消任务的归档状态
    pub async fn set_unarchived_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        updated_at: DateTime<Utc>,
    ) -> AppResult<()> {
        let query = r#"
            UPDATE tasks
            SET archived_at = NULL, updated_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(updated_at)
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(())
    }
}
