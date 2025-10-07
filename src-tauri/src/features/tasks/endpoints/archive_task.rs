/// 归档任务 API - 单文件组件
///
/// 归档任务，使其不在常规视图中显示
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use serde::Serialize;

use crate::{
    entities::TaskCardDto,
    features::tasks::shared::{repositories::TaskRepository, TaskAssembler},
    shared::{
        core::{AppError, AppResult},
        http::{error_handler::success_response, extractors::extract_correlation_id},
    },
    startup::AppState,
};

/// 归档任务的响应
#[derive(Debug, Serialize)]
pub struct ArchiveTaskResponse {
    pub task: TaskCardDto,
}

// ==================== 文档层 ====================
/*
CABC for `archive_task`

## 1. 端点签名 (Endpoint Signature)

POST /api/tasks/{id}/archive

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我归档一个任务时，我希望系统能够：
> 1. 标记任务为已归档
> 2. 该任务不再在常规视图中显示
> 3. 保留所有任务数据（日程、时间块等）以供将来查看

### 2.2. 核心业务逻辑 (Core Business Logic)

归档任务，设置 `archived_at` 时间戳。归档的任务：
- 不会出现在任何常规看板视图中（staging、daily、calendar等）
- **删除当天及之后的所有日程和关联的时间块**（过去的日程保留）
- 可以通过取消归档恢复（但已删除的日程不会自动恢复）
- 可以在归档视图中查看

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
  "task": {
    "id": "uuid",
    "title": "string",
    "is_archived": true,
    "archived_at": "2025-10-05T12:00:00Z",
    ...
  }
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Task not found: {id}"
}
```

**409 Conflict:**

```json
{
  "error_code": "CONFLICT",
  "message": "任务已经归档"
}
```

## 4. 验证规则 (Validation Rules)

- `task_id`:
    - **必须**是有效的 UUID 格式。
    - **必须**存在于数据库中。
    - 违反时返回 `404 NOT_FOUND`
- **业务规则验证:**
    - 任务**不能**已经归档（`archived_at IS NOT NULL`）。
    - 违反时返回 `409 CONFLICT`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1. 获取当前时间 `now`。
2. 获取写入许可（`app_state.acquire_write_permit()`）。
3. 启动数据库事务（`TransactionHelper::begin`）。
4. 查询任务（`TaskRepository::find_by_id_in_tx`）。
5. 如果任务不存在，返回 404 错误。
6. 检查任务是否已归档，如果是，返回 409 冲突。
7. **删除当天及之后的所有日程**：
   - 查找所有当天及之后的日程日期
   - 对每个日期：
     - 查找该日期的所有时间块
     - 删除任务到时间块的链接（`task_time_block_links`）
     - 软删除"孤儿"时间块（没有其他任务关联的时间块）
   - 删除所有当天及之后的日程记录（`task_schedules`）
8. 更新任务的 `archived_at` 字段。
9. 重新查询任务并组装 `TaskCardDto`。
10. 填充 `schedules` 字段（只剩下过去的日程）。
11. 根据 schedules 设置正确的 `schedule_status`（通常为 `Staging`）。
12. 写入领域事件到 outbox。
13. 提交事务。
14. 返回归档后的任务。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **任务已归档:** 返回 `409` 冲突（幂等性保护）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次查询任务。
    - **`SELECT`:** 1次查询当天及之后的日程日期。
    - **`SELECT`:** N次查询时间块（N = 未来日程数量）。
    - **`DELETE`:** 0-M条记录在 `task_time_block_links` 表（M = 关联的时间块数量）。
    - **`UPDATE`:** 0-M条记录在 `time_blocks` 表（软删除孤儿时间块）。
    - **`DELETE`:** 0-N条记录在 `task_schedules` 表（删除当天及之后的日程）。
    - **`UPDATE`:** 1条记录在 `tasks` 表（设置 `archived_at`）。
    - **`SELECT`:** 1次重新查询任务。
    - **`SELECT`:** 1次查询剩余的 schedules。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `task.archived` 事件，包含归档的任务（`TaskCardDto`）
    - 注意：删除的日程和时间块不会单独发送事件，因为任务已归档不会再显示

*（无其他已知副作用）*
*/

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
    ) -> AppResult<ArchiveTaskResponse> {
        let now = app_state.clock().now_utc();

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 开始事务（✅ 使用 TransactionHelper）
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 1. 查找任务（✅ 使用共享 Repository）
        let task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 2. 检查是否已归档
        if task.archived_at.is_some() {
            return Err(AppError::conflict("任务已经归档"));
        }

        // 3. 删除当天及之后的所有日程（包括关联的时间块）
        let today = now.date_naive();
        database::delete_today_and_future_schedules_in_tx(&mut tx, task_id, today).await?;

        // 4. 更新任务的 archived_at 字段
        database::set_archived_in_tx(&mut tx, task_id, now).await?;

        // 5. 重新查询任务并组装完整 TaskCard
        let updated_task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;
        let mut task_card = TaskAssembler::task_to_card_basic(&updated_task);

        // 6. ✅ 在事务内填充 schedules 字段
        // ⚠️ 必须在写入 SSE 之前填充，确保 SSE 和 HTTP 返回的数据一致！
        task_card.schedules = TaskAssembler::assemble_schedules_in_tx(&mut tx, task_id).await?;

        // 7. ✅ 根据 schedules 设置正确的 schedule_status
        use crate::entities::ScheduleStatus;
        use chrono::Utc;
        let today = Utc::now().date_naive();

        let has_future_schedule = task_card
            .schedules
            .as_ref()
            .map(|schedules| {
                schedules.iter().any(|s| {
                    if let Ok(schedule_date) =
                        chrono::NaiveDate::parse_from_str(&s.scheduled_day, "%Y-%m-%d")
                    {
                        schedule_date >= today
                    } else {
                        false
                    }
                })
            })
            .unwrap_or(false);

        task_card.schedule_status = if has_future_schedule {
            ScheduleStatus::Scheduled
        } else {
            ScheduleStatus::Staging
        };

        // 8. 在事务中写入领域事件到 outbox
        use crate::shared::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        {
            let payload = serde_json::json!({
                "task": task_card,
            });
            let mut event = DomainEvent::new("task.archived", "task", task_id.to_string(), payload)
                .with_aggregate_version(now.timestamp_millis());

            if let Some(cid) = correlation_id {
                event = event.with_correlation_id(cid);
            }

            outbox_repo.append_in_tx(&mut tx, &event).await?;
        }

        // 9. 提交事务（✅ 使用 TransactionHelper）
        TransactionHelper::commit(tx).await?;

        // 10. 返回结果
        Ok(ArchiveTaskResponse { task: task_card })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::features::tasks::shared::repositories::TaskTimeBlockLinkRepository;
    use crate::features::time_blocks::shared::TimeBlockRepository;
    use chrono::{DateTime, NaiveDate, Utc};
    use sqlx::{Sqlite, Transaction};

    /// 删除当天及之后的所有日程（包括清理时间块）
    pub async fn delete_today_and_future_schedules_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        today: NaiveDate,
    ) -> AppResult<()> {
        // 1. 查找当天及之后的所有日程日期
        let future_dates = find_future_schedule_dates(tx, task_id, today).await?;

        // 2. 对每个日期，清理时间块链接和孤儿时间块
        for scheduled_day in future_dates {
            // 查找该日期的所有时间块
            let time_blocks = find_time_blocks_for_day(tx, task_id, scheduled_day).await?;

            // 删除时间块链接
            for block in &time_blocks {
                delete_task_time_block_link(tx, task_id, block.id).await?;

                // 检查是否成为孤儿时间块
                let remaining_links =
                    TaskTimeBlockLinkRepository::count_remaining_tasks_in_block_in_tx(tx, block.id)
                        .await?;

                if remaining_links == 0 {
                    TimeBlockRepository::soft_delete_in_tx(tx, block.id).await?;
                }
            }
        }

        // 3. 删除所有当天及之后的日程记录
        let query = r#"
            DELETE FROM task_schedules
            WHERE task_id = ? AND DATE(scheduled_day) >= DATE(?)
        "#;

        sqlx::query(query)
            .bind(task_id.to_string())
            .bind(today.and_hms_opt(0, 0, 0).unwrap().and_utc().to_rfc3339())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(())
    }

    /// 查找当天及之后的日程日期
    async fn find_future_schedule_dates(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        today: NaiveDate,
    ) -> AppResult<Vec<DateTime<Utc>>> {
        let query = r#"
            SELECT scheduled_day
            FROM task_schedules
            WHERE task_id = ? AND DATE(scheduled_day) >= DATE(?)
            ORDER BY scheduled_day ASC
        "#;

        let rows = sqlx::query_as::<_, (String,)>(query)
            .bind(task_id.to_string())
            .bind(today.and_hms_opt(0, 0, 0).unwrap().and_utc().to_rfc3339())
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        let dates = rows
            .into_iter()
            .filter_map(|(date_str,)| {
                chrono::DateTime::parse_from_rfc3339(&date_str)
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
            })
            .collect();

        Ok(dates)
    }

    /// 查找某天的所有时间块
    async fn find_time_blocks_for_day(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        scheduled_day: DateTime<Utc>,
    ) -> AppResult<Vec<TimeBlockInfo>> {
        let query = r#"
            SELECT DISTINCT tb.id
            FROM time_blocks tb
            INNER JOIN task_time_block_links ttbl ON tb.id = ttbl.time_block_id
            WHERE ttbl.task_id = ?
              AND DATE(tb.start_time) = DATE(?)
              AND tb.is_deleted = false
        "#;

        let rows = sqlx::query_as::<_, (String,)>(query)
            .bind(task_id.to_string())
            .bind(scheduled_day.to_rfc3339())
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        let blocks = rows
            .into_iter()
            .filter_map(|(id_str,)| {
                let id = Uuid::parse_str(&id_str).ok()?;
                Some(TimeBlockInfo { id })
            })
            .collect();

        Ok(blocks)
    }

    /// 删除任务到时间块的链接
    async fn delete_task_time_block_link(
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

    /// 时间块简要信息
    struct TimeBlockInfo {
        id: Uuid,
    }

    /// 设置任务为已归档
    pub async fn set_archived_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        archived_at: DateTime<Utc>,
    ) -> AppResult<()> {
        let query = r#"
            UPDATE tasks
            SET archived_at = ?, updated_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(archived_at)
            .bind(archived_at)
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(())
    }
}
