/// 任务返回暂存区 API - 单文件组件
///
/// POST /api/tasks/:id/return-to-staging
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{TaskCardDto, TimeBlock},
    features::tasks::shared::{
        assemblers::TimeBlockAssembler,
        repositories::{TaskRepository, TaskTimeBlockLinkRepository},
        TaskAssembler,
    },
    features::time_blocks::shared::repositories::TimeBlockRepository,
    shared::{
        core::{AppError, AppResult},
        http::{error_handler::success_response, extractors::extract_correlation_id},
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `return_to_staging`

## 1. 端点签名 (Endpoint Signature)

POST /api/tasks/{id}/return-to-staging

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我想要重置一个任务的所有未来安排时，我希望系统能够：
> 1. 删除所有今天及未来的日程和时间块
> 2. 保留过去的历史记录（记录我的努力）
> 3. 如果任务已完成，自动重新打开它
> 4. 将任务返回到 Staging 区

### 2.2. 核心业务逻辑 (Core Business Logic)

将任务返回 Staging 区，清理所有今天及未来的安排，但保留过去的历史记录。
具体操作：
1. 删除今天及未来的所有 `task_schedules` 记录
2. 删除今天及未来的所有 `task_time_block_links` 记录
3. 软删除"孤儿"时间块
4. 如果任务已完成，自动重新打开（设置 `completed_at = NULL`）
5. `schedule_status` 变为 `Staging`

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 任务ID

**请求头 (Request Headers):**
- `X-Correlation-ID` (optional): 用于前端去重和请求追踪

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`

```json
{
  "task_card": {
    "id": "uuid",
    "title": "string",
    "schedule_status": "staging",
    "is_completed": false,
    "schedules": null,
    ...
  }
}
```

**注意：** 副作用（删除的时间块）通过 SSE 事件推送。

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
    - **必须**存在于数据库中且未删除。
    - 违反时返回 `404 NOT_FOUND`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  获取当前时间 `now`。
2.  计算"今天"的本地日期（UTC零点表示）（`utc_time_to_local_date_utc_midnight`）。
3.  获取写入许可（`app_state.acquire_write_permit()`）。
4.  启动数据库事务（`TransactionHelper::begin`）。
5.  查询任务（`TaskRepository::find_by_id_in_tx`）。
6.  如果任务不存在，返回 404 错误。
7.  查找今天及未来的所有时间块（`database::find_future_time_blocks`）。
8.  对每个时间块，删除任务到时间块的链接（`database::delete_task_time_block_link`）。
9.  对每个时间块，检查是否变成"孤儿"（`TaskTimeBlockLinkRepository::count_remaining_tasks_in_block_in_tx`）。
10. 如果时间块没有剩余任务，软删除该时间块（`TimeBlockRepository::soft_delete_in_tx`）。
11. 在删除之前，查询被删除的时间块的完整数据（用于 SSE 事件）。
12. 删除今天及未来的所有日程记录（`database::delete_future_schedules`）。
13. 如果任务已完成，重新打开它（`TaskRepository::set_reopened_in_tx`）。
14. 重新查询任务并组装 `TaskCardDto`。
15. 在事务内填充 `schedules` 字段。
16. 设置 `schedule_status` 为 `Staging`（因为已删除所有未来日程）。
17. 写入领域事件到 outbox（`task.returned_to_staging` 事件）。
18. 提交事务（`TransactionHelper::commit`）。
19. 返回更新后的任务。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **任务没有任何日程和时间块:** 返回成功（幂等操作）。
- **任务已在 Staging 区:** 返回成功（幂等操作）。
- **任务已完成:** 自动重新打开（`completed_at` 设为 NULL）。
- **只有过去的日程:** 保留过去的日程，只删除今天及未来的。
- **时间块还有其他任务:** 不删除时间块（避免影响其他任务）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次查询 `tasks` 表（验证任务存在）。
    - **`SELECT`:** 1次查询 `time_blocks` 表（查找今天及未来的时间块）。
    - **`DELETE`:** 0-N 条记录在 `task_time_block_links` 表。
    - **`SELECT`:** 0-N 次查询 `task_time_block_links` 表（检查孤儿状态）。
    - **`UPDATE`:** 0-N 条记录在 `time_blocks` 表（软删除孤儿时间块）。
    - **`DELETE`:** 0-N 条记录在 `task_schedules` 表（删除今天及未来的日程）。
    - **`UPDATE`:** 0-1 条记录在 `tasks` 表（如果已完成，重新打开）。
    - **`SELECT`:** 1次查询 `tasks` 表（重新获取数据）。
    - **`SELECT`:** 1次查询 `task_schedules` 表（填充 schedules）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `task.returned_to_staging` 事件，包含：
        - 更新后的任务（`TaskCardDto`）
        - 副作用：删除的时间块列表（`TimeBlockViewDto[]`）
- **日志记录:**
    - 记录删除的孤儿时间块 ID。
    - 失败时，记录详细错误信息。

*（无其他已知副作用）*
*/

// ==================== 响应结构体 ====================
#[derive(Debug, Serialize)]
pub struct ReturnToStagingResponse {
    pub task_card: TaskCardDto,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    let correlation_id = extract_correlation_id(&headers);
    match logic::execute(&app_state, task_id, correlation_id).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;
    use crate::features::shared::TransactionHelper;

    pub async fn execute(
        app_state: &AppState,
        task_id: Uuid,
        correlation_id: Option<String>,
    ) -> AppResult<ReturnToStagingResponse> {
        let now = app_state.clock().now_utc();

        // 1. 计算"今天"的本地日期（UTC零点表示）
        // 🔧 FIX: 使用系统时区提取本地日期
        use crate::shared::core::utils::time_utils::utc_time_to_local_date_utc_midnight;
        let today_utc = utc_time_to_local_date_utc_midnight(now);

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 2. 开始事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 3. 查找任务
        let task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 4. 查找今天及未来的所有时间块
        let time_blocks = database::find_future_time_blocks(&mut tx, task_id, today_utc).await?;

        // 5. 删除 task_time_block_links
        let time_block_ids: Vec<Uuid> = time_blocks.iter().map(|b| b.id).collect();
        for &block_id in &time_block_ids {
            database::delete_task_time_block_link(&mut tx, task_id, block_id).await?;
        }

        // 6. 软删除"孤儿"时间片
        let mut deleted_time_block_ids = Vec::new();
        for block in &time_blocks {
            let remaining_links =
                TaskTimeBlockLinkRepository::count_remaining_tasks_in_block_in_tx(
                    &mut tx, block.id,
                )
                .await?;

            if remaining_links == 0 {
                TimeBlockRepository::soft_delete_in_tx(&mut tx, block.id).await?;
                deleted_time_block_ids.push(block.id);
            }
        }

        // 7. 查询被删除的时间块的完整数据（用于事件）
        let deleted_time_blocks =
            TimeBlockAssembler::assemble_for_event_in_tx(&mut tx, &deleted_time_block_ids).await?;

        // 8. 删除今天及未来的所有 schedules
        database::delete_future_schedules(&mut tx, task_id, today_utc).await?;

        // 9. 如果任务已完成，重新打开它
        if task.completed_at.is_some() {
            TaskRepository::set_reopened_in_tx(&mut tx, task_id, now).await?;
        }

        // 10. 重新查询任务并组装 TaskCard
        // 注意：schedule_status 是派生字段，由装配器根据 task_schedules 表计算
        let updated_task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        let mut task_card = TaskAssembler::task_to_card_basic(&updated_task);

        // 11. ✅ 在事务内填充 schedules 字段
        // ⚠️ 必须在写入 SSE 之前填充，确保 SSE 和 HTTP 返回的数据一致！
        task_card.schedules = TaskAssembler::assemble_schedules_in_tx(&mut tx, task_id).await?;

        // 11.5. ✅ 根据 schedules 设置正确的 schedule_status
        // staging 定义：今天和未来没有排期的任务，过去的排期不影响
        // 返回暂存区操作删除了所有未来排期，所以这里应该是 Staging
        use crate::entities::ScheduleStatus;
        task_card.schedule_status = ScheduleStatus::Staging;

        // 12. 写入领域事件到 outbox
        use crate::shared::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        let payload = serde_json::json!({
            "task": task_card,
            "side_effects": {
                "deleted_time_blocks": deleted_time_blocks,
            }
        });

        let mut event = DomainEvent::new(
            "task.returned_to_staging",
            "task",
            task_id.to_string(),
            payload,
        )
        .with_aggregate_version(now.timestamp_millis());

        if let Some(cid) = correlation_id {
            event = event.with_correlation_id(cid);
        }

        outbox_repo.append_in_tx(&mut tx, &event).await?;

        // 13. 提交事务
        TransactionHelper::commit(tx).await?;

        // 14. 返回结果
        Ok(ReturnToStagingResponse { task_card })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;

    /// 查找任务在今天及未来的所有时间块
    pub async fn find_future_time_blocks(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        today: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<Vec<TimeBlock>> {
        let query = r#"
            SELECT tb.id, tb.title, tb.glance_note, tb.detail_note, tb.start_time, tb.end_time,
                   tb.area_id, tb.recurrence_rule, tb.recurrence_parent_id, tb.recurrence_original_time,
                   tb.created_at, tb.updated_at, tb.is_deleted
            FROM time_blocks tb
            JOIN task_time_block_links ttbl ON ttbl.time_block_id = tb.id
            WHERE ttbl.task_id = ?
              AND DATE(tb.start_time) >= DATE(?)
              AND tb.is_deleted = false
        "#;

        let rows = sqlx::query_as::<_, crate::entities::TimeBlockRow>(query)
            .bind(task_id.to_string())
            .bind(today.to_rfc3339())
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        let time_blocks = rows
            .into_iter()
            .map(|row| {
                TimeBlock::try_from(row).map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::QueryError(e))
                })
            })
            .collect::<AppResult<Vec<TimeBlock>>>()?;

        Ok(time_blocks)
    }

    /// 删除任务到时间块的链接
    pub async fn delete_task_time_block_link(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        time_block_id: Uuid,
    ) -> AppResult<()> {
        let query = r#"
            DELETE FROM task_time_block_links
            WHERE task_id = ? AND time_block_id = ?
        "#;

        sqlx::query(query)
            .bind(task_id.to_string())
            .bind(time_block_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(())
    }

    /// 删除今天及未来的所有 schedules
    pub async fn delete_future_schedules(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        today: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<()> {
        let query = r#"
            DELETE FROM task_schedules
            WHERE task_id = ? AND DATE(scheduled_day) >= DATE(?)
        "#;

        sqlx::query(query)
            .bind(task_id.to_string())
            .bind(today.to_rfc3339())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(())
    }
}
