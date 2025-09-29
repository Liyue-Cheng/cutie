/// 任务功能的验证逻辑
/// 
/// 提供所有任务API端点共享的验证函数

use crate::shared::core::{Task, ValidationError, AppResult};
use super::dtos::{CreateTaskRequest, UpdateTaskRequest};

/// 验证创建任务请求
pub fn validate_create_task_request(request: &CreateTaskRequest) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // 验证标题
    if request.title.trim().is_empty() {
        errors.push(ValidationError::new(
            "title",
            "任务标题不能为空",
            "TITLE_EMPTY",
        ));
    }

    if request.title.len() > 255 {
        errors.push(ValidationError::new(
            "title",
            "任务标题不能超过255个字符",
            "TITLE_TOO_LONG",
        ));
    }

    // 验证预估时长
    if let Some(duration) = request.estimated_duration {
        if duration < 0 {
            errors.push(ValidationError::new(
                "estimated_duration",
                "预估时长不能为负数",
                "DURATION_NEGATIVE",
            ));
        }
    }

    // 验证截止日期
    if request.due_date.is_some() && request.due_date_type.is_none() {
        errors.push(ValidationError::new(
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

/// 验证更新任务请求
pub fn validate_update_task_request(request: &UpdateTaskRequest) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // 验证标题
    if let Some(title) = &request.title {
        if title.trim().is_empty() {
            errors.push(ValidationError::new(
                "title",
                "任务标题不能为空",
                "TITLE_EMPTY",
            ));
        }

        if title.len() > 255 {
            errors.push(ValidationError::new(
                "title",
                "任务标题不能超过255个字符",
                "TITLE_TOO_LONG",
            ));
        }
    }

    // 验证预估时长
    if let Some(Some(duration)) = request.estimated_duration {
        if duration < 0 {
            errors.push(ValidationError::new(
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

/// 验证任务业务规则
pub fn validate_task_business_rules(task: &Task) -> AppResult<()> {
    // 验证标题长度
    if task.title.len() > 255 {
        return Err(crate::shared::core::AppError::validation_error(
            "title",
            "任务标题不能超过255个字符",
            "TITLE_TOO_LONG",
        ));
    }

    // 验证预估时长
    if let Some(duration) = task.estimated_duration {
        if duration < 0 {
            return Err(crate::shared::core::AppError::validation_error(
                "estimated_duration",
                "预估时长不能为负数",
                "DURATION_NEGATIVE",
            ));
        }
        if duration > 24 * 60 * 7 {
            // 一周的分钟数
            return Err(crate::shared::core::AppError::validation_error(
                "estimated_duration",
                "预估时长不能超过一周",
                "DURATION_TOO_LONG",
            ));
        }
    }

    // 验证截止日期
    if let Some(due_date) = task.due_date {
        if due_date < task.created_at {
            return Err(crate::shared::core::AppError::validation_error(
                "due_date",
                "截止日期不能早于创建时间",
                "DUE_DATE_TOO_EARLY",
            ));
        }
    }

    // 验证子任务数量
    if let Some(subtasks) = &task.subtasks {
        if subtasks.len() > 50 {
            return Err(crate::shared::core::AppError::validation_error(
                "subtasks",
                "子任务数量不能超过50个",
                "TOO_MANY_SUBTASKS",
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_validate_create_task_request() {
        let valid_request = CreateTaskRequest {
            title: "Test Task".to_string(),
            glance_note: None,
            detail_note: None,
            estimated_duration: Some(60),
            subtasks: None,
            area_id: None,
            due_date: None,
            due_date_type: None,
            context: CreationContext {
                context_type: crate::shared::core::ContextType::Misc,
                context_id: "floating".to_string(),
            },
        };

        assert!(validate_create_task_request(&valid_request).is_ok());

        let invalid_request = CreateTaskRequest {
            title: "".to_string(),
            glance_note: None,
            detail_note: None,
            estimated_duration: Some(-10),
            subtasks: None,
            area_id: None,
            due_date: Some(Utc::now()),
            due_date_type: None,
            context: CreationContext {
                context_type: crate::shared::core::ContextType::Misc,
                context_id: "floating".to_string(),
            },
        };

        let validation_result = validate_create_task_request(&invalid_request);
        assert!(validation_result.is_err());
        let errors = validation_result.unwrap_err();
        assert!(errors.len() >= 3); // 空标题、负时长、缺少日期类型
    }
}
