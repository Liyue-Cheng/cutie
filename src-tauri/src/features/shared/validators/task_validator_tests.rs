/// Task 验证器单元测试
#[cfg(test)]
mod tests {
    use crate::{
        entities::{CreateTaskRequest, UpdateTaskRequest},
        features::shared::validators::TaskValidator,
        infra::core::AppError,
    };

    #[test]
    fn test_validate_create_request_valid() {
        let request = CreateTaskRequest {
            title: "Valid Task".to_string(),
            glance_note: Some("Short note".to_string()),
            detail_note: Some("Detailed note".to_string()),
            estimated_duration: Some(3600),
            subtasks: None,
            area_id: None,
            project_id: None,
            due_date: None,
            due_date_type: None,
        };

        assert!(TaskValidator::validate_create_request(&request).is_ok());
    }

    #[test]
    fn test_validate_create_request_empty_title() {
        let request = CreateTaskRequest {
            title: "   ".to_string(), // 空白标题
            glance_note: None,
            detail_note: None,
            estimated_duration: None,
            subtasks: None,
            area_id: None,
            project_id: None,
            due_date: None,
            due_date_type: None,
        };

        let result = TaskValidator::validate_create_request(&request);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationFailed(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].field, "title");
                assert_eq!(errors[0].code, "REQUIRED");
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_validate_create_request_title_too_long() {
        let request = CreateTaskRequest {
            title: "a".repeat(201), // 超过200字符
            glance_note: None,
            detail_note: None,
            estimated_duration: None,
            subtasks: None,
            area_id: None,
            project_id: None,
            due_date: None,
            due_date_type: None,
        };

        let result = TaskValidator::validate_create_request(&request);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationFailed(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].field, "title");
                assert_eq!(errors[0].code, "MAX_LENGTH");
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_validate_create_request_glance_note_too_long() {
        let request = CreateTaskRequest {
            title: "Valid Title".to_string(),
            glance_note: Some("a".repeat(501)), // 超过500字符
            detail_note: None,
            estimated_duration: None,
            subtasks: None,
            area_id: None,
            project_id: None,
            due_date: None,
            due_date_type: None,
        };

        let result = TaskValidator::validate_create_request(&request);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationFailed(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].field, "glance_note");
                assert_eq!(errors[0].code, "MAX_LENGTH");
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_validate_create_request_detail_note_too_long() {
        let request = CreateTaskRequest {
            title: "Valid Title".to_string(),
            glance_note: None,
            detail_note: Some("a".repeat(5001)), // 超过5000字符
            estimated_duration: None,
            subtasks: None,
            area_id: None,
            project_id: None,
            due_date: None,
            due_date_type: None,
        };

        let result = TaskValidator::validate_create_request(&request);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationFailed(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].field, "detail_note");
                assert_eq!(errors[0].code, "MAX_LENGTH");
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_validate_create_request_invalid_estimated_duration() {
        let request = CreateTaskRequest {
            title: "Valid Title".to_string(),
            glance_note: None,
            detail_note: None,
            estimated_duration: Some(-100), // 负数
            subtasks: None,
            area_id: None,
            project_id: None,
            due_date: None,
            due_date_type: None,
        };

        let result = TaskValidator::validate_create_request(&request);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationFailed(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].field, "estimated_duration");
                assert_eq!(errors[0].code, "MIN_VALUE");
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_validate_create_request_estimated_duration_too_large() {
        let request = CreateTaskRequest {
            title: "Valid Title".to_string(),
            glance_note: None,
            detail_note: None,
            estimated_duration: Some(24 * 60 * 60 + 1), // 超过24小时
            subtasks: None,
            area_id: None,
            project_id: None,
            due_date: None,
            due_date_type: None,
        };

        let result = TaskValidator::validate_create_request(&request);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationFailed(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].field, "estimated_duration");
                assert_eq!(errors[0].code, "MAX_VALUE");
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_validate_create_request_multiple_errors() {
        let request = CreateTaskRequest {
            title: "".to_string(),              // 空标题
            glance_note: Some("a".repeat(501)), // 简要说明太长
            detail_note: None,
            estimated_duration: Some(-1), // 无效时长
            subtasks: None,
            area_id: None,
            project_id: None,
            due_date: None,
            due_date_type: None,
        };

        let result = TaskValidator::validate_create_request(&request);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationFailed(errors) => {
                assert_eq!(errors.len(), 3); // 3个错误
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_validate_update_request_valid() {
        let request = UpdateTaskRequest {
            title: Some("Updated Title".to_string()),
            glance_note: Some(Some("Updated note".to_string())),
            detail_note: None,
            estimated_duration: Some(Some(7200)),
            subtasks: None,
            area_id: None,
            project_id: None,
            due_date: None,
            due_date_type: None,
        };

        assert!(TaskValidator::validate_update_request(&request).is_ok());
    }

    #[test]
    fn test_validate_update_request_empty_title() {
        let request = UpdateTaskRequest {
            title: Some("   ".to_string()), // 空白标题
            glance_note: None,
            detail_note: None,
            estimated_duration: None,
            subtasks: None,
            area_id: None,
            project_id: None,
            due_date: None,
            due_date_type: None,
        };

        let result = TaskValidator::validate_update_request(&request);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationFailed(errors) => {
                assert_eq!(errors[0].field, "title");
                assert_eq!(errors[0].code, "REQUIRED");
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_validate_update_request_nested_option_glance_note() {
        let request = UpdateTaskRequest {
            title: Some("Valid Title".to_string()),
            glance_note: Some(Some("a".repeat(501))), // 超过500字符
            detail_note: None,
            estimated_duration: None,
            subtasks: None,
            area_id: None,
            project_id: None,
            due_date: None,
            due_date_type: None,
        };

        let result = TaskValidator::validate_update_request(&request);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationFailed(errors) => {
                assert_eq!(errors[0].field, "glance_note");
                assert_eq!(errors[0].code, "MAX_LENGTH");
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_validate_title_valid() {
        assert!(TaskValidator::validate_title("Valid Title").is_ok());
    }

    #[test]
    fn test_validate_title_empty() {
        let result = TaskValidator::validate_title("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_title_too_long() {
        let result = TaskValidator::validate_title(&"a".repeat(201));
        assert!(result.is_err());
    }
}
