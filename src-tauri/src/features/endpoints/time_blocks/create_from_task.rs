/// 从任务创建时间块 API - 单文件组件
///
/// 专门处理"拖动任务到日历"的场景
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    entities::{LinkedTaskSummary, TimeBlock, TimeBlockViewDto},
    features::{
        shared::{repositories::TimeBlockRepository, TimeBlockConflictChecker},
        shared::{
            repositories::{TaskRepository, TaskScheduleRepository, TaskTimeBlockLinkRepository},
            TaskAssembler,
        },
    },
    infra::{
        core::{AppError, AppResult},
        http::error_handler::created_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `create_time_block_from_task`

## 1. 端点签名 (Endpoint Signature)

POST /api/time-blocks/from-task

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我将一个任务拖动到日历的特定时间段时，
> 我希望系统能够：
> 1. 为这个任务创建一个时间块（分配具体的执行时间）
> 2. 自动创建任务的日程记录（标记任务在该日期有安排）
> 3. 更新任务的状态为"已排期"
> 4. 返回完整的任务信息，以便我能看到更新后的状态

### 2.2. 核心业务逻辑 (Core Business Logic)

这是专门为"拖动任务到日历"场景设计的端点，执行一系列原子操作：
1. 创建时间块（记录具体的执行时间段）
2. 建立任务与时间块的链接关系
3. 创建或更新任务的日程记录（task_schedules），标记任务在该日期有安排
4. 时间块的标题默认使用任务标题（可自定义）
5. 时间块的 area 继承任务的 area
6. 返回完整的时间块视图和更新后的任务卡片

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**请求体 (Request Body):** `application/json`

```json
{
  "task_id": "UUID (required)",
  "start_time": "string (ISO 8601 UTC, required)",
  "end_time": "string (ISO 8601 UTC, required)",
  "start_time_local": "string | null (optional, HH:MM:SS format, 本地开始时间)",
  "end_time_local": "string | null (optional, HH:MM:SS format, 本地结束时间)",
  "time_type": "string | null (optional, 'FLOATING' | 'FIXED', 默认 'FLOATING')",
  "creation_timezone": "string | null (optional, 创建时的时区，占位字段)",
  "title": "string | null (optional, 默认使用任务标题)",
  "is_all_day": "boolean | null (optional, 是否为全天事件)"
}
```

### 3.2. 响应 (Responses)

**201 Created:**

*   **Content-Type:** `application/json`

```json
{
  "time_block": {
    "id": "uuid",
    "start_time": "2025-10-05T14:00:00Z",
    "end_time": "2025-10-05T15:00:00Z",
    "start_time_local": "14:00:00",
    "end_time_local": "15:00:00",
    "time_type": "FLOATING",
    "creation_timezone": "Asia/Shanghai",
    "is_all_day": false,
    "title": "string",
    "glance_note": null,
    "detail_note": null,
    "area_id": "uuid | null",
    "linked_tasks": [
      {
        "id": "uuid",
        "title": "string",
        "is_completed": false
      }
    ],
    "is_recurring": false
  },
  "updated_task": {
    "id": "uuid",
    "title": "string",
    "schedule_status": "scheduled",
    "is_completed": false,
    "area": {...} | null,
    "schedules": [
      {
        "scheduled_day": "2025-10-05",
        "outcome": null
      }
    ],
    ...
  }
}
```

**400 Bad Request:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "开始时间必须早于结束时间",
  "details": [
    { "field": "time_range", "code": "INVALID_TIME_RANGE", "message": "开始时间必须早于结束时间" }
  ]
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Task not found: {task_id}"
}
```

**409 Conflict:**

```json
{
  "error_code": "CONFLICT",
  "message": "该时间段与现有时间块重叠"
}
```

## 4. 验证规则 (Validation Rules)

- `task_id`:
    - **必须**存在。
    - **必须**是有效的 UUID 格式。
    - 对应的任务**必须**存在于数据库中。
    - 违反时返回错误码：`NOT_FOUND`
- `start_time`:
    - **必须**存在。
    - **必须**是有效的 ISO 8601 UTC 时间格式。
    - **必须**早于 `end_time`。
    - 违反时返回错误码：`INVALID_TIME_RANGE`
- `end_time`:
    - **必须**存在。
    - **必须**是有效的 ISO 8601 UTC 时间格式。
    - **必须**晚于 `start_time`。
    - 违反时返回错误码：`INVALID_TIME_RANGE`
- **时间冲突验证**:
    - 新时间块的时间范围**不能**与现有时间块重叠。
    - 违反时返回错误码：`CONFLICT`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `validation::validate_request` 验证请求体。
2.  启动数据库事务（`app_state.db_pool().begin()`）。
3.  调用 `TaskRepository::find_by_id_in_tx` 查询任务：
    - 如果任务不存在，返回 404 错误
4.  调用 `TimeBlockConflictChecker::check_in_tx` 检查时间冲突：
    - 查询时间范围重叠的现有时间块
    - 如果存在重叠，返回 409 冲突错误
5.  通过 `IdGenerator` 生成新的 `block_id`（UUID）。
6.  通过 `Clock` 服务获取当前时间 `now`。
7.  确定时间块标题：使用请求中的自定义标题，如果没有则使用任务标题。
8.  构造 `TimeBlock` 领域实体对象：
    - 设置 `id`, `title`（来自请求或任务）
    - 设置 `start_time`, `end_time`
    - 设置 `area_id`（继承任务的 area）
    - 设置 `created_at = now`, `updated_at = now`
    - 设置 `deleted_at IS NULL`
9.  调用 `TimeBlockRepository::insert_in_tx` 持久化时间块。
10. 调用 `TaskTimeBlockLinkRepository::link_in_tx` 建立任务与时间块的链接。
11. 计算日程日期：
    - 使用 `utc_time_to_local_date_utc_midnight` 将 UTC 时间转换为本地日期的 UTC 零点
    - 例如：`2025-10-02T18:00:00Z (UTC)` → `2025-10-03T00:00:00Z`（如果在 UTC+8 时区）
12. 检查该日期是否已有日程记录（`TaskScheduleRepository::has_schedule_for_day_in_tx`）。
13. 如果没有日程记录，创建新的日程（`TaskScheduleRepository::create_in_tx`）。
14. 提交数据库事务。
15. 组装返回的 `TimeBlockViewDto`：
    - 填充所有基础字段
    - 填充 `linked_tasks`（包含任务摘要）
16. 组装返回的 `TaskCardDto`：
    - 调用 `TaskAssembler::task_to_card_basic` 创建基础卡片
    - 设置 `schedule_status = Scheduled`
    - 填充 `schedules` 字段（包含新创建的日程）
17. 返回 `201 Created` 和包含时间块与任务的响应对象。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **`start_time >= end_time`:** 返回 `400` 错误，错误码 `INVALID_TIME_RANGE`。
- **时间范围与现有时间块重叠:** 返回 `409` 错误，错误码 `CONFLICT`。
- **该日期已有日程记录:** 不重复创建，保持幂等性。
- **跨时区的时间处理:** 使用系统时区正确计算日程日期（例如：UTC 晚上 10 点在 UTC+8 时区算第二天）。
- **任务已完成:** 当前实现允许为已完成的任务创建时间块（未来可能需要限制）。
- **并发创建:** 事务隔离保证数据一致性。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次，查询任务是否存在。
    - **`SELECT`:** 1次，查询重叠的时间块（冲突检测）。
    - **`SELECT`:** 1次，检查日程是否已存在。
    - **`INSERT`:** 1条记录到 `time_blocks` 表。
    - **`INSERT`:** 1条记录到 `task_time_block_links` 表。
    - **`INSERT`:** 0-1条记录到 `task_schedules` 表（如果该日期尚无日程）。
    - **`SELECT`:** 1次，查询任务的完整日程列表（用于返回）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **日志记录:**
    - 记录时间块创建和日程创建的详细信息（包含时间转换日志）。
    - 失败时，记录详细错误信息。

*（无其他已知副作用，不发送 SSE 事件）*
*/

// ==================== 请求/响应结构 ====================
#[derive(Debug, Deserialize)]
pub struct CreateFromTaskRequest {
    pub task_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub start_time_local: Option<String>, // 本地开始时间 (HH:MM:SS)
    pub end_time_local: Option<String>,   // 本地结束时间 (HH:MM:SS)
    pub time_type: Option<crate::entities::time_block::TimeType>, // 时间类型
    pub creation_timezone: Option<String>, // 创建时的时区
    pub title: Option<String>,            // 可选，默认使用任务标题
    pub is_all_day: Option<bool>,         // 可选，支持在日历全天槽位创建全天事件
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(request): Json<CreateFromTaskRequest>,
) -> Response {
    let correlation_id = crate::infra::http::extract_correlation_id(&headers);
    match logic::execute(&app_state, request, correlation_id).await {
        Ok(response) => created_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;
    use chrono::Local;

    pub fn validate_request(request: &CreateFromTaskRequest) -> AppResult<()> {
        if request.start_time >= request.end_time {
            return Err(AppError::validation_error(
                "time_range",
                "开始时间必须早于结束时间",
                "INVALID_TIME_RANGE",
            ));
        }

        // 验证分时事件不能跨天
        let is_all_day = request.is_all_day.unwrap_or(false);
        if !is_all_day {
            // ✅ 根据时间类型选择不同的跨天检测方式
            let time_type = request
                .time_type
                .unwrap_or(crate::entities::time_block::TimeType::Floating);
            let crosses_day = if time_type == crate::entities::time_block::TimeType::Floating {
                // 浮动时间：检测本地时间部分是否跨天
                if let (Some(start_local), Some(end_local)) =
                    (&request.start_time_local, &request.end_time_local)
                {
                    // 对于浮动时间，只要 end_local < start_local 就说明跨天了
                    // 例如：start_local = "23:00:00", end_local = "01:00:00" → 跨天
                    end_local < start_local
                } else {
                    // 如果没有本地时间信息，回退到UTC检测
                    let local_start = request.start_time.with_timezone(&Local);
                    let local_end = request.end_time.with_timezone(&Local);
                    local_start.date_naive() != local_end.date_naive()
                }
            } else {
                // 固定时间：检测UTC转本地后是否跨天
                let local_start = request.start_time.with_timezone(&Local);
                let local_end = request.end_time.with_timezone(&Local);
                local_start.date_naive() != local_end.date_naive()
            };

            if crosses_day {
                return Err(AppError::validation_error(
                    "time_range",
                    "分时事件不能跨天，请使用全天事件或将时间块拆分为多个",
                    "CROSS_DAY_TIMED_EVENT",
                ));
            }
        }

        Ok(())
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        request: CreateFromTaskRequest,
        correlation_id: Option<String>,
    ) -> AppResult<crate::entities::TimeBlockTransactionResult> {
        // 1. 验证
        validation::validate_request(&request)?;

        // ✅ 获取写入许可，确保写操作串行执行（覆盖所有后续事务）
        let _permit = app_state.acquire_write_permit().await;

        // 2. 开始事务
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
        })?;

        // 3. 检查任务是否存在（✅ 使用共享 Repository）
        let task = TaskRepository::find_by_id_in_tx(&mut tx, request.task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", request.task_id.to_string()))?;

        // 4. 检查时间冲突（✅ 使用共享 ConflictChecker）
        let is_all_day = request.is_all_day.unwrap_or(false);
        let has_conflict = TimeBlockConflictChecker::check_in_tx(
            &mut tx,
            &request.start_time,
            &request.end_time,
            is_all_day,
            None,
        )
        .await?;

        if has_conflict {
            return Err(AppError::conflict("该时间段与现有时间块重叠"));
        }

        // 5. 生成 UUID 和时间戳
        let block_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // 6. 创建时间块（使用任务标题或自定义标题）
        let title = request.title.or_else(|| Some(task.title.clone()));

        let time_block = TimeBlock {
            id: block_id,
            title,
            glance_note: None,
            detail_note: None,
            start_time: request.start_time,
            end_time: request.end_time,
            start_time_local: request.start_time_local, // 使用请求中的字段
            end_time_local: request.end_time_local,     // 使用请求中的字段
            time_type: request.time_type.unwrap_or_default(), // 使用请求中的字段，默认FLOATING
            creation_timezone: request.creation_timezone, // 使用请求中的字段
            is_all_day,
            area_id: task.area_id, // 继承任务的 area
            created_at: now,
            updated_at: now,
            is_deleted: false,
            source_info: Some(crate::entities::SourceInfo {
                source_type: "native::from_task".to_string(),
                description: None,
                url: None,
                created_by_task_id: Some(request.task_id),
            }),
            external_source_id: None,
            external_source_provider: None,
            external_source_metadata: None,
            recurrence_rule: None,
            recurrence_parent_id: None,
            recurrence_original_date: None,
        };

        TimeBlockRepository::insert_in_tx(&mut tx, &time_block).await?;

        // 7. 链接任务到时间块（✅ 使用共享 Repository）
        TaskTimeBlockLinkRepository::link_in_tx(&mut tx, request.task_id, block_id).await?;

        // 8. 创建日程记录（✅ 使用共享 Repository）
        // 提取日期字符串（YYYY-MM-DD）- 使用系统本地时区
        use crate::infra::core::utils::time_utils;
        use chrono::Local;
        let local_start = request.start_time.with_timezone(&Local);
        let scheduled_date = time_utils::format_date_yyyy_mm_dd(&local_start.date_naive());

        tracing::info!(
            "[create_from_task] start_time (UTC): {}, scheduled_date: {}",
            request.start_time,
            scheduled_date
        );

        let has_schedule = TaskScheduleRepository::has_schedule_for_day_in_tx(
            &mut tx,
            request.task_id,
            &scheduled_date,
        )
        .await?;
        if !has_schedule {
            TaskScheduleRepository::create_in_tx(&mut tx, request.task_id, &scheduled_date).await?;
        }

        // 9. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::infra::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 10. 组装返回数据（✅ area_id 已直接从 time_block 获取）
        let time_block_view = TimeBlockViewDto {
            id: time_block.id,
            start_time: time_block.start_time,
            end_time: time_block.end_time,
            start_time_local: time_block.start_time_local,
            end_time_local: time_block.end_time_local,
            time_type: time_block.time_type,
            creation_timezone: time_block.creation_timezone,
            is_all_day: time_block.is_all_day,
            title: time_block.title,
            glance_note: time_block.glance_note,
            detail_note: time_block.detail_note,
            area_id: time_block.area_id,
            linked_tasks: vec![LinkedTaskSummary {
                id: task.id,
                title: task.title.clone(),
                is_completed: task.is_completed(),
            }],
            is_recurring: false,
            recurrence_id: None,
            recurrence_original_date: None,
        };

        // 11. 组装更新后的 TaskCard（✅ area_id 已由 TaskAssembler 填充）
        let mut updated_task = TaskAssembler::task_to_card_basic(&task);

        // 12. ✅ 填充 schedules 字段（事务已提交，使用 pool 查询）
        // ⚠️ 必须填充完整数据，否则前端筛选会失败！
        updated_task.schedules =
            TaskAssembler::assemble_schedules(app_state.db_pool(), request.task_id).await?;
        // schedule_status 已删除 - 前端根据 schedules 字段实时计算

        // 13. 发送 SSE 事件（通知其他视图时间块已创建）
        use crate::features::shared::TransactionHelper;
        use crate::infra::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };

        let mut outbox_tx = TransactionHelper::begin(app_state.db_pool()).await?;
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        let payload = serde_json::json!({
            "time_block_id": block_id,
            "task_id": request.task_id,
            "time_block": time_block_view,
            "updated_task": updated_task,
        });

        let mut event = DomainEvent::new(
            "time_blocks.created",
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

        Ok(crate::entities::TimeBlockTransactionResult {
            time_block: time_block_view,
            side_effects: crate::entities::TimeBlockSideEffects {
                updated_tasks: Some(vec![updated_task]),
                updated_time_blocks: None,
            },
        })
    }
}

// ==================== 数据访问层 ====================
// ✅ 已全部迁移到共享 Repository：
// - TaskRepository::find_by_id_in_tx
// - TimeBlockConflictChecker::check_in_tx
// - TimeBlockRepository::insert_in_tx
// - TaskTimeBlockLinkRepository::link_in_tx
// - TaskScheduleRepository::has_schedule_for_day_in_tx, create_in_tx
