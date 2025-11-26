/// 视图层的 TaskCard 装配助手
/// 封装通用的装配逻辑
use sqlx::SqlitePool;

use crate::{
    entities::{Task, TaskCardDto},
    features::shared::TaskAssembler,
    infra::core::AppResult,
};

pub struct ViewTaskCardAssembler;

impl ViewTaskCardAssembler {
    /// 为 Task 组装完整 TaskCard（包含 schedules 和 time_blocks）
    ///
    /// schedule_status 已删除 - 前端根据 schedules 字段实时计算：
    /// - schedules 为 Some(_) 且非空 => scheduled
    /// - schedules 为 None 或空 => staging
    pub async fn assemble_full(task: &Task, pool: &SqlitePool) -> AppResult<TaskCardDto> {
        let mut card = TaskAssembler::task_to_card_basic(task);

        // 组装完整的 schedules（包含 time_blocks）
        let schedules = TaskAssembler::assemble_schedules(pool, task.id).await?;
        card.schedules = schedules;

        // 填充 recurrence_expiry_behavior
        TaskAssembler::fill_recurrence_expiry_behavior(&mut card, pool).await?;

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

        // 批量填充 recurrence_expiry_behavior（优化性能）
        TaskAssembler::fill_recurrence_expiry_behavior_batch(&mut task_cards, pool).await?;

        Ok(task_cards)
    }
}
