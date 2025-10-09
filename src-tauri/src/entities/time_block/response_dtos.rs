/// TimeBlock 响应 DTOs - 对应前端的视图模型
///
/// 这些 DTOs 与前端的 dtos.ts 中定义的类型一一对应
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::TimeType;

/// TimeBlockView (时间块视图模型)
///
/// 对应前端: src/types/dtos.ts 中的 TimeBlockView
/// 用途: 在日历时间轴上显示的时间块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBlockViewDto {
    // --- 核心身份与时间 ---
    pub id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    /// 本地开始时间 (HH:MM:SS)，仅在time_type=FLOATING时有值
    pub start_time_local: Option<String>,
    /// 本地结束时间 (HH:MM:SS)，仅在time_type=FLOATING时有值
    pub end_time_local: Option<String>,
    /// 时间类型
    pub time_type: TimeType,
    /// 创建时的时区（占位字段）
    pub creation_timezone: Option<String>,
    pub is_all_day: bool,

    // --- 显示内容 ---
    pub title: Option<String>,
    pub glance_note: Option<String>,
    pub detail_note: Option<String>,

    // --- 染色信息 ---
    pub area_id: Option<Uuid>, // ✅ 前端通过 area_id 从 area store 获取完整信息

    // --- 关联的任务摘要 ---
    pub linked_tasks: Vec<LinkedTaskSummary>,

    // --- 其他元信息 ---
    pub is_recurring: bool,
}

/// 关联任务摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedTaskSummary {
    pub id: Uuid,
    pub title: String,
    pub is_completed: bool,
}

// 注意：从 TimeBlock 实体转换为 TimeBlockViewDto 的逻辑
// 应该在 features/time_blocks/shared/assembler.rs 中实现
