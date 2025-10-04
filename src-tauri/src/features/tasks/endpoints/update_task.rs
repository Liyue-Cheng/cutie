/// 更新任务 API - 单文件组件
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    entities::{ScheduleStatus, TaskCardDto, UpdateTaskRequest},
    features::tasks::shared::{
        assemblers::TimeBlockAssembler,
        repositories::{TaskRepository, TaskScheduleRepository, TaskTimeBlockLinkRepository},
        TaskAssembler,
    },
    shared::{
        core::{AppError, AppResult},
        http::{error_handler::success_response, extractors::extract_correlation_id},
    },
    startup::AppState,
};

/// 更新任务的响应
#[derive(Debug, Serialize)]
pub struct UpdateTaskResponse {
    pub task: TaskCardDto,
    // 注意：副作用（updated time blocks）已通过 SSE 推送
}

// ==================== 文档层 ====================
/*
CABC for `update_task`

## 1. 端点签名 (Endpoint Signature)

PATCH /api/tasks/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要修改任务的标题、笔记、子任务等信息，
> 并且当我修改任务标题或 area 时，系统能自动同步更新相关的时间块，
> 以保持数据一致性。

### 2.2. 核心业务逻辑 (Core Business Logic)

更新任务的可变字段（标题、笔记、子任务、area 等）。
特殊业务逻辑：当标题或 area 有变更时，自动更新所有"唯一关联且自动创建"的时间块，
确保时间块的标题和 area 与任务保持一致。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**请求体 (Request Body):** `application/json`

所有字段都是可选的（部分更新）：

```json
{
  "title": "string | null (optional, 1-255 chars)",
  "glance_note": "string | null (optional, 支持置空)",
  "detail_note": "string | null (optional, 支持置空)",
  "estimated_duration": "number | null (optional, 0-10080)",
  "area_id": "UUID | null (optional, 支持置空)",
  "due_date": "string (YYYY-MM-DD) | null (optional)",
  "due_date_type": "'soft' | 'hard' | null (optional)",
  "subtasks": "array | null (optional, 最多50个, 支持置空)"
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
    "glance_note": "string | null",
    "schedule_status": "staging" | "scheduled",
    "is_completed": false,
    "area": {...} | null,
    "project_id": null,
    "subtasks": [...] | null,
    "schedules": [...] | null,
    "due_date": {...} | null,
    "has_detail_note": boolean
  }
}
```

**注意：** 副作用（更新的时间块）通过 SSE 事件推送，不在 HTTP 响应中包含。

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "任务不存在"
}
```

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [...]
}
```

## 4. 验证规则 (Validation Rules)

- `title`:
    - 如果提供，**必须**为非空字符串 (trim后)。
    - 如果提供，长度**必须**小于等于 255 个字符。
    - 违反时返回错误码：`TITLE_EMPTY` 或 `TITLE_TOO_LONG`
- `subtasks`:
    - 如果提供，数组长度**必须**小于等于 50。
    - 违反时返回错误码：`TOO_MANY_SUBTASKS`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `validation::validate_update_request` 验证请求体。
2.  获取当前时间 `now`。
3.  获取写入许可（`app_state.acquire_write_permit()`）。
4.  启动数据库事务（`TransactionHelper::begin`）。
5.  查询旧任务数据（`TaskRepository::find_by_id_in_tx`）。
6.  如果任务不存在，返回 404 错误。
7.  更新任务（`TaskRepository::update_in_tx`）。
8.  检查标题或 area 是否有变更。
9.  如果有变更，查询所有链接的时间块（`TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx`）。
10. 对每个时间块：
    - 检查是否是唯一关联（`is_exclusive_link_in_tx`）
    - 检查是否是自动创建的（标题与旧任务标题一致）
    - 如果是唯一关联且自动创建，更新时间块的标题和 area
11. 查询更新后的完整时间块数据（`TimeBlockAssembler::assemble_for_event_in_tx`）。
12. 重新查询任务并组装 `TaskCardDto`。
13. 在事务内填充 `schedules` 字段。
14. 根据 schedules 设置正确的 `schedule_status`。
15. 写入领域事件到 outbox（包含更新的任务和副作用的时间块）。
16. 提交事务（`TransactionHelper::commit`）。
17. 返回更新后的任务。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **`title` 为空或全空格:** 返回 `422` 错误，错误码 `TITLE_EMPTY`。
- **`title` 超过 255 字符:** 返回 `422` 错误，错误码 `TITLE_TOO_LONG`。
- **`subtasks` 超过 50 个:** 返回 `422` 错误，错误码 `TOO_MANY_SUBTASKS`。
- **时间块是手动创建的（标题与任务不一致）:** 不自动更新。
- **时间块关联多个任务:** 不自动更新（避免影响其他任务）。
- **幂等性:** 相同参数重复调用，结果一致，副作用只执行一次（通过 correlation_id 实现）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 查询旧任务、链接的时间块、排他性检查。
    - **`UPDATE`:** 1条记录在 `tasks` 表。
    - **`UPDATE`:** 0-N 条记录在 `time_blocks` 表（仅更新唯一关联且自动创建的时间块）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `task.updated` 事件，包含：
        - 更新后的任务（`TaskCardDto`）
        - 副作用：更新的时间块列表（`TimeBlockViewDto[]`）
- **日志记录:**
    - 成功时，记录更新的时间块 ID。
    - 失败时，记录详细错误信息。

*（无其他已知副作用）*
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<UpdateTaskRequest>,
) -> Response {
    let correlation_id = extract_correlation_id(&headers);
    match logic::execute(&app_state, task_id, request, correlation_id).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate_update_request(request: &UpdateTaskRequest) -> AppResult<()> {
        tracing::trace!("Entering validation::validate_update_request");
        println!("Entering validation::validate_update_request");
        // 检查是否为空更新
        // if request.is_empty() {
        //     return Err(AppError::validation_error(
        //         "request",
        //         "至少需要更新一个字段",
        //         "EMPTY_UPDATE",
        //     ));
        // }

        // 验证标题
        if let Some(title) = &request.title {
            if title.trim().is_empty() {
                return Err(AppError::validation_error(
                    "title",
                    "标题不能为空",
                    "TITLE_EMPTY",
                ));
            }
            if title.len() > 255 {
                return Err(AppError::validation_error(
                    "title",
                    "标题不能超过255个字符",
                    "TITLE_TOO_LONG",
                ));
            }
        }

        // 验证子任务数量
        if let Some(Some(subtasks)) = &request.subtasks {
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
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;
    use crate::features::shared::TransactionHelper;

    pub async fn execute(
        app_state: &AppState,
        task_id: Uuid,
        request: UpdateTaskRequest,
        correlation_id: Option<String>,
    ) -> AppResult<UpdateTaskResponse> {
        // 1. 验证
        validation::validate_update_request(&request)?;
        println!("Exiting validation::validate_update_request");

        let now = app_state.clock().now_utc();

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 2. 开启事务（✅ 使用 TransactionHelper）
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 3. 查询旧任务数据（✅ 使用共享 Repository）
        let old_task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 4. 更新任务（✅ 使用共享 Repository）
        TaskRepository::update_in_tx(&mut tx, task_id, &request).await?;

        // 5. 检查标题或 area 是否有变更
        let title_changed =
            request.title.is_some() && request.title.as_ref() != Some(&old_task.title);
        let area_changed = request.area_id.is_some() && request.area_id != Some(old_task.area_id);

        // 6. 如果标题或 area 有变更，更新唯一关联的时间块（✅ 使用共享 Repository）
        let mut updated_time_block_ids = Vec::new();
        if title_changed || area_changed {
            let linked_blocks =
                TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx(&mut tx, task_id)
                    .await?;

            for block in linked_blocks {
                // 检查是否是唯一关联（✅ 使用共享 Repository）
                let is_exclusive = TaskTimeBlockLinkRepository::is_exclusive_link_in_tx(
                    &mut tx, block.id, task_id,
                )
                .await?;
                if !is_exclusive {
                    continue;
                }

                // 检查标题是否一致（自动创建的时间块）
                let is_auto_created = block
                    .title
                    .as_ref()
                    .map(|t| t == &old_task.title)
                    .unwrap_or(false);

                if !is_auto_created {
                    // 手动创建的时间块，不自动更新
                    continue;
                }

                // 更新时间块的标题和 area（✅ 调用数据访问层）
                database::update_time_block_title_and_area_in_tx(
                    &mut tx,
                    block.id,
                    request.title.as_deref(),
                    request.area_id,
                    now,
                )
                .await?;

                updated_time_block_ids.push(block.id);
                tracing::info!(
                    "Updated exclusive time block {} for task {}",
                    block.id,
                    task_id
                );
            }
        }

        // 7. 查询更新后的完整时间块数据（✅ 使用共享装配器）
        let updated_blocks =
            TimeBlockAssembler::assemble_for_event_in_tx(&mut tx, &updated_time_block_ids).await?;

        // 8. 重新查询任务以获取最新数据（✅ 使用共享 Repository）
        let task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 9. 组装 TaskCardDto（用于事件载荷）
        let mut task_card_for_event = TaskAssembler::task_to_card_basic(&task);

        // 10. ✅ 在事务内填充 schedules 字段
        // ⚠️ 必须在写入 SSE 之前填充，确保 SSE 和 HTTP 返回的数据一致！
        task_card_for_event.schedules =
            TaskAssembler::assemble_schedules_in_tx(&mut tx, task_id).await?;

        // 9.1. 根据 schedules 设置正确的 schedule_status
        // staging 定义：今天和未来没有排期的任务，过去的排期不影响
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

        // 11. 在事务中写入领域事件到 outbox
        use crate::shared::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        {
            let payload = serde_json::json!({
                "task": task_card_for_event,
                "side_effects": {
                    "updated_time_blocks": updated_blocks,
                }
            });
            let mut event = DomainEvent::new("task.updated", "task", task_id.to_string(), payload)
                .with_aggregate_version(now.timestamp_millis());

            // 关联 correlation_id（用于前端去重和请求追踪）
            if let Some(cid) = correlation_id {
                event = event.with_correlation_id(cid);
            }

            outbox_repo.append_in_tx(&mut tx, &event).await?;
        }

        // 12. 提交事务（✅ 使用 TransactionHelper）
        TransactionHelper::commit(tx).await?;

        // 13. 返回结果（复用事件中的 task_card）
        // HTTP 响应与 SSE 事件载荷保持一致
        Ok(UpdateTaskResponse {
            task: task_card_for_event,
        })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use chrono::{DateTime, Utc};
    use sqlx::{Sqlite, Transaction};

    /// 更新时间块的标题和 area（仅用于任务更新时的联动更新）
    pub async fn update_time_block_title_and_area_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        block_id: Uuid,
        new_title: Option<&str>,
        new_area_id: Option<Option<Uuid>>, // None: 不更新; Some(None): 置 NULL; Some(Some(id)): 设置
        now: DateTime<Utc>,
    ) -> AppResult<()> {
        let mut set_clauses = Vec::new();
        let mut binds: Vec<String> = Vec::new();

        if let Some(title) = new_title {
            set_clauses.push("title = ?");
            binds.push(title.to_string());
        }

        if let Some(area_opt) = new_area_id {
            set_clauses.push("area_id = ?");
            binds.push(area_opt.map(|id| id.to_string()).unwrap_or_default());
        }

        if set_clauses.is_empty() {
            return Ok(()); // 没有需要更新的字段
        }

        set_clauses.push("updated_at = ?");
        let update_clause = set_clauses.join(", ");
        let query = format!("UPDATE time_blocks SET {} WHERE id = ?", update_clause);

        let mut q = sqlx::query(&query);
        for bind in binds {
            q = q.bind(bind);
        }
        q = q.bind(now.to_rfc3339());
        q = q.bind(block_id.to_string());

        q.execute(&mut **tx).await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        Ok(())
    }
}

// ✅ 已迁移到共享 Repository：
// - TaskRepository::find_by_id_in_tx, update_in_tx
// - TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx, is_exclusive_link_in_tx
// - TaskScheduleRepository::has_any_schedule
// - TimeBlockAssembler::assemble_for_event_in_tx
