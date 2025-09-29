/// 时间块功能模块的HTTP载荷定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::{
    core::{TimeBlock, ValidationError},
    http::extractors::Validate,
};

/// 创建时间块请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTimeBlockPayload {
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

    /// 要关联的任务ID列表
    pub task_ids: Vec<Uuid>,
}

impl Validate for CreateTimeBlockPayload {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // 验证时间范围
        if self.start_time >= self.end_time {
            errors.push(ValidationError::new(
                "time_range",
                "开始时间必须早于结束时间",
                "INVALID_TIME_RANGE",
            ));
        }

        // 验证时间不能是过去（可选的业务规则）
        let now = Utc::now();
        if self.end_time < now {
            errors.push(ValidationError::new(
                "end_time",
                "结束时间不能是过去时间",
                "END_TIME_IN_PAST",
            ));
        }

        // 验证时间块不能超过24小时
        let duration = self.end_time - self.start_time;
        if duration > chrono::Duration::hours(24) {
            errors.push(ValidationError::new(
                "duration",
                "时间块不能超过24小时",
                "DURATION_TOO_LONG",
            ));
        }

        // 验证标题长度
        if let Some(title) = &self.title {
            if title.len() > 255 {
                errors.push(ValidationError::new(
                    "title",
                    "标题不能超过255个字符",
                    "TITLE_TOO_LONG",
                ));
            }
        }

        // 验证关联的任务数量
        if self.task_ids.len() > 10 {
            errors.push(ValidationError::new(
                "task_ids",
                "一个时间块最多只能关联10个任务",
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

/// 更新时间块请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTimeBlockPayload {
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

impl Validate for UpdateTimeBlockPayload {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // 验证时间范围（如果都提供了）
        if let (Some(start), Some(end)) = (&self.start_time, &self.end_time) {
            if start >= end {
                errors.push(ValidationError::new(
                    "time_range",
                    "开始时间必须早于结束时间",
                    "INVALID_TIME_RANGE",
                ));
            }

            // 验证时间块不能超过24小时
            let duration = *end - *start;
            if duration > chrono::Duration::hours(24) {
                errors.push(ValidationError::new(
                    "duration",
                    "时间块不能超过24小时",
                    "DURATION_TOO_LONG",
                ));
            }
        }

        // 验证标题长度
        if let Some(Some(title)) = &self.title {
            if title.len() > 255 {
                errors.push(ValidationError::new(
                    "title",
                    "标题不能超过255个字符",
                    "TITLE_TOO_LONG",
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

/// 时间块查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBlockQuery {
    /// 特定日期
    pub date: Option<DateTime<Utc>>,

    /// 开始日期
    pub start_date: Option<DateTime<Utc>>,

    /// 结束日期
    pub end_date: Option<DateTime<Utc>>,

    /// 任务ID
    pub task_id: Option<Uuid>,

    /// 领域ID
    pub area_id: Option<Uuid>,
}

impl Default for TimeBlockQuery {
    fn default() -> Self {
        Self {
            date: None,
            start_date: None,
            end_date: None,
            task_id: None,
            area_id: None,
        }
    }
}

/// 时间冲突检查查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConflictQuery {
    /// 开始时间
    pub start_time: DateTime<Utc>,

    /// 结束时间
    pub end_time: DateTime<Utc>,

    /// 排除的时间块ID
    pub exclude_id: Option<Uuid>,
}

impl Validate for TimeConflictQuery {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if self.start_time >= self.end_time {
            errors.push(ValidationError::new(
                "time_range",
                "开始时间必须早于结束时间",
                "INVALID_TIME_RANGE",
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 空闲时间段查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeSlotsQuery {
    /// 开始时间
    pub start_time: DateTime<Utc>,

    /// 结束时间
    pub end_time: DateTime<Utc>,

    /// 最小时长（分钟）
    pub min_duration_minutes: i32,
}

impl Validate for FreeSlotsQuery {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if self.start_time >= self.end_time {
            errors.push(ValidationError::new(
                "time_range",
                "开始时间必须早于结束时间",
                "INVALID_TIME_RANGE",
            ));
        }

        if self.min_duration_minutes <= 0 {
            errors.push(ValidationError::new(
                "min_duration_minutes",
                "最小时长必须大于0",
                "INVALID_MIN_DURATION",
            ));
        }

        if self.min_duration_minutes > 24 * 60 {
            errors.push(ValidationError::new(
                "min_duration_minutes",
                "最小时长不能超过24小时",
                "MIN_DURATION_TOO_LONG",
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 链接任务到时间块请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkTaskPayload {
    /// 任务ID
    pub task_id: Uuid,
}

impl Validate for LinkTaskPayload {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        // 基本验证，更复杂的验证在服务层进行
        Ok(())
    }
}

/// 时间冲突检查响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConflictResponse {
    /// 是否有冲突
    pub has_conflict: bool,

    /// 查询的开始时间
    pub start_time: DateTime<Utc>,

    /// 查询的结束时间
    pub end_time: DateTime<Utc>,

    /// 冲突的时间块列表（如果有）
    pub conflicting_blocks: Option<Vec<TimeBlock>>,
}

/// 时间块统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBlockStatsResponse {
    /// 总时间块数
    pub total_count: i64,

    /// 今日时间块数
    pub today_count: i64,

    /// 本周时间块数
    pub this_week_count: i64,

    /// 本月时间块数
    pub this_month_count: i64,

    /// 总计划时间（分钟）
    pub total_planned_minutes: i64,

    /// 平均时间块时长（分钟）
    pub avg_duration_minutes: f64,

    /// 按领域分组的统计
    pub by_area: std::collections::HashMap<String, i64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_create_time_block_payload_validation() {
        let now = Utc::now();
        let valid_payload = CreateTimeBlockPayload {
            title: Some("Meeting".to_string()),
            glance_note: None,
            detail_note: None,
            start_time: now + chrono::Duration::hours(1),
            end_time: now + chrono::Duration::hours(2),
            area_id: None,
            task_ids: vec![Uuid::new_v4()],
        };

        assert!(valid_payload.validate().is_ok());

        // 测试无效时间范围
        let invalid_payload = CreateTimeBlockPayload {
            title: None,
            glance_note: None,
            detail_note: None,
            start_time: now + chrono::Duration::hours(2),
            end_time: now + chrono::Duration::hours(1),
            area_id: None,
            task_ids: vec![],
        };

        assert!(invalid_payload.validate().is_err());
    }

    #[test]
    fn test_time_conflict_query_validation() {
        let now = Utc::now();
        let valid_query = TimeConflictQuery {
            start_time: now,
            end_time: now + chrono::Duration::hours(1),
            exclude_id: None,
        };

        assert!(valid_query.validate().is_ok());

        let invalid_query = TimeConflictQuery {
            start_time: now + chrono::Duration::hours(1),
            end_time: now,
            exclude_id: None,
        };

        assert!(invalid_query.validate().is_err());
    }

    #[test]
    fn test_free_slots_query_validation() {
        let now = Utc::now();
        let valid_query = FreeSlotsQuery {
            start_time: now,
            end_time: now + chrono::Duration::hours(8),
            min_duration_minutes: 30,
        };

        assert!(valid_query.validate().is_ok());

        let invalid_query = FreeSlotsQuery {
            start_time: now,
            end_time: now + chrono::Duration::hours(8),
            min_duration_minutes: -10,
        };

        assert!(invalid_query.validate().is_err());
    }
}
