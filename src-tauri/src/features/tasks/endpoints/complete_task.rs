/// 完成任务 API - 单文件组件
///
/// 按照 Cutie 的精确业务逻辑实现
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use serde::Serialize;

use crate::{
    entities::{TaskCardDto, TimeBlock},
    features::tasks::shared::{
        assemblers::TimeBlockAssembler,
        repositories::{TaskRepository, TaskScheduleRepository, TaskTimeBlockLinkRepository},
        TaskAssembler,
    },
    features::time_blocks::shared::repositories::TimeBlockRepository,
    shared::{
        core::{AppError, AppResult},
        http::{error_handler::success_response, extractors::extract_correlation_id},
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

## 1. 端点签名 (Endpoint Signature)

POST /api/tasks/{id}/completion

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我完成一个任务时，我希望系统能够：
> 1. 标记任务为已完成
> 2. 保留当天的日程记录（记录我的努力）
> 3. 清理未来的日程和时间块（因为任务已完成，不需要未来的安排）
> 4. 智能处理正在进行的时间块（截断到当前时间）

### 2.2. 核心业务逻辑 (Core Business Logic)

完成任务，并根据 Cutie 的业务规则智能处理相关的日程和时间块：
1. **当天日程**:
   - 如果今天已有日程：设置为已完成（`outcome = 'COMPLETED_ON_DAY'`）
   - 如果今天没有日程：创建一条新日程并设置为已完成（确保任务保留在今天的看板中）
2. **未来日程**: 删除
3. **时间块处理**（仅针对唯一关联且自动创建的时间块）:
   - 在过去：保留
   - 正在进行（start_time <= now < end_time）：截断到当前时间
   - 在未来：删除

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
    "is_completed": true,
    "completed_at": "2025-10-05T12:00:00Z",
    ...
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

## 4. 验证规则 (Validation Rules)

- `task_id`:
    - **必须**是有效的 UUID 格式。
    - **必须**存在于数据库中。
    - 违反时返回 `404 NOT_FOUND`
- **业务规则验证:**
    - 任务**不能**已经完成（`completed_at IS NOT NULL`）。
    - 违反时返回 `409 CONFLICT`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  获取当前时间 `now`。
2.  获取写入许可（`app_state.acquire_write_permit()`）。
3.  启动数据库事务（`TransactionHelper::begin`）。
4.  查询任务（`TaskRepository::find_by_id_in_tx`）。
5.  如果任务不存在，返回 404 错误。
6.  检查任务是否已完成，如果是，返回 409 冲突。
7.  设置任务为已完成（`TaskRepository::set_completed_in_tx`）。
8.  处理日程:
    - 检查今天是否有日程（`TaskScheduleRepository::has_schedule_for_day_in_tx`）
    - 如果有：更新当天日程为已完成（`TaskScheduleRepository::update_today_to_completed_in_tx`）
    - 如果没有：创建今天的日程（`TaskScheduleRepository::create_in_tx`），然后更新为已完成
    - 删除未来日程（`TaskScheduleRepository::delete_future_schedules_in_tx`）
9.  查询所有链接的时间块（`TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx`）。
10. 对每个时间块，调用 `classify_time_block_action` 分类处理动作：
    - 检查是否是唯一关联（`is_exclusive_link_in_tx`）
    - 检查是否是自动创建的（标题与任务标题一致）
    - 根据时间判断动作：保留/截断/删除
11. 在执行删除/截断之前，先查询完整的时间块数据（用于 SSE 事件）。
12. 执行时间块的删除和截断操作：
    - 删除未来的时间块（`TimeBlockRepository::soft_delete_in_tx`）
    - 截断正在进行的时间块（`TimeBlockRepository::truncate_to_in_tx`）
13. 查询被截断的时间块的完整数据。
14. 重新查询任务并组装 `TaskCardDto`。
15. 在事务内填充 `schedules` 字段。
16. 根据 schedules 设置正确的 `schedule_status`。
17. 写入领域事件到 outbox（包含完成的任务和副作用的时间块）。
18. 提交事务（`TransactionHelper::commit`）。
19. 返回完成后的任务。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **任务已完成:** 返回 `409` 冲突（幂等性保护）。
- **今天没有日程:** 自动创建一条日程并标记为已完成，确保任务保留在今天的看板中。
- **今天已有日程:** 直接更新为已完成。
- **时间块是手动创建的（标题与任务不一致）:** 保留，不删除也不截断。
- **时间块关联多个任务:** 保留，不删除也不截断（避免影响其他任务）。
- **时间块在过去:** 保留（记录已完成的工作）。
- **时间块正在进行:** 截断到当前时间（记录部分努力）。
- **时间块在未来:** 删除（因为任务已完成，不需要未来的时间安排）。
- **无日程和时间块的任务:** 创建今天的日程并标记为已完成，然后更新 `completed_at` 字段。
- **幂等性:** 通过 `completed_at` 检查和 correlation_id 实现。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 查询任务、链接的时间块、排他性检查、检查今天是否有日程。
    - **`UPDATE`:** 1条记录在 `tasks` 表（设置 `completed_at`）。
    - **`INSERT`:** 0-1 条记录在 `task_schedules` 表（如果今天没有日程，创建一条）。
    - **`UPDATE`:** 1 条记录在 `task_schedules` 表（今天的日程设为完成）。
    - **`DELETE`:** 0-N 条记录在 `task_schedules` 表（删除未来日程）。
    - **`UPDATE`:** 0-N 条记录在 `time_blocks` 表（软删除或截断）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `task.completed` 事件，包含：
        - 完成的任务（`TaskCardDto`）
        - 副作用：删除的时间块列表（`TimeBlockViewDto[]`）
        - 副作用：截断的时间块列表（`TimeBlockViewDto[]`）
- **日志记录:**
    - 记录删除和截断的时间块 ID。
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
    ) -> AppResult<CompleteTaskResponse> {
        let now = app_state.clock().now_utc();

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

        // 3. 设置任务为已完成（✅ 使用共享 Repository）
        TaskRepository::set_completed_in_tx(&mut tx, task_id, now).await?;

        // 4. 处理日程：确保今天有日程并设为完成，删除未来日程
        // 4.1. 检查今天是否有日程
        use crate::shared::core::utils::time_utils;
        let today_date = time_utils::format_date_yyyy_mm_dd(&now.date_naive());
        let has_today_schedule =
            TaskScheduleRepository::has_schedule_for_day_in_tx(&mut tx, task_id, &today_date).await?;

        if has_today_schedule {
            // 今天已有日程，更新为已完成
            TaskScheduleRepository::update_today_to_completed_in_tx(&mut tx, task_id, now).await?;
        } else {
            // 今天没有日程，创建一条新的
            TaskScheduleRepository::create_in_tx(&mut tx, task_id, &today_date).await?;
            // 立即更新为已完成
            TaskScheduleRepository::update_today_to_completed_in_tx(&mut tx, task_id, now).await?;
            tracing::info!("Created today's schedule for completed task {}", task_id);
        }

        // 4.2. 删除未来日程
        TaskScheduleRepository::delete_future_schedules_in_tx(&mut tx, task_id, now).await?;

        // 5. 查询所有链接的时间块（✅ 使用共享 Repository）
        let linked_blocks =
            TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx(&mut tx, task_id).await?;

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

        // 7. 查询将被删除的时间块的完整数据（✅ 使用共享装配器）
        let deleted_time_block_ids: Vec<Uuid> = blocks_to_delete.iter().map(|b| b.id).collect();
        let deleted_blocks =
            TimeBlockAssembler::assemble_for_event_in_tx(&mut tx, &deleted_time_block_ids).await?;

        // 8. 现在执行删除和截断操作（✅ 使用共享 Repository）
        for block in blocks_to_delete {
            TimeBlockRepository::soft_delete_in_tx(&mut tx, block.id).await?;
            tracing::info!("Deleted future block {}", block.id);
        }

        let mut truncated_time_block_ids = Vec::new();
        for block in blocks_to_truncate {
            TimeBlockRepository::truncate_to_in_tx(&mut tx, block.id, now).await?;
            truncated_time_block_ids.push(block.id);
            tracing::info!("Truncated ongoing block {} to {}", block.id, now);
        }

        // 9. 查询被截断的时间块的完整数据（✅ 使用共享装配器）
        let truncated_blocks =
            TimeBlockAssembler::assemble_for_event_in_tx(&mut tx, &truncated_time_block_ids)
                .await?;

        // 10. 重新查询任务并组装完整 TaskCard（✅ 使用共享 Repository）
        let updated_task_in_tx = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;
        let mut task_card_for_event = TaskAssembler::task_to_card_basic(&updated_task_in_tx);

        // 10.1. area_id 已由 TaskAssembler 填充

        // 11. ✅ 在事务内填充 schedules 字段
        // ⚠️ 必须在写入 SSE 之前填充，确保 SSE 和 HTTP 返回的数据一致！
        task_card_for_event.schedules =
            TaskAssembler::assemble_schedules_in_tx(&mut tx, task_id).await?;

        // 11.5. ✅ 根据 schedules 设置正确的 schedule_status
        // staging 定义：今天和未来没有排期的任务，过去的排期不影响
        use crate::entities::ScheduleStatus;
        use chrono::Utc;
        let today = Utc::now().date_naive();

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

        // 12. 在事务中写入领域事件到 outbox
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
                    "deleted_time_blocks": deleted_blocks,
                    "truncated_time_blocks": truncated_blocks,
                }
            });
            let mut event =
                DomainEvent::new("task.completed", "task", task_id.to_string(), payload)
                    .with_aggregate_version(now.timestamp_millis());

            if let Some(cid) = correlation_id {
                event = event.with_correlation_id(cid);
            }

            outbox_repo.append_in_tx(&mut tx, &event).await?;
        }

        // 13. 提交事务（✅ 使用 TransactionHelper）
        TransactionHelper::commit(tx).await?;

        // 14. 返回结果（复用事件中的 task_card）
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
