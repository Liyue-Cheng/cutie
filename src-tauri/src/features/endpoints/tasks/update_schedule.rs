/// 更新任务日程 API - 单文件组件
///
/// PATCH /api/tasks/:id/schedules/:date
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::{Outcome, SideEffects, TaskTransactionResult, TimeBlock},
    features::shared::{
        assemblers::TimeBlockAssembler,
        repositories::{
            TaskRepository, TaskScheduleRepository, TaskTimeBlockLinkRepository,
            TimeBlockRepository,
        },
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
CABC for `update_schedule`

## 1. 端点签名 (Endpoint Signature)

PATCH /api/tasks/{id}/schedules/{date}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要修改任务的日程安排，可以更改日期或更新完成状态，
> 以便我能灵活调整我的任务计划。

### 2.2. 核心业务逻辑 (Core Business Logic)

更新任务在指定日期的日程记录。支持两种更新：
1. 更改日期（`new_date`）：将日程从原日期移动到新日期
2. 更新结果状态（`outcome`）：标记日程的完成情况（PLANNED/PRESENCE_LOGGED/COMPLETED_ON_DAY/CARRIED_OVER）

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 任务ID
- `date` (YYYY-MM-DD, required): 原日期

**请求体 (Request Body):** `application/json`

```json
{
  "new_date": "string (YYYY-MM-DD) | null (optional)",
  "outcome": "string ('PLANNED' | 'PRESENCE_LOGGED' | 'COMPLETED_ON_DAY' | 'CARRIED_OVER') | null (optional)"
}
```

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
    "schedule_status": "scheduled",
    "schedules": [...],
    ...
  },
  "side_effects": {
    "deleted_time_blocks": [
      {
        "id": "uuid",
        "title": "string",
        "start_time": "2025-01-01T09:00:00Z",
        "end_time": "2025-01-01T10:00:00Z",
        ...
      }
    ]
  }
}
```

**注意：**
- 当改期到不同日期时，原日期的孤儿浮动时间片会被删除并包含在 `side_effects.deleted_time_blocks` 中
- HTTP 响应和 SSE 事件使用完全相同的数据结构

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Task not found: {id}" | "Schedule not found: Task {id} on {date}"
}
```

**409 Conflict:**

