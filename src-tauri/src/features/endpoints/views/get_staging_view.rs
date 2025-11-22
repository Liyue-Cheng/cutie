use crate::{
    entities::{ScheduleStatus, Task, TaskCardDto},
    features::shared::TaskAssembler,
    infra::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};
/// 获取 Staging 视图 API - 单文件组件
///
/// 按照单文件组件模式实现
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};

// ==================== 文档层 ====================
/*
CABC for `get_staging_view`

## 1. 端点签名 (Endpoint Signature)

GET /api/views/staging

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要查看所有未排期（Staging）的任务列表，
> 以便我能看到所有还没被安排到具体日期的待办事项，并进行后续的排期规划。

### 2.2. 核心业务逻辑 (Core Business Logic)

从数据库中查询所有符合"Staging"定义的任务：
- 未删除、未完成、且今天及未来不存在 task_schedules 记录的任务（过去的排期不影响）
- 排除 EXPIRE 类型且已过期的循环任务（recurrence_original_date < today）

为每个任务组装完整的 TaskCardDto（包含 area 信息、schedules 等上下文），并明确标记 schedule_status 为 Staging。

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
    "schedule_status": "staging",
    "is_completed": false,
    "area": { "id": "uuid", "name": "string", "color": "#RRGGBB" } | null,
    "schedules": null,
    "due_date": { "date": "ISO8601", "type": "deadline" | "scheduled" } | null,
    "has_detail_note": boolean
  },
  ...
]
```

**注意：** 空列表返回 `[]`，而不是错误。

## 4. 验证规则 (Validation Rules)

- 无输入参数，无需验证。
- "Staging" 的定义由后端逻辑保证：
  - `is_deleted = false`
  - `completed_at IS NULL`
  - 不存在于 `task_schedules` 表中
  - 排除 EXPIRE 类型且已过期的循环任务

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `database::find_staging_tasks` 查询数据库：
    - 查询 `tasks` 表，过滤 `is_deleted = false` 和 `completed_at IS NULL`
    - 通过 `NOT EXISTS` 子查询排除所有在 `task_schedules` 中有记录的任务
    - 排除 EXPIRE 类型且已过期的循环任务（recurrence_original_date < today）
    - 按 `created_at` 降序排列（最新的在前）
2.  遍历每个任务，调用 `assemble_task_card` 进行组装：
    - 调用 `TaskAssembler::task_to_card_basic` 创建基础 TaskCard
    - 调用 `TaskAssembler::assemble_schedules` 查询完整的 schedules（对于 staging 任务应该为 None）
    - 明确设置 `schedule_status = Staging`
3.  返回 `200 OK` 和任务列表（`Vec<TaskCardDto>`）。

## 6. 边界情况 (Edge Cases)

- **数据库中没有 staging 任务:** 返回空数组 `[]`（200 OK）。
- **所有任务都已排期或已完成:** 返回空数组 `[]`（200 OK）。
- **任务数量很大:** 当前无分页机制，可能返回大量数据（性能考虑）。
- **任务只有过去的 schedule 但今天/未来无 schedule:** 该任务**会**出现在 staging 视图（因为 SQL 查询只检查今天及未来的 schedule，过去的 schedule 不影响）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库查询:**
    - **`SELECT`:** 1次，查询 `tasks` 表（带 `NOT EXISTS` 子查询过滤 `task_schedules`）。
    - **`SELECT`:** N次（N = staging 任务数量），每个任务查询 `task_schedules` 表（用于组装 schedules，预期为空）。
    - **`SELECT`:** 0-N次，查询 `areas` 表（如果任务有 area_id，由 `TaskAssembler` 内部查询）。
- **无写操作:** 此端点为只读查询，不修改任何数据。
- **无 SSE 事件:** 不发送任何事件。
- **日志记录:**
    - 失败时（数据库错误），以 `ERROR` 级别记录详细错误信息。

*（无其他已知副作用）*
*/

