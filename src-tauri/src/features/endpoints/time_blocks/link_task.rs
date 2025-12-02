/// 链接任务到时间块 API - 单文件组件
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::TaskCardDto,
    features::{
        shared::repositories::TimeBlockRepository,
        shared::TransactionHelper,
        shared::{
            repositories::{TaskRepository, TaskScheduleRepository, TaskTimeBlockLinkRepository},
            TaskAssembler,
        },
    },
    infra::{
        core::{utils::time_utils, AppError, AppResult},
        http::{error_handler::success_response, extractors::extract_correlation_id},
    },
    startup::AppState,
};

/// 链接任务到时间块的请求
#[derive(Debug, Deserialize)]
pub struct LinkTaskRequest {
    pub task_id: Uuid,
}

/// 链接任务到时间块的响应
#[derive(Debug, Serialize)]
pub struct LinkTaskResponse {
    pub task: TaskCardDto,
    pub time_block_id: Uuid,
}

// ==================== 文档层 ====================
/*
CABC for `link_task`

## 1. 端点签名 (Endpoint Signature)

POST /api/time-blocks/{block_id}/link-task

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要将一个任务与日历上已有的时间块关联起来，
> 而不是创建新的时间块，这样我可以将多个任务安排到同一个时间段内。

### 2.2. 核心业务逻辑 (Core Business Logic)

1. 验证时间块和任务都存在。
2. 建立任务与时间块的链接关系。
3. 如果任务在该天没有 schedule 记录，创建该天的日程记录。
4. 返回更新后的完整 TaskCardDto（包含新的 schedules、时间块信息）。
5. 发布 SSE 事件通知前端更新 UI。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**路径参数:**
- `block_id`: UUID - 时间块的ID

**请求体 (Request Body):** `application/json`

```json
{
  "task_id": "uuid"
}
```

**请求头 (Request Headers):**
- `X-Correlation-ID` (optional): 用于前端去重和请求追踪

### 3.2. 响应 (Responses)

**200 OK:**

```json
{
  "task": {
    "id": "uuid",
    "title": "string",
    "schedule_status": "scheduled",
    "schedules": [
      {
        "scheduled_day": "2025-10-07",
        "outcome": null,
        "time_blocks": [
          {
            "id": "uuid",
            "start_time": "2025-10-07T10:00:00Z",
            "end_time": "2025-10-07T11:00:00Z",
            "start_time_local": "10:00:00",
            "end_time_local": "11:00:00",
            "time_type": "FLOATING",
            "creation_timezone": "Asia/Shanghai",
            "is_all_day": false,
            "title": "任务标题",
            "glance_note": null
          }
        ]
      }
    ],
    // ... 其他 TaskCard 字段
  },
  "time_block_id": "uuid"
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "时间块不存在" 或 "任务不存在"
}
```

## 4. 验证规则 (Validation Rules)

- `block_id`:
    - **必须**存在于数据库中且未被软删除。
    - 违反时返回错误码：`NOT_FOUND`
- `task_id`:
    - **必须**存在于数据库中且未被软删除。
    - 违反时返回错误码：`NOT_FOUND`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1. 获取写入许可。
2. 启动数据库事务。
3. 验证时间块存在（`TimeBlockRepository::find_by_id_in_tx`）。
4. 验证任务存在（`TaskRepository::find_by_id_in_tx`）。
5. 检查链接是否已存在（幂等性）。
6. 如果不存在，创建链接（`TaskTimeBlockLinkRepository::link_in_tx`）。
7. 从时间块的 start_time 提取本地日期作为 scheduled_day。
8. 检查该任务在该天是否有 schedule 记录。
9. 如果没有，创建该天的 schedule 记录（`TaskScheduleRepository::create_in_tx`）。
10. 提交事务。
11. 重新查询任务并组装完整的 TaskCardDto（包含 schedules、area 等）。
12. 发布 SSE 事件 `time_blocks.linked`。
13. 返回响应。

## 6. 边界情况 (Edge Cases)

- **时间块不存在:** 返回 `404` 错误。
- **任务不存在:** 返回 `404` 错误。
- **链接已存在:** 幂等，直接返回当前状态（不报错）。
- **任务当天已有 schedule:** 不重复创建。
- **幂等性:** 相同参数重复调用，结果一致。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`INSERT`:** 0-1 条记录到 `task_time_block_links` 表（若不存在）。
    - **`INSERT`:** 0-1 条记录到 `task_schedules` 表（若该天无 schedule）。
    - **`INSERT`:** 1 条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `time_blocks.linked` 事件，包含：
        - `time_block_id`: 时间块ID
        - `linked_task_id`: 被链接的任务ID
        - `affected_task_ids`: 受影响的任务ID列表（包含被链接的任务）
- **日志记录:**
    - 成功时，记录链接的 task_id 和 block_id。
    - 失败时，记录详细错误信息。

*（无其他已知副作用）*
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(block_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<LinkTaskRequest>,
) -> Response {
    let correlation_id = extract_correlation_id(&headers);
    match logic::execute(&app_state, block_id, request, correlation_id).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        block_id: Uuid,
        request: LinkTaskRequest,
        correlation_id: Option<String>,
    ) -> AppResult<LinkTaskResponse> {
        let task_id = request.task_id;
        let now = app_state.clock().now_utc();

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 1. 开启事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 2. 验证时间块存在
        let time_block = TimeBlockRepository::find_by_id_in_tx(&mut tx, block_id).await?;

        // 3. 验证任务存在
        let task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 4. 检查链接是否已存在（幂等性）
        let link_exists = database::check_link_exists_in_tx(&mut tx, task_id, block_id).await?;

        // 5. 如果不存在，创建链接
        if !link_exists {
            TaskTimeBlockLinkRepository::link_in_tx(&mut tx, task_id, block_id).await?;
            tracing::info!(
                "Created link between task {} and time block {}",
                task_id,
                block_id
            );
        } else {
            tracing::info!(
                "Link already exists between task {} and time block {}",
                task_id,
                block_id
            );
        }

        // 5.5. 如果时间块没有 area，则继承任务的 area
        let should_update_area = time_block.area_id.is_none() && task.area_id.is_some();
        if should_update_area {
            let update_request = crate::entities::UpdateTimeBlockRequest {
                title: None,
                glance_note: None,
                detail_note: None,
                start_time: None,
                end_time: None,
                start_time_local: None,
                end_time_local: None,
                time_type: None,
                creation_timezone: None,
                is_all_day: None,
                area_id: Some(task.area_id),
            };
            TimeBlockRepository::update_in_tx(&mut tx, block_id, &update_request, now).await?;
            tracing::info!(
                "Updated time block {} area_id to {:?} (inherited from task)",
                block_id,
                task.area_id
            );
        }

        // 6. 从时间块的 start_time 提取日期字符串 - 使用系统本地时区
        use chrono::Local;
        let local_start = time_block.start_time.with_timezone(&Local);
        let scheduled_date = time_utils::format_date_yyyy_mm_dd(&local_start.date_naive());

        // 7. 检查该任务在该天是否有 schedule 记录
        let has_schedule =
            TaskScheduleRepository::has_schedule_for_day_in_tx(&mut tx, task_id, &scheduled_date)
                .await?;

        // 8. 如果没有，创建该天的 schedule 记录
        if !has_schedule {
            TaskScheduleRepository::create_in_tx(&mut tx, task_id, &scheduled_date).await?;
            tracing::info!(
                "Created schedule for task {} on day {}",
                task_id,
                scheduled_date
            );
        }

        // 9. 提交事务
        TransactionHelper::commit(tx).await?;

        // 10. 重新查询任务并组装完整的 TaskCardDto
        let pool = app_state.db_pool();

        let mut task_card = TaskAssembler::task_to_card_basic(&task);

        // ✅ 填充 schedules 字段（必须在 SSE 之前）
        task_card.schedules = TaskAssembler::assemble_schedules(pool, task_id).await?;
        // schedule_status 已删除 - 前端根据 schedules 字段实时计算

        // 11. 组装更新后的时间块数据（用于SSE事件）
        let updated_time_block = TimeBlockRepository::find_by_id(pool, block_id).await?;
        let time_block_view = crate::entities::TimeBlockViewDto {
            id: updated_time_block.id,
            start_time: updated_time_block.start_time,
            end_time: updated_time_block.end_time,
            start_time_local: updated_time_block.start_time_local,
            end_time_local: updated_time_block.end_time_local,
            time_type: updated_time_block.time_type,
            creation_timezone: updated_time_block.creation_timezone,
            is_all_day: updated_time_block.is_all_day,
            title: updated_time_block.title,
            glance_note: updated_time_block.glance_note,
            detail_note: updated_time_block.detail_note,
            area_id: updated_time_block.area_id, // ✅ 包含更新后的 area_id
            linked_tasks: vec![crate::entities::LinkedTaskSummary {
                id: task.id,
                title: task.title.clone(),
                is_completed: task.is_completed(),
            }],
            is_recurring: updated_time_block.recurrence_rule.is_some(),
            recurrence_id: None, // TODO: 从 time_block_recurrence_links 查询
            recurrence_original_date: updated_time_block.recurrence_original_date,
        };

        // 12. 发布 SSE 事件（包含完整的时间块数据和受影响的任务）
        use crate::infra::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };

        let mut outbox_tx = TransactionHelper::begin(app_state.db_pool()).await?;
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        let payload = serde_json::json!({
            "time_block_id": block_id,
            "linked_task_id": task_id,
            "affected_task_ids": vec![task_id],
            "affected_tasks": vec![task_card.clone()], // ✅ 克隆以便后续使用
            "time_block": time_block_view, // ✅ 包含完整数据
        });

        let mut event = DomainEvent::new(
            "time_blocks.linked",
            "TimeBlock",
            block_id.to_string(),
            payload,
        )
        .with_aggregate_version(now.timestamp_millis());

        // 关联 correlation_id（用于前端去重和请求追踪）
        if let Some(cid) = correlation_id {
            event = event.with_correlation_id(cid);
        }

        outbox_repo.append_in_tx(&mut outbox_tx, &event).await?;
        TransactionHelper::commit(outbox_tx).await?;

        // 12. 返回结果
        Ok(LinkTaskResponse {
            task: task_card,
            time_block_id: block_id,
        })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use sqlx::{Sqlite, Transaction};

    /// 检查链接是否已存在
    pub async fn check_link_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        block_id: Uuid,
    ) -> AppResult<bool> {
        let query = r#"
            SELECT COUNT(*) as count
            FROM task_time_block_links
            WHERE task_id = ? AND time_block_id = ?
        "#;

        let count: i64 = sqlx::query_scalar(query)
            .bind(task_id.to_string())
            .bind(block_id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
            })?;

        Ok(count > 0)
    }
}
