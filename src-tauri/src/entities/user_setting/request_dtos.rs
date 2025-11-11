use serde::{Deserialize, Serialize};

use super::model::ValueType;

/// 更新单个设置的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSettingRequest {
    /// 设置值 (JSON 序列化的字符串)
    pub value: serde_json::Value,
    
    /// 值的类型
    pub value_type: ValueType,
}

/// 批量更新设置的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBatchSettingsRequest {
    /// 要更新的设置列表
    pub settings: Vec<SettingUpdate>,
}

/// 单个设置更新项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingUpdate {
    /// 设置键
    pub key: String,
    
    /// 设置值
    pub value: serde_json::Value,
    
    /// 值的类型
    pub value_type: ValueType,
}

