/// 任务功能模块的HTTP载荷定义
/// 
/// 定义任务相关API端点的请求体和响应体结构

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::{
    core::{DueDateType, Subtask, ContextType},
    http::extractors::Validate,
};

/// 创建任务请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskPayload {
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
    
    /// 创建上下文
    pub context: CreationContextPayload,
}

impl Validate for CreateTaskPayload {
    fn validate(&self) -> Result<(), Vec<crate::shared::core::ValidationError>> {
        let mut errors = Vec::new();
        
        // 验证标题
        if self.title.trim().is_empty() {
            errors.push(crate::shared::core::ValidationError::new(
                "title",
                "任务标题不能为空",
                "TITLE_EMPTY",
            ));
        }
        
        if self.title.len() > 255 {
            errors.push(crate::shared::core::ValidationError::new(
                "title",
                "任务标题不能超过255个字符",
                "TITLE_TOO_LONG",
            ));
        }
        
        // 验证预估时长
        if let Some(duration) = self.estimated_duration {
            if duration < 0 {
                errors.push(crate::shared::core::ValidationError::new(
                    "estimated_duration",
                    "预估时长不能为负数",
                    "DURATION_NEGATIVE",
                ));
            }
        }
        
        // 验证截止日期
        if self.due_date.is_some() && self.due_date_type.is_none() {
            errors.push(crate::shared::core::ValidationError::new(
                "due_date_type",
                "设置截止日期时必须指定日期类型",
                "DUE_DATE_TYPE_REQUIRED",
            ));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 创建上下文载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreationContextPayload {
    /// 上下文类型
    pub context_type: ContextType,
    
    /// 上下文ID
    pub context_id: String,
}

/// 更新任务请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskPayload {
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

impl Validate for UpdateTaskPayload {
    fn validate(&self) -> Result<(), Vec<crate::shared::core::ValidationError>> {
        let mut errors = Vec::new();
        
        // 验证标题
        if let Some(title) = &self.title {
            if title.trim().is_empty() {
                errors.push(crate::shared::core::ValidationError::new(
                    "title",
                    "任务标题不能为空",
                    "TITLE_EMPTY",
                ));
            }
            
            if title.len() > 255 {
                errors.push(crate::shared::core::ValidationError::new(
                    "title",
                    "任务标题不能超过255个字符",
                    "TITLE_TOO_LONG",
                ));
            }
        }
        
        // 验证预估时长
        if let Some(Some(duration)) = self.estimated_duration {
            if duration < 0 {
                errors.push(crate::shared::core::ValidationError::new(
                    "estimated_duration",
                    "预估时长不能为负数",
                    "DURATION_NEGATIVE",
                ));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 任务搜索查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSearchQuery {
    /// 搜索关键词
    pub q: Option<String>,
    
    /// 限制返回数量
    pub limit: Option<usize>,
}

impl Default for TaskSearchQuery {
    fn default() -> Self {
        Self {
            q: None,
            limit: Some(50),
        }
    }
}

/// 任务统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatsResponse {
    /// 总任务数
    pub total_count: i64,
    
    /// 已完成任务数
    pub completed_count: i64,
    
    /// 待处理任务数
    pub pending_count: i64,
    
    /// 逾期任务数
    pub overdue_count: i64,
    
    /// 今日任务数
    pub today_count: i64,
    
    /// 本周任务数
    pub this_week_count: i64,
    
    /// 本月任务数
    pub this_month_count: i64,
}

/// 任务完成率统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompletionStats {
    /// 日期
    pub date: DateTime<Utc>,
    
    /// 完成任务数
    pub completed_count: i64,
    
    /// 总任务数
    pub total_count: i64,
    
    /// 完成率（百分比）
    pub completion_rate: f64,
}

/// 任务创建趋势统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreationTrend {
    /// 日期
    pub date: DateTime<Utc>,
    
    /// 创建任务数
    pub created_count: i64,
    
    /// 累计任务数
    pub cumulative_count: i64,
}

/// 批量操作载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkTaskOperation {
    /// 任务ID列表
    pub task_ids: Vec<Uuid>,
    
    /// 操作类型
    pub operation: BulkOperationType,
    
    /// 操作参数（可选）
    pub parameters: Option<serde_json::Value>,
}

/// 批量操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BulkOperationType {
    /// 批量删除
    Delete,
    
    /// 批量完成
    Complete,
    
    /// 批量重新打开
    Reopen,
    
    /// 批量更新领域
    UpdateArea,
    
    /// 批量更新项目
    UpdateProject,
}

impl Validate for BulkTaskOperation {
    fn validate(&self) -> Result<(), Vec<crate::shared::core::ValidationError>> {
        let mut errors = Vec::new();
        
        if self.task_ids.is_empty() {
            errors.push(crate::shared::core::ValidationError::new(
                "task_ids",
                "任务ID列表不能为空",
                "TASK_IDS_EMPTY",
            ));
        }
        
        if self.task_ids.len() > 100 {
            errors.push(crate::shared::core::ValidationError::new(
                "task_ids",
                "批量操作不能超过100个任务",
                "TOO_MANY_TASKS",
            ));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task_payload_validation() {
        let valid_payload = CreateTaskPayload {
            title: "Test Task".to_string(),
            glance_note: None,
            detail_note: None,
            estimated_duration: Some(60),
            subtasks: None,
            area_id: None,
            due_date: None,
            due_date_type: None,
            context: CreationContextPayload {
                context_type: ContextType::Misc,
                context_id: "floating".to_string(),
            },
        };
        
        assert!(valid_payload.validate().is_ok());
        
        let invalid_payload = CreateTaskPayload {
            title: "".to_string(),
            glance_note: None,
            detail_note: None,
            estimated_duration: Some(-10),
            subtasks: None,
            area_id: None,
            due_date: Some(Utc::now()),
            due_date_type: None,
            context: CreationContextPayload {
                context_type: ContextType::Misc,
                context_id: "floating".to_string(),
            },
        };
        
        let validation_result = invalid_payload.validate();
        assert!(validation_result.is_err());
        let errors = validation_result.unwrap_err();
        assert!(errors.len() >= 3); // 空标题、负时长、缺少日期类型
    }

    #[test]
    fn test_bulk_task_operation_validation() {
        let valid_operation = BulkTaskOperation {
            task_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
            operation: BulkOperationType::Complete,
            parameters: None,
        };
        
        assert!(valid_operation.validate().is_ok());
        
        let invalid_operation = BulkTaskOperation {
            task_ids: vec![],
            operation: BulkOperationType::Delete,
            parameters: None,
        };
        
        assert!(invalid_operation.validate().is_err());
    }
}

