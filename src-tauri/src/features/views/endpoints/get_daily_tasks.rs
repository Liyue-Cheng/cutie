/// 获取指定日期的任务视图 - 单文件组件
///
/// GET /api/views/daily/:date
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use chrono::NaiveDate;
use serde::Serialize;

use crate::{
    entities::{Task, TaskCardDto},
    features::views::shared::ViewTaskCardAssembler,
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `get_daily_tasks`

## 1. 端点签名 (Endpoint Signature)

GET /api/views/daily/:date

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要查看指定日期的所有任务列表（包括已完成和未完成的任务），
> 以便我能了解某一天的工作安排和完成情况。

### 2.2. 核心业务逻辑 (Core Business Logic)

根据 URL 路径中的日期参数（YYYY-MM-DD 格式），查询该日期在 task_schedules 表中的所有任务。
为每个任务组装完整的 TaskCardDto（包含完整的 schedules、time_blocks 和 area 信息），
返回包含任务列表、日期和数量的响应结构。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `date` (string, required): 日期字符串，格式为 YYYY-MM-DD（例如：2025-10-05）

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `GetDailyTasksResponse`

```json
{
  "tasks": [
    {
      "id": "uuid",
      "title": "string",
      "glance_note": "string | null",
      "schedule_status": "scheduled",
      "is_completed": boolean,
      "area": { "id": "uuid", "name": "string", "color": "#RRGGBB" } | null,
      "schedules": [
        {
          "id": "uuid",
          "scheduled_day": "YYYY-MM-DD",
          "time_blocks": [...]
        }
      ],
      "due_date": { "date": "ISO8601", "type": "deadline" | "scheduled" } | null,
      "has_detail_note": boolean
    },
    ...
  ],
  "date": "YYYY-MM-DD",
  "count": number
}
```

**400 Bad Request:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "date", "code": "INVALID_DATE_FORMAT", "message": "日期格式错误，请使用 YYYY-MM-DD 格式" }
  ]
}
```

**注意：** 如果该日期没有任务，返回 `{ "tasks": [], "date": "...", "count": 0 }`。

## 4. 验证规则 (Validation Rules)

- `date` 参数：
    - **必须**存在于 URL 路径中。
    - **必须**符合 YYYY-MM-DD 格式（例如：2025-10-05）。
    - **必须**能够成功解析为有效的日期（NaiveDate）。
    - 违反时返回 `400 VALIDATION_FAILED`，错误码 `INVALID_DATE_FORMAT`。

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  **验证层:** 调用 `validation::parse_date` 解析日期字符串：
    - 使用 `NaiveDate::parse_from_str` 解析 YYYY-MM-DD 格式
    - 转换为 UTC DateTime（时间设为 00:00:00）
    - 解析失败返回 `400 VALIDATION_FAILED` 错误
2.  **数据查询:** 调用 `database::find_tasks_for_date` 查询数据库：
    - 通过 `INNER JOIN task_schedules` 查询该日期的所有任务
    - 使用 `DATE(ts.scheduled_day) = DATE(?)` 进行日期匹配
    - 过滤 `is_deleted = false`（包含已完成和未完成任务）
    - 使用 `DISTINCT` 去重（一个任务在同一天可能有多个时间块）
    - 按 `created_at DESC` 排序（最新创建的在前）
3.  **任务组装:** 调用 `ViewTaskCardAssembler::assemble_batch` 批量组装：
    - 为每个任务查询完整的 schedules（包含 time_blocks）
    - 查询 area 信息（如果有）
    - 组装成 TaskCardDto
4.  **构建响应:** 返回 `GetDailyTasksResponse`：
    - `tasks`: 任务列表
    - `date`: 原始日期字符串
    - `count`: 任务数量
5.  返回 `200 OK` 和响应结构。

## 6. 边界情况 (Edge Cases)

- **日期格式错误:** 返回 `400 VALIDATION_FAILED`，错误码 `INVALID_DATE_FORMAT`。
- **无效日期（如 2025-02-30）:** 返回 `400 VALIDATION_FAILED`，错误码 `INVALID_DATE`。
- **该日期没有任务:** 返回 `{ "tasks": [], "date": "...", "count": 0 }`（200 OK）。
- **任务在该日期有多个时间块:** 任务只出现一次（通过 `DISTINCT` 去重），schedules 字段包含所有时间块。
- **包含已完成和未完成任务:** 不过滤 `completed_at`，两种任务都会返回。

## 7. 预期副作用 (Expected Side Effects)

- **数据库查询:**
    - **`SELECT`:** 1次，通过 `INNER JOIN` 查询 `tasks` 和 `task_schedules` 表（带日期过滤）。
    - **`SELECT`:** N次（N = 该日期的任务数量），每个任务查询完整的 schedules。
    - **`SELECT`:** 0-M次（M = schedules 总数），查询 `time_blocks` 表（每个 schedule 可能有时间块）。
    - **`SELECT`:** 0-N次，查询 `areas` 表（如果任务有 area_id）。
- **无写操作:** 此端点为只读查询，不修改任何数据。
- **无 SSE 事件:** 不发送任何事件。
- **日志记录:**
    - 成功时，以 `INFO` 级别记录 "Found N tasks for date YYYY-MM-DD"。
    - 失败时（日期格式错误或数据库错误），以 `WARN` 或 `ERROR` 级别记录详细错误信息。

*（无其他已知副作用）*
*/

// ==================== 响应结构体 ====================
#[derive(Debug, Serialize)]
pub struct GetDailyTasksResponse {
    pub tasks: Vec<TaskCardDto>,
    pub date: String,
    pub count: usize,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>, Path(date_str): Path<String>) -> Response {
    match logic::execute(&app_state, &date_str).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;
    use crate::shared::core::utils::time_utils;

    pub fn parse_date(date_str: &str) -> AppResult<NaiveDate> {
        time_utils::parse_date_yyyy_mm_dd(date_str).map_err(|_| {
            AppError::validation_error(
                "date",
                "日期格式错误，请使用 YYYY-MM-DD 格式",
                "INVALID_DATE_FORMAT",
            )
        })
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, date_str: &str) -> AppResult<GetDailyTasksResponse> {
        // 1. 解析日期为 NaiveDate
        let target_date = validation::parse_date(date_str)?;

        // 2. 查询该日期的所有任务（使用日期字符串匹配）
        let tasks = database::find_tasks_for_date(app_state.db_pool(), target_date).await?;

        tracing::info!("Found {} tasks for date {}", tasks.len(), date_str);

        // 3. 组装完整的 TaskCards（使用共享装配器）
        let task_cards = ViewTaskCardAssembler::assemble_batch(tasks, app_state.db_pool()).await?;

        // 4. 返回结果
        Ok(GetDailyTasksResponse {
            count: task_cards.len(),
            date: date_str.to_string(),
            tasks: task_cards,
        })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::shared::core::utils::time_utils;

    /// 查询指定日期的所有任务
    pub async fn find_tasks_for_date(
        pool: &sqlx::SqlitePool,
        target_date: NaiveDate,
    ) -> AppResult<Vec<Task>> {
        let date_str = time_utils::format_date_yyyy_mm_dd(&target_date);

        let query = r#"
            SELECT DISTINCT t.id, t.title, t.glance_note, t.detail_note, t.estimated_duration,
                   t.subtasks, t.project_id, t.area_id, t.due_date, t.due_date_type, t.completed_at, t.archived_at,
                   t.created_at, t.updated_at, t.deleted_at, t.source_info,
                   t.external_source_id, t.external_source_provider, t.external_source_metadata,
                   t.recurrence_id, t.recurrence_original_date
            FROM tasks t
            INNER JOIN task_schedules ts ON ts.task_id = t.id
            WHERE ts.scheduled_date = ?
              AND t.deleted_at IS NULL
              AND t.archived_at IS NULL
            ORDER BY t.created_at DESC
        "#;

        let rows = sqlx::query_as::<_, crate::entities::TaskRow>(query)
            .bind(date_str)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        let tasks = rows
            .into_iter()
            .map(|row| {
                Task::try_from(row).map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::QueryError(e))
                })
            })
            .collect::<AppResult<Vec<Task>>>()?;

        Ok(tasks)
    }
}
