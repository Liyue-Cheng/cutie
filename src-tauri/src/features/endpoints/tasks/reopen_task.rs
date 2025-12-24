/// 重新打开任务 API - 单文件组件
///
/// 将已完成的任务重新打开，使其回到未完成状态
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

/// 重新打开任务的响应
/// ✅ HTTP 响应和 SSE 事件使用相同的数据结构
#[derive(Debug, Serialize)]
pub struct ReopenTaskResponse {
    #[serde(flatten)]
    pub result: TaskTransactionResult,
}

// ==================== 文档层 ====================
/*
CABC for `reopen_task`

## 1. 端点签名 (Endpoint Signature)

DELETE /api/tasks/{id}/completion

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我误标记了一个任务为已完成，或者需要重新开始一个已完成的任务时，
> 我希望能够将其重新打开，使其回到未完成状态。

### 2.2. 核心业务逻辑 (Core Business Logic)

将已完成的任务重新打开，设置 `completed_at = NULL`，使任务回到未完成状态。
这是 `complete_task` 的逆操作，但不会恢复已删除或截断的时间块（这些是完成任务时的副作用，不可逆）。

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
    "schedule_status": "staging" | "scheduled",
    "is_completed": false,
    "completed_at": null,
    "schedules": [...] | null,
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
  "message": "任务尚未完成"
}
```

## 4. 验证规则 (Validation Rules)

- `task_id`:
    - **必须**是有效的 UUID 格式。
    - **必须**存在于数据库中且未删除。
    - 违反时返回 `404 NOT_FOUND`
- **业务规则验证:**
    - 任务**必须**已完成（`completed_at IS NOT NULL`）。
    - 违反时返回 `409 CONFLICT`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  获取当前时间 `now`。
2.  获取写入许可（`app_state.acquire_write_permit()`）。
3.  启动数据库事务（`TransactionHelper::begin`）。
4.  查询任务（`TaskRepository::find_by_id_in_tx`）。
5.  如果任务不存在，返回 404 错误。
6.  检查任务是否已完成，如果未完成，返回 409 冲突。
7.  设置任务为未完成（`TaskRepository::set_reopened_in_tx`，设置 `completed_at = NULL`, `updated_at = now`）。
8.  提交事务（`TransactionHelper::commit`）。
9.  重新查询任务（`TaskRepository::find_by_id`）。
10. 组装 `TaskCardDto`（`TaskAssembler::task_to_card_basic`）。
11. 填充 `schedules` 字段（`TaskAssembler::assemble_schedules`）。
12. 根据 schedules 设置正确的 `schedule_status`：
    - 如果今天或未来有日程：`Scheduled`
    - 否则：`Staging`
13. 返回重新打开后的任务。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **任务已删除 (`is_deleted = true`):** 返回 `404` 错误（视为不存在）。
- **任务未完成:** 返回 `409` 冲突。
- **日程记录:** 不影响已有的日程记录（包括 outcome 状态），只改变任务的完成状态。
- **时间块:** 不恢复完成任务时已删除或截断的时间块（这些副作用不可逆）。
- **幂等性:** 对已未完成的任务调用会返回 409 错误（不具有幂等性）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次查询 `tasks` 表（事务内）。
    - **`UPDATE`:** 1条记录在 `tasks` 表（设置 `completed_at = NULL`, `updated_at = now`）。
    - **`SELECT`:** 1次查询 `tasks` 表（事务后，重新获取数据）。
    - **`SELECT`:** 1次查询 `task_schedules` 表（填充 schedules）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **无 SSE 事件:** 当前实现不发送 SSE 事件（可能需要补充 `task.reopened` 事件）。
- **日志记录:**
    - 成功时，记录性能指标（各阶段耗时）。
    - 失败时，记录详细错误信息。

*（无其他已知副作用）*
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    let handler_start = std::time::Instant::now();
    tracing::info!("[PERF] reopen_task HANDLER_START for task_id={}", task_id);

    let correlation_id = extract_correlation_id(&headers);

    let logic_start = std::time::Instant::now();
    let result = logic::execute(&app_state, task_id, correlation_id).await;
    tracing::info!(
        "[PERF] reopen_task LOGIC took {:.3}ms",
        logic_start.elapsed().as_secs_f64() * 1000.0
    );

    let response_start = std::time::Instant::now();
    let response = match result {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    };
    tracing::info!(
        "[PERF] reopen_task RESPONSE_BUILD took {:.3}ms",
        response_start.elapsed().as_secs_f64() * 1000.0
    );

    tracing::info!(
        "[PERF] reopen_task HANDLER_TOTAL took {:.3}ms",
        handler_start.elapsed().as_secs_f64() * 1000.0
    );

    response
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;
    use crate::features::shared::TransactionHelper;

    pub async fn execute(
        app_state: &AppState,
        task_id: Uuid,
        _correlation_id: Option<String>,
    ) -> AppResult<ReopenTaskResponse> {
        let start_time = std::time::Instant::now();
        tracing::info!("[PERF] reopen_task START for task_id={}", task_id);

        let now = app_state.clock().now_utc();

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // ⏱️ 1. 取连接（✅ 使用 TransactionHelper）
        let acquire_start = std::time::Instant::now();
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;
        tracing::info!(
            "[PERF] reopen_task ACQUIRE_CONNECTION took {:.3}ms",
            acquire_start.elapsed().as_secs_f64() * 1000.0
        );

        // ⏱️ 2. 查找任务（✅ 使用共享 Repository）
        let find_task_start = std::time::Instant::now();
        let task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;
        tracing::info!(
            "[PERF] reopen_task FIND_TASK took {:.3}ms",
            find_task_start.elapsed().as_secs_f64() * 1000.0
        );

        // ⏱️ 3. 检查是否未完成
        let check_start = std::time::Instant::now();
        if task.completed_at.is_none() {
            return Err(AppError::conflict("任务尚未完成"));
        }
        tracing::info!(
            "[PERF] reopen_task CHECK_STATUS took {:.3}ms",
            check_start.elapsed().as_secs_f64() * 1000.0
        );

        // ⏱️ 4. 重新打开任务（✅ 使用共享 Repository）
        let update_start = std::time::Instant::now();
        TaskRepository::set_reopened_in_tx(&mut tx, task_id, now).await?;
        tracing::info!(
            "[PERF] reopen_task UPDATE_TASK took {:.3}ms",
            update_start.elapsed().as_secs_f64() * 1000.0
        );

        // ⏱️ 5. 提交事务（✅ 使用 TransactionHelper）
        let commit_start = std::time::Instant::now();
        TransactionHelper::commit(tx).await?;
        tracing::info!(
            "[PERF] reopen_task COMMIT took {:.3}ms",
            commit_start.elapsed().as_secs_f64() * 1000.0
        );

        // ⏱️ 6. 重新查询并组装返回数据（✅ 使用共享 Repository）
        let refetch_start = std::time::Instant::now();
        let updated_task = TaskRepository::find_by_id(app_state.db_pool(), task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;
        tracing::info!(
            "[PERF] reopen_task REFETCH_TASK took {:.3}ms",
            refetch_start.elapsed().as_secs_f64() * 1000.0
        );

        // ⏱️ 7. 组装响应（✅ area_id 已由 TaskAssembler 填充）
        let assemble_start = std::time::Instant::now();
        let mut task_card = TaskAssembler::task_to_card_basic(&updated_task);

        // ✅ 填充 schedules 字段（事务已提交，使用 pool 查询）
        task_card.schedules =
            TaskAssembler::assemble_schedules(app_state.db_pool(), task_id).await?;
        // schedule_status 已删除 - 前端根据 schedules 字段实时计算

        // 填充 recurrence_expiry_behavior
        TaskAssembler::fill_recurrence_expiry_behavior(&mut task_card, app_state.db_pool()).await?;

        tracing::info!(
            "[PERF] reopen_task ASSEMBLE_RESPONSE took {:.3}ms",
            assemble_start.elapsed().as_secs_f64() * 1000.0
        );

        tracing::info!(
            "[PERF] reopen_task TOTAL took {:.3}ms",
            start_time.elapsed().as_secs_f64() * 1000.0
        );

        Ok(ReopenTaskResponse {
            result: TaskTransactionResult {
                task: task_card,
                side_effects: SideEffects::empty(),
            },
        })
    }
}

// ==================== 数据访问层 ====================
// ✅ 已全部迁移到共享 Repository：
// - TaskRepository::find_by_id_in_tx, find_by_id, set_reopened_in_tx
// - TaskScheduleRepository::has_any_schedule
