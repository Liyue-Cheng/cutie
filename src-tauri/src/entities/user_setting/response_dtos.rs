use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::model::{SettingCategory, UserSetting, ValueType};

/// 用户设置响应DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettingDto {
    /// 设置键
    pub setting_key: String,

    /// 设置值 (JSON)
    pub setting_value: String,

    /// 值的类型
    pub value_type: ValueType,

    /// 设置分类
    pub category: SettingCategory,

    /// 最后更新时间
    pub updated_at: DateTime<Utc>,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl From<UserSetting> for UserSettingDto {
    fn from(setting: UserSetting) -> Self {
        Self {
            setting_key: setting.setting_key,
            setting_value: setting.setting_value,
            value_type: setting.value_type,
            category: setting.category,
            updated_at: setting.updated_at,
            created_at: setting.created_at,
        }
    }
}

/// 批量更新响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpdateResponse {
    /// 更新的设置数量
    pub updated_count: usize,

    /// 更新后的设置列表
    pub settings: Vec<UserSettingDto>,
}

/// 重置响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetResponse {
    /// 重置的设置数量
    pub reset_count: usize,

    /// 默认设置列表
    pub settings: Vec<UserSettingDto>,
}
