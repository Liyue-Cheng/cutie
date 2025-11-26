/// 添加任务日程 API - 单文件组件
///
/// POST /api/tasks/:id/schedules
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
    entities::{SideEffects, TaskTransactionResult},
    features::shared::{
        repositories::{TaskRepository, TaskScheduleRepository},
        TaskAssembler,
    },
    infra::{
        core::{AppError, AppResult},
        http::{error_handler::created_response, extractors::extract_correlation_id},
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `add_schedule`

## 1. 端点签名 (Endpoint Signature)

POST /api/tasks/{id}/schedules

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要为任务添加日程安排，指定任务在某天需要完成，
> 以便我能更好地规划我的每日工作。

### 2.2. 核心业务逻辑 (Core Business Logic)

为任务添加日程记录到 `task_schedules` 表，初始 `outcome` 为 `PLANNED`。
如果这是任务的第一个日程，任务的 `schedule_status` 会从 `Staging` 变为 `Scheduled`。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 任务ID

**请求体 (Request Body):** `application/json`

```json
{
  "scheduled_day": "string (YYYY-MM-DD, required)"
}
```

**请求头 (Request Headers):**
- `X-Correlation-ID` (optional): 用于前端去重和请求追踪

### 3.2. 响应 (Responses)

**201 Created:**

*   **Content-Type:** `application/json`

```json
{
  "task_card": {
    "id": "uuid",
    "title": "string",
    "schedule_status": "scheduled",
    "schedules": [
      {
        "id": "uuid",
        "scheduled_day": "2025-10-05",
        "outcome": "PLANNED",
        "time_blocks": []
      }
    ],
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
  "message": "该日期已有日程安排"
}
```

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "scheduled_day", "code": "INVALID_DATE_FORMAT", "message": "日期格式错误，请使用 YYYY-MM-DD 格式" }
  ]
}
```

## 4. 验证规则 (Validation Rules)

- `scheduled_day`:
    - **必须**存在。
    - **必须**符合 `YYYY-MM-DD` 格式。
    - 违反时返回错误码：`INVALID_DATE_FORMAT`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  解析日期字符串为 `DateTime<Utc>`（`validation::parse_date`）。
2.  获取写入许可（`app_state.acquire_write_permit()`）。
3.  启动数据库事务（`TransactionHelper::begin`）。
4.  查询任务（`TaskRepository::find_by_id_in_tx`）。
5.  如果任务不存在，返回 404 错误。
6.  检查该日期是否已有日程（`TaskScheduleRepository::has_schedule_for_day_in_tx`）。
7.  如果已有日程，返回 409 冲突。
8.  创建日程记录（`TaskScheduleRepository::create_in_tx`，初始 `outcome = PLANNED`）。
9.  重新查询任务（`TaskRepository::find_by_id_in_tx`）。
10. 组装 `TaskCardDto`（`TaskAssembler::task_to_card_basic`）。
11. 在事务内填充 `schedules` 字段（`TaskAssembler::assemble_schedules_in_tx`）。
12. 根据 schedules 设置正确的 `schedule_status`（应为 `Scheduled`，因为刚添加了日程）。
13. 写入领域事件到 outbox（`task.scheduled` 事件）。
14. 提交事务（`TransactionHelper::commit`）。
15. 返回 `201 Created` 和更新后的任务。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **该日期已有日程:** 返回 `409` 冲突。
- **日期格式错误:** 返回 `422` 验证错误。
- **添加过去的日期:** 允许（系统不限制日期范围）。
- **添加未来很远的日期:** 允许（系统不限制日期范围）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次查询 `tasks` 表（验证任务存在）。
    - **`SELECT`:** 1次查询 `task_schedules` 表（检查日期冲突）。
    - **`INSERT`:** 1条记录到 `task_schedules` 表。
    - **`SELECT`:** 1次查询 `tasks` 表（重新获取数据）。
    - **`SELECT`:** 1次查询 `task_schedules` 表（填充 schedules）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `task.scheduled` 事件，包含：
        - 更新后的任务（`TaskCardDto`）
        - 新增的日期（`scheduled_day`）
- **日志记录:**
    - 成功时，记录日程创建信息。
    - 失败时，记录详细错误信息。

*（无其他已知副作用）*
*/

// ==================== 请求/响应结构体 ====================
#[derive(Debug, Deserialize)]
pub struct AddScheduleRequest {
    /// 安排日期（YYYY-MM-DD 格式）
    pub scheduled_day: String,
}

/// 添加日程的响应
/// ✅ HTTP 响应和 SSE 事件使用相同的数据结构
#[derive(Debug, Serialize)]
pub struct AddScheduleResponse {
    #[serde(flatten)]
    pub result: TaskTransactionResult,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<AddScheduleRequest>,
) -> Response {
    let correlation_id = extract_correlation_id(&headers);
    match logic::execute(&app_state, task_id, request, correlation_id).await {
        Ok(response) => created_response(response).into_response(),
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
        request: AddScheduleRequest,
        correlation_id: Option<String>,
    ) -> AppResult<AddScheduleResponse> {
        let now = app_state.clock().now_utc();

        // 1. 解析日期
        let scheduled_day = validation::parse_date(&request.scheduled_day)?;

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 2. 开始事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 3. 检查任务是否存在
        let _task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 4. 检查该日期是否已有日程
        let has_schedule =
            TaskScheduleRepository::has_schedule_for_day_in_tx(&mut tx, task_id, &scheduled_day)
                .await?;

        if has_schedule {
            return Err(AppError::conflict("该日期已有日程安排"));
        }

        // 5. 创建日程记录
        TaskScheduleRepository::create_in_tx(&mut tx, task_id, &scheduled_day).await?;

        // 6. 重新查询任务并组装 TaskCard
        // 注意：schedule_status 是派生字段，由装配器根据 task_schedules 表计算
        let updated_task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        let mut task_card = TaskAssembler::task_to_card_basic(&updated_task);

        // 7. ✅ 在事务内填充 schedules 字段
        // ⚠️ 必须在写入 SSE 之前填充，确保 SSE 和 HTTP 返回的数据一致！
        task_card.schedules = TaskAssembler::assemble_schedules_in_tx(&mut tx, task_id).await?;
        // schedule_status 已删除 - 前端根据 schedules 字段实时计算

        // 8. 构建统一的事务结果
        // ✅ HTTP 响应和 SSE 事件使用相同的数据结构
        let transaction_result = TaskTransactionResult {
            task: task_card,
            side_effects: SideEffects::empty(),
        };

        // 9. 写入领域事件到 outbox
        use crate::infra::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        {
            // ✅ 使用统一的事务结果作为事件载荷
            let payload = serde_json::to_value(&transaction_result)?;

            let mut event =
                DomainEvent::new("task.scheduled", "task", task_id.to_string(), payload)
                    .with_aggregate_version(now.timestamp_millis());

            if let Some(cid) = correlation_id {
                event = event.with_correlation_id(cid);
            }

            outbox_repo.append_in_tx(&mut tx, &event).await?;
        }

        // 10. 提交事务
        TransactionHelper::commit(tx).await?;

        // 11. 返回结果
        // ✅ HTTP 响应与 SSE 事件载荷完全一致
        Ok(AddScheduleResponse {
            result: transaction_result,
        })
    }
}

// ==================== 数据访问层 ====================
// ✅ 所有数据库操作已迁移到共享 Repository
// schedule_status 是派生字段，不存储在数据库中，由装配器根据 task_schedules 表计算
