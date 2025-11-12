/// AI 自动分类服务
///
/// 为任务提供异步的 AI 自动分类功能
use crate::{
    entities::ScheduleStatus,
    features::{
        ai::shared::{classify_task_area, load_quick_model_config, AreaOption},
        shared::TransactionHelper,
    },
    infra::{
        core::{AppError, AppResult},
        events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        },
    },
};
use chrono::Utc;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::features::shared::{repositories::TaskRepository, TaskAssembler};

/// AI 自动分类服务
pub struct AiClassificationService;

impl AiClassificationService {
    /// 异步分类并更新任务
    ///
    /// 此函数应该在任务创建后异步调用，不阻塞 HTTP 响应
    pub async fn classify_and_update_task(
        task_id: Uuid,
        task_title: &str,
        pool: &SqlitePool,
    ) -> AppResult<()> {
        tracing::info!(
            target: "SERVICE:TASKS:auto_classify",
            task_id = %task_id,
            title = %task_title,
            "Starting AI classification"
        );

        // 1. 获取所有可用的 Area
        let areas = Self::fetch_available_areas(pool).await?;

        if areas.is_empty() {
            tracing::debug!(
                target: "SERVICE:TASKS:auto_classify",
                "No areas available, skipping classification"
            );
            return Ok(());
        }

        tracing::debug!(
            target: "SERVICE:TASKS:auto_classify",
            area_count = areas.len(),
            "Fetched available areas"
        );

        // 2. 加载快速模型配置
        let quick_model_config = load_quick_model_config(pool).await?;

        // 3. 调用 AI 分类（带超时）
        let area_id = classify_task_area(task_title, &areas, &quick_model_config).await?;

        // 4. 如果 AI 返回了 area_id，更新任务并发送 SSE 事件
        if let Some(area_id) = area_id {
            tracing::info!(
                target: "SERVICE:TASKS:auto_classify",
                task_id = %task_id,
                area_id = %area_id,
                "AI suggested area, updating task"
            );

            Self::update_task_area_with_event(pool, task_id, area_id).await?;

            tracing::info!(
                target: "SERVICE:TASKS:auto_classify",
                task_id = %task_id,
                area_id = %area_id,
                "Task area updated successfully"
            );
        } else {
            tracing::debug!(
                target: "SERVICE:TASKS:auto_classify",
                task_id = %task_id,
                "AI did not suggest an area"
            );
        }

        Ok(())
    }

    /// 获取所有可用的 Area
    async fn fetch_available_areas(pool: &SqlitePool) -> AppResult<Vec<AreaOption>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name
            FROM areas
            WHERE is_deleted = FALSE
            ORDER BY name
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?;

        let areas = rows
            .into_iter()
            .filter_map(|row| {
                let id: String = row.get("id");
                let name: String = row.get("name");
                let id = Uuid::parse_str(&id).ok()?;
                Some(AreaOption { id, name })
            })
            .collect();

        Ok(areas)
    }

    /// 更新任务的 area_id 并发送 SSE 事件
    async fn update_task_area_with_event(
        pool: &SqlitePool,
        task_id: Uuid,
        area_id: Uuid,
    ) -> AppResult<()> {
        let mut tx = TransactionHelper::begin(pool).await?;
        let now = Utc::now();

        // 1. 更新任务的 area_id
        sqlx::query(
            r#"
            UPDATE tasks
            SET area_id = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(area_id.to_string())
        .bind(now.to_rfc3339())
        .bind(task_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?;

        // 2. 重新查询任务以获取最新数据
        let task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 3. 组装 TaskCardDto
        let mut task_card = TaskAssembler::task_to_card_basic(&task);

        // 5. 在事务内填充 schedules 字段
        task_card.schedules = TaskAssembler::assemble_schedules_in_tx(&mut tx, task_id).await?;

        // 6. 根据 schedules 设置正确的 schedule_status
        let local_today = now.date_naive();

        let has_future_schedule = task_card
            .schedules
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
            ScheduleStatus::Scheduled
        } else {
            ScheduleStatus::Staging
        };

        // 7. 写入领域事件到 outbox
        let outbox_repo = SqlxEventOutboxRepository::new(pool.clone());

        let payload = serde_json::json!({
            "task": task_card,
            "side_effects": {
                "ai_classified": true,
                "area_id": area_id.to_string(),
            }
        });

        let event = DomainEvent::new("task.updated", "task", task_id.to_string(), payload)
            .with_aggregate_version(now.timestamp_millis());

        outbox_repo.append_in_tx(&mut tx, &event).await?;

        // 8. 提交事务
        TransactionHelper::commit(tx).await?;

        Ok(())
    }
}
