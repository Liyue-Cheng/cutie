use crate::entities::{CreateTaskRequest, UpdateTaskRequest};
/// Task 验证器
///
/// 负责 Task 相关的数据验证逻辑
use crate::infra::core::{AppError, AppResult, ValidationError};

pub struct TaskValidator;

impl TaskValidator {
    /// 验证创建任务请求
    pub fn validate_create_request(request: &CreateTaskRequest) -> AppResult<()> {
        let mut errors = Vec::new();

        // 验证标题
        if request.title.trim().is_empty() {
            errors.push(ValidationError::new("title", "标题不能为空", "REQUIRED"));
        }

        if request.title.len() > 200 {
            errors.push(ValidationError::new(
                "title",
                "标题长度不能超过200个字符",
                "MAX_LENGTH",
            ));
        }

        // 验证简要说明
        if let Some(ref glance_note) = request.glance_note {
            if glance_note.len() > 500 {
                errors.push(ValidationError::new(
                    "glance_note",
                    "简要说明长度不能超过500个字符",
                    "MAX_LENGTH",
                ));
            }
        }

        // 验证详细说明
        if let Some(ref detail_note) = request.detail_note {
            if detail_note.len() > 5000 {
                errors.push(ValidationError::new(
                    "detail_note",
                    "详细说明长度不能超过5000个字符",
                    "MAX_LENGTH",
                ));
            }
        }

        // 验证预估时长
        if let Some(duration) = request.estimated_duration {
            if duration <= 0 {
                errors.push(ValidationError::new(
                    "estimated_duration",
                    "预估时长必须大于0",
                    "MIN_VALUE",
                ));
            }
            if duration > 24 * 60 * 60 {
                // 24小时
                errors.push(ValidationError::new(
                    "estimated_duration",
                    "预估时长不能超过24小时",
                    "MAX_VALUE",
                ));
            }
        }

        if !errors.is_empty() {
            return Err(AppError::ValidationFailed(errors));
        }

        Ok(())
    }

    /// 验证更新任务请求
    pub fn validate_update_request(request: &UpdateTaskRequest) -> AppResult<()> {
        let mut errors = Vec::new();

        // 验证标题
        if let Some(ref title) = request.title {
            if title.trim().is_empty() {
                errors.push(ValidationError::new("title", "标题不能为空", "REQUIRED"));
            }
            if title.len() > 200 {
                errors.push(ValidationError::new(
                    "title",
                    "标题长度不能超过200个字符",
                    "MAX_LENGTH",
                ));
            }
        }

        // 验证简要说明 (Option<Option<String>>)
        if let Some(Some(ref glance_note)) = request.glance_note {
            if glance_note.len() > 500 {
                errors.push(ValidationError::new(
                    "glance_note",
                    "简要说明长度不能超过500个字符",
                    "MAX_LENGTH",
                ));
            }
        }

        // 验证详细说明 (Option<Option<String>>)
        if let Some(Some(ref detail_note)) = request.detail_note {
            if detail_note.len() > 5000 {
                errors.push(ValidationError::new(
                    "detail_note",
                    "详细说明长度不能超过5000个字符",
                    "MAX_LENGTH",
                ));
            }
        }

        // 验证预估时长 (Option<Option<i32>>)
        if let Some(Some(duration)) = request.estimated_duration {
            if duration <= 0 {
                errors.push(ValidationError::new(
                    "estimated_duration",
                    "预估时长必须大于0",
                    "MIN_VALUE",
                ));
            }
            if duration > 24 * 60 * 60 {
                // 24小时
                errors.push(ValidationError::new(
                    "estimated_duration",
                    "预估时长不能超过24小时",
                    "MAX_VALUE",
                ));
            }
        }

        if !errors.is_empty() {
            return Err(AppError::ValidationFailed(errors));
        }

        Ok(())
    }

    /// 验证任务标题
    pub fn validate_title(title: &str) -> AppResult<()> {
        if title.trim().is_empty() {
            return Err(AppError::validation_error(
                "title",
                "标题不能为空",
                "REQUIRED",
            ));
        }

        if title.len() > 200 {
            return Err(AppError::validation_error(
                "title",
                "标题长度不能超过200个字符",
                "MAX_LENGTH",
            ));
        }

        Ok(())
    }
}
