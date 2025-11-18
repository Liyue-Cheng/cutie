/// 创建任务并添加日程 API - 单文件组件
///
/// POST /api/tasks/with-schedule
///
/// 这个端点将创建任务和添加日程合并为一个原子操作，
/// 避免前端发送两个请求，提高性能和用户体验。
use axum::{
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};

use crate::{
    entities::{ScheduleStatus, Task, TaskCardDto},
    features::shared::{
        repositories::TaskRepository, repositories::TaskScheduleRepository, TaskAssembler,
    },
    infra::{
        core::{AppError, AppResult},
        http::{error_handler::created_response, extractors::extract_correlation_id},
    },
    startup::AppState,
};
use serde::Deserialize;

// ==================== 文档层 ====================
/*
CABC for `create_task_with_schedule`

## 1. 端点签名 (Endpoint Signature)

POST /api/tasks/with-schedule

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我在日期视图中快速添加任务时，
> 我希望系统能够一次性创建任务并安排到指定日期，
> 而不需要等待两次网络请求。

### 2.2. 核心业务逻辑 (Core Business Logic)

在一个事务中完成两个操作：
1. 创建新任务（类似 POST /api/tasks）
2. 为新任务添加日程安排（类似 POST /api/tasks/:id/schedules）

返回的任务直接带有 schedule_status = "scheduled" 和对应的 schedules 数组。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**请求体 (Request Body):** `application/json`

```json
{
  "title": "string (required, 1-255 chars)",
  "scheduled_day": "string (YYYY-MM-DD, required)",
  "glance_note": "string | null (optional)",
  "detail_note": "string | null (optional)",
  "estimated_duration": "number | null (optional, 分钟数，0-10080)",
  "area_id": "string (UUID) | null (optional)",
  "due_date": "string (YYYY-MM-DD) | null (optional)",
  "due_date_type": "'soft' | 'hard' | null (optional)",
  "subtasks": "array | null (optional, 最多50个)"
}
```

**请求头 (Request Headers):**
- `X-Correlation-ID` (optional): 用于前端去重和请求追踪

### 3.2. 响应 (Responses)

**201 Created:**

*   **Content-Type:** `application/json`
*   **Schema:** `TaskCardDto`

```json
{
  "id": "uuid",
  "title": "string",
  "glance_note": "string | null",
  "schedule_status": "scheduled",
  "is_completed": false,
  "area": { "id": "uuid", "name": "string", "color": "string" } | null,
  "project_id": null,
  "subtasks": [...] | null,
  "schedules": [
    {
      "id": "uuid",
      "scheduled_day": "2025-10-05",
      "outcome": "PLANNED",
      "time_blocks": []
    }
  ],
  "due_date": {...} | null,
  "has_detail_note": boolean
}
```

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "title", "code": "TITLE_EMPTY", "message": "任务标题不能为空" }
  ]
}
```

## 4. 验证规则 (Validation Rules)

- `title`:
    - **必须**存在。
    - **必须**为非空字符串 (trim后)。
    - 长度**必须**小于等于 255 个字符。
    - 违反时返回错误码：`TITLE_EMPTY` 或 `TITLE_TOO_LONG`
- `scheduled_day`:
    - **必须**存在。
    - **必须**符合 `YYYY-MM-DD` 格式。
    - 违反时返回错误码：`INVALID_DATE_FORMAT`
- `estimated_duration`:
    - 如果提供，**必须**是大于等于 0 的整数。
    - 如果提供，**必须**小于等于 10080 (7天 = 7*24*60 分钟)。
    - 违反时返回错误码：`DURATION_NEGATIVE` 或 `DURATION_TOO_LONG`
- `subtasks`:
    - 如果提供，数组长度**必须**小于等于 50。
    - 违反时返回错误码：`TOO_MANY_SUBTASKS`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `validation::validate_request` 验证请求体（包括任务字段和日期格式）。
2.  获取写入许可（`app_state.acquire_write_permit()`），确保写操作串行执行。
3.  启动数据库事务（`TransactionHelper::begin`）。
4.  通过 `IdGenerator` 生成新的 `task_id`（UUID）。
5.  通过 `Clock` 服务获取当前时间 `now`。
6.  构造 `Task` 领域实体对象（和 create_task 相同）。
7.  调用 `TaskRepository::insert_in_tx` 持久化任务到 `tasks` 表。
8.  调用 `TaskScheduleRepository::create_in_tx` 创建日程记录。
9.  调用 `TaskAssembler::task_to_card_basic` 组装 `TaskCardDto`。
10. 在事务内填充 `schedules` 字段（`TaskAssembler::assemble_schedules_in_tx`）。
11. 设置 `task_card.schedule_status = Scheduled`（因为已有日程）。
12. 写入领域事件到 outbox（`task.created_with_schedule` 事件）。
13. 提交数据库事务（`TransactionHelper::commit`）。
14. 返回 `201 Created` 和组装好的 `TaskCardDto`。

## 6. 边界情况 (Edge Cases)

- **`title` 为空或全空格:** 返回 `422` 错误，错误码 `TITLE_EMPTY`。
- **`title` 超过 255 字符:** 返回 `422` 错误，错误码 `TITLE_TOO_LONG`。
- **`scheduled_day` 格式错误:** 返回 `422` 错误，错误码 `INVALID_DATE_FORMAT`。
- **`estimated_duration` 为负数:** 返回 `422` 错误，错误码 `DURATION_NEGATIVE`。
- **`estimated_duration` 超过 10080:** 返回 `422` 错误，错误码 `DURATION_TOO_LONG`。
- **`subtasks` 超过 50 个:** 返回 `422` 错误，错误码 `TOO_MANY_SUBTASKS`。
- **并发创建:** 使用写入许可确保写操作串行执行，避免并发问题。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`INSERT`:** 1条记录到 `tasks` 表。
    - **`INSERT`:** 1条记录到 `task_schedules` 表。
    - **`SELECT`:** 1次查询 `task_schedules` 表（填充 schedules）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `task.created_with_schedule` 事件，包含：
        - 创建的任务（`TaskCardDto`）
        - 日程日期（`scheduled_day`）
- **日志记录:**
    - 成功时，以 `INFO` 级别记录 "Task created with schedule" 及任务ID。
    - 失败时（验证失败或数据库错误），以 `WARN` 或 `ERROR` 级别记录详细错误信息。

*/

