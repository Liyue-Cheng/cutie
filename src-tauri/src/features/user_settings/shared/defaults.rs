use crate::entities::user_setting::{SettingCategory, UserSetting, ValueType};
use std::collections::HashMap;

/// 默认设置项定义
pub struct DefaultSetting {
    pub key: &'static str,
    pub value: &'static str,
    pub value_type: ValueType,
    pub category: SettingCategory,
}

/// 获取所有默认设置
pub fn get_default_settings() -> Vec<DefaultSetting> {
    vec![
        // Appearance 设置
        DefaultSetting {
            key: "appearance.language",
            value: "\"en\"",
            value_type: ValueType::String,
            category: SettingCategory::Appearance,
        },
        DefaultSetting {
            key: "appearance.display_scale",
            value: "100",
            value_type: ValueType::Number,
            category: SettingCategory::Appearance,
        },
        DefaultSetting {
            key: "appearance.theme",
            value: "\"auto\"",
            value_type: ValueType::String,
            category: SettingCategory::Appearance,
        },
        // Behavior 设置
        DefaultSetting {
            key: "behavior.default_task_duration",
            value: "30",
            value_type: ValueType::Number,
            category: SettingCategory::Behavior,
        },
        DefaultSetting {
            key: "behavior.work_hours_start",
            value: "\"09:00\"",
            value_type: ValueType::String,
            category: SettingCategory::Behavior,
        },
        DefaultSetting {
            key: "behavior.work_hours_end",
            value: "\"18:00\"",
            value_type: ValueType::String,
            category: SettingCategory::Behavior,
        },
        // Data 设置
        DefaultSetting {
            key: "data.auto_archive_days",
            value: "30",
            value_type: ValueType::Number,
            category: SettingCategory::Data,
        },
        // Account 设置
        DefaultSetting {
            key: "account.user_name",
            value: "\"\"",
            value_type: ValueType::String,
            category: SettingCategory::Account,
        },
        DefaultSetting {
            key: "account.user_email",
            value: "\"\"",
            value_type: ValueType::String,
            category: SettingCategory::Account,
        },
    ]
}

/// 获取默认设置的 Map
pub fn get_default_settings_map() -> HashMap<String, DefaultSetting> {
    get_default_settings()
        .into_iter()
        .map(|s| (s.key.to_string(), s))
        .collect()
}

/// 获取指定 key 的默认值
pub fn get_default_value(key: &str) -> Option<DefaultSetting> {
    get_default_settings()
        .into_iter()
        .find(|s| s.key == key)
}

/// 创建默认设置实体
pub fn create_default_setting_entity(key: &str) -> Option<UserSetting> {
    get_default_value(key).map(|default| {
        UserSetting::new(
            key.to_string(),
            default.value.to_string(),
            default.value_type,
            default.category,
        )
    })
}

/// 创建所有默认设置实体
pub fn create_all_default_entities() -> Vec<UserSetting> {
    get_default_settings()
        .into_iter()
        .map(|default| {
            UserSetting::new(
                default.key.to_string(),
                default.value.to_string(),
                default.value_type,
                default.category,
            )
        })
        .collect()
}