```json
{
  "error_code": "CONFLICT",
  "message": "目标日期已有日程安排"
}
```

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "request", "code": "EMPTY_REQUEST", "message": "必须提供 new_date 或 outcome 至少一个字段" }
  ]
}
```

## 4. 验证规则 (Validation Rules)

- **请求完整性:**
    - `new_date` 和 `outcome` **至少提供一个**。
    - 违反时返回错误码：`EMPTY_REQUEST`
- `new_date`:
    - 如果提供，**必须**符合 `YYYY-MM-DD` 格式。
    - 违反时返回错误码：`INVALID_DATE_FORMAT`
- `outcome`:
    - 如果提供，**必须**是有效值之一：`PLANNED`, `PRESENCE_LOGGED`, `COMPLETED_ON_DAY`, `CARRIED_OVER`。
    - 违反时返回错误码：`INVALID_OUTCOME`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  验证请求（`validation::validate_request`，确保至少提供一个字段）。
2.  解析原始日期（`validation::parse_date`）。
3.  获取写入许可（`app_state.acquire_write_permit()`）。
4.  启动数据库事务（`TransactionHelper::begin`）。
5.  查询任务（`TaskRepository::find_by_id_in_tx`）。
6.  如果任务不存在，返回 404 错误。
7.  检查原始日期是否有日程（`TaskScheduleRepository::has_schedule_for_day_in_tx`）。
8.  如果原始日期没有日程，返回 404 错误。
9.  如果提供了 `new_date`：
    - 解析新日期
    - 如果新日期与原日期不同，检查新日期是否已有日程
    - 如果新日期已有日程，返回 409 冲突
    - 更新日程的日期（`database::update_schedule_date`）
10. 如果提供了 `outcome`：
    - 解析 outcome 枚举值（`validation::parse_outcome`）
    - 确定目标日期（如果更改了日期，使用新日期；否则使用原日期）
    - 更新日程的 outcome（`database::update_schedule_outcome`）
11. 重新查询任务并组装 `TaskCardDto`。
12. 在事务内填充 `schedules` 字段。
13. 根据 schedules 设置正确的 `schedule_status`。
14. 写入领域事件到 outbox（`task.schedule_updated` 事件）。
15. 提交事务（`TransactionHelper::commit`）。
16. 返回更新后的任务。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **原日期没有日程:** 返回 `404` 错误。
- **新日期已有日程:** 返回 `409` 冲突。
- **新日期与原日期相同:** 允许（仅视为 outcome 更新）。
- **两个字段都不提供:** 返回 `422` 验证错误。
- **outcome 值无效:** 返回 `422` 验证错误。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次查询 `tasks` 表（验证任务存在）。
    - **`SELECT`:** 1-2次查询 `task_schedules` 表（检查原日期和新日期）。
    - **`UPDATE`:** 1条记录在 `task_schedules` 表（更新日期和/或 outcome）。
    - **`SELECT`:** 1次查询 `tasks` 表（重新获取数据）。
    - **`SELECT`:** 1次查询 `task_schedules` 表（填充 schedules）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `task.schedule_updated` 事件，包含：
        - 更新后的任务（`TaskCardDto`）
        - 原日期（`original_date`）
        - 新日期（`new_date`，如果有）
        - 新 outcome（`outcome`，如果有）
- **日志记录:**
    - 成功时，记录日程更新信息。
    - 失败时，记录详细错误信息。

*（无其他已知副作用）*
*/

// ==================== 请求/响应结构体 ====================
#[derive(Debug, Deserialize)]
pub struct UpdateScheduleRequest {
    /// 新日期（YYYY-MM-DD 格式，可选）
    pub new_date: Option<String>,
    /// 新的结局状态（可选）
    pub outcome: Option<String>,
}

/// 更新日程的响应
/// ✅ HTTP 响应和 SSE 事件使用相同的数据结构
#[derive(Debug, Serialize)]
pub struct UpdateScheduleResponse {
    #[serde(flatten)]
    pub result: TaskTransactionResult,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path((task_id, date_str)): Path<(Uuid, String)>,
    headers: HeaderMap,
    Json(request): Json<UpdateScheduleRequest>,
) -> Response {
    let correlation_id = extract_correlation_id(&headers);
    match logic::execute(&app_state, task_id, &date_str, request, correlation_id).await {
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

    pub fn parse_outcome(outcome_str: &str) -> AppResult<Outcome> {
        match outcome_str {
            "PLANNED" => Ok(Outcome::Planned),
            "PRESENCE_LOGGED" => Ok(Outcome::PresenceLogged),
            "COMPLETED_ON_DAY" => Ok(Outcome::CompletedOnDay),
            "CARRIED_OVER" => Ok(Outcome::CarriedOver),
            _ => Err(AppError::validation_error(
                "outcome",
                format!("无效的 outcome 值: {}", outcome_str),
                "INVALID_OUTCOME",
            )),
        }
    }

    pub fn validate_request(request: &UpdateScheduleRequest) -> AppResult<()> {
        if request.new_date.is_none() && request.outcome.is_none() {
            return Err(AppError::validation_error(
                "request",
                "必须提供 new_date 或 outcome 至少一个字段",
                "EMPTY_REQUEST",
            ));
        }
        Ok(())
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
        request: UpdateScheduleRequest,
        correlation_id: Option<String>,
    ) -> AppResult<UpdateScheduleResponse> {
        let now = app_state.clock().now_utc();

        // 1. 验证请求
        validation::validate_request(&request)?;

        // 2. 解析原始日期
        let original_date = validation::parse_date(date_str)?;

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 3. 开始事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 4. 查找任务
        let _task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 5. 检查原始日期是否有日程
        let has_original_schedule =
            TaskScheduleRepository::has_schedule_for_day_in_tx(&mut tx, task_id, &original_date)
                .await?;

        if !has_original_schedule {
            return Err(AppError::not_found(
                "Schedule",
                format!("Task {} on {}", task_id, date_str),
            ));
        }

        // 6. 处理更新逻辑
        let mut deleted_time_blocks = Vec::new();

        if let Some(ref new_date_str) = request.new_date {
            // 解析新日期
            let new_date = validation::parse_date(new_date_str)?;

            // 检查新日期是否已有日程（如果不是同一天）
            if original_date != new_date {
                let has_new_date_schedule =
                    TaskScheduleRepository::has_schedule_for_day_in_tx(&mut tx, task_id, &new_date)
                        .await?;

                if has_new_date_schedule {
                    return Err(AppError::conflict("目标日期已有日程安排"));
                }

                // 🔥 改期到不同日期时的正确逻辑：删除原日程，创建新日程
                // 1. 查找原日期的所有浮动时间片
                let time_blocks =
                    database::find_floating_time_blocks_for_day(&mut tx, task_id, &original_date)
                        .await?;

                // 2. 删除时间片链接
                let time_block_ids: Vec<Uuid> = time_blocks.iter().map(|b| b.id).collect();
                for &block_id in &time_block_ids {
                    database::delete_task_time_block_link(&mut tx, task_id, block_id).await?;
                }

                // 3. 软删除孤儿浮动时间片
                let mut deleted_time_block_ids = Vec::new();
                for block in &time_blocks {
                    let remaining_links =
                        TaskTimeBlockLinkRepository::count_remaining_tasks_in_block_in_tx(
                            &mut tx, block.id,
                        )
                        .await?;

                    // 只有当时间片没有任何剩余任务链接时才删除（孤儿检查）
                    if remaining_links == 0 {
                        TimeBlockRepository::soft_delete_in_tx(&mut tx, block.id).await?;
                        deleted_time_block_ids.push(block.id);
                    }
                }

                // 4. 查询被删除的时间片的完整数据（用于副作用）
                deleted_time_blocks =
                    TimeBlockAssembler::assemble_for_event_in_tx(&mut tx, &deleted_time_block_ids)
                        .await?;
            }

            // 更新日期（直接更新现有日程记录）
            database::update_schedule_date(&mut tx, task_id, &original_date, &new_date, now)
                .await?;
        }

        // 7. 处理 outcome 更新
        if let Some(ref outcome_str) = request.outcome {
            let outcome = validation::parse_outcome(outcome_str)?;
            let target_date = if let Some(ref new_date_str) = request.new_date {
                validation::parse_date(new_date_str)?
            } else {
                original_date
            };
            database::update_schedule_outcome(&mut tx, task_id, &target_date, outcome, now).await?;
        }

        // 8. 重新查询任务并组装 TaskCard
        let updated_task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        let mut task_card = TaskAssembler::task_to_card_basic(&updated_task);

        // 9. ✅ 在事务内填充 schedules 字段
        // ⚠️ 必须在写入 SSE 之前填充，确保 SSE 和 HTTP 返回的数据一致！
        task_card.schedules = TaskAssembler::assemble_schedules_in_tx(&mut tx, task_id).await?;

        // 9.5. ✅ 根据 schedules 设置正确的 schedule_status
        // staging 定义：今天和未来没有排期的任务，过去的排期不影响
        use crate::entities::ScheduleStatus;
        // ✅ 使用本地时间确定"今天"的日期，避免时区问题
        let local_today = chrono::Local::now().date_naive();

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

        // 10. 构建统一的事务结果
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

        // 11. 写入领域事件到 outbox
        use crate::infra::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        {
            // ✅ 使用统一的事务结果作为事件载荷
            let payload = serde_json::to_value(&transaction_result)?;

            let mut event = DomainEvent::new(
                "task.schedule_updated",
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

        // 12. 提交事务
        TransactionHelper::commit(tx).await?;

        // 13. 返回结果
        // ✅ HTTP 响应与 SSE 事件载荷完全一致
        Ok(UpdateScheduleResponse {
            result: transaction_result,
        })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use sqlx::{Sqlite, Transaction};

    /// 查找任务在指定日期的所有 floating 时间片（只有 floating 类型可以被删除）
    pub async fn find_floating_time_blocks_for_day(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        scheduled_date: &str, // YYYY-MM-DD 字符串
    ) -> AppResult<Vec<TimeBlock>> {
        // 🔥 正确的查询：先获取所有浮动时间片，然后在代码中按本地日期过滤
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
              AND tb.time_type = 'FLOATING'
              AND tb.is_deleted = false
        "#;

        let rows = sqlx::query_as::<_, crate::entities::TimeBlockRow>(query)
            .bind(task_id.to_string())
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        let mut time_blocks = Vec::new();

        // 🔥 在代码中按本地日期过滤（与 TaskAssembler 相同的逻辑）
        for row in rows {
            let time_block = TimeBlock::try_from(row)
                .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::QueryError(e)))?;

            // 🔥 使用系统本地时区转换 UTC 时间到本地日期
            use chrono::Local;
            let local_start = time_block.start_time.with_timezone(&Local);
            let formatted_date = crate::infra::core::utils::time_utils::format_date_yyyy_mm_dd(
                &local_start.date_naive(),
            );

            // 只保留匹配日期的时间片
            if formatted_date == scheduled_date {
                time_blocks.push(time_block);
            }
        }

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

    /// 更新日程的日期
    pub async fn update_schedule_date(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        original_date: &str, // YYYY-MM-DD 字符串
        new_date: &str,      // YYYY-MM-DD 字符串
        updated_at: chrono::DateTime<Utc>,
    ) -> AppResult<()> {
        let query = r#"
            UPDATE task_schedules
            SET scheduled_date = ?, updated_at = ?
            WHERE task_id = ? AND scheduled_date = ?
        "#;

        sqlx::query(query)
            .bind(new_date)
            .bind(updated_at.to_rfc3339())
            .bind(task_id.to_string())
            .bind(original_date)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(())
    }

    /// 更新日程的 outcome
    pub async fn update_schedule_outcome(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        scheduled_date: &str, // YYYY-MM-DD 字符串
        outcome: Outcome,
        updated_at: chrono::DateTime<Utc>,
    ) -> AppResult<()> {
        let outcome_str = match outcome {
            Outcome::Planned => "PLANNED",
            Outcome::PresenceLogged => "PRESENCE_LOGGED",
            Outcome::CompletedOnDay => "COMPLETED_ON_DAY",
            Outcome::CarriedOver => "CARRIED_OVER",
        };

        let query = r#"
            UPDATE task_schedules
            SET outcome = ?, updated_at = ?
            WHERE task_id = ? AND scheduled_date = ?
        "#;

        sqlx::query(query)
            .bind(outcome_str)
            .bind(updated_at.to_rfc3339())
            .bind(task_id.to_string())
            .bind(scheduled_date)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(())
    }
}
