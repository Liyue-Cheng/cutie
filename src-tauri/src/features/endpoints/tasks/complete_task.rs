/// 完成任务 API - 单文件组件
///
/// 按照 Cutie 的精确业务逻辑实现
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use serde::Serialize;

use crate::{
    entities::{task::request_dtos::CompleteTaskRequest, SideEffects, TaskTransactionResult, TimeBlock},
    features::shared::repositories::TimeBlockRepository,
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

/// 完成任务的响应
/// ✅ HTTP 响应和 SSE 事件使用相同的数据结构
#[derive(Debug, Serialize)]
pub struct CompleteTaskResponse {
    #[serde(flatten)]
    pub result: TaskTransactionResult,
}

// ==================== 文档层 ====================
/*
CABC for `complete_task`

## 1. 端点签名 (Endpoint Signature)

POST /api/tasks/{id}/completion

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我完成一个任务时，我希望系统能够：
> 1. 使用我设备的当前时间作为完成时间（避免时区问题）
> 2. 根据我所在的视图上下文，智能决定日程记录位置：
>    - 在过去日期视图完成 → 补记录到那天（修正历史）
>    - 在今天/未来日期视图完成 → 记录在今天（正常完成/提前完成）
>    - 在通用视图完成 → 记录在今天
> 3. 清理未来的日程和时间块（因为任务已完成）
> 4. 智能处理正在进行的时间块（截断到当前时间）

### 2.2. 核心业务逻辑 (Core Business Logic)

完成任务，并根据**视图上下文**和**客户端时间**智能处理相关的日程和时间块：

**日程处理逻辑：**
- 解析视图上下文 (`view_context`)，确定日程创建位置 (`schedule_date`)
- 规则：
  - 过去日期视图 (`daily::2025-10-01` < today) → 记录在那天（补记录历史）
  - 今天/未来日期视图 (`daily::2025-10-05` >= today) → 记录在今天（正常/提前完成）
  - 通用视图 (`misc::*`, `area::*`, `project::*`) → 记录在今天
- 如果 `schedule_date` 已有日程：设置为已完成（`outcome = 'COMPLETED_ON_DAY'`）
- 如果 `schedule_date` 没有日程：创建一条新日程并设置为已完成
- 删除 `> schedule_date` 的所有日程

**时间块处理**（仅针对唯一关联且自动创建的时间块）:
- 使用客户端时间 (`completed_at_client`) 判断时间状态
- 在过去 (`end_time < completed_at_client`)：保留
- 正在进行 (`start_time <= completed_at_client < end_time`)：截断到 `completed_at_client`
- 在未来 (`start_time > completed_at_client`)：删除

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 任务ID

**Request Body:**
```json
{
  "completed_at_client": "2025-10-05T14:30:00+08:00",
  "view_context": "daily::2025-10-01"
}
```

**Body Schema:**
- `completed_at_client` (DateTime<Utc>, required): 客户端时间（用户实际完成的时刻）
- `view_context` (String, required): 视图上下文，格式 `{type}::{identifier}`
  - 例如：`"daily::2025-10-01"`, `"misc::staging"`, `"area::{uuid}"`

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
    "is_completed": true,
    "completed_at": "2025-10-05T14:30:00+08:00",
    "schedules": [
      {
        "scheduled_day": "2025-10-01",
        "outcome": "completed_on_day"
      }
    ],
    ...
  },
  "side_effects": {
    "deleted_time_blocks": [...],
    "truncated_time_blocks": [...]
  }
}
```

**注意：** 副作用（删除/截断的时间块）通过 SSE 事件推送。

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
  "message": "任务已经完成"
}
```

**400 Bad Request:**

```json
{
  "error_code": "VALIDATION_ERROR",
  "field": "view_context",
  "message": "Invalid view context format"
}
```

## 4. 验证规则 (Validation Rules)

- `task_id`:
    - **必须**是有效的 UUID 格式。
    - **必须**存在于数据库中。
    - 违反时返回 `404 NOT_FOUND`
- `completed_at_client`:
    - **必须**是有效的 ISO 8601 日期时间格式。
    - **必须**包含时区信息。
- `view_context`:
    - **必须**是有效的上下文格式 `{type}::{identifier}`。
    - `type` 必须是 `daily`, `misc`, `area`, 或 `project`。
    - 对于 `daily` 类型，必须包含日期 `YYYY-MM-DD`。
    - 违反时返回 `400 VALIDATION_ERROR`
- **业务规则验证:**
    - 任务**不能**已经完成（`completed_at IS NOT NULL`）。
    - 违反时返回 `409 CONFLICT`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  获取写入许可（`app_state.acquire_write_permit()`）。
2.  启动数据库事务（`TransactionHelper::begin`）。
3.  查询任务（`TaskRepository::find_by_id_in_tx`）。
4.  如果任务不存在，返回 404 错误。
5.  检查任务是否已完成，如果是，返回 409 冲突。
6.  **解析视图上下文，确定日程创建位置**（`determine_schedule_date`）：
    - 获取今天的日期（使用本地时间）
    - 如果是 `daily` 类型：
      - 视图日期 < 今天 → 返回视图日期（补记录历史）
      - 视图日期 >= 今天 → 返回今天（正常/提前完成）
    - 如果是其他类型 → 返回今天
7.  设置任务为已完成（`TaskRepository::set_completed_in_tx`），使用 `completed_at_client`。
8.  更新子任务：将所有子任务标记为已完成。
9.  **处理日程**：
    - 检查 `schedule_date` 是否有日程（`has_schedule_for_day_in_tx`）
    - 如果有：更新为已完成（`update_day_to_completed_in_tx`）
    - 如果没有：创建日程 + 更新为已完成
    - 删除 `> schedule_date` 的所有日程（`delete_schedules_after_in_tx`）
10. 查询所有链接的时间块（`find_linked_time_blocks_in_tx`）。
11. 对每个时间块，调用 `classify_time_block_action` 分类处理动作：
    - 检查是否是唯一关联（`is_exclusive_link_in_tx`）
    - 检查是否是自动创建的（标题与任务标题一致）
    - 使用 `completed_at_client` 判断时间状态：保留/截断/删除
12. 在执行删除/截断之前，先查询完整的时间块数据（用于 SSE 事件）。
13. 执行时间块的删除和截断操作：
    - 删除未来的时间块（`soft_delete_in_tx`）
    - 截断正在进行的时间块（`truncate_to_in_tx`），截断到 `completed_at_client`
14. 查询被截断的时间块的完整数据。
15. 重新查询任务并组装 `TaskCardDto`。
16. 在事务内填充 `schedules` 字段。
17. 根据 schedules 设置正确的 `schedule_status`。
18. 写入领域事件到 outbox（包含完成的任务和副作用的时间块）。
19. 提交事务（`TransactionHelper::commit`）。
20. 返回完成后的任务。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **任务已完成:** 返回 `409` 冲突（幂等性保护）。
- **视图上下文格式错误:** 返回 `400` 验证错误。
- **过去日期视图中完成:**
  - 例如：今天是 10-05，在 `daily::2025-10-01` 视图中完成
  - 结果：任务 `completed_at = 10-05T14:30:00`，日程在 **10-01**，删除 10-02~10-05 的日程
  - 语义：补记录历史，"那天其实完成了"
- **未来日期视图中完成:**
  - 例如：今天是 10-05，在 `daily::2025-10-10` 视图中完成
  - 结果：任务 `completed_at = 10-05T14:30:00`，日程在 **10-05**（今天），删除 10-06~10-10 的日程
  - 语义：提前完成，"今天提前做完了"
- **今天日期视图中完成:**
  - 例如：今天是 10-05，在 `daily::2025-10-05` 视图中完成
  - 结果：日程在 **10-05**，删除 > 10-05 的日程
  - 语义：正常完成
- **通用视图中完成:**
  - 例如：在 `misc::staging` 视图中完成
  - 结果：日程在**今天**，删除 > 今天的日程
  - 语义：正常完成
- **schedule_date 没有日程:** 自动创建一条日程并标记为已完成。
- **schedule_date 已有日程:** 直接更新为已完成。
- **时间块是手动创建的（标题与任务不一致）:** 保留，不删除也不截断。
- **时间块关联多个任务:** 保留，不删除也不截断（避免影响其他任务）。
- **时间块在过去:** 保留（记录已完成的工作）。
- **时间块正在进行:** 截断到 `completed_at_client`（记录部分努力）。
- **时间块在未来:** 删除（因为任务已完成，不需要未来的时间安排）。
- **幂等性:** 通过 `completed_at` 检查和 correlation_id 实现。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 查询任务、链接的时间块、排他性检查、检查 schedule_date 是否有日程。
    - **`UPDATE`:** 1条记录在 `tasks` 表（设置 `completed_at = completed_at_client`）。
    - **`INSERT`:** 0-1 条记录在 `task_schedules` 表（如果 schedule_date 没有日程，创建一条）。
    - **`UPDATE`:** 1 条记录在 `task_schedules` 表（schedule_date 的日程设为完成）。
    - **`DELETE`:** 0-N 条记录在 `task_schedules` 表（删除 > schedule_date 的日程）。
    - **`UPDATE`:** 0-N 条记录在 `time_blocks` 表（软删除或截断）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `task.completed` 事件，包含：
        - 完成的任务（`TaskCardDto`），`completed_at` 使用客户端时间
        - 副作用：删除的时间块列表（`TimeBlockViewDto[]`）
        - 副作用：截断的时间块列表（`TimeBlockViewDto[]`）
- **日志记录:**
    - 记录视图上下文和计算出的 schedule_date。
    - 记录删除和截断的时间块 ID。
    - 失败时，记录详细错误信息。

## 8. 契约 (Contract)

### 8.1. 前置条件 (Preconditions)

- 任务必须存在且未完成。
- `completed_at_client` 必须是有效的 UTC 时间。
- `view_context` 必须是有效的上下文格式。

### 8.2. 后置条件 (Postconditions)

- 任务的 `completed_at` 被设置为 `completed_at_client`。
- 任务的所有子任务被标记为已完成。
- 在 `schedule_date` 有一条已完成的日程记录。
- `> schedule_date` 的所有日程被删除。
- 唯一关联且自动创建的时间块根据时间状态被保留/截断/删除。
- 手动创建或多任务共享的时间块被保留。

### 8.3. 不变量 (Invariants)

- 已完成的任务不能被再次完成（幂等性）。
- 日程记录的位置由视图上下文决定，反映用户的操作意图。
- 时间块的处理基于客户端时间，确保时区一致性。
- 历史数据（过去的日程和时间块）总是被保留。

*（无其他已知副作用）*
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<CompleteTaskRequest>,
) -> Response {
    let correlation_id = extract_correlation_id(&headers);
    match logic::execute(&app_state, task_id, request, correlation_id).await {
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
        request: CompleteTaskRequest,
        correlation_id: Option<String>,
    ) -> AppResult<CompleteTaskResponse> {
        // ✅ 使用客户端时间作为完成时间
        let completed_at = request.completed_at_client;

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 开始事务（✅ 使用 TransactionHelper）
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 1. 查找任务（✅ 使用共享 Repository）
        let task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 2. 检查是否已完成
        if task.completed_at.is_some() {
            return Err(AppError::conflict("任务已经完成"));
        }

        // 3. 解析视图上下文，确定日程创建位置
        let schedule_date = determine_schedule_date(&request.view_context)?;
        tracing::info!(
            "Complete task {} in view context '{}', schedule_date: {}",
            task_id,
            request.view_context,
            schedule_date
        );

        // 4. 设置任务为已完成（✅ 使用客户端时间）
        TaskRepository::set_completed_in_tx(&mut tx, task_id, completed_at).await?;

        // 5. 如果有子任务，将所有子任务标记为已完成
        if let Some(mut subtasks) = task.subtasks.clone() {
            for subtask in &mut subtasks {
                subtask.is_completed = true;
            }
            TaskRepository::update_subtasks_in_tx(&mut tx, task_id, Some(subtasks)).await?;
        }

        // 6. 处理日程：确保在 schedule_date 有已完成的日程，删除之后的日程
        let has_schedule = TaskScheduleRepository::has_schedule_for_day_in_tx(
            &mut tx,
            task_id,
            &schedule_date,
        )
        .await?;

        if has_schedule {
            // 已有日程，更新为已完成
            TaskScheduleRepository::update_day_to_completed_in_tx(
                &mut tx,
                task_id,
                &schedule_date,
                completed_at,
            )
            .await?;
        } else {
            // 没有日程，创建一条新的
            TaskScheduleRepository::create_in_tx(&mut tx, task_id, &schedule_date).await?;
            // 立即更新为已完成
            TaskScheduleRepository::update_day_to_completed_in_tx(
                &mut tx,
                task_id,
                &schedule_date,
                completed_at,
            )
            .await?;
            tracing::info!(
                "Created schedule on {} for completed task {}",
                schedule_date,
                task_id
            );
        }

        // 7. 删除 > schedule_date 的所有日程
        TaskScheduleRepository::delete_schedules_after_in_tx(&mut tx, task_id, &schedule_date)
            .await?;

        // 8. 查询所有链接的时间块（✅ 使用共享 Repository）
        let linked_blocks =
            TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx(&mut tx, task_id).await?;

        // 9. 第一遍：收集需要删除/截断的时间块（✅ 在删除之前先查询完整数据）
        let mut blocks_to_delete = Vec::new();
        let mut blocks_to_truncate = Vec::new();

        for block in linked_blocks {
            let action = classify_time_block_action(
                &block,
                &task.title,
                task_id,
                completed_at, // ✅ 使用客户端时间判断
                &mut tx,
            )
            .await?;
            match action {
                TimeBlockAction::Deleted => blocks_to_delete.push(block),
                TimeBlockAction::Truncated => blocks_to_truncate.push(block),
                TimeBlockAction::None => {}
            }
        }

        // 10. 查询将被删除的时间块的完整数据（✅ 使用共享装配器）
        let deleted_time_block_ids: Vec<Uuid> = blocks_to_delete.iter().map(|b| b.id).collect();
        let deleted_blocks =
            TimeBlockAssembler::assemble_for_event_in_tx(&mut tx, &deleted_time_block_ids).await?;

        // 11. 现在执行删除和截断操作（✅ 使用共享 Repository）
        for block in blocks_to_delete {
            TimeBlockRepository::soft_delete_in_tx(&mut tx, block.id).await?;
            tracing::info!("Deleted future block {}", block.id);
        }

        let mut truncated_time_block_ids = Vec::new();
        for block in blocks_to_truncate {
            TimeBlockRepository::truncate_to_in_tx(&mut tx, block.id, completed_at).await?;
            truncated_time_block_ids.push(block.id);
            tracing::info!("Truncated ongoing block {} to {}", block.id, completed_at);
        }

        // 12. 查询被截断的时间块的完整数据（✅ 使用共享装配器）
        let truncated_blocks =
            TimeBlockAssembler::assemble_for_event_in_tx(&mut tx, &truncated_time_block_ids)
                .await?;

        // 13. 重新查询任务并组装完整 TaskCard（✅ 使用共享 Repository）
        let updated_task_in_tx = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;
        let mut task_card_for_event = TaskAssembler::task_to_card_basic(&updated_task_in_tx);

        // 14. ✅ 在事务内填充 schedules 字段
        // ⚠️ 必须在写入 SSE 之前填充，确保 SSE 和 HTTP 返回的数据一致！
        task_card_for_event.schedules =
            TaskAssembler::assemble_schedules_in_tx(&mut tx, task_id).await?;

        // 15. ✅ 根据 schedules 设置正确的 schedule_status
        // staging 定义：今天和未来没有排期的任务，过去的排期不影响
        use crate::entities::ScheduleStatus;
        // ✅ 使用本地时间确定"今天"的日期，避免时区问题
        let today = chrono::Local::now().date_naive();

        let has_future_schedule = task_card_for_event
            .schedules
            .as_ref()
            .map(|schedules| {
                schedules.iter().any(|s| {
                    if let Ok(schedule_date) =
                        chrono::NaiveDate::parse_from_str(&s.scheduled_day, "%Y-%m-%d")
                    {
                        schedule_date >= today
                    } else {
                        false
                    }
                })
            })
            .unwrap_or(false);

        task_card_for_event.schedule_status = if has_future_schedule {
            ScheduleStatus::Scheduled
        } else {
            ScheduleStatus::Staging
        };

        // 16. 构建统一的事务结果
        // ✅ HTTP 响应和 SSE 事件使用相同的数据结构
        let transaction_result = TaskTransactionResult {
            task: task_card_for_event,
            side_effects: SideEffects {
                deleted_time_blocks: if deleted_blocks.is_empty() {
                    None
                } else {
                    Some(deleted_blocks)
                },
                truncated_time_blocks: if truncated_blocks.is_empty() {
                    None
                } else {
                    Some(truncated_blocks)
                },
                ..Default::default()
            },
        };

        // 17. 在事务中写入领域事件到 outbox
        // ✅ 一个业务事务 = 一个领域事件（包含所有副作用的完整数据）
        use crate::infra::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        {
            // ✅ 使用统一的事务结果作为事件载荷
            let payload = serde_json::to_value(&transaction_result)?;

            let mut event =
                DomainEvent::new("task.completed", "task", task_id.to_string(), payload)
                    .with_aggregate_version(completed_at.timestamp_millis());

            if let Some(cid) = correlation_id {
                event = event.with_correlation_id(cid);
            }

            outbox_repo.append_in_tx(&mut tx, &event).await?;
        }

        // 18. 提交事务（✅ 使用 TransactionHelper）
        TransactionHelper::commit(tx).await?;

        // 19. 返回结果
        // ✅ HTTP 响应与 SSE 事件载荷完全一致
        Ok(CompleteTaskResponse {
            result: transaction_result,
        })
    }

    /// 根据视图上下文确定日程创建位置
    ///
    /// # 规则
    /// - 过去日期视图 (daily::2025-10-01 < today): 记录在那天（补记录历史）
    /// - 今天/未来日期视图 (daily::2025-10-05 >= today): 记录在今天（正常完成/提前完成）
    /// - 通用视图 (misc::*, area::*, project::*): 记录在今天
    fn determine_schedule_date(view_context: &str) -> AppResult<String> {
        let parts: Vec<&str> = view_context.split("::").collect();

        if parts.is_empty() {
            return Err(AppError::validation_error(
                "view_context",
                "Invalid view context format",
                "INVALID_VIEW_CONTEXT",
            ));
        }

        // 获取今天的日期（使用本地时间）
        use crate::infra::core::utils::time_utils;
        let today = chrono::Local::now().date_naive();
        let today_str = time_utils::format_date_yyyy_mm_dd(&today);

        match parts[0] {
            "daily" => {
                if parts.len() < 2 {
                    return Err(AppError::validation_error(
                        "view_context",
                        "Invalid daily context: missing date",
                        "MISSING_DATE",
                    ));
                }

                let view_date = parts[1];

                // 🔥 核心逻辑：比较视图日期和今天
                if view_date < today_str.as_str() {
                    // 过去：补记录历史
                    tracing::info!(
                        "View date {} is in the past (today: {}), schedule on that date",
                        view_date,
                        today_str
                    );
                    Ok(view_date.to_string())
                } else {
                    // 今天/未来：记录在今天
                    tracing::info!(
                        "View date {} is today or future (today: {}), schedule on today",
                        view_date,
                        today_str
                    );
                    Ok(today_str)
                }
            }
            "misc" | "area" | "project" => {
                // 通用视图：总是今天
                Ok(today_str)
            }
            _ => Err(AppError::validation_error(
                "view_context",
                "Unknown view context type",
                "UNKNOWN_VIEW_CONTEXT_TYPE",
            )),
        }
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
        // 1. 检查是否仅链接此任务（✅ 使用共享 Repository）
        let is_exclusive =
            TaskTimeBlockLinkRepository::is_exclusive_link_in_tx(tx, block.id, task_id).await?;
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
// ✅ 已全部迁移到共享 Repository：
// - TaskRepository::find_by_id_in_tx, set_completed_in_tx
// - TaskScheduleRepository::update_today_to_completed_in_tx, delete_future_schedules_in_tx
// - TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx, is_exclusive_link_in_tx
// - TimeBlockRepository::soft_delete_in_tx, truncate_to_in_tx
// - TimeBlockAssembler::assemble_for_event_in_tx
