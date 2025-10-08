/// 获取已排期任务 API - 单文件组件
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};

use crate::{
    entities::{ScheduleStatus, Task, TaskCardDto},
    features::tasks::shared::TaskAssembler,
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `get_planned`

## 1. 端点签名 (Endpoint Signature)

GET /api/views/planned

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要查看所有已排期（Planned）的未完成任务列表，
> 以便我能看到所有已被安排到具体日期的待办事项，了解未来的工作安排。

### 2.2. 核心业务逻辑 (Core Business Logic)

从数据库中查询所有符合"Planned"定义的任务：未删除、未完成、且至少存在一条 task_schedules 记录的任务。
为每个任务组装完整的 TaskCardDto（包含完整的 schedules、time_blocks 和 area 信息），并明确标记 schedule_status 为 Scheduled。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- 无

**Query Parameters:**
- 无（当前版本不支持分页、过滤、排序参数）

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `TaskCardDto[]`

```json
[
  {
    "id": "uuid",
    "title": "string",
    "glance_note": "string | null",
    "schedule_status": "scheduled",
    "is_completed": false,
    "area": { "id": "uuid", "name": "string", "color": "#RRGGBB" } | null,
    "schedules": [
      {
        "id": "uuid",
        "scheduled_day": "YYYY-MM-DD",
        "time_blocks": [
          {
            "id": "uuid",
            "start_time": "HH:MM",
            "end_time": "HH:MM"
          }
        ]
      }
    ],
    "due_date": { "date": "ISO8601", "type": "deadline" | "scheduled" } | null,
    "has_detail_note": boolean
  },
  ...
]
```

**注意：** 空列表返回 `[]`，而不是错误。

## 4. 验证规则 (Validation Rules)

- 无输入参数，无需验证。
- "Planned" 的定义由后端逻辑保证：
  - `is_deleted = false`
  - `completed_at IS NULL`
  - 存在至少一条 `task_schedules` 记录（通过 INNER JOIN 保证）

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `database::find_planned_tasks` 查询数据库：
    - 通过 `INNER JOIN task_schedules` 查询 `tasks` 表
    - 过滤条件：`is_deleted = false` 和 `completed_at IS NULL`
    - 使用 `DISTINCT` 去重（一个任务可能有多个 schedules）
    - 按 `scheduled_day ASC, created_at DESC` 排序（最近的日期优先，同一天内最新创建的在前）
2.  遍历每个任务，调用 `assemble_task_card` 进行组装：
    - 调用 `TaskAssembler::task_to_card_basic` 创建基础 TaskCard
    - 调用 `TaskAssembler::assemble_schedules` 查询完整的 schedules（包含 time_blocks）
    - 明确设置 `schedule_status = Scheduled`
3.  返回 `200 OK` 和任务列表（`Vec<TaskCardDto>`）。

## 6. 边界情况 (Edge Cases)

- **数据库中没有已排期任务:** 返回空数组 `[]`（200 OK）。
- **所有任务都在 staging 或已完成:** 返回空数组 `[]`（200 OK）。
- **任务有多个 schedules:** 任务只出现一次（通过 `DISTINCT` 去重），schedules 字段包含所有日程。
- **任务数量很大:** 当前无分页机制，可能返回大量数据（性能考虑）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库查询:**
    - **`SELECT`:** 1次，通过 `INNER JOIN` 查询 `tasks` 和 `task_schedules` 表。
    - **`SELECT`:** N次（N = planned 任务数量），每个任务查询完整的 schedules。
    - **`SELECT`:** 0-M次（M = schedules 总数），查询 `time_blocks` 表（每个 schedule 可能有时间块）。
    - **`SELECT`:** 0-N次，查询 `areas` 表（如果任务有 area_id）。
- **无写操作:** 此端点为只读查询，不修改任何数据。
- **无 SSE 事件:** 不发送任何事件。
- **日志记录:**
    - 失败时（数据库错误），以 `ERROR` 级别记录详细错误信息。

*（无其他已知副作用）*
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>) -> Response {
    match logic::execute(&app_state).await {
        Ok(task_cards) => success_response(task_cards).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState) -> AppResult<Vec<TaskCardDto>> {
        let pool = app_state.db_pool();

        // 1. 获取所有已排期任务
        let tasks = database::find_planned_tasks(pool).await?;

        // 2. 为每个任务组装 TaskCardDto
        let mut task_cards = Vec::new();
        for task in tasks {
            let task_card = assemble_task_card(&task, pool).await?;
            task_cards.push(task_card);
        }

        Ok(task_cards)
    }

    /// 组装单个任务的 TaskCard（包含完整的 schedules + time_blocks）
    async fn assemble_task_card(task: &Task, pool: &sqlx::SqlitePool) -> AppResult<TaskCardDto> {
        let mut card = TaskAssembler::task_to_card_basic(task);

        // 组装完整的 schedules（包含 time_blocks）
        let schedules = TaskAssembler::assemble_schedules(pool, task.id).await?;
        card.schedules = schedules;

        // 明确设置为 scheduled
        card.schedule_status = ScheduleStatus::Scheduled;

        Ok(card)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::entities::TaskRow;

    pub async fn find_planned_tasks(pool: &sqlx::SqlitePool) -> AppResult<Vec<Task>> {
        let query = r#"
            SELECT DISTINCT
                t.id, t.title, t.glance_note, t.detail_note, t.estimated_duration,
                t.subtasks, t.project_id, t.area_id, t.due_date, t.due_date_type, t.completed_at, t.archived_at,
                t.created_at, t.updated_at, t.deleted_at, t.source_info,
                t.external_source_id, t.external_source_provider, t.external_source_metadata,
                t.recurrence_rule, t.recurrence_parent_id, t.recurrence_original_date, t.recurrence_exclusions
            FROM tasks t
            INNER JOIN task_schedules ts ON t.id = ts.task_id
            WHERE t.deleted_at IS NULL AND t.completed_at IS NULL AND t.archived_at IS NULL
            ORDER BY ts.scheduled_day ASC, t.created_at DESC
        "#;

        let rows = sqlx::query_as::<_, TaskRow>(query)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        let tasks: Result<Vec<Task>, _> = rows.into_iter().map(Task::try_from).collect();

        tasks.map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }
}
