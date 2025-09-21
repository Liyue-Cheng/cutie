use crate::core::db::DbPool;
use crate::core::models::Setting;
use anyhow::{anyhow, Result};
use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequest},
    Client,
};
use futures_util::Stream;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct AiConfig {
    api_key: String,
    base_url: String,
    model: String,
}

pub enum ModelType {
    Fast,
    Slow,
}

/// Fetches AI configuration from the database settings.
/// It looks for keys like 'ai_fast_model_id', 'ai_fast_api_key', 'ai_fast_base_url'.
async fn get_ai_config(pool: &DbPool, model_type: ModelType) -> Result<AiConfig> {
    let prefix = match model_type {
        ModelType::Fast => "ai_fast",
        ModelType::Slow => "ai_slow",
    };

    let settings = sqlx::query_as::<_, Setting>("SELECT key, value FROM settings WHERE key LIKE $1")
        .bind(format!("{}%", prefix))
        .fetch_all(pool)
        .await?;

    let mut config_map: HashMap<String, String> = HashMap::new();
    for setting in settings {
        if let Some(value) = setting.value.as_str() {
            config_map.insert(setting.key, value.to_string());
        }
    }

    let model_key = format!("{}_model_id", prefix);
    let api_key_key = format!("{}_api_key", prefix);
    let base_url_key = format!("{}_base_url", prefix);

    let config = AiConfig {
        model: config_map
            .get(&model_key)
            .ok_or_else(|| anyhow!("Missing setting: {}", model_key))?
            .clone(),
        api_key: config_map
            .get(&api_key_key)
            .ok_or_else(|| anyhow!("Missing setting: {}", api_key_key))?
            .clone(),
        base_url: config_map
            .get(&base_url_key)
            .ok_or_else(|| anyhow!("Missing setting: {}", base_url_key))?
            .clone(),
    };

    Ok(config)
}

/// Creates a streaming chat completion request.
pub async fn stream_chat_completion(
    pool: &DbPool,
    model_type: ModelType,
    messages: Vec<ChatCompletionRequestMessage>,
) -> Result<impl Stream<Item = Result<String, async_openai::error::OpenAIError>>> {
    // 1. Get AI config from the database
    let ai_config = get_ai_config(pool, model_type).await?;

    // 2. Create OpenAI client with dynamic configuration
    let config = OpenAIConfig::new()
        .with_api_key(ai_config.api_key)
        .with_api_base(ai_config.base_url);

    let client = Client::with_config(config);

    // 3. Create the chat request
    let request = CreateChatCompletionRequest {
        model: ai_config.model,
        messages,
        stream: Some(true),
        ..Default::default()
    };

    // 4. Get the stream
    let mut stream = client.chat().create_stream(request).await?;

    // 5. Map the stream to extract just the text content
    let mapped_stream = async_stream::stream! {
        use futures_util::StreamExt;
        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    if let Some(choice) = response.choices.first() {
                        if let Some(content) = &choice.delta.content {
                            yield Ok(content.clone());
                        }
                    }
                }
                Err(e) => {
                    yield Err(e.into());
                    break;
                }
            }
        }
    };

    Ok(mapped_stream)
}