// ==================== HTTP 处理器 ====================
/// 获取 Staging 视图的 HTTP 处理器
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
        // 只读操作，不需要事务，直接使用连接池
        let pool = app_state.db_pool();

        // 1. 计算本地时间的今天日期
        use crate::infra::core::utils::time_utils;
        use chrono::Local;
        let today = Local::now().date_naive();
        let today_str = time_utils::format_date_yyyy_mm_dd(&today);

        // 2. 获取所有 staging 任务
        let tasks = database::find_staging_tasks(pool, &today_str).await?;

        // 3. 为每个任务获取额外信息并组装成 TaskCardDto
        let mut task_cards = Vec::new();
        for task in tasks {
            let task_card = assemble_task_card(&task, pool).await?;
            task_cards.push(task_card);
        }

        Ok(task_cards)
    }

    /// 组装单个任务的 TaskCard（包含完整的 schedules + time_blocks）
    async fn assemble_task_card(task: &Task, pool: &sqlx::SqlitePool) -> AppResult<TaskCardDto> {
        // 1. 创建基础 TaskCard
        let mut card = TaskAssembler::task_to_card_basic(task);

        // 2. 组装完整的 schedules（对于 staging 任务应该是 None）
        let schedules = TaskAssembler::assemble_schedules(pool, task.id).await?;
        card.schedules = schedules;

        // 3. 设置 schedule_status 为 staging
        card.schedule_status = ScheduleStatus::Staging;

        // 4. 填充 recurrence_expiry_behavior
        TaskAssembler::fill_recurrence_expiry_behavior(&mut card, pool).await?;

        Ok(card)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::entities::TaskRow;

    /// 查询所有 staging 任务
    ///
    /// 条件：
    /// - is_deleted = false
    /// - completed_at IS NULL
    /// - archived_at IS NULL
    /// - 今天及未来不存在 schedule（过去的 schedule 不影响）
    /// - 排除 EXPIRE 类型且已过期的循环任务（recurrence_original_date < today）
    ///
    /// # 参数
    /// - `today`: 本地时间的今天日期 (YYYY-MM-DD 格式)
    pub async fn find_staging_tasks(pool: &sqlx::SqlitePool, today: &str) -> AppResult<Vec<Task>> {
        let query = r#"
            SELECT
                t.id, t.title, t.glance_note, t.detail_note, t.estimated_duration,
                t.subtasks, t.sort_positions, t.project_id, t.section_id, t.area_id, t.due_date, t.due_date_type,
                t.completed_at, t.archived_at, t.created_at, t.updated_at, t.deleted_at, t.source_info,
                t.external_source_id, t.external_source_provider, t.external_source_metadata,
                t.recurrence_id, t.recurrence_original_date
            FROM tasks t
            WHERE t.deleted_at IS NULL
              AND t.completed_at IS NULL
              AND t.archived_at IS NULL
              AND NOT EXISTS (
                  SELECT 1 FROM task_schedules ts
                  WHERE ts.task_id = t.id
                    AND ts.scheduled_day >= ?
              )
              AND NOT (
                  -- 排除 EXPIRE 类型且已过期的循环任务
                  t.recurrence_id IS NOT NULL
                  AND t.recurrence_original_date IS NOT NULL
                  AND t.recurrence_original_date < ?
                  AND EXISTS (
                      SELECT 1 FROM task_recurrences tr
                      WHERE tr.id = t.recurrence_id
                        AND tr.expiry_behavior = 'EXPIRE'
                  )
              )
            ORDER BY t.created_at DESC
        "#;

        let rows = sqlx::query_as::<_, TaskRow>(query)
            .bind(today) // 用于 task_schedules.scheduled_day >= ?
            .bind(today) // 用于 recurrence_original_date < ?
            .fetch_all(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
            })?;

        let tasks: Result<Vec<Task>, _> = rows.into_iter().map(Task::try_from).collect();

        tasks.map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::QueryError(e)))
    }
}
