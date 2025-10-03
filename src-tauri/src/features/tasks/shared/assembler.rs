/// Task 装配器 (Assembler)
///
/// 职责：将数据库实体（Task）和其他相关数据组装成响应 DTO
///
/// 装配器模式的优势：
/// - entities 层保持纯粹的数据结构定义
/// - 业务逻辑集中在 features 层
/// - 易于测试和维护
use chrono::Utc;

use crate::entities::{
    DueDateInfo, DueDateType, ProjectSummary, ScheduleInfo, ScheduleRecord, ScheduleStatus,
    SubtaskDto, Task, TaskCardDto, TaskDetailDto,
};

/// Task 装配器
pub struct TaskAssembler;

impl TaskAssembler {
    /// 从 Task 实体创建 TaskCardDto（基础版本）
    ///
    /// 只填充可以直接从 Task 实体获取的字段
    /// 需要额外数据的字段保持默认值，调用者需要手动设置：
    /// - schedule_status（需要从 Schedule 表判断）
    /// - schedule_info（需要从 Schedule 表计算）
    pub fn task_to_card_basic(task: &Task) -> TaskCardDto {
        TaskCardDto {
            id: task.id,
            title: task.title.clone(),
            glance_note: task.glance_note.clone(),
            is_completed: task.is_completed(),
            schedule_status: ScheduleStatus::Staging, // 默认 Staging，需要后续判断
            subtasks: task.subtasks.as_ref().map(|subtasks| {
                subtasks
                    .iter()
                    .map(|s| SubtaskDto::from(s.clone()))
                    .collect()
            }),
            area_id: task.area_id, // ✅ 直接传递 area_id，前端从 area store 获取完整信息
            project_id: task.project_id,
            schedule_info: None, // 需要后续填充
            due_date: task.due_date.map(|date| DueDateInfo {
                date,
                due_date_type: task.due_date_type.clone().unwrap_or(DueDateType::Soft),
                is_overdue: Utc::now() > date,
            }),
            has_detail_note: task.detail_note.is_some(),
        }
    }

    /// 从 Task 实体创建完整的 TaskCardDto
    ///
    /// 包含所有必要的上下文信息
    #[allow(dead_code)]
    pub fn task_to_card_full(
        task: &Task,
        schedule_status: ScheduleStatus,
        schedule_info: Option<ScheduleInfo>,
    ) -> TaskCardDto {
        let mut card = Self::task_to_card_basic(task);
        card.schedule_status = schedule_status;
        card.schedule_info = schedule_info;
        card
    }

    /// 从 TaskCardDto 和 Task 实体创建 TaskDetailDto（基础版本）
    ///
    /// 需要额外数据的字段保持默认值：
    /// - schedules（需要从 Schedule 表获取）
    /// - project（需要从 Project 表获取）
    pub fn card_and_task_to_detail_basic(card: TaskCardDto, task: &Task) -> TaskDetailDto {
        TaskDetailDto {
            card,
            detail_note: task.detail_note.clone(),
            schedules: Vec::new(), // 需要后续填充
            project: None,         // 需要后续填充
            created_at: task.created_at,
            updated_at: task.updated_at,
        }
    }

    /// 从 TaskCardDto 和额外信息创建完整的 TaskDetailDto
    #[allow(dead_code)]
    pub fn card_to_detail_full(
        card: TaskCardDto,
        task: &Task,
        schedules: Vec<ScheduleRecord>,
        project: Option<ProjectSummary>,
    ) -> TaskDetailDto {
        TaskDetailDto {
            card,
            detail_note: task.detail_note.clone(),
            schedules,
            project,
            created_at: task.created_at,
            updated_at: task.updated_at,
        }
    }

    /// 从 Task 实体直接创建 TaskDetailDto（基础版本）
    ///
    /// 这是一个便捷方法，内部调用 task_to_card_basic 和 card_and_task_to_detail_basic
    #[allow(dead_code)]
    pub fn task_to_detail_basic(task: &Task) -> TaskDetailDto {
        let card = Self::task_to_card_basic(task);
        Self::card_and_task_to_detail_basic(card, task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_task() -> Task {
        Task {
            id: Uuid::new_v4(),
            title: "Test Task".to_string(),
            glance_note: Some("Test note".to_string()),
            detail_note: Some("Detailed note".to_string()),
            estimated_duration: Some(60),
            subtasks: None,
            project_id: None,
            area_id: None,
            due_date: None,
            due_date_type: None,
            completed_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_deleted: false,
            source_info: None,
            external_source_id: None,
            external_source_provider: None,
            external_source_metadata: None,
            recurrence_rule: None,
            recurrence_parent_id: None,
            recurrence_original_date: None,
            recurrence_exclusions: None,
        }
    }

    #[test]
    fn test_task_to_card_basic() {
        let task = create_test_task();
        let card = TaskAssembler::task_to_card_basic(&task);

        assert_eq!(card.id, task.id);
        assert_eq!(card.title, task.title);
        assert_eq!(card.glance_note, task.glance_note);
        assert!(!card.is_completed);
        assert_eq!(card.schedule_status, ScheduleStatus::Staging);
        assert!(card.has_detail_note);
    }

    #[test]
    fn test_task_to_detail_basic() {
        let task = create_test_task();
        let detail = TaskAssembler::task_to_detail_basic(&task);

        assert_eq!(detail.card.id, task.id);
        assert_eq!(detail.detail_note, task.detail_note);
        assert_eq!(detail.created_at, task.created_at);
        assert_eq!(detail.updated_at, task.updated_at);
    }
}
