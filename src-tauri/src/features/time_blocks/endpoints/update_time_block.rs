/// 更新时间块 API - 单文件组件
///
/// 按照单文件组件模式实现
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use crate::{
    entities::{TimeBlockViewDto, UpdateTimeBlockRequest},
    features::{
        tasks::shared::{
            assemblers::LinkedTaskAssembler, repositories::TaskTimeBlockLinkRepository,
        },
        time_blocks::shared::{repositories::TimeBlockRepository, TimeBlockConflictChecker},
    },
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `update_time_block`

## 1. 端点签名 (Endpoint Signature)

PATCH /api/time-blocks/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要调整日历上时间块的时间、标题、笔记或所属区域，
> 以便我能够灵活管理我的日程安排，适应计划的变化。
> 特别是在拖动时间块调整时间时，系统应该自动验证是否与其他时间块冲突。

### 2.2. 核心业务逻辑 (Core Business Logic)

更新现有时间块的可变字段（时间范围、标题、笔记、area 等）。
支持部分更新（PATCH 语义），只需提供要更改的字段。
关键业务规则：
1. 如果更新时间范围，必须确保新时间范围不与其他时间块重叠（排除自身）
2. 更新后自动刷新 `updated_at` 时间戳
3. 返回完整的时间块视图（包含关联的任务信息）

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 时间块ID

**请求体 (Request Body):** `application/json`

所有字段都是可选的（部分更新）：

```json
{
  "start_time": "string (ISO 8601 UTC) | null (optional)",
  "end_time": "string (ISO 8601 UTC) | null (optional)",
  "title": "string | null (optional, 最多255字符, 支持置空)",
  "glance_note": "string | null (optional, 支持置空)",
  "detail_note": "string | null (optional, 支持置空)",
  "area_id": "UUID | null (optional, 支持置空)"
}
```

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `TimeBlockViewDto`

```json
{
  "id": "uuid",
  "start_time": "2025-10-05T14:00:00Z",
  "end_time": "2025-10-05T16:00:00Z",
  "title": "string | null",
  "glance_note": "string | null",
  "detail_note": "string | null",
  "area_id": "uuid | null",
  "linked_tasks": [
    {
      "id": "uuid",
      "title": "string",
      "is_completed": false
    }
  ],
  "is_recurring": false
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
  "message": "TimeBlock not found: {id}"
}
```

**409 Conflict:**

```json
{
  "error_code": "CONFLICT",
  "message": "该时间段与现有时间块重叠，时间块不允许重叠"
}
```

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "title", "code": "TITLE_TOO_LONG", "message": "标题不能超过255个字符" }
  ]
}
```

## 4. 验证规则 (Validation Rules)

- `id`:
    - **必须**是有效的 UUID 格式。
    - 对应的时间块**必须**存在且未被删除。
    - 违反时返回错误码：`NOT_FOUND`
- `start_time`:
    - 如果提供，**必须**是有效的 ISO 8601 UTC 时间格式。
    - 如果同时提供 `start_time` 和 `end_time`，**必须**满足 `start_time < end_time`。
    - 违反时返回错误码：`INVALID_TIME_RANGE`
- `end_time`:
    - 如果提供，**必须**是有效的 ISO 8601 UTC 时间格式。
    - 如果同时提供 `start_time` 和 `end_time`，**必须**满足 `start_time < end_time`。
    - 违反时返回错误码：`INVALID_TIME_RANGE`
- **最终时间范围验证**:
    - 合并现有值和新值后，**必须**满足 `final_start_time < final_end_time`。
    - 违反时返回错误码：`INVALID_TIME_RANGE`
- `title`:
    - 如果提供，长度**必须**小于等于 255 个字符。
    - 违反时返回错误码：`TITLE_TOO_LONG`
- **时间冲突验证**:
    - 如果更新了时间范围，新时间范围**不能**与其他时间块重叠（排除自身）。
    - 违反时返回错误码：`CONFLICT`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `validation::validate_update_request` 验证请求体（初步验证）。
2.  启动数据库事务（`app_state.db_pool().begin()`）。
3.  调用 `TimeBlockRepository::find_by_id_in_tx` 查询现有时间块：
    - 如果时间块不存在，返回 404 错误
4.  确定最终的时间范围：
    - `final_start_time = request.start_time.unwrap_or(existing_block.start_time)`
    - `final_end_time = request.end_time.unwrap_or(existing_block.end_time)`
5.  再次验证最终时间范围：
    - 检查 `final_start_time < final_end_time`
    - 如果不满足，返回 400 错误
6.  如果时间范围发生变化（`request.start_time` 或 `request.end_time` 非空）：
    - 调用 `TimeBlockConflictChecker::check_in_tx` 检查时间冲突
    - 传入 `Some(id)` 排除当前时间块
    - 如果存在重叠，返回 409 冲突错误
7.  通过 `Clock` 服务获取当前时间 `now`。
8.  调用 `TimeBlockRepository::update_in_tx` 更新时间块。
9.  提交数据库事务。
10. 重新查询时间块以获取最新数据（`TimeBlockRepository::find_by_id`）。
11. 组装返回的 `TimeBlockViewDto`：
    - 填充所有基础字段
    - 调用 `LinkedTaskAssembler::get_for_time_block` 填充关联任务
12. 返回 `200 OK` 和组装好的 `TimeBlockViewDto`。

## 6. 边界情况 (Edge Cases)

- **时间块不存在:** 返回 `404` 错误。
- **只更新 `start_time`:** 结合现有 `end_time`，验证最终时间范围。
- **只更新 `end_time`:** 结合现有 `start_time`，验证最终时间范围。
- **最终时间范围无效:** 返回 `400` 错误，错误码 `INVALID_TIME_RANGE`。
- **时间范围与其他时间块重叠:** 返回 `409` 错误，错误码 `CONFLICT`。
- **`title` 超过 255 字符:** 返回 `422` 错误，错误码 `TITLE_TOO_LONG`。
- **清空 `title`（设置为 `null`）:** 允许，时间块可以没有标题。
- **更新 `area_id` 为 `null`:** 允许，时间块可以不属于任何区域。
- **空更新（不提供任何字段）:** 仍然更新 `updated_at` 时间戳，返回成功。
- **并发更新:** 事务隔离保证数据一致性，后者可能因冲突检测失败。
- **幂等性:** 相同参数重复调用，结果一致（`updated_at` 会变化）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次，查询现有时间块。
    - **`SELECT`:** 0-1次，查询重叠的时间块（仅当时间范围变化时）。
    - **`UPDATE`:** 1条记录在 `time_blocks` 表。
    - **`SELECT`:** 1次，重新查询更新后的时间块。
    - **`SELECT`:** 1次，查询关联的任务列表。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **日志记录:**
    - 成功时，记录时间块更新信息（包含 block_id）。
    - 失败时，记录详细错误信息。

*（无其他已知副作用，不发送 SSE 事件）*
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateTimeBlockRequest>,
) -> Response {
    match logic::execute(&app_state, id, request).await {
        Ok(time_block_view) => success_response(time_block_view).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate_update_request(request: &UpdateTimeBlockRequest) -> AppResult<()> {
        // 如果同时更新开始和结束时间，验证时间范围
        if let (Some(start), Some(end)) = (request.start_time, request.end_time) {
            if start >= end {
                return Err(AppError::validation_error(
                    "time_range",
                    "开始时间必须早于结束时间",
                    "INVALID_TIME_RANGE",
                ));
            }
        }

        // 验证标题长度（如果有）
        if let Some(Some(title)) = &request.title {
            if title.len() > 255 {
                return Err(AppError::validation_error(
                    "title",
                    "标题不能超过255个字符",
                    "TITLE_TOO_LONG",
                ));
            }
        }

        Ok(())
    }

    /// 验证最终时间范围（包括跨天检测）
    pub fn validate_final_time_range(
        final_start_time: &chrono::DateTime<chrono::Utc>,
        final_end_time: &chrono::DateTime<chrono::Utc>,
        final_is_all_day: bool,
    ) -> AppResult<()> {
        // 基本验证：开始时间必须早于结束时间
        if final_start_time >= final_end_time {
            return Err(AppError::validation_error(
                "time_range",
                "开始时间必须早于结束时间",
                "INVALID_TIME_RANGE",
            ));
        }

        // 验证分时事件不能跨天
        if !final_is_all_day && !is_same_day(final_start_time, final_end_time) {
            return Err(AppError::validation_error(
                "time_range",
                "分时事件不能跨天，请使用全天事件或将时间块拆分为多个",
                "CROSS_DAY_TIMED_EVENT",
            ));
        }

        Ok(())
    }

    /// 检查两个时间是否在同一天（系统本地时区）
    fn is_same_day(
        time1: &chrono::DateTime<chrono::Utc>,
        time2: &chrono::DateTime<chrono::Utc>,
    ) -> bool {
        use chrono::Local;
        let local1 = time1.with_timezone(&Local);
        let local2 = time2.with_timezone(&Local);
        local1.date_naive() == local2.date_naive()
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        id: Uuid,
        request: UpdateTimeBlockRequest,
    ) -> AppResult<TimeBlockViewDto> {
        // 1. 验证请求
        validation::validate_update_request(&request)?;

        // 2. 开始事务
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 3. 获取现有时间块（确保存在）（✅ 使用共享 Repository）
        let existing_block = TimeBlockRepository::find_by_id_in_tx(&mut tx, id).await?;

        // 4. 确定最终的时间范围和全天状态
        let final_start_time = request.start_time.unwrap_or(existing_block.start_time);
        let final_end_time = request.end_time.unwrap_or(existing_block.end_time);
        let final_is_all_day = request.is_all_day.unwrap_or(existing_block.is_all_day);

        // 5. 验证最终时间范围（包括跨天检测）
        validation::validate_final_time_range(
            &final_start_time,
            &final_end_time,
            final_is_all_day,
        )?;

        // 6. 如果时间范围或全天状态发生变化，检查时间冲突（✅ 使用共享 ConflictChecker）
        if request.start_time.is_some()
            || request.end_time.is_some()
            || request.is_all_day.is_some()
        {
            let has_conflict = TimeBlockConflictChecker::check_in_tx(
                &mut tx,
                &final_start_time,
                &final_end_time,
                final_is_all_day,
                Some(id), // 排除当前时间块
            )
            .await?;

            if has_conflict {
                return Err(AppError::conflict(
                    "该时间段与现有时间块重叠，时间块不允许重叠",
                ));
            }
        }

        // 7. 查询受影响的任务ID（如果时间发生变化）
        let should_publish_event = request.start_time.is_some() || request.end_time.is_some();
        let affected_task_ids = if should_publish_event {
            TaskTimeBlockLinkRepository::get_task_ids_for_block_in_tx(&mut tx, id).await?
        } else {
            Vec::new()
        };

        // 8. 获取当前时间戳
        let now = app_state.clock().now_utc();

        // 9. 更新时间块（✅ 使用共享 Repository）
        TimeBlockRepository::update_in_tx(&mut tx, id, &request, now).await?;

        // 10. 写入事件到 outbox（在事务内，仅当时间变化且有关联任务时）
        if should_publish_event && !affected_task_ids.is_empty() {
            use crate::shared::events::{
                models::DomainEvent,
                outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
            };

            let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

            let payload = serde_json::json!({
                "time_block_id": id,
                "affected_task_ids": affected_task_ids,
            });

            let event =
                DomainEvent::new("time_blocks.updated", "TimeBlock", id.to_string(), payload);

            outbox_repo.append_in_tx(&mut tx, &event).await?;
        }

        // 11. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 12. 重新查询时间块以获取最新数据（✅ 使用共享 Repository）
        let updated_block = TimeBlockRepository::find_by_id(app_state.db_pool(), id).await?;

        // 13. 组装返回的 TimeBlockViewDto（✅ area_id 已直接从 updated_block 获取）
        let mut time_block_view = TimeBlockViewDto {
            id: updated_block.id,
            start_time: updated_block.start_time,
            end_time: updated_block.end_time,
            start_time_local: updated_block.start_time_local,
            end_time_local: updated_block.end_time_local,
            time_type: updated_block.time_type,
            creation_timezone: updated_block.creation_timezone,
            is_all_day: updated_block.is_all_day,
            title: updated_block.title,
            glance_note: updated_block.glance_note,
            detail_note: updated_block.detail_note,
            area_id: updated_block.area_id,
            linked_tasks: Vec::new(),
            is_recurring: updated_block.recurrence_rule.is_some(),
        };

        // 14. 获取关联的任务摘要（✅ 使用共享 Assembler）
        time_block_view.linked_tasks =
            LinkedTaskAssembler::get_for_time_block(app_state.db_pool(), id).await?;

        tracing::info!("Updated time block: {}", id);

        Ok(time_block_view)
    }
}

// ==================== 数据访问层 ====================
// ✅ 已全部迁移到共享 Repository：
// - TimeBlockRepository::find_by_id_in_tx, find_by_id
// - TimeBlockConflictChecker::check_in_tx
// - TimeBlockRepository::update_in_tx
// - LinkedTaskAssembler::get_for_time_block
