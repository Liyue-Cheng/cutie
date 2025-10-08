/// 创建时间块 API - 单文件组件
///
/// 按照单文件组件模式实现
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};

use crate::{
    entities::{CreateTimeBlockRequest, TimeBlock, TimeBlockViewDto},
    features::time_blocks::shared::{repositories::TimeBlockRepository, TimeBlockConflictChecker},
    shared::{
        core::{AppError, AppResult},
        http::error_handler::created_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `create_time_block`

## 1. 端点签名 (Endpoint Signature)

POST /api/time-blocks

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要在日历上创建一个纯时间块（会议、约会、独立事件），
> 以便我能够管理我的日程安排，而不必关联到具体的任务。

### 2.2. 核心业务逻辑 (Core Business Logic)

创建一个独立的时间块，不关联任何任务。此端点专注于纯时间块的创建（会议、约会等）。
关键业务规则：时间块不允许重叠，系统会自动检测并拒绝重叠的时间段。
如果需要创建与任务关联的时间块，应使用专门的 `POST /api/time-blocks/from-task` 端点。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**请求体 (Request Body):** `application/json`

```json
{
  "start_time": "string (ISO 8601 UTC, required)",
  "end_time": "string (ISO 8601 UTC, required)",
  "title": "string | null (optional, 最多255字符)",
  "glance_note": "string | null (optional)",
  "detail_note": "string | null (optional)",
  "area_id": "UUID | null (optional)"
}
```

### 3.2. 响应 (Responses)

**201 Created:**

*   **Content-Type:** `application/json`
*   **Schema:** `TimeBlockViewDto`

```json
{
  "id": "uuid",
  "start_time": "2025-10-05T14:00:00Z",
  "end_time": "2025-10-05T15:00:00Z",
  "title": "string | null",
  "glance_note": "string | null",
  "detail_note": "string | null",
  "area_id": "uuid | null",
  "linked_tasks": [],
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
- `title`:
    - 如果提供，长度**必须**小于等于 255 个字符。
    - 违反时返回错误码：`TITLE_TOO_LONG`
- **时间冲突验证**:
    - 新时间块的时间范围**不能**与现有时间块重叠。
    - 违反时返回错误码：`CONFLICT`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `validation::validate_create_request` 验证请求体。
2.  启动数据库事务（`app_state.db_pool().begin()`）。
3.  调用 `TimeBlockConflictChecker::check_in_tx` 检查时间冲突：
    - 查询时间范围重叠的现有时间块
    - 如果存在重叠，返回 409 冲突错误
4.  通过 `IdGenerator` 生成新的 `block_id`（UUID）。
5.  通过 `Clock` 服务获取当前时间 `now`。
6.  构造 `TimeBlock` 领域实体对象：
    - 设置 `id`, `title`, `glance_note`, `detail_note`, `area_id`
    - 设置 `start_time`, `end_time`
    - 设置 `created_at = now`, `updated_at = now`
    - 设置 `is_deleted = false`
    - 设置循环相关字段为 `None`（当前版本不支持循环）
7.  调用 `TimeBlockRepository::insert_in_tx` 持久化时间块到 `time_blocks` 表。
8.  提交数据库事务。
9.  组装返回的 `TimeBlockViewDto`：
    - 填充所有基础字段
    - 设置 `linked_tasks = []`（纯时间块无关联任务）
    - 设置 `is_recurring = false`
10. 返回 `201 Created` 和组装好的 `TimeBlockViewDto`。

## 6. 边界情况 (Edge Cases)

- **`start_time >= end_time`:** 返回 `400` 错误，错误码 `INVALID_TIME_RANGE`。
- **时间范围与现有时间块重叠:** 返回 `409` 错误，错误码 `CONFLICT`。
- **`title` 超过 255 字符:** 返回 `422` 错误，错误码 `TITLE_TOO_LONG`。
- **`area_id` 不存在:** 当前实现中正常返回（area_id 字段为提供的值），未来可能需要验证。
- **无标题的时间块:** 允许创建，`title` 为 `null`。
- **并发创建重叠时间块:** 事务隔离保证只有一个会成功，其他会收到冲突错误。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次，查询重叠的时间块（冲突检测）。
    - **`INSERT`:** 1条记录到 `time_blocks` 表。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **日志记录:**
    - 成功时，可能记录时间块创建信息（如有配置）。
    - 失败时（验证失败或数据库错误），以 `WARN` 或 `ERROR` 级别记录详细错误信息。

*（无其他已知副作用，不发送 SSE 事件）*
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateTimeBlockRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(time_block_view) => created_response(time_block_view).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;
    use chrono::{DateTime, Utc};

    pub fn validate_create_request(request: &CreateTimeBlockRequest) -> AppResult<()> {
        // 验证时间范围
        if request.start_time >= request.end_time {
            return Err(AppError::validation_error(
                "time_range",
                "开始时间必须早于结束时间",
                "INVALID_TIME_RANGE",
            ));
        }

        // 验证分时事件不能跨天
        let is_all_day = request.is_all_day.unwrap_or(false);
        if !is_all_day && !is_same_day(&request.start_time, &request.end_time) {
            return Err(AppError::validation_error(
                "time_range",
                "分时事件不能跨天，请使用全天事件或将时间块拆分为多个",
                "CROSS_DAY_TIMED_EVENT",
            ));
        }

        // 验证标题长度（如果有）
        if let Some(title) = &request.title {
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

    /// 检查两个时间是否在同一天（本地时区）
    fn is_same_day(time1: &DateTime<Utc>, time2: &DateTime<Utc>) -> bool {
        use crate::shared::core::utils::time_utils::extract_local_date_from_utc;
        let d1 = extract_local_date_from_utc(time1.clone());
        let d2 = extract_local_date_from_utc(time2.clone());
        d1 == d2
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        request: CreateTimeBlockRequest,
    ) -> AppResult<TimeBlockViewDto> {
        // 1. 验证请求
        validation::validate_create_request(&request)?;

        // 2. 开始事务
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 3. 检查时间冲突（✅ 使用共享 ConflictChecker）
        let is_all_day = request.is_all_day.unwrap_or(false);
        let has_conflict = TimeBlockConflictChecker::check_in_tx(
            &mut tx,
            &request.start_time,
            &request.end_time,
            is_all_day,
            None, // 新建时没有要排除的ID
        )
        .await?;

        if has_conflict {
            return Err(AppError::conflict(
                "该时间段与现有时间块重叠，时间块不允许重叠",
            ));
        }

        // 4. 生成 UUID 和时间戳
        let block_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // 5. 创建时间块实体
        let time_block = TimeBlock {
            id: block_id,
            title: request.title.clone(),
            glance_note: request.glance_note.clone(),
            detail_note: request.detail_note.clone(),
            start_time: request.start_time,
            end_time: request.end_time,
            is_all_day,
            area_id: request.area_id,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            source_info: Some(crate::entities::SourceInfo {
                source_type: "native::manual".to_string(),
                description: None,
                url: None,
                created_by_task_id: None,
            }),
            external_source_id: None,
            external_source_provider: None,
            external_source_metadata: None,
            recurrence_rule: None,
            recurrence_parent_id: None,
            recurrence_original_date: None,
            recurrence_exclusions: None,
        };

        // 6. 插入时间块到数据库（✅ 使用共享 Repository）
        TimeBlockRepository::insert_in_tx(&mut tx, &time_block).await?;

        // 7. 提交事务
        // 🔧 REMOVED: 任务关联逻辑已移除，职责分离
        // 任务关联应使用 POST /time-blocks/from-task 端点
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 8. 组装返回的 TimeBlockViewDto（✅ area_id 已直接从 time_block 获取）
        let time_block_view = TimeBlockViewDto {
            id: time_block.id,
            start_time: time_block.start_time,
            end_time: time_block.end_time,
            is_all_day: time_block.is_all_day,
            title: time_block.title,
            glance_note: time_block.glance_note,
            detail_note: time_block.detail_note,
            area_id: time_block.area_id,
            linked_tasks: Vec::new(), // 🔧 纯时间块不关联任务
            is_recurring: time_block.recurrence_rule.is_some(),
        };

        Ok(time_block_view)
    }
}

// ==================== 数据访问层 ====================
// ✅ 已全部迁移到共享 Repository：
// - TimeBlockConflictChecker::check_in_tx
// - TimeBlockRepository::insert_in_tx
//
// 🔧 职责分离说明：
// 此端点仅创建纯时间块，不涉及任务关联
// 任务关联使用专门的 POST /time-blocks/from-task 端点
