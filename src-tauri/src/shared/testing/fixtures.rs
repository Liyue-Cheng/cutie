/// 测试数据构造器（Fixtures）
///
/// 提供快速创建测试数据的工具
use crate::entities::{Area, Task};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// 任务测试数据构造器
pub struct TaskFixture {
    pub id: Uuid,
    pub title: String,
    pub glance_note: Option<String>,
    pub detail_note: Option<String>,
    pub estimated_duration: Option<i32>,
    pub area_id: Option<Uuid>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for TaskFixture {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: "Test Task".to_string(),
            glance_note: None,
            detail_note: None,
            estimated_duration: None,
            area_id: None,
            completed_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

impl TaskFixture {
    /// 创建新的任务 fixture
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置标题
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// 设置区域
    pub fn area_id(mut self, area_id: Uuid) -> Self {
        self.area_id = Some(area_id);
        self
    }

    /// 设置为已完成
    pub fn completed(mut self) -> Self {
        self.completed_at = Some(Utc::now());
        self
    }

    /// 设置预估时长（分钟）
    pub fn duration(mut self, minutes: i32) -> Self {
        self.estimated_duration = Some(minutes);
        self
    }

    /// 构建 Task 实体
    pub fn build(self) -> Task {
        Task {
            id: self.id,
            title: self.title,
            glance_note: self.glance_note,
            detail_note: self.detail_note,
            estimated_duration: self.estimated_duration,
            subtasks: None,
            project_id: None,
            area_id: self.area_id,
            due_date: None,
            due_date_type: None,
            completed_at: self.completed_at,
            archived_at: None,
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: None,
            source_info: None,
            external_source_id: None,
            external_source_provider: None,
            external_source_metadata: None,
            recurrence_id: None,
            recurrence_original_date: None,
        }
    }
}

/// 区域测试数据构造器
pub struct AreaFixture {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for AreaFixture {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: "Test Area".to_string(),
            color: "#FF0000".to_string(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl AreaFixture {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.color = color.into();
        self
    }

    pub fn build(self) -> Area {
        Area {
            id: self.id,
            name: self.name,
            color: self.color,
            parent_area_id: None,
            is_deleted: false,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_fixture() {
        let task = TaskFixture::new()
            .title("My Task")
            .duration(60)
            .completed()
            .build();

        assert_eq!(task.title, "My Task");
        assert_eq!(task.estimated_duration, Some(60));
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn test_area_fixture() {
        let area = AreaFixture::new().name("Work").color("#00FF00").build();

        assert_eq!(area.name, "Work");
        assert_eq!(area.color, "#00FF00");
    }
}
