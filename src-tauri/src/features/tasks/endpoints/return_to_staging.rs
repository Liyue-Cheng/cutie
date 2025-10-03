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

## API端点
POST /api/tasks/:id/return-to-staging

## 预期行为简介
将任务返回暂存区，删除所有未来的日程和时间块，保留历史记录。

## 业务逻辑
1. 验证任务存在且未删除
2. 计算"今天"的UTC日期
3. 查找今天及未来的所有时间块
4. 删除 task_time_block_links
5. 软删除"孤儿"时间片
6. 删除今天及未来的所有 schedules
7. 更新任务状态：
   - schedule_status = 'staging'
   - is_completed = false（如果已完成）
   - completed_at = null（如果已完成）
8. 通过 SSE 推送 task.returned_to_staging 事件

## 输入输出规范
- **前置条件**:
  - 任务存在且未删除
- **后置条件**:
  - 删除今天及未来的 task_schedules
  - 删除今天及未来的 task_time_block_links
  - 软删除孤儿 time_blocks
  - 更新任务状态为 staging
  - 重新打开已完成的任务

## 边界情况
- 任务不存在 → 404
- 即使没有任何 schedule/time_block，也返回成功
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

        // 1. 计算"今天"的UTC日期（零点）
        // TODO: 未来实现用户配置后，考虑 day_start_hour
        let today_utc = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();

        // 2. 开始事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 3. 查找任务
        let _task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
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

        // 9. 更新任务状态为 staging 并重新打开任务
        database::return_to_staging(&mut tx, task_id, now).await?;

        // 10. 重新查询任务并组装 TaskCard
        let updated_task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        let task_card = TaskAssembler::task_to_card_basic(&updated_task);

        // 11. 写入领域事件到 outbox
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

        // 12. 提交事务
        TransactionHelper::commit(tx).await?;

        // 13. 返回结果
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

    /// 更新任务状态为 staging 并重新打开任务
    pub async fn return_to_staging(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        updated_at: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<()> {
        let query = r#"
            UPDATE tasks
            SET schedule_status = 'staging',
                is_completed = false,
                completed_at = null,
                updated_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(updated_at.to_rfc3339())
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(())
    }
}
