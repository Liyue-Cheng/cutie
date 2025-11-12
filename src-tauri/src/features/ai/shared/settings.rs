use sqlx::SqlitePool;

use crate::{
    features::user_settings::shared::{get_default_value, UserSettingRepository},
    infra::core::{AppError, AppResult},
};

/// 对话模型 API Base URL 设置键
pub const AI_CONVERSATION_API_BASE_URL: &str = "ai.conversation.api_base_url";
/// 对话模型 API Key 设置键
pub const AI_CONVERSATION_API_KEY: &str = "ai.conversation.api_key";
/// 对话模型 Model 设置键
pub const AI_CONVERSATION_MODEL: &str = "ai.conversation.model";

/// 快速模型 API Base URL 设置键
pub const AI_QUICK_API_BASE_URL: &str = "ai.quick.api_base_url";
/// 快速模型 API Key 设置键
pub const AI_QUICK_API_KEY: &str = "ai.quick.api_key";
/// 快速模型 Model 设置键
pub const AI_QUICK_MODEL: &str = "ai.quick.model";

/// AI 模型配置
#[derive(Debug, Clone)]
pub struct AiModelConfig {
    pub api_base_url: String,
    pub api_key: String,
    pub model: String,
}

impl AiModelConfig {
    /// 检查配置是否完整
    pub fn is_complete(&self) -> bool {
        !self.api_base_url.is_empty() && !self.api_key.is_empty() && !self.model.is_empty()
    }

    /// 校验配置是否完整，不完整时返回配置错误
    pub fn require_complete(self, model_kind: &str) -> AppResult<Self> {
        if self.is_complete() {
            Ok(self)
        } else {
            Err(AppError::configuration_error(format!(
                "{model_kind} configuration is incomplete. Please set api_base_url, api_key and model."
            )))
        }
    }
}

/// 加载对话模型配置
pub async fn load_conversation_model_config(pool: &SqlitePool) -> AppResult<AiModelConfig> {
    load_model_config(
        pool,
        AI_CONVERSATION_API_BASE_URL,
        AI_CONVERSATION_API_KEY,
        AI_CONVERSATION_MODEL,
    )
    .await
}

/// 加载快速模型配置
pub async fn load_quick_model_config(pool: &SqlitePool) -> AppResult<AiModelConfig> {
    load_model_config(
        pool,
        AI_QUICK_API_BASE_URL,
        AI_QUICK_API_KEY,
        AI_QUICK_MODEL,
    )
    .await
}

async fn load_model_config(
    pool: &SqlitePool,
    base_url_key: &str,
    api_key_key: &str,
    model_key: &str,
) -> AppResult<AiModelConfig> {
    let api_base_url = fetch_setting_string(pool, base_url_key).await?;
    let api_key = fetch_setting_string(pool, api_key_key).await?;
    let model = fetch_setting_string(pool, model_key).await?;

    Ok(AiModelConfig {
        api_base_url: sanitize_url(api_base_url),
        api_key: api_key.trim().to_string(),
        model: model.trim().to_string(),
    })
}

async fn fetch_setting_string(pool: &SqlitePool, key: &str) -> AppResult<String> {
    if let Some(setting) = UserSettingRepository::find_by_key(pool, key).await? {
        return parse_string_value(&setting.setting_value, key);
    }

    if let Some(default) = get_default_value(key) {
        return parse_string_value(default.value, key);
    }

    Ok(String::new())
}

fn parse_string_value(value: &str, key: &str) -> AppResult<String> {
    serde_json::from_str::<String>(value).map_err(|err| {
        AppError::configuration_error(format!(
            "Setting '{}' has invalid string value: {}",
            key, err
        ))
    })
}

fn sanitize_url(url: String) -> String {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        trimmed.trim_end_matches('/').to_string()
    }
}
