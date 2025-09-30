/// Task相关的值对象
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// 子任务结构
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
pub struct Subtask {
    /// 子任务ID
    pub id: Uuid,
    /// 子任务标题
    pub title: String,
    /// 是否完成
    pub is_completed: bool,
    /// 排序顺序
    pub sort_order: String,
}

/// 来源信息结构
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/")]
pub struct SourceInfo {
    /// 来源类型
    pub source_type: String,
    /// 来源描述
    pub description: Option<String>,
    /// 来源URL
    pub url: Option<String>,
}
