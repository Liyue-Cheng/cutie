/// Task 请求 DTOs
///
/// 只包含 API 请求相关的数据传输对象
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{DueDateType, Subtask};

/// 创建任务的请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub glance_note: Option<String>,
    pub detail_note: Option<String>,
    pub estimated_duration: Option<i32>,
    pub subtasks: Option<Vec<Subtask>>,
    pub area_id: Option<Uuid>,
    pub due_date: Option<DateTime<Utc>>,
    pub due_date_type: Option<DueDateType>,
    pub project_id: Option<Uuid>,
}

/// 更新任务的请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub glance_note: Option<Option<String>>,
    pub detail_note: Option<Option<String>>,
    pub estimated_duration: Option<Option<i32>>,
    pub subtasks: Option<Option<Vec<Subtask>>>,
    pub project_id: Option<Option<Uuid>>,
    pub area_id: Option<Option<Uuid>>,
    pub due_date: Option<Option<DateTime<Utc>>>,
    pub due_date_type: Option<Option<DueDateType>>,
}

impl UpdateTaskRequest {
    /// 检查请求是否为空，即所有字段都是None
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.glance_note.is_none()
            && self.detail_note.is_none()
            && self.estimated_duration.is_none()
            && self.subtasks.is_none()
            && self.project_id.is_none()
            && self.area_id.is_none()
            && self.due_date.is_none()
            && self.due_date_type.is_none()
    }
}

/// 搜索查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchTasksQuery {
    pub q: Option<String>,
    pub limit: Option<usize>,
}

impl Default for SearchTasksQuery {
    fn default() -> Self {
        Self {
            q: None,
            limit: Some(50),
        }
    }
}
