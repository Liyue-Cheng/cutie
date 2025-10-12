/// 视图层的 TaskCard 装配助手
/// 封装通用的装配逻辑
use sqlx::SqlitePool;

use crate::{
    entities::{ScheduleStatus, Task, TaskCardDto},
    features::shared::TaskAssembler,
    infra::core::AppResult,
};

pub struct ViewTaskCardAssembler;

impl ViewTaskCardAssembler {
    /// 为 Task 组装完整 TaskCard（包含 schedules 和 time_blocks）
    pub async fn assemble_full(task: &Task, pool: &SqlitePool) -> AppResult<TaskCardDto> {
        let mut card = TaskAssembler::task_to_card_basic(task);

        // 组装完整的 schedules（包含 time_blocks）
        let schedules = TaskAssembler::assemble_schedules(pool, task.id).await?;

        // 根据 schedules 设置 schedule_status
        card.schedule_status = if schedules.is_some() {
            ScheduleStatus::Scheduled
        } else {
            ScheduleStatus::Staging
        };

        card.schedules = schedules;

        Ok(card)
    }

    /// 批量组装 TaskCards
    pub async fn assemble_batch(
        tasks: Vec<Task>,
        pool: &SqlitePool,
    ) -> AppResult<Vec<TaskCardDto>> {
        let mut task_cards = Vec::new();
        for task in tasks {
            let task_card = Self::assemble_full(&task, pool).await?;
            task_cards.push(task_card);
        }
        Ok(task_cards)
    }

    /// 组装 TaskCard 并明确设置 schedule_status（用于 planned 和 staging 视图）
    pub async fn assemble_with_status(
        task: &Task,
        pool: &SqlitePool,
        status: ScheduleStatus,
    ) -> AppResult<TaskCardDto> {
        let mut card = TaskAssembler::task_to_card_basic(task);

        // 组装完整的 schedules（包含 time_blocks）
        let schedules = TaskAssembler::assemble_schedules(pool, task.id).await?;

        // 明确设置 schedule_status
        card.schedule_status = status;
        card.schedules = schedules;

        Ok(card)
    }
}