// ==================== 请求/响应结构体 ====================
#[derive(Debug, Deserialize)]
pub struct CreateTaskWithScheduleRequest {
    /// 任务标题
    pub title: String,
    /// 安排日期（YYYY-MM-DD 格式）
    pub scheduled_day: String,
    /// 速览备注
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glance_note: Option<String>,
    /// 详细备注
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail_note: Option<String>,
    /// 预估时长（分钟）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_duration: Option<i32>,
    /// 区域ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub area_id: Option<uuid::Uuid>,
    /// 截止日期（YYYY-MM-DD 格式）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<chrono::NaiveDate>,
    /// 截止日期类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date_type: Option<crate::entities::DueDateType>,
    /// 子任务列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtasks: Option<Vec<crate::entities::Subtask>>,
    /// 项目ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<uuid::Uuid>,
    /// 项目章节ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_id: Option<uuid::Uuid>,
}

// ==================== HTTP 处理器 ====================
/// 创建任务并添加日程的 HTTP 处理器
pub async fn handle(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<CreateTaskWithScheduleRequest>,
) -> Response {
    let correlation_id = extract_correlation_id(&headers);
    match logic::execute(&app_state, request, correlation_id).await {
        Ok(task_card) => created_response(task_card).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate_request(request: &CreateTaskWithScheduleRequest) -> AppResult<()> {
        // 验证标题
        if request.title.trim().is_empty() {
            return Err(AppError::validation_error(
                "title",
                "任务标题不能为空",
                "TITLE_EMPTY",
            ));
        }

        if request.title.len() > 255 {
            return Err(AppError::validation_error(
                "title",
                "任务标题不能超过255个字符",
                "TITLE_TOO_LONG",
            ));
        }

        // 验证日期格式
        parse_date(&request.scheduled_day)?;

        // 验证预估时长
        if let Some(duration) = request.estimated_duration {
            if duration < 0 {
                return Err(AppError::validation_error(
                    "estimated_duration",
                    "预估时长不能为负数",
                    "DURATION_NEGATIVE",
                ));
            }
            if duration > 24 * 60 * 7 {
                return Err(AppError::validation_error(
                    "estimated_duration",
                    "预估时长不能超过一周",
                    "DURATION_TOO_LONG",
                ));
            }
        }

        // 验证子任务数量
        if let Some(subtasks) = &request.subtasks {
            if subtasks.len() > 50 {
                return Err(AppError::validation_error(
                    "subtasks",
                    "子任务数量不能超过50个",
                    "TOO_MANY_SUBTASKS",
                ));
            }
        }

        Ok(())
    }

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
        request: CreateTaskWithScheduleRequest,
        correlation_id: Option<String>,
    ) -> AppResult<TaskCardDto> {
        // 1. 验证请求
        validation::validate_request(&request)?;

        // 2. 解析日期
        let scheduled_day = validation::parse_date(&request.scheduled_day)?;

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 3. 开始事务（✅ 使用 TransactionHelper）
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 4. 生成 UUID 和时间戳
        let task_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // 5. 创建任务实体
        let task = Task {
            id: task_id,
            title: request.title.clone(),
            glance_note: request.glance_note.clone(),
            detail_note: request.detail_note.clone(),
            estimated_duration: request.estimated_duration,
            subtasks: request.subtasks.clone(),
            project_id: request.project_id,
            section_id: request.section_id,
            area_id: request.area_id,
            due_date: request.due_date,
            due_date_type: request.due_date_type.clone(),
            completed_at: None,
            archived_at: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            source_info: None,
            external_source_id: None,
            external_source_provider: None,
            external_source_metadata: None,
            recurrence_id: None,
            recurrence_original_date: None,
        };

        // 6. 插入任务到数据库（✅ 使用共享 Repository）
        TaskRepository::insert_in_tx(&mut tx, &task).await?;

        // 7. 创建日程记录
        TaskScheduleRepository::create_in_tx(&mut tx, task_id, &scheduled_day).await?;

        // 8. 组装返回的 TaskCardDto
        let mut task_card = TaskAssembler::task_to_card_basic(&task);

        // 9. ✅ 在事务内填充 schedules 字段
        task_card.schedules = TaskAssembler::assemble_schedules_in_tx(&mut tx, task_id).await?;

        // 10. ✅ 根据 schedules 设置正确的 schedule_status
        // staging 定义：今天和未来没有排期的任务，过去的排期不影响
        use chrono::Utc;
        let today = Utc::now().date_naive();

        let has_future_schedule = task_card
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

        task_card.schedule_status = if has_future_schedule {
            ScheduleStatus::Scheduled
        } else {
            ScheduleStatus::Staging
        };

        // 11. 写入领域事件到 outbox
        use crate::infra::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        let payload = serde_json::json!({
            "task": task_card,
            "scheduled_day": scheduled_day,
        });

        let mut event = DomainEvent::new(
            "task.created_with_schedule",
            "task",
            task_id.to_string(),
            payload,
        )
        .with_aggregate_version(now.timestamp_millis());

        if let Some(cid) = correlation_id {
            event = event.with_correlation_id(cid);
        }

        outbox_repo.append_in_tx(&mut tx, &event).await?;

        // 12. 提交事务（✅ 使用 TransactionHelper）
        TransactionHelper::commit(tx).await?;

        tracing::info!(
            "Task created with schedule successfully: task_id={}, scheduled_day={}",
            task_id,
            scheduled_day
        );

        // 13. ✅ 异步 AI 自动分类（不阻塞返回）
        // 条件：未指定 area_id 且不是从模板创建
        if task.area_id.is_none() && task.source_info.is_none() {
            let task_id = task.id;
            let task_title = task.title.clone();
            let pool = app_state.db_pool().clone();

            tracing::debug!(
                target: "SERVICE:TASKS:create_task_with_schedule",
                task_id = %task_id,
                "Spawning AI classification task"
            );

            // 异步任务：不阻塞当前请求
            tokio::spawn(async move {
                use crate::features::shared::AiClassificationService;

                if let Err(e) =
                    AiClassificationService::classify_and_update_task(task_id, &task_title, &pool)
                        .await
                {
                    tracing::error!(
                        target: "SERVICE:TASKS:auto_classify",
                        task_id = %task_id,
                        error = %e,
                        "Failed to auto-classify task"
                    );
                }
            });
        }

        Ok(task_card)
    }
}

// ==================== 数据访问层 ====================
// ✅ 已迁移到共享 Repository：
// - TaskRepository::insert_in_tx
// - TaskScheduleRepository::create_in_tx
