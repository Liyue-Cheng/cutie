/// 模板功能模块的HTTP载荷定义

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::{
    core::{ContextType, Subtask, Template, ValidationError},
    http::extractors::Validate,
};

/// 创建模板请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplatePayload {
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

impl Validate for CreateTemplatePayload {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // 验证名称
        if self.name.trim().is_empty() {
            errors.push(ValidationError::new(
                "name",
                "模板名称不能为空",
                "NAME_EMPTY",
            ));
        }

        if self.name.len() > 100 {
            errors.push(ValidationError::new(
                "name",
                "模板名称不能超过100个字符",
                "NAME_TOO_LONG",
            ));
        }

        // 验证标题模板
        if self.title_template.trim().is_empty() {
            errors.push(ValidationError::new(
                "title_template",
                "标题模板不能为空",
                "TITLE_TEMPLATE_EMPTY",
            ));
        }

        // 验证模板语法
        if let Err(err) = crate::shared::core::validate_template_syntax(&self.title_template) {
            errors.push(ValidationError::new(
                "title_template",
                &err,
                "INVALID_TEMPLATE_SYNTAX",
            ));
        }

        if let Some(glance_note) = &self.glance_note_template {
            if let Err(err) = crate::shared::core::validate_template_syntax(glance_note) {
                errors.push(ValidationError::new(
                    "glance_note_template",
                    &err,
                    "INVALID_TEMPLATE_SYNTAX",
                ));
            }
        }

        if let Some(detail_note) = &self.detail_note_template {
            if let Err(err) = crate::shared::core::validate_template_syntax(detail_note) {
                errors.push(ValidationError::new(
                    "detail_note_template",
                    &err,
                    "INVALID_TEMPLATE_SYNTAX",
                ));
            }
        }

        // 验证预估时长
        if let Some(duration) = self.estimated_duration_template {
            if duration < 0 {
                errors.push(ValidationError::new(
                    "estimated_duration_template",
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

/// 更新模板请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTemplatePayload {
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

impl Validate for UpdateTemplatePayload {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // 验证名称
        if let Some(name) = &self.name {
            if name.trim().is_empty() {
                errors.push(ValidationError::new(
                    "name",
                    "模板名称不能为空",
                    "NAME_EMPTY",
                ));
            }

            if name.len() > 100 {
                errors.push(ValidationError::new(
                    "name",
                    "模板名称不能超过100个字符",
                    "NAME_TOO_LONG",
                ));
            }
        }

        // 验证标题模板
        if let Some(title_template) = &self.title_template {
            if title_template.trim().is_empty() {
                errors.push(ValidationError::new(
                    "title_template",
                    "标题模板不能为空",
                    "TITLE_TEMPLATE_EMPTY",
                ));
            }

            if let Err(err) = crate::shared::core::validate_template_syntax(title_template) {
                errors.push(ValidationError::new(
                    "title_template",
                    &err,
                    "INVALID_TEMPLATE_SYNTAX",
                ));
            }
        }

        // 验证预估时长
        if let Some(Some(duration)) = self.estimated_duration_template {
            if duration < 0 {
                errors.push(ValidationError::new(
                    "estimated_duration_template",
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

/// 基于模板创建任务请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskFromTemplatePayload {
    /// 创建上下文
    pub context: CreationContextPayload,

    /// 自定义变量（可选）
    pub custom_variables: Option<std::collections::HashMap<String, String>>,
}

/// 创建上下文载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreationContextPayload {
    /// 上下文类型
    pub context_type: ContextType,

    /// 上下文ID
    pub context_id: String,
}

impl Validate for CreateTaskFromTemplatePayload {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        // 基本验证，更复杂的验证在服务层进行
        Ok(())
    }
}

/// 克隆模板请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneTemplatePayload {
    /// 新模板名称
    pub new_name: String,
}

impl Validate for CloneTemplatePayload {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if self.new_name.trim().is_empty() {
            errors.push(ValidationError::new(
                "new_name",
                "新模板名称不能为空",
                "NEW_NAME_EMPTY",
            ));
        }

        if self.new_name.len() > 100 {
            errors.push(ValidationError::new(
                "new_name",
                "新模板名称不能超过100个字符",
                "NEW_NAME_TOO_LONG",
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 模板查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateQuery {
    /// 搜索关键词
    pub q: Option<String>,

    /// 领域ID
    pub area_id: Option<Uuid>,

    /// 变量名
    pub variable: Option<String>,

    /// 限制返回数量
    pub limit: Option<usize>,
}

impl Default for TemplateQuery {
    fn default() -> Self {
        Self {
            q: None,
            area_id: None,
            variable: None,
            limit: Some(50),
        }
    }
}

/// 模板统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateStatsResponse {
    /// 总模板数
    pub total_count: i64,

    /// 按领域分组的统计
    pub by_area: std::collections::HashMap<String, i64>,

    /// 包含变量的模板数
    pub templates_with_variables: i64,

    /// 最常用的变量
    pub most_used_variables: Vec<VariableUsage>,

    /// 平均模板复杂度
    pub avg_complexity: f64,
}

/// 变量使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableUsage {
    /// 变量名
    pub variable_name: String,

    /// 使用次数
    pub usage_count: i64,

    /// 使用该变量的模板数
    pub template_count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_template_payload_validation() {
        let valid_payload = CreateTemplatePayload {
            name: "Daily Standup".to_string(),
            title_template: "Daily Standup - {{date}}".to_string(),
            glance_note_template: Some("Standup for {{date}}".to_string()),
            detail_note_template: None,
            estimated_duration_template: Some(15),
            subtasks_template: None,
            area_id: None,
        };

        assert!(valid_payload.validate().is_ok());

        // 测试无效模板语法
        let invalid_payload = CreateTemplatePayload {
            name: "Invalid Template".to_string(),
            title_template: "Unclosed {{variable".to_string(),
            glance_note_template: None,
            detail_note_template: None,
            estimated_duration_template: None,
            subtasks_template: None,
            area_id: None,
        };

        assert!(invalid_payload.validate().is_err());
    }

    #[test]
    fn test_clone_template_payload_validation() {
        let valid_payload = CloneTemplatePayload {
            new_name: "Cloned Template".to_string(),
        };

        assert!(valid_payload.validate().is_ok());

        let invalid_payload = CloneTemplatePayload {
            new_name: "".to_string(),
        };

        assert!(invalid_payload.validate().is_err());
    }
}
