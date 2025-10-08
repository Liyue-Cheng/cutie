/// 获取任务详情 API - 单文件组件
///
/// 按照单文件组件模式实现
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    entities::{Task, TaskDetailDto},
    features::tasks::shared::TaskAssembler,
    shared::{
        core::{AppError, AppResult, utils::time_utils},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `get_task`

## 1. 端点签名 (Endpoint Signature)

GET /api/tasks/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要查看任务的详细信息（包括完整笔记、日程、时间块等），
> 以便我能全面了解任务的状态和相关安排。

### 2.2. 核心业务逻辑 (Core Business Logic)

根据任务 ID 查询任务的完整详情，包括基础信息、详细笔记、所有日程（包含关联的时间块）。
返回 `TaskDetailDto`，其中包含比 `TaskCardDto` 更详细的信息（如 detail_note）。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 任务ID

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `TaskDetailDto`

```json
{
  "card": {
    "id": "uuid",
    "title": "string",
    "glance_note": "string | null",
    "schedule_status": "staging" | "scheduled",
    "is_completed": boolean,
    "area": {...} | null,
    "schedules": [...] | null,
    "due_date": {...} | null,
    "has_detail_note": boolean
  },
  "detail_note": "string | null",
  "project": null,
  "created_at": "2025-10-05T12:00:00Z",
  "updated_at": "2025-10-05T12:00:00Z"
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Task not found: {id}"
}
```

## 4. 验证规则 (Validation Rules)

- `task_id`:
    - **必须**是有效的 UUID 格式。
    - **必须**存在于数据库中且未删除（`deleted_at IS NULL`）。
    - 违反时返回 `404 NOT_FOUND`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  查询任务实体（`TaskRepository::find_by_id`）。
2.  如果任务不存在或已删除，返回 404 错误。
3.  组装基础 `TaskCardDto`（`TaskAssembler::task_to_card_basic`）。
4.  查询并组装完整的 schedules 数据（`TaskAssembler::assemble_schedules`）。
5.  根据 schedules 判断并设置正确的 `schedule_status`:
    - 如果今天或未来有日程：`Scheduled`
    - 否则：`Staging`
6.  组装 `TaskDetailDto`，包含：
    - `card`: 完整的任务卡片
    - `detail_note`: 详细笔记
    - `project`: 项目信息（暂未实现，返回 null）
    - `created_at`, `updated_at`: 时间戳
7.  返回 `TaskDetailDto`。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **任务已删除 (`is_deleted = true`):** 返回 `404` 错误（视为不存在）。
- **任务无 schedules:** `schedules` 字段为 `None`，`schedule_status` 为 `Staging`。
- **任务无 detail_note:** `detail_note` 字段为 `null`。

## 7. 预期副作用 (Expected Side Effects)

- **数据库查询:**
    - **`SELECT`:** 1次查询 `tasks` 表（获取任务基础信息）。
    - **`SELECT`:** 1次查询 `task_schedules` 表（获取所有日程）。
    - **`SELECT`:** 0-N 次查询 `time_blocks` 表（获取每个日程关联的时间块）。
    - **`SELECT`:** 0-1 次查询 `areas` 表（获取 area 信息，如果有）。
- **无写操作:** 此端点为只读查询，不修改任何数据。
- **无 SSE 事件:** 不发送任何事件。
- **日志记录:**
    - 失败时（如任务不存在），以 `WARN` 级别记录错误信息。

*（无其他已知副作用）*
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>, Path(task_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, task_id).await {
        Ok(task_detail) => success_response(task_detail).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, task_id: Uuid) -> AppResult<TaskDetailDto> {
        let pool = app_state.db_pool();

        // 1. 查询任务
        let task = database::find_task_by_id(pool, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 2. 组装基础 TaskCard
        let mut task_card = TaskAssembler::task_to_card_basic(&task);

        // 3. 组装完整的 schedules（包含 time_blocks）
        let schedules = TaskAssembler::assemble_schedules(pool, task_id).await?;

        // 4. ✅ 关键：根据实际 schedules 判断 schedule_status
        // staging 定义：今天和未来没有排期的任务，过去的排期不影响
        use chrono::Utc;
        let local_today = time_utils::extract_local_date_from_utc(Utc::now());

        let has_future_schedule = schedules
            .as_ref()
            .map(|schedules| {
                schedules.iter().any(|s| {
                    if let Ok(schedule_date) =
                        chrono::NaiveDate::parse_from_str(&s.scheduled_day, "%Y-%m-%d")
                    {
                        schedule_date >= local_today
                    } else {
                        false
                    }
                })
            })
            .unwrap_or(false);

        task_card.schedule_status = if has_future_schedule {
            crate::entities::ScheduleStatus::Scheduled
        } else {
            crate::entities::ScheduleStatus::Staging
        };

        task_card.schedules = schedules;

        // 5. 组装 TaskDetailDto
        let task_detail = TaskDetailDto {
            card: task_card,
            detail_note: task.detail_note.clone(),
            // schedules 已通过 flatten 从 TaskCardDto 继承
            project: None, // TODO: 查询项目信息
            created_at: task.created_at,
            updated_at: task.updated_at,
        };

        Ok(task_detail)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;

    pub async fn find_task_by_id(
        pool: &sqlx::SqlitePool,
        task_id: Uuid,
    ) -> AppResult<Option<Task>> {
        use crate::features::tasks::shared::repositories::TaskRepository;
        TaskRepository::find_by_id(pool, task_id).await
    }
}

// ✅ 已迁移到共享 Repository：
// - TaskRepository::find_by_id
// - TaskScheduleRepository::get_all_for_task
