use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 用户设置实体
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSetting {
    /// 设置项的唯一标识符
    pub setting_key: String,

    /// 设置值 (JSON 格式)
    pub setting_value: String,

    /// 设置值的数据类型
    pub value_type: ValueType,

    /// 设置项的分类
    pub category: SettingCategory,

    /// 最后更新时间
    pub updated_at: DateTime<Utc>,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 设置值的数据类型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum ValueType {
    String,
    Number,
    Boolean,
    Object,
    Array,
}

/// 设置项的分类
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum SettingCategory {
    Appearance,
    Behavior,
    Data,
    Account,
    Debug,
    System,
}

impl UserSetting {
    /// 创建新的设置项
    pub fn new(
        setting_key: String,
        setting_value: String,
        value_type: ValueType,
        category: SettingCategory,
    ) -> Self {
        let now = Utc::now();
        Self {
            setting_key,
            setting_value,
            value_type,
            category,
            updated_at: now,
            created_at: now,
        }
    }

    /// 更新设置值
    pub fn update_value(&mut self, setting_value: String, value_type: ValueType) {
        self.setting_value = setting_value;
        self.value_type = value_type;
        self.updated_at = Utc::now();
    }
}
