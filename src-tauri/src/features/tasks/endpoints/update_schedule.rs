/// 更新任务日程 API - 单文件组件
///
/// PATCH /api/tasks/:id/schedules/:date
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::{Outcome, TaskCardDto},
    features::tasks::shared::{
        repositories::{TaskRepository, TaskScheduleRepository},
        TaskAssembler,
    },
    shared::{
        core::{AppError, AppResult, utils::time_utils},
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
  "task_card": {
    "id": "uuid",
    "title": "string",
    "schedule_status": "scheduled",
    "schedules": [...],
    ...
  }
}
```

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

#[derive(Debug, Serialize)]
pub struct UpdateScheduleResponse {
    pub task_card: TaskCardDto,
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

    pub fn parse_date(date_str: &str) -> AppResult<chrono::DateTime<Utc>> {
        let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
            AppError::validation_error(
                "scheduled_day",
                "日期格式错误，请使用 YYYY-MM-DD 格式",
                "INVALID_DATE_FORMAT",
            )
        })?;

        // 🔧 FIX: 直接使用 NaiveDate 转换为 UTC 零点
        use crate::shared::core::utils::time_utils::local_date_to_utc_midnight;
        Ok(local_date_to_utc_midnight(naive_date))
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
            TaskScheduleRepository::has_schedule_for_day_in_tx(&mut tx, task_id, original_date)
                .await?;

        if !has_original_schedule {
            return Err(AppError::not_found(
                "Schedule",
                format!("Task {} on {}", task_id, date_str),
            ));
        }

        // 6. 处理更新逻辑
        if let Some(ref new_date_str) = request.new_date {
            // 解析新日期
            let new_date = validation::parse_date(new_date_str)?;

            // 检查新日期是否已有日程（如果不是同一天）
            if original_date.date_naive() != new_date.date_naive() {
                let has_new_date_schedule =
                    TaskScheduleRepository::has_schedule_for_day_in_tx(&mut tx, task_id, new_date)
                        .await?;

                if has_new_date_schedule {
                    return Err(AppError::conflict("目标日期已有日程安排"));
                }
            }

            // 更新日期
            database::update_schedule_date(&mut tx, task_id, original_date, new_date, now).await?;
        }

        // 7. 处理 outcome 更新
        if let Some(ref outcome_str) = request.outcome {
            let outcome = validation::parse_outcome(outcome_str)?;
            let target_date = if let Some(ref new_date_str) = request.new_date {
                validation::parse_date(new_date_str)?
            } else {
                original_date
            };
            database::update_schedule_outcome(&mut tx, task_id, target_date, outcome, now).await?;
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
        use chrono::Utc;
        let local_today = time_utils::extract_local_date_from_utc(Utc::now());

        let has_future_schedule = task_card.schedules.as_ref().map(|schedules| {
            schedules.iter().any(|s| {
                if let Ok(schedule_date) = chrono::NaiveDate::parse_from_str(&s.scheduled_day, "%Y-%m-%d") {
                    schedule_date >= local_today
                } else {
                    false
                }
            })
        }).unwrap_or(false);
        
        task_card.schedule_status = if has_future_schedule {
            ScheduleStatus::Scheduled
        } else {
            ScheduleStatus::Staging
        };

        // 10. 写入领域事件到 outbox
        use crate::shared::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        let payload = serde_json::json!({
            "task": task_card,
            "original_date": date_str,
            "new_date": request.new_date,
            "outcome": request.outcome,
        });

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

        // 11. 提交事务
        TransactionHelper::commit(tx).await?;

        // 12. 返回结果
        Ok(UpdateScheduleResponse { task_card })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use sqlx::{Sqlite, Transaction};

    /// 更新日程的日期
    pub async fn update_schedule_date(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        original_date: chrono::DateTime<Utc>,
        new_date: chrono::DateTime<Utc>,
        updated_at: chrono::DateTime<Utc>,
    ) -> AppResult<()> {
        let query = r#"
            UPDATE task_schedules
            SET scheduled_day = ?, updated_at = ?
            WHERE task_id = ? AND DATE(scheduled_day) = DATE(?)
        "#;

        sqlx::query(query)
            .bind(new_date.to_rfc3339())
            .bind(updated_at.to_rfc3339())
            .bind(task_id.to_string())
            .bind(original_date.to_rfc3339())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(())
    }

    /// 更新日程的 outcome
    pub async fn update_schedule_outcome(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        scheduled_day: chrono::DateTime<Utc>,
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
            WHERE task_id = ? AND DATE(scheduled_day) = DATE(?)
        "#;

        sqlx::query(query)
            .bind(outcome_str)
            .bind(updated_at.to_rfc3339())
            .bind(task_id.to_string())
            .bind(scheduled_day.to_rfc3339())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(())
    }
}
