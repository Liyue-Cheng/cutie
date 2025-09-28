/// 服务层数据传输对象 (DTOs)
/// 
/// 定义了服务层方法的输入输出数据结构

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::models::{DueDateType, Subtask, ContextType};

/// 创建任务数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskData {
    /// 任务标题
    pub title: String,
    
    /// 快览笔记
    pub glance_note: Option<String>,
    
    /// 详细笔记
    pub detail_note: Option<String>,
    
    /// 预估时长（分钟）
    pub estimated_duration: Option<i32>,
    
    /// 子任务列表
    pub subtasks: Option<Vec<Subtask>>,
    
    /// 领域ID
    pub area_id: Option<Uuid>,
    
    /// 截止日期
    pub due_date: Option<DateTime<Utc>>,
    
    /// 截止日期类型
    pub due_date_type: Option<DueDateType>,
}

/// 更新任务数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskData {
    /// 任务标题
    pub title: Option<String>,
    
    /// 快览笔记
    pub glance_note: Option<Option<String>>,
    
    /// 详细笔记
    pub detail_note: Option<Option<String>>,
    
    /// 预估时长（分钟）
    pub estimated_duration: Option<Option<i32>>,
    
    /// 子任务列表
    pub subtasks: Option<Option<Vec<Subtask>>>,
    
    /// 项目ID
    pub project_id: Option<Option<Uuid>>,
    
    /// 领域ID
    pub area_id: Option<Option<Uuid>>,
    
    /// 截止日期
    pub due_date: Option<Option<DateTime<Utc>>>,
    
    /// 截止日期类型
    pub due_date_type: Option<Option<DueDateType>>,
}

/// 创建上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreationContext {
    /// 上下文类型
    pub context_type: ContextType,
    
    /// 上下文ID
    pub context_id: String,
}

impl CreationContext {
    /// 创建项目列表上下文
    pub fn project_list(project_id: Uuid) -> Self {
        Self {
            context_type: ContextType::ProjectList,
            context_id: format!("project::{}", project_id),
        }
    }
    
    /// 创建每日看板上下文
    pub fn daily_kanban(day: DateTime<Utc>) -> Self {
        Self {
            context_type: ContextType::DailyKanban,
            context_id: day.timestamp().to_string(),
        }
    }
    
    /// 创建杂项上下文
    pub fn misc(context_id: String) -> Self {
        Self {
            context_type: ContextType::Misc,
            context_id,
        }
    }
    
    /// 创建浮动任务上下文
    pub fn floating() -> Self {
        Self::misc("floating".to_string())
    }
    
    /// 创建暂存区上下文
    pub fn staging() -> Self {
        Self::misc("staging_all".to_string())
    }
}

/// 创建时间块数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTimeBlockData {
    /// 时间块标题
    pub title: Option<String>,
    
    /// 快览笔记
    pub glance_note: Option<String>,
    
    /// 详细笔记
    pub detail_note: Option<String>,
    
    /// 开始时间
    pub start_time: DateTime<Utc>,
    
    /// 结束时间
    pub end_time: DateTime<Utc>,
    
    /// 领域ID
    pub area_id: Option<Uuid>,
    
    /// 关联的任务ID列表
    pub task_ids: Vec<Uuid>,
}

/// 更新时间块数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTimeBlockData {
    /// 时间块标题
    pub title: Option<Option<String>>,
    
    /// 快览笔记
    pub glance_note: Option<Option<String>>,
    
    /// 详细笔记
    pub detail_note: Option<Option<String>>,
    
    /// 开始时间
    pub start_time: Option<DateTime<Utc>>,
    
    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,
    
    /// 领域ID
    pub area_id: Option<Option<Uuid>>,
}

/// 排序更新命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOrderCommand {
    /// 任务ID
    pub task_id: Uuid,
    
    /// 上下文类型
    pub context_type: ContextType,
    
    /// 上下文ID
    pub context_id: String,
    
    /// 新的排序位置
    pub new_sort_order: String,
}

/// 创建模板数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplateData {
    /// 模板名称
    pub name: String,
    
    /// 标题模板
    pub title_template: String,
    
    /// 快览笔记模板
    pub glance_note_template: Option<String>,
    
    /// 详细笔记模板
    pub detail_note_template: Option<String>,
    
    /// 预估时长模板
    pub estimated_duration_template: Option<i32>,
    
    /// 子任务模板
    pub subtasks_template: Option<Vec<Subtask>>,
    
    /// 领域ID
    pub area_id: Option<Uuid>,
}

/// 更新模板数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTemplateData {
    /// 模板名称
    pub name: Option<String>,
    
    /// 标题模板
    pub title_template: Option<String>,
    
    /// 快览笔记模板
    pub glance_note_template: Option<Option<String>>,
    
    /// 详细笔记模板
    pub detail_note_template: Option<Option<String>>,
    
    /// 预估时长模板
    pub estimated_duration_template: Option<Option<i32>>,
    
    /// 子任务模板
    pub subtasks_template: Option<Option<Vec<Subtask>>>,
    
    /// 领域ID
    pub area_id: Option<Option<Uuid>>,
}

