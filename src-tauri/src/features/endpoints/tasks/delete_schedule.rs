/// 删除任务日程 API - 单文件组件
///
/// DELETE /api/tasks/:id/schedules/:date
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use serde::Serialize;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{SideEffects, TaskTransactionResult, TimeBlock},
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

// ==================== 文档层 ====================
/*
CABC for `delete_schedule`

## 1. 端点签名 (Endpoint Signature)

DELETE /api/tasks/{id}/schedules/{date}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我取消某天的任务安排时，我希望系统能够：
> 1. 删除该日期的日程记录
> 2. 清理该日期关联的时间块链接
> 3. 智能清理"孤儿"时间块（只关联该任务且没有其他用途的时间块）

### 2.2. 核心业务逻辑 (Core Business Logic)

删除任务在指定日期的日程记录，并清理相关数据：
1. 删除 `task_schedules` 记录
2. 删除该日期所有时间块的 `task_time_block_links` 记录
3. 软删除"孤儿"时间块（删除链接后没有任何关联任务的时间块）
4. 如果任务没有剩余日程，`schedule_status` 会变回 `Staging`

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 任务ID
- `date` (YYYY-MM-DD, required): 日程日期

**请求头 (Request Headers):**
- `X-Correlation-ID` (optional): 用于前端去重和请求追踪

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`

```json
{
  "task_card": {
    "id": "uuid",
    "title": "string",
    "schedule_status": "staging" | "scheduled",
    "schedules": [...] | null,
    ...
  }
}
```

**注意：** 副作用（删除的时间块）通过 SSE 事件推送。

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Task not found: {id}" | "Schedule not found: Task {id} on {date}"
}
```

## 4. 验证规则 (Validation Rules)

- `task_id`:
    - **必须**是有效的 UUID 格式。
    - **必须**存在于数据库中且未删除。
    - 违反时返回 `404 NOT_FOUND`
- `date`:
    - **必须**符合 `YYYY-MM-DD` 格式。
    - 该日期**必须**有日程记录。
    - 违反时返回 `404 NOT_FOUND` 或 `422 VALIDATION_FAILED`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  解析日期字符串为 `DateTime<Utc>`（`validation::parse_date`）。
2.  获取写入许可（`app_state.acquire_write_permit()`）。
3.  启动数据库事务（`TransactionHelper::begin`）。
4.  查询任务（`TaskRepository::find_by_id_in_tx`）。
5.  如果任务不存在，返回 404 错误。
6.  检查该日期是否有日程（`TaskScheduleRepository::has_schedule_for_day_in_tx`）。
7.  如果该日期没有日程，返回 404 错误。
8.  查找该日期的所有时间块（`database::find_time_blocks_for_day`）。
9.  对每个时间块，删除任务到时间块的链接（`database::delete_task_time_block_link`）。
10. 对每个时间块，检查是否变成"孤儿"（`TaskTimeBlockLinkRepository::count_remaining_tasks_in_block_in_tx`）。
11. 如果时间块没有剩余任务，软删除该时间块（`TimeBlockRepository::soft_delete_in_tx`）。
12. 在删除之前，查询被删除的时间块的完整数据（用于 SSE 事件）。
13. 删除日程记录（`database::delete_schedule`）。
14. 重新查询任务并组装 `TaskCardDto`。
15. 在事务内填充 `schedules` 字段。
16. 根据 schedules 设置正确的 `schedule_status`（如果没有剩余日程，应为 `Staging`）。
17. 写入领域事件到 outbox（`task.schedule_deleted` 事件）。
18. 提交事务（`TransactionHelper::commit`）。
19. 返回更新后的任务。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **该日期没有日程:** 返回 `404` 错误。
- **时间块还有其他任务:** 不删除时间块（避免影响其他任务）。
- **该日期没有时间块:** 只删除日程记录。
- **删除最后一个日程:** `schedule_status` 变为 `Staging`。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次查询 `tasks` 表（验证任务存在）。
    - **`SELECT`:** 1次查询 `task_schedules` 表（检查日程是否存在）。
    - **`SELECT`:** 1次查询 `time_blocks` 表（查找该日期的时间块）。
    - **`DELETE`:** 0-N 条记录在 `task_time_block_links` 表。
    - **`SELECT`:** 0-N 次查询 `task_time_block_links` 表（检查孤儿状态）。
    - **`UPDATE`:** 0-N 条记录在 `time_blocks` 表（软删除孤儿时间块）。
    - **`DELETE`:** 1条记录在 `task_schedules` 表。
    - **`SELECT`:** 1次查询 `tasks` 表（重新获取数据）。
    - **`SELECT`:** 1次查询 `task_schedules` 表（填充 schedules）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `task.schedule_deleted` 事件，包含：
        - 更新后的任务（`TaskCardDto`）
        - 删除的日期（`deleted_date`）
        - 副作用：删除的时间块列表（`TimeBlockViewDto[]`）
- **日志记录:**
    - 记录删除的孤儿时间块 ID。
    - 失败时，记录详细错误信息。

*（无其他已知副作用）*
*/

