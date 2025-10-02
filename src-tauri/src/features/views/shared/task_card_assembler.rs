/// 视图层的 TaskCard 装配助手
/// 封装通用的装配逻辑
use sqlx::SqlitePool;

use crate::{
    entities::{ScheduleStatus, Task, TaskCardDto},
    features::{
        shared::repositories::AreaRepository,
        tasks::shared::{repositories::TaskScheduleRepository, TaskAssembler},
    },
    shared::core::AppResult,
};

pub struct ViewTaskCardAssembler;

impl ViewTaskCardAssembler {
    /// 为 Task 组装完整 TaskCard（包括 area、schedule_status）
    pub async fn assemble_full(task: &Task, pool: &SqlitePool) -> AppResult<TaskCardDto> {
        let mut card = TaskAssembler::task_to_card_basic(task);

        // 判断 schedule_status
        let has_schedule = TaskScheduleRepository::has_any_schedule(pool, task.id).await?;
        card.schedule_status = if has_schedule {
            ScheduleStatus::Scheduled
        } else {
            ScheduleStatus::Staging
        };

        // 获取 area
        if let Some(area_id) = task.area_id {
            card.area = AreaRepository::get_summary(pool, area_id).await?;
        }

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

        // 明确设置 schedule_status
        card.schedule_status = status;

        // 获取 area
        if let Some(area_id) = task.area_id {
            card.area = AreaRepository::get_summary(pool, area_id).await?;
        }

        Ok(card)
    }
}

