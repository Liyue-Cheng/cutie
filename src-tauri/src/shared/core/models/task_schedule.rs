use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Outcome;

/// TaskSchedule (任务日程) 实体定义
///
/// 一条关联记录，精确定义了一个Task在哪一天被安排，以及它在那一天的最终"结局"。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskSchedule {
    /// 日程ID (主键)
    pub id: Uuid,

    /// 任务ID (外键)
    ///
    /// **不变量:** 必须永远指向一个有效的、未被物理删除的Task
    pub task_id: Uuid,

    /// 安排日期
    ///
    /// **前置条件:** 必须是当日的零点零分零秒（UTC）
    pub scheduled_day: DateTime<Utc>,

    /// 结局
    ///
    /// **后置条件:** 忠实记录当日的结局，不应被未来发生的事件（如任务的全局完成）追溯修改
    pub outcome: Outcome,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl TaskSchedule {
    /// 创建新的任务日程
    pub fn new(
        id: Uuid,
        task_id: Uuid,
        scheduled_day: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            task_id,
            scheduled_day,
            outcome: Outcome::Planned,
            created_at,
            updated_at: created_at,
        }
    }

    /// 更新结局
    pub fn update_outcome(&mut self, outcome: Outcome, updated_at: DateTime<Utc>) {
        self.outcome = outcome;
        self.updated_at = updated_at;
    }

    /// 重新安排到新日期
    pub fn reschedule(&mut self, new_day: DateTime<Utc>, updated_at: DateTime<Utc>) {
        self.scheduled_day = new_day;
        self.outcome = Outcome::Planned; // 重置为计划状态
        self.updated_at = updated_at;
    }

    /// 记录努力
    pub fn log_presence(&mut self, updated_at: DateTime<Utc>) -> Result<(), String> {
        if self.outcome == Outcome::CompletedOnDay {
            return Err("Cannot log presence for already completed task".to_string());
        }

        self.outcome = Outcome::PresenceLogged;
        self.updated_at = updated_at;
        Ok(())
    }

    /// 标记为当日完成
    pub fn mark_completed_on_day(&mut self, updated_at: DateTime<Utc>) {
        self.outcome = Outcome::CompletedOnDay;
        self.updated_at = updated_at;
    }

    /// 标记为延期
    pub fn mark_carried_over(&mut self, updated_at: DateTime<Utc>) {
        self.outcome = Outcome::CarriedOver;
        self.updated_at = updated_at;
    }

    /// 检查是否已完成
    pub fn is_completed(&self) -> bool {
        self.outcome == Outcome::CompletedOnDay
    }

    /// 检查是否已记录努力
    pub fn has_logged_presence(&self) -> bool {
        matches!(
            self.outcome,
            Outcome::PresenceLogged | Outcome::CompletedOnDay
        )
    }
}
