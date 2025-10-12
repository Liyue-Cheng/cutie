use crate::infra::core::{AppError, AppResult, ValidationError};
/// TimeBlock 验证器
///
/// 负责 TimeBlock 相关的数据验证逻辑
use chrono::{DateTime, Utc};

pub struct TimeBlockValidator;

impl TimeBlockValidator {
    /// 验证时间范围
    pub fn validate_time_range(
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> AppResult<()> {
        let mut errors = Vec::new();

        if start_time >= end_time {
            errors.push(ValidationError::new(
                "time_range",
                "开始时间必须早于结束时间",
                "INVALID_RANGE",
            ));
        }

        let duration = end_time - start_time;
        if duration.num_hours() > 24 {
            errors.push(ValidationError::new(
                "duration",
                "时间块长度不能超过24小时",
                "MAX_DURATION",
            ));
        }

        if duration.num_minutes() < 1 {
            errors.push(ValidationError::new(
                "duration",
                "时间块长度不能少于1分钟",
                "MIN_DURATION",
            ));
        }

        if !errors.is_empty() {
            return Err(AppError::ValidationFailed(errors));
        }

        Ok(())
    }

    /// 验证标题
    pub fn validate_title(title: &str) -> AppResult<()> {
        if title.trim().is_empty() {
            return Err(AppError::validation_error(
                "title",
                "标题不能为空",
                "REQUIRED",
            ));
        }

        if title.len() > 100 {
            return Err(AppError::validation_error(
                "title",
                "标题长度不能超过100个字符",
                "MAX_LENGTH",
            ));
        }

        Ok(())
    }

    /// 验证时间不能是过去时间（允许5分钟的误差）
    pub fn validate_not_past_time(start_time: DateTime<Utc>) -> AppResult<()> {
        let now = Utc::now();
        if start_time < now - chrono::Duration::minutes(5) {
            return Err(AppError::validation_error(
                "start_time",
                "开始时间不能是过去时间",
                "PAST_TIME",
            ));
        }

        Ok(())
    }
}
