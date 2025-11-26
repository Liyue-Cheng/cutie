/// 获取所有未完成任务 API - 单文件组件
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use chrono::NaiveDate;

use crate::{
    entities::{Task, TaskCardDto},
    features::shared::{RecurrenceInstantiationService, TaskAssembler},
    infra::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `get_all_incomplete`

## 1. 端点签名 (Endpoint Signature)

GET /api/views/all-incomplete

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要查看所有未完成任务的列表（无论是否已排期），
> 以便我能专注于需要处理的待办事项，而不被已完成的任务干扰。

### 2.2. 核心业务逻辑 (Core Business Logic)

从数据库中查询所有未删除且未完成的任务（不限制排期状态）。
为每个任务组装完整的 TaskCardDto（包含 schedules、time_blocks 和 area 信息），
并根据实际 schedules 情况动态设置 schedule_status。

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
    "schedule_status": "staging" | "scheduled",
    "is_completed": false,
    "area": { "id": "uuid", "name": "string", "color": "#RRGGBB" } | null,
    "schedules": [...] | null,
    "due_date": { "date": "ISO8601", "type": "deadline" | "scheduled" } | null,
    "has_detail_note": boolean
  },
  ...
]
```

**注意：**
- 空列表返回 `[]`，而不是错误。
- 响应中所有任务的 `is_completed` 均为 `false`。

## 4. 验证规则 (Validation Rules)

- 无输入参数，无需验证。
- 查询条件：
  - `is_deleted = false`（排除已删除任务）
  - `completed_at IS NULL`（排除已完成任务）

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `database::find_all_incomplete_tasks` 查询数据库：
    - 查询 `tasks` 表，过滤 `is_deleted = false` 和 `completed_at IS NULL`
    - 按 `created_at DESC` 排序（最新创建的在前）
2.  遍历每个任务，调用 `assemble_task_card` 进行组装：
    - 调用 `TaskAssembler::task_to_card_basic` 创建基础 TaskCard
    - 调用 `TaskAssembler::assemble_schedules` 查询完整的 schedules（包含 time_blocks）
    - schedule_status 已删除，前端根据 schedules 字段实时计算
3.  对任务列表按 `id` 降序排序（保证稳定的显示顺序）。
4.  返回 `200 OK` 和任务列表（`Vec<TaskCardDto>`）。

## 6. 边界情况 (Edge Cases)

- **数据库中没有未完成任务:** 返回空数组 `[]`（200 OK）。
- **所有任务都已完成或已删除:** 返回空数组 `[]`（200 OK）。
- **任务数量很大:** 当前无分页机制，可能返回大量数据（性能考虑，建议添加分页）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库查询:**
    - **`SELECT`:** 1次，查询 `tasks` 表（过滤 `is_deleted = false` 和 `completed_at IS NULL`，按 `created_at DESC` 排序）。
    - **`SELECT`:** N次（N = 未完成任务数量），每个任务查询完整的 schedules。
    - **`SELECT`:** 0-M次（M = schedules 总数），查询 `time_blocks` 表（每个 schedule 可能有时间块）。
    - **`SELECT`:** 0-N次，查询 `areas` 表（如果任务有 area_id）。
- **无写操作:** 此端点为只读查询，不修改任何数据。
- **无 SSE 事件:** 不发送任何事件。
- **日志记录:**
    - 失败时（数据库错误），以 `ERROR` 级别记录详细错误信息。

*（无其他已知副作用）*

**性能考虑：**
1. 当前实现会一次性返回所有未完成任务，没有分页机制。
2. 如果未完成任务数量超过数百个，建议添加分页参数（limit/offset 或 cursor-based）。
3. 考虑添加客户端缓存或 SSE 订阅机制，减少重复查询。
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
    use crate::features::shared::TaskRecurrenceRepository;
    use rrule::RRuleSet;

    pub async fn execute(app_state: &AppState) -> AppResult<Vec<TaskCardDto>> {
        let pool = app_state.db_pool();

        // 1. 为所有活跃的循环规则实例化未来最近的一个实例
        instantiate_next_recurrence_instances(app_state).await?;

        // 2. 获取所有未完成任务（包括刚实例化的循环任务）
        let tasks = database::find_all_incomplete_tasks(pool).await?;

        // 3. 为每个任务组装 TaskCardDto
        let mut task_cards = Vec::new();
        for task in tasks {
            let task_card = assemble_task_card(&task, pool).await?;
            task_cards.push(task_card);
        }

        // 4. 按 created_at 倒序（最新的在前）
        task_cards.sort_by(|a, b| b.id.cmp(&a.id));

        Ok(task_cards)
    }

    /// 为所有活跃的循环规则实例化未来最近的一个实例
    async fn instantiate_next_recurrence_instances(app_state: &AppState) -> AppResult<()> {
        let pool = app_state.db_pool();
        let today = app_state.clock().now_utc().date_naive();

        // 1. 获取所有活跃的循环规则
        let recurrences = TaskRecurrenceRepository::find_all_active(pool).await?;

        tracing::info!(
            "🔄 [ALL_INCOMPLETE] Found {} active recurrences to check",
            recurrences.len()
        );

        // 2. 对每个循环规则，找到未来最近的实例日期并实例化
        for recurrence in recurrences {
            // 计算下一个实例日期
            if let Some(next_date) = find_next_occurrence(&recurrence, &today) {
                tracing::debug!(
                    "🔄 [ALL_INCOMPLETE] Recurrence {} next occurrence: {}",
                    recurrence.id,
                    next_date
                );

                // 实例化该日期
                let _ = RecurrenceInstantiationService::instantiate_for_date(
                    pool,
                    app_state.id_generator().as_ref(),
                    app_state.clock().as_ref(),
                    &next_date,
                )
                .await;
            }
        }

        Ok(())
    }

    /// 计算循环规则的下一个实例日期（从今天开始）
    fn find_next_occurrence(
        recurrence: &crate::entities::TaskRecurrence,
        from_date: &NaiveDate,
    ) -> Option<NaiveDate> {
        // 确定 DTSTART
        let dtstart_date = recurrence.start_date.clone().unwrap_or_else(|| {
            crate::infra::core::utils::time_utils::format_date_yyyy_mm_dd(
                &recurrence.created_at.date_naive(),
            )
        });

        // 构建完整的 RRULE 字符串
        let start_date_rrule = dtstart_date.replace("-", "");
        let full_rrule = format!("DTSTART:{}\nRRULE:{}", start_date_rrule, recurrence.rule);

        // 解析 RRULE
        let rrule_set: RRuleSet = match full_rrule.parse() {
            Ok(set) => set,
            Err(e) => {
                tracing::warn!(
                    "Failed to parse RRULE for recurrence {}: {:?}",
                    recurrence.id,
                    e
                );
                return None;
            }
        };

        // 找到从今天开始的第一个实例
        for occurrence in rrule_set.into_iter().take(1000) {
            let occ_date = occurrence.date_naive();
            if occ_date >= *from_date {
                return Some(occ_date);
            }
        }

        None
    }

    /// 组装单个任务的 TaskCard（包含完整的 schedules + time_blocks）
    ///
    /// schedule_status 已删除 - 前端根据 schedules 字段实时计算
    async fn assemble_task_card(task: &Task, pool: &sqlx::SqlitePool) -> AppResult<TaskCardDto> {
        let mut card = TaskAssembler::task_to_card_basic(task);

        // 组装完整的 schedules（包含 time_blocks）
        let schedules = TaskAssembler::assemble_schedules(pool, task.id).await?;
        card.schedules = schedules;

        Ok(card)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::entities::TaskRow;

    pub async fn find_all_incomplete_tasks(pool: &sqlx::SqlitePool) -> AppResult<Vec<Task>> {
        let query = r#"
            SELECT
                id, title, glance_note, detail_note, estimated_duration,
                subtasks, sort_positions, project_id, section_id, area_id, due_date, due_date_type, completed_at, archived_at,
                created_at, updated_at, deleted_at, source_info,
                external_source_id, external_source_provider, external_source_metadata,
                recurrence_id, recurrence_original_date
            FROM tasks
            WHERE deleted_at IS NULL AND completed_at IS NULL AND archived_at IS NULL
            ORDER BY created_at DESC
        "#;

        let rows = sqlx::query_as::<_, TaskRow>(query)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
            })?;

        let tasks: Result<Vec<Task>, _> = rows.into_iter().map(Task::try_from).collect();

        tasks.map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::QueryError(e)))
    }
}
