use crate::entities::user_setting::{UserSetting, ValueType};
use std::collections::HashMap;

/// 默认设置项定义
pub struct DefaultSetting {
    pub key: &'static str,
    pub value: &'static str,
    pub value_type: ValueType,
}

/// 获取所有默认设置
pub fn get_default_settings() -> Vec<DefaultSetting> {
    vec![
        // Appearance 设置
        DefaultSetting {
            key: "appearance.theme",
            value: "\"business\"",
            value_type: ValueType::String,
        },
        // AI 设置 - 会根据模型类型区分对话与快速模型
        DefaultSetting {
            key: "ai.conversation.api_base_url",
            value: "\"\"",
            value_type: ValueType::String,
        },
        DefaultSetting {
            key: "ai.conversation.api_key",
            value: "\"\"",
            value_type: ValueType::String,
        },
        DefaultSetting {
            key: "ai.conversation.model",
            value: "\"\"",
            value_type: ValueType::String,
        },
        DefaultSetting {
            key: "ai.quick.api_base_url",
            value: "\"\"",
            value_type: ValueType::String,
        },
        DefaultSetting {
            key: "ai.quick.api_key",
            value: "\"\"",
            value_type: ValueType::String,
        },
        DefaultSetting {
            key: "ai.quick.model",
            value: "\"\"",
            value_type: ValueType::String,
        },
        // Debug 测试设置 - 用于测试各种数据类型和功能
        DefaultSetting {
            key: "debug.test_string",
            value: "\"Hello World\"",
            value_type: ValueType::String,
        },
        DefaultSetting {
            key: "debug.test_number",
            value: "42",
            value_type: ValueType::Number,
        },
        DefaultSetting {
            key: "debug.test_boolean",
            value: "false",
            value_type: ValueType::Boolean,
        },
        DefaultSetting {
            key: "debug.test_float",
            value: "3.14",
            value_type: ValueType::Number,
        },
        DefaultSetting {
            key: "debug.test_toggle",
            value: "true",
            value_type: ValueType::Boolean,
        },
        // ==================== Internal Settings ====================
        // 这些设置不在设置面板显示，通过 UI 交互自动保存
        // CalendarPanel 设置 - 被 HomeView 和 CalendarView 共享
        DefaultSetting {
            key: "internal.calendar.default_view_type",
            value: "\"month\"",
            value_type: ValueType::String,
        },
        DefaultSetting {
            key: "internal.calendar.default_zoom",
            value: "1",
            value_type: ValueType::Number,
        },
        DefaultSetting {
            key: "internal.calendar.month_filter.recurring",
            value: "true",
            value_type: ValueType::Boolean,
        },
        DefaultSetting {
            key: "internal.calendar.month_filter.scheduled",
            value: "true",
            value_type: ValueType::Boolean,
        },
        DefaultSetting {
            key: "internal.calendar.month_filter.due_dates",
            value: "true",
            value_type: ValueType::Boolean,
        },
        DefaultSetting {
            key: "internal.calendar.month_filter.all_day",
            value: "true",
            value_type: ValueType::Boolean,
        },
        // Home - RecentTaskPanel 设置
        DefaultSetting {
            key: "internal.home.recent.default_days",
            value: "3",
            value_type: ValueType::Number,
        },
        DefaultSetting {
            key: "internal.home.recent.show_completed",
            value: "true",
            value_type: ValueType::Boolean,
        },
        DefaultSetting {
            key: "internal.home.recent.show_daily_recurring",
            value: "true",
            value_type: ValueType::Boolean,
        },
        // ==================== Task Behavior Settings ====================
        // 任务行为相关设置
        DefaultSetting {
            key: "task.completion.create_schedule_on_complete",
            value: "true",
            value_type: ValueType::Boolean,
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
    get_default_settings().into_iter().find(|s| s.key == key)
}

/// 创建默认设置实体
pub fn create_default_setting_entity(key: &str) -> Option<UserSetting> {
    get_default_value(key).map(|default| {
        UserSetting::new(
            key.to_string(),
            default.value.to_string(),
            default.value_type,
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
            )
        })
        .collect()
}
