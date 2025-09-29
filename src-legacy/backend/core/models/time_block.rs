use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::SourceInfo;

/// TimeBlock (时间块) 实体定义
///
/// 代表日历上的一个有明确开始和结束时间的持续性时间段。
///
/// ## 不变量
/// - start_time必须永远小于或等于end_time
/// - end_time必须永远大于或等于start_time
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeBlock {
    /// 时间块ID (主键)
    pub id: Uuid,

    /// 时间块标题 (可选)
    ///
    /// **后置条件:** 如果存在，它将覆盖掉关联任务的标题，作为时间块的独立主题
    pub title: Option<String>,

    /// 快览笔记 (可选)
    pub glance_note: Option<String>,

    /// 详细笔记 (可选)
    pub detail_note: Option<String>,

    /// 开始时间
    ///
    /// **不变量:** start_time必须永远小于或等于end_time
    pub start_time: DateTime<Utc>,

    /// 结束时间
    ///
    /// **不变量:** end_time必须永远大于或等于start_time
    pub end_time: DateTime<Utc>,

    /// 领域ID (外键, 可选)
    ///
    /// **后置条件:** 决定此时间块在日历上的染色。它的值独立于其关联的任何Task的area_id
    pub area_id: Option<Uuid>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 逻辑删除标记
    pub is_deleted: bool,

    /// 来源信息 (可选)
    pub source_info: Option<SourceInfo>,

    /// 外部来源ID (可选)
    pub external_source_id: Option<String>,

    /// 外部来源提供商 (可选)
    pub external_source_provider: Option<String>,

    /// 外部来源元数据 (可选)
    pub external_source_metadata: Option<serde_json::Value>,

    /// 重复规则 (可选)
    pub recurrence_rule: Option<String>,

    /// 重复任务父ID (可选)
    pub recurrence_parent_id: Option<Uuid>,

    /// 重复任务原始日期 (可选)
    pub recurrence_original_date: Option<DateTime<Utc>>,

    /// 重复任务排除日期 (可选)
    pub recurrence_exclusions: Option<Vec<DateTime<Utc>>>,
}

impl TimeBlock {
    /// 创建新的时间块
    pub fn new(
        id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Result<Self, String> {
        if start_time > end_time {
            return Err("start_time must be less than or equal to end_time".to_string());
        }

        Ok(Self {
            id,
            title: None,
            glance_note: None,
            detail_note: None,
            start_time,
            end_time,
            area_id: None,
            created_at,
            updated_at: created_at,
            is_deleted: false,
            source_info: None,
            external_source_id: None,
            external_source_provider: None,
            external_source_metadata: None,
            recurrence_rule: None,
            recurrence_parent_id: None,
            recurrence_original_date: None,
            recurrence_exclusions: None,
        })
    }

    /// 获取时间块持续时间（分钟）
    pub fn duration_minutes(&self) -> i64 {
        (self.end_time - self.start_time).num_minutes()
    }

    /// 更新时间范围
    pub fn update_time_range(
        &mut self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<(), String> {
        if start_time > end_time {
            return Err("start_time must be less than or equal to end_time".to_string());
        }

        self.start_time = start_time;
        self.end_time = end_time;
        self.updated_at = updated_at;
        Ok(())
    }
}