// ==================== 响应结构体 ====================
/// 删除日程的响应
/// ✅ HTTP 响应和 SSE 事件使用相同的数据结构
#[derive(Debug, Serialize)]
pub struct DeleteScheduleResponse {
    #[serde(flatten)]
    pub result: TaskTransactionResult,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path((task_id, date_str)): Path<(Uuid, String)>,
    headers: HeaderMap,
) -> Response {
    let correlation_id = extract_correlation_id(&headers);
    match logic::execute(&app_state, task_id, &date_str, correlation_id).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn parse_date(date_str: &str) -> AppResult<String> {
        use crate::infra::core::utils::time_utils;
        time_utils::parse_date_yyyy_mm_dd(date_str)
            .map(|date| time_utils::format_date_yyyy_mm_dd(&date))
            .map_err(|_| {
                AppError::validation_error(
                    "scheduled_day",
                    "日期格式错误，请使用 YYYY-MM-DD 格式",
                    "INVALID_DATE_FORMAT",
                )
            })
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;
    use crate::features::shared::TransactionHelper;

    pub async fn execute(
        app_state: &AppState,
        task_id: Uuid,
        date_str: &str,
        correlation_id: Option<String>,
    ) -> AppResult<DeleteScheduleResponse> {
        let now = app_state.clock().now_utc();

        // 1. 解析日期
        let scheduled_day = validation::parse_date(date_str)?;

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 2. 开始事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 3. 查找任务
        let _task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 4. 检查该日期是否有日程
        let has_schedule =
            TaskScheduleRepository::has_schedule_for_day_in_tx(&mut tx, task_id, &scheduled_day)
                .await?;

        if !has_schedule {
            return Err(AppError::not_found(
                "Schedule",
                format!("Task {} on {}", task_id, date_str),
            ));
        }

        // 5. 查找该日期的所有 time_blocks
        let time_blocks =
            database::find_time_blocks_for_day(&mut tx, task_id, &scheduled_day).await?;

        // 6. 删除 task_time_block_links
        let time_block_ids: Vec<Uuid> = time_blocks.iter().map(|b| b.id).collect();
        for &block_id in &time_block_ids {
            database::delete_task_time_block_link(&mut tx, task_id, block_id).await?;
        }

        // 7. 软删除"孤儿"时间片
        let mut deleted_time_block_ids = Vec::new();
        for block in &time_blocks {
            let remaining_links =
                TaskTimeBlockLinkRepository::count_remaining_tasks_in_block_in_tx(
                    &mut tx, block.id,
                )
                .await?;

            if remaining_links == 0 {
                TimeBlockRepository::soft_delete_in_tx(&mut tx, block.id).await?;
                deleted_time_block_ids.push(block.id);
            }
        }

        // 8. 查询被删除的时间块的完整数据（用于事件）
        let deleted_time_blocks =
            TimeBlockAssembler::assemble_for_event_in_tx(&mut tx, &deleted_time_block_ids).await?;

        // 9. 删除 schedule 记录
        database::delete_schedule(&mut tx, task_id, &scheduled_day).await?;

        // 10. 重新查询任务并组装 TaskCard
        // 注意：schedule_status 是派生字段，由装配器根据 task_schedules 表计算
        let updated_task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        let mut task_card = TaskAssembler::task_to_card_basic(&updated_task);

        // 13. ✅ 在事务内填充 schedules 字段
        // ⚠️ 必须在写入 SSE 之前填充，确保 SSE 和 HTTP 返回的数据一致！
        task_card.schedules = TaskAssembler::assemble_schedules_in_tx(&mut tx, task_id).await?;

        // 13.5. ✅ 根据 schedules 设置正确的 schedule_status
        // staging 定义：今天和未来没有排期的任务，过去的排期不影响
        use crate::entities::ScheduleStatus;
        use chrono::Utc;
        let local_today = Utc::now().date_naive();

        let has_future_schedule = task_card
            .schedules
            .as_ref()
            .map(|schedules| {
                schedules.iter().any(|s| {
                    if let Ok(schedule_date) =
                        chrono::NaiveDate::parse_from_str(&s.scheduled_day, "%Y-%m-%d")
                    {
                        schedule_date >= local_today
                    } else {
                        false
                    }
                })
            })
            .unwrap_or(false);

        task_card.schedule_status = if has_future_schedule {
            ScheduleStatus::Scheduled
        } else {
            ScheduleStatus::Staging
        };

        // 14. 构建统一的事务结果
        // ✅ HTTP 响应和 SSE 事件使用相同的数据结构
        let transaction_result = TaskTransactionResult {
            task: task_card,
            side_effects: SideEffects {
                deleted_time_blocks: if deleted_time_blocks.is_empty() {
                    None
                } else {
                    Some(deleted_time_blocks)
                },
                ..Default::default()
            },
        };

        // 15. 写入领域事件到 outbox
        use crate::infra::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        {
            // ✅ 使用统一的事务结果作为事件载荷
            let payload = serde_json::to_value(&transaction_result)?;

            let mut event = DomainEvent::new(
                "task.schedule_deleted",
                "task",
                task_id.to_string(),
                payload,
            )
            .with_aggregate_version(now.timestamp_millis());

            if let Some(cid) = correlation_id {
                event = event.with_correlation_id(cid);
            }

            outbox_repo.append_in_tx(&mut tx, &event).await?;
        }

        // 16. 提交事务
        TransactionHelper::commit(tx).await?;

        // 17. 返回结果
        // ✅ HTTP 响应与 SSE 事件载荷完全一致
        Ok(DeleteScheduleResponse {
            result: transaction_result,
        })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;

    /// 查找任务在指定日期的所有时间块
    pub async fn find_time_blocks_for_day(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        scheduled_date: &str, // YYYY-MM-DD 字符串
    ) -> AppResult<Vec<TimeBlock>> {
        let query = r#"
            SELECT tb.id, tb.title, tb.glance_note, tb.detail_note, tb.start_time, tb.end_time,
                   tb.start_time_local, tb.end_time_local, tb.time_type, tb.creation_timezone,
                   tb.is_all_day, tb.source_info, tb.external_source_id, tb.external_source_provider,
                   tb.external_source_metadata,
                   tb.area_id, tb.recurrence_rule, tb.recurrence_parent_id, tb.recurrence_original_date,
                   tb.created_at, tb.updated_at, tb.is_deleted
            FROM time_blocks tb
            JOIN task_time_block_links ttbl ON ttbl.time_block_id = tb.id
            WHERE ttbl.task_id = ?
              AND DATE(tb.start_time) = ?
              AND tb.is_deleted = false
        "#;

        let rows = sqlx::query_as::<_, crate::entities::TimeBlockRow>(query)
            .bind(task_id.to_string())
            .bind(scheduled_date)
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        let time_blocks = rows
            .into_iter()
            .map(|row| {
                TimeBlock::try_from(row).map_err(|e| {
                    AppError::DatabaseError(crate::infra::core::DbError::QueryError(e))
                })
            })
            .collect::<AppResult<Vec<TimeBlock>>>()?;

        Ok(time_blocks)
    }

    /// 删除任务到时间块的链接
    pub async fn delete_task_time_block_link(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        time_block_id: Uuid,
    ) -> AppResult<()> {
        let query = r#"
            DELETE FROM task_time_block_links
            WHERE task_id = ? AND time_block_id = ?
        "#;

        sqlx::query(query)
            .bind(task_id.to_string())
            .bind(time_block_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(())
    }

    /// 删除日程记录
    pub async fn delete_schedule(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        scheduled_date: &str, // YYYY-MM-DD 字符串
    ) -> AppResult<()> {
        let query = r#"
            DELETE FROM task_schedules
            WHERE task_id = ? AND scheduled_date = ?
        "#;

        sqlx::query(query)
            .bind(task_id.to_string())
            .bind(scheduled_date)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(())
    }
}