/// 创建领域数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAreaData {
    /// 领域名称
    pub name: String,
    
    /// 颜色代码
    pub color: String,
    
    /// 父领域ID
    pub parent_area_id: Option<Uuid>,
}

/// 更新领域数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAreaData {
    /// 领域名称
    pub name: Option<String>,
    
    /// 颜色代码
    pub color: Option<String>,
    
    /// 父领域ID
    pub parent_area_id: Option<Option<Uuid>>,
}

impl CreateTaskData {
    /// 验证创建任务数据
    pub fn validate(&self) -> Result<(), Vec<crate::common::error::ValidationError>> {
        let mut errors = Vec::new();
        
        // 验证标题
        if self.title.is_empty() {
            errors.push(crate::common::error::ValidationError::new(
                "title",
                "Title cannot be empty",
                "TITLE_EMPTY"
            ));
        }
        
        if self.title.len() > 255 {
            errors.push(crate::common::error::ValidationError::new(
                "title",
                "Title cannot exceed 255 characters",
                "TITLE_TOO_LONG"
            ));
        }
        
        // 验证预估时长
        if let Some(duration) = self.estimated_duration {
            if duration <= 0 {
                errors.push(crate::common::error::ValidationError::new(
                    "estimated_duration",
                    "Estimated duration must be positive",
                    "DURATION_INVALID"
                ));
            }
        }
        
        // 验证截止日期一致性
        match (&self.due_date, &self.due_date_type) {
            (Some(_), None) => {
                errors.push(crate::common::error::ValidationError::new(
                    "due_date_type",
                    "Due date type must be specified when due date is set",
                    "DUE_DATE_TYPE_MISSING"
                ));
            }
            (None, Some(_)) => {
                errors.push(crate::common::error::ValidationError::new(
                    "due_date",
                    "Due date must be specified when due date type is set",
                    "DUE_DATE_MISSING"
                ));
            }
            _ => {}
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl CreateTimeBlockData {
    /// 验证创建时间块数据
    pub fn validate(&self) -> Result<(), Vec<crate::common::error::ValidationError>> {
        let mut errors = Vec::new();
        
        // 验证时间范围
        if self.start_time >= self.end_time {
            errors.push(crate::common::error::ValidationError::new(
                "time_range",
                "Start time must be before end time",
                "TIME_RANGE_INVALID"
            ));
        }
        
        // 验证时间不能在过去（可选验证）
        let now = chrono::Utc::now();
        if self.end_time < now {
            errors.push(crate::common::error::ValidationError::new(
                "end_time",
                "End time cannot be in the past",
                "END_TIME_PAST"
            ));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl CreateTemplateData {
    /// 验证创建模板数据
    pub fn validate(&self) -> Result<(), Vec<crate::common::error::ValidationError>> {
        let mut errors = Vec::new();
        
        // 验证名称
        if self.name.is_empty() {
            errors.push(crate::common::error::ValidationError::new(
                "name",
                "Template name cannot be empty",
                "NAME_EMPTY"
            ));
        }
        
        if self.name.len() > 100 {
            errors.push(crate::common::error::ValidationError::new(
                "name",
                "Template name cannot exceed 100 characters",
                "NAME_TOO_LONG"
            ));
        }
        
        // 验证标题模板
        if self.title_template.is_empty() {
            errors.push(crate::common::error::ValidationError::new(
                "title_template",
                "Title template cannot be empty",
                "TITLE_TEMPLATE_EMPTY"
            ));
        }
        
        // 验证模板语法
        if let Err(e) = crate::common::utils::template_utils::validate_template_syntax(&self.title_template) {
            errors.push(crate::common::error::ValidationError::new(
                "title_template",
                format!("Invalid template syntax: {}", e),
                "TEMPLATE_SYNTAX_ERROR"
            ));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl CreateAreaData {
    /// 验证创建领域数据
    pub fn validate(&self) -> Result<(), Vec<crate::common::error::ValidationError>> {
        let mut errors = Vec::new();
        
        // 验证名称
        if self.name.is_empty() {
            errors.push(crate::common::error::ValidationError::new(
                "name",
                "Area name cannot be empty",
                "NAME_EMPTY"
            ));
        }
        
        if self.name.len() > 100 {
            errors.push(crate::common::error::ValidationError::new(
                "name",
                "Area name cannot exceed 100 characters",
                "NAME_TOO_LONG"
            ));
        }
        
        // 验证颜色格式
        if !crate::core::models::Area::validate_color(&self.color) {
            errors.push(crate::common::error::ValidationError::new(
                "color",
                "Invalid color format, must be #RRGGBB",
                "COLOR_INVALID"
            ));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
