/// 日程功能模块的HTTP载荷定义
///
/// 定义日程相关API端点的请求体和响应体结构
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::{
    core::{Outcome, ValidationError},
    http::extractors::Validate,
};

/// 安排任务请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTaskPayload {
    /// 任务ID
    pub task_id: Uuid,

    /// 目标日期（零点时间戳）
    pub target_day: DateTime<Utc>,

    /// 操作模式：move（移动）或link（链接）
    pub mode: ScheduleMode,

    /// 源日程ID（仅在move模式下需要）
    pub source_schedule_id: Option<Uuid>,
}

impl Validate for ScheduleTaskPayload {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // 验证目标日期必须是当天的零点
        let normalized_day = crate::shared::core::normalize_to_day_start(self.target_day);
        if self.target_day != normalized_day {
            errors.push(ValidationError::new(
                "target_day",
                "目标日期必须是当天的零点时间戳",
                "INVALID_DAY_TIMESTAMP",
            ));
        }

        // 验证move模式必须提供源日程ID
        if self.mode == ScheduleMode::Move && self.source_schedule_id.is_none() {
            errors.push(ValidationError::new(
                "source_schedule_id",
                "移动模式下必须提供源日程ID",
                "SOURCE_SCHEDULE_ID_REQUIRED",
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 安排模式枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScheduleMode {
    /// 链接模式 - 为任务创建额外的日程安排
    Link,
    /// 移动模式 - 移动现有日程到新日期
    Move,
}

/// 记录努力请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPresencePayload {
    /// 可选的备注信息
    pub note: Option<String>,
}

impl Validate for LogPresencePayload {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // 验证备注长度
        if let Some(note) = &self.note {
            if note.len() > 500 {
                errors.push(ValidationError::new(
                    "note",
                    "备注不能超过500个字符",
                    "NOTE_TOO_LONG",
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

/// 日程统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleStatsResponse {
    /// 总日程数
    pub total_count: i64,

    /// 已计划日程数
    pub planned_count: i64,

    /// 已记录努力日程数
    pub presence_logged_count: i64,

    /// 当日完成日程数
    pub completed_on_day_count: i64,

    /// 延期日程数
    pub carried_over_count: i64,

    /// 本周日程数
    pub this_week_count: i64,

    /// 本月日程数
    pub this_month_count: i64,

    /// 完成率（百分比）
    pub completion_rate: f64,
}

/// 日程查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleQuery {
    /// 特定日期
    pub date: Option<DateTime<Utc>>,

    /// 开始日期
    pub start_date: Option<DateTime<Utc>>,

    /// 结束日期
    pub end_date: Option<DateTime<Utc>>,

    /// 任务ID
    pub task_id: Option<Uuid>,

    /// 结局过滤
    pub outcome: Option<Outcome>,
}

impl Validate for ScheduleQuery {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // 验证日期查询的互斥性
        if self.date.is_some() && (self.start_date.is_some() || self.end_date.is_some()) {
            errors.push(ValidationError::new(
                "date_query",
                "不能同时使用单日查询和日期范围查询",
                "CONFLICTING_DATE_QUERIES",
            ));
        }

        // 验证日期范围
        if let (Some(start), Some(end)) = (&self.start_date, &self.end_date) {
            if start >= end {
                errors.push(ValidationError::new(
                    "date_range",
                    "开始日期必须早于结束日期",
                    "INVALID_DATE_RANGE",
                ));
            }
        }

        // 验证日期范围查询的完整性
        if self.start_date.is_some() && self.end_date.is_none() {
            errors.push(ValidationError::new(
                "end_date",
                "使用日期范围查询时必须同时提供开始和结束日期",
                "END_DATE_REQUIRED",
            ));
        }

        if self.end_date.is_some() && self.start_date.is_none() {
            errors.push(ValidationError::new(
                "start_date",
                "使用日期范围查询时必须同时提供开始和结束日期",
                "START_DATE_REQUIRED",
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 批量日程操作载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkScheduleOperation {
    /// 日程ID列表
    pub schedule_ids: Vec<Uuid>,

    /// 操作类型
    pub operation: BulkScheduleOperationType,

    /// 操作参数（可选）
    pub parameters: Option<serde_json::Value>,
}

/// 批量日程操作类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BulkScheduleOperationType {
    /// 批量删除
    Delete,

    /// 批量记录努力
    LogPresence,

    /// 批量标记为延期
    MarkCarriedOver,

    /// 批量移动到新日期
    MoveToDate,
}

impl Validate for BulkScheduleOperation {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if self.schedule_ids.is_empty() {
            errors.push(ValidationError::new(
                "schedule_ids",
                "日程ID列表不能为空",
                "SCHEDULE_IDS_EMPTY",
            ));
        }

        if self.schedule_ids.len() > 100 {
            errors.push(ValidationError::new(
                "schedule_ids",
                "批量操作不能超过100个日程",
                "TOO_MANY_SCHEDULES",
            ));
        }

        // 验证移动操作的参数
        if self.operation == BulkScheduleOperationType::MoveToDate {
            if let Some(params) = &self.parameters {
                if !params.get("target_date").is_some() {
                    errors.push(ValidationError::new(
                        "parameters.target_date",
                        "移动操作必须提供目标日期",
                        "TARGET_DATE_REQUIRED",
                    ));
                }
            } else {
                errors.push(ValidationError::new(
                    "parameters",
                    "移动操作必须提供参数",
                    "PARAMETERS_REQUIRED",
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_schedule_task_payload_validation() {
        let valid_payload = ScheduleTaskPayload {
            task_id: Uuid::new_v4(),
            target_day: crate::shared::core::normalize_to_day_start(Utc::now()),
            mode: ScheduleMode::Link,
            source_schedule_id: None,
        };

        assert!(valid_payload.validate().is_ok());

        // 测试move模式缺少源日程ID
        let invalid_payload = ScheduleTaskPayload {
            task_id: Uuid::new_v4(),
            target_day: crate::shared::core::normalize_to_day_start(Utc::now()),
            mode: ScheduleMode::Move,
            source_schedule_id: None,
        };

        assert!(invalid_payload.validate().is_err());
    }

    #[test]
    fn test_schedule_query_validation() {
        // 测试冲突的日期查询
        let invalid_query = ScheduleQuery {
            date: Some(Utc::now()),
            start_date: Some(Utc::now()),
            end_date: Some(Utc::now()),
            task_id: None,
            outcome: None,
        };

        assert!(invalid_query.validate().is_err());

        // 测试有效的单日查询
        let valid_single_date = ScheduleQuery {
            date: Some(Utc::now()),
            start_date: None,
            end_date: None,
            task_id: None,
            outcome: None,
        };

        assert!(valid_single_date.validate().is_ok());

        // 测试有效的日期范围查询
        let valid_range = ScheduleQuery {
            date: None,
            start_date: Some(Utc::now()),
            end_date: Some(Utc::now() + chrono::Duration::days(7)),
            task_id: None,
            outcome: None,
        };

        assert!(valid_range.validate().is_ok());
    }

    #[test]
    fn test_bulk_schedule_operation_validation() {
        let valid_operation = BulkScheduleOperation {
            schedule_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
            operation: BulkScheduleOperationType::LogPresence,
            parameters: None,
        };

        assert!(valid_operation.validate().is_ok());

        // 测试空ID列表
        let invalid_operation = BulkScheduleOperation {
            schedule_ids: vec![],
            operation: BulkScheduleOperationType::Delete,
            parameters: None,
        };

        assert!(invalid_operation.validate().is_err());
    }
}
