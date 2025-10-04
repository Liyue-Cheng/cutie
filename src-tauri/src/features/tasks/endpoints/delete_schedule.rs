/// 删除任务日程 API - 单文件组件
///
/// DELETE /api/tasks/:id/schedules/:date
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use chrono::{NaiveDate, Utc};
use serde::Serialize;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{TaskCardDto, TimeBlock},
    features::tasks::shared::{
        assemblers::TimeBlockAssembler,
        repositories::{TaskRepository, TaskScheduleRepository, TaskTimeBlockLinkRepository},
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
CABC for `delete_schedule`

## API端点
DELETE /api/tasks/:id/schedules/:date

## 预期行为简介
删除任务在指定日期的日程，并智能清理相关的时间块。

## 业务逻辑
1. 验证任务存在且未删除
2. 验证该日期有日程记录
3. 查找该日期的所有时间块
4. 删除 task_time_block_links
5. 软删除"孤儿"时间片（没有其他任务链接的）
6. 删除 schedule 记录
7. 如果没有任何 schedule 了，更新 schedule_status 为 'staging'
8. 通过 SSE 推送 task.schedule_deleted 事件

## 输入输出规范
- **前置条件**:
  - 任务存在且未删除
  - 该日期有日程记录
- **后置条件**:
  - 删除 task_schedules 记录
  - 删除 task_time_block_links
  - 软删除孤儿 time_blocks
  - 更新 schedule_status（如果需要）

## 边界情况
- 任务不存在 → 404
- 该日期没有日程 → 404
*/

// ==================== 响应结构体 ====================
#[derive(Debug, Serialize)]
pub struct DeleteScheduleResponse {
    pub task_card: TaskCardDto,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path((task_id, date_str)): Path<(Uuid, String)>,
    headers: HeaderMap,
) -> Response {
    let correlation_id = extract_correlation_id(&headers);
    match logic::execute(&app_state, task_id, &date_str, correlation_id).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn parse_date(date_str: &str) -> AppResult<chrono::DateTime<Utc>> {
        let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
            AppError::validation_error(
                "scheduled_day",
                "日期格式错误，请使用 YYYY-MM-DD 格式",
                "INVALID_DATE_FORMAT",
            )
        })?;

        // 🔧 FIX: 直接使用 NaiveDate 转换为 UTC 零点
        use crate::shared::core::utils::time_utils::local_date_to_utc_midnight;
        Ok(local_date_to_utc_midnight(naive_date))
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;
    use crate::features::shared::TransactionHelper;

    pub async fn execute(
        app_state: &AppState,
        task_id: Uuid,
        date_str: &str,
        correlation_id: Option<String>,
    ) -> AppResult<DeleteScheduleResponse> {
        let now = app_state.clock().now_utc();

        // 1. 解析日期
        let scheduled_day = validation::parse_date(date_str)?;

        // 2. 开始事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 3. 查找任务
        let _task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 4. 检查该日期是否有日程
        let has_schedule =
            TaskScheduleRepository::has_schedule_for_day_in_tx(&mut tx, task_id, scheduled_day)
                .await?;

        if !has_schedule {
            return Err(AppError::not_found(
                "Schedule",
                format!("Task {} on {}", task_id, date_str),
            ));
        }

        // 5. 查找该日期的所有 time_blocks
        let time_blocks =
            database::find_time_blocks_for_day(&mut tx, task_id, scheduled_day).await?;

        // 6. 删除 task_time_block_links
        let time_block_ids: Vec<Uuid> = time_blocks.iter().map(|b| b.id).collect();
        for &block_id in &time_block_ids {
            database::delete_task_time_block_link(&mut tx, task_id, block_id).await?;
        }

        // 7. 软删除"孤儿"时间片
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

        // 8. 查询被删除的时间块的完整数据（用于事件）
        let deleted_time_blocks =
            TimeBlockAssembler::assemble_for_event_in_tx(&mut tx, &deleted_time_block_ids).await?;

        // 9. 删除 schedule 记录
        database::delete_schedule(&mut tx, task_id, scheduled_day).await?;

        // 10. 重新查询任务并组装 TaskCard
        // 注意：schedule_status 是派生字段，由装配器根据 task_schedules 表计算
        let updated_task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        let mut task_card = TaskAssembler::task_to_card_basic(&updated_task);

        // 13. ✅ 在事务内填充 schedules 字段
        // ⚠️ 必须在写入 SSE 之前填充，确保 SSE 和 HTTP 返回的数据一致！
        task_card.schedules = TaskAssembler::assemble_schedules_in_tx(&mut tx, task_id).await?;

        // 14. 写入领域事件到 outbox
        use crate::shared::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        let payload = serde_json::json!({
            "task": task_card,
            "deleted_date": date_str,
            "side_effects": {
                "deleted_time_blocks": deleted_time_blocks,
            }
        });

        let mut event = DomainEvent::new(
            "task.schedule_deleted",
            "task",
            task_id.to_string(),
            payload,
        )
        .with_aggregate_version(now.timestamp_millis());

        if let Some(cid) = correlation_id {
            event = event.with_correlation_id(cid);
        }

        outbox_repo.append_in_tx(&mut tx, &event).await?;

        // 15. 提交事务
        TransactionHelper::commit(tx).await?;

        // 16. 返回结果
        Ok(DeleteScheduleResponse { task_card })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;

    /// 查找任务在指定日期的所有时间块
    pub async fn find_time_blocks_for_day(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        scheduled_day: chrono::DateTime<Utc>,
    ) -> AppResult<Vec<TimeBlock>> {
        let query = r#"
            SELECT tb.id, tb.title, tb.glance_note, tb.detail_note, tb.start_time, tb.end_time,
                   tb.area_id, tb.recurrence_rule, tb.recurrence_parent_id, tb.recurrence_original_time,
                   tb.created_at, tb.updated_at, tb.is_deleted
            FROM time_blocks tb
            JOIN task_time_block_links ttbl ON ttbl.time_block_id = tb.id
            WHERE ttbl.task_id = ?
              AND DATE(tb.start_time) = DATE(?)
              AND tb.is_deleted = false
        "#;

        let rows = sqlx::query_as::<_, crate::entities::TimeBlockRow>(query)
            .bind(task_id.to_string())
            .bind(scheduled_day.to_rfc3339())
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

    /// 删除日程记录
    pub async fn delete_schedule(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        scheduled_day: chrono::DateTime<Utc>,
    ) -> AppResult<()> {
        let query = r#"
            DELETE FROM task_schedules
            WHERE task_id = ? AND DATE(scheduled_day) = DATE(?)
        "#;

        sqlx::query(query)
            .bind(task_id.to_string())
            .bind(scheduled_day.to_rfc3339())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(())
    }
}
