/// TimeBlock 验证器单元测试
#[cfg(test)]
mod tests {
    use crate::{features::shared::validators::TimeBlockValidator, infra::core::AppError};
    use chrono::{Duration, Utc};

    #[test]
    fn test_validate_time_range_valid() {
        let start = Utc::now();
        let end = start + Duration::hours(2);

        assert!(TimeBlockValidator::validate_time_range(start, end).is_ok());
    }

    #[test]
    fn test_validate_time_range_start_after_end() {
        let start = Utc::now();
        let end = start - Duration::hours(1); // 结束时间在开始时间之前

        let result = TimeBlockValidator::validate_time_range(start, end);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationFailed(errors) => {
                // 会有2个错误：INVALID_RANGE 和 MIN_DURATION
                assert!(errors.len() >= 1);
                assert!(errors.iter().any(|e| e.code == "INVALID_RANGE"));
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_validate_time_range_equal() {
        let start = Utc::now();
        let end = start; // 开始和结束时间相同

        let result = TimeBlockValidator::validate_time_range(start, end);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_time_range_too_long() {
        let start = Utc::now();
        let end = start + Duration::hours(25); // 超过24小时

        let result = TimeBlockValidator::validate_time_range(start, end);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationFailed(errors) => {
                let has_max_duration_error = errors.iter().any(|e| e.code == "MAX_DURATION");
                assert!(has_max_duration_error);
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_validate_time_range_too_short() {
        let start = Utc::now();
        let end = start + Duration::seconds(30); // 少于1分钟

        let result = TimeBlockValidator::validate_time_range(start, end);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationFailed(errors) => {
                let has_min_duration_error = errors.iter().any(|e| e.code == "MIN_DURATION");
                assert!(has_min_duration_error);
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_validate_time_range_multiple_errors() {
        let start = Utc::now();
        let end = start; // 时间相同（无效范围）+ 少于1分钟

        let result = TimeBlockValidator::validate_time_range(start, end);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationFailed(errors) => {
                assert!(errors.len() >= 1); // 至少有1个错误
            }
            _ => panic!("Expected ValidationFailed error"),
        }
    }

    #[test]
    fn test_validate_title_valid() {
        assert!(TimeBlockValidator::validate_title("Valid TimeBlock").is_ok());
    }

    #[test]
    fn test_validate_title_empty() {
        let result = TimeBlockValidator::validate_title("   ");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationFailed(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].field, "title");
                assert_eq!(errors[0].code, "REQUIRED");
            }
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[test]
    fn test_validate_title_too_long() {
        let result = TimeBlockValidator::validate_title(&"a".repeat(101));
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationFailed(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].field, "title");
                assert_eq!(errors[0].code, "MAX_LENGTH");
            }
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[test]
    fn test_validate_not_past_time_valid() {
        let future_time = Utc::now() + Duration::hours(1);
        assert!(TimeBlockValidator::validate_not_past_time(future_time).is_ok());
    }

    #[test]
    fn test_validate_not_past_time_within_tolerance() {
        let recent_time = Utc::now() - Duration::minutes(3); // 3分钟前，在5分钟误差内
        assert!(TimeBlockValidator::validate_not_past_time(recent_time).is_ok());
    }

    #[test]
    fn test_validate_not_past_time_past() {
        let past_time = Utc::now() - Duration::hours(1); // 1小时前
        let result = TimeBlockValidator::validate_not_past_time(past_time);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::ValidationFailed(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].field, "start_time");
                assert_eq!(errors[0].code, "PAST_TIME");
            }
            _ => panic!("Expected ValidationFailed"),
        }
    }

    #[test]
    fn test_validate_not_past_time_exactly_5_minutes() {
        let time = Utc::now() - Duration::minutes(5) + Duration::seconds(1); // 4分59秒前
                                                                             // 应该通过（在5分钟误差内）
        assert!(TimeBlockValidator::validate_not_past_time(time).is_ok());
    }

    #[test]
    fn test_validate_not_past_time_just_over_5_minutes() {
        let time = Utc::now() - Duration::minutes(6); // 6分钟前
        let result = TimeBlockValidator::validate_not_past_time(time);
        assert!(result.is_err());
    }
}
