use crate::shared::core::error::{AppError, AppResult};
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, ChatCompletionRequestUserMessageContentPart,
        CreateChatCompletionRequestArgs, ImageDetail, ImageUrl,
    },
    Client,
};

use super::config::{
    DEFAULT_MAX_TOKENS, DEFAULT_MODEL, OPENAI_API_KEY, OPENAI_BASE_URL, REQUEST_TIMEOUT_SECS,
};

/// OpenAI 客户端封装
pub struct OpenAIClient {
    client: Client<OpenAIConfig>,
}

impl OpenAIClient {
    /// 创建新的 OpenAI 客户端
    pub fn new() -> Self {
        let config = OpenAIConfig::new()
            .with_api_key(OPENAI_API_KEY)
            .with_api_base(OPENAI_BASE_URL);

        Self {
            client: Client::with_config(config),
        }
    }

    /// 发送聊天请求（支持多轮对话上下文）
    pub async fn chat(
        &self,
        messages: Vec<crate::entities::ai::ChatMessage>,
        system: Option<String>,
        max_tokens: Option<u32>,
    ) -> AppResult<crate::entities::ai::AiChatResponse> {
        self.chat_internal(messages, system, max_tokens, false)
            .await
    }

    /// 发送聊天请求（禁用思维链）
    pub async fn chat_no_thinking(
        &self,
        messages: Vec<crate::entities::ai::ChatMessage>,
        system: Option<String>,
        max_tokens: Option<u32>,
    ) -> AppResult<crate::entities::ai::AiChatResponse> {
        self.chat_internal(messages, system, max_tokens, true).await
    }

    /// 内部聊天方法
    async fn chat_internal(
        &self,
        messages: Vec<crate::entities::ai::ChatMessage>,
        system: Option<String>,
        max_tokens: Option<u32>,
        disable_thinking: bool,
    ) -> AppResult<crate::entities::ai::AiChatResponse> {
        // 开始计时
        let start_time = std::time::Instant::now();
        // 构建消息列表
        let mut request_messages: Vec<ChatCompletionRequestMessage> = Vec::new();

        // 添加系统消息（如果有）
        if let Some(system_content) = system {
            let system_message = ChatCompletionRequestSystemMessageArgs::default()
                .content(system_content)
                .build()
                .map_err(|e| {
                    AppError::external_dependency_failed(
                        "OpenAI",
                        format!("Failed to build system message: {}", e),
                    )
                })?;
            request_messages.push(ChatCompletionRequestMessage::System(system_message));
        }

        // 添加对话历史（支持 user 和 assistant）
        for msg in messages {
            if msg.role == "assistant" {
                // 添加助手消息
                let assistant_message =
                    async_openai::types::ChatCompletionRequestAssistantMessageArgs::default()
                        .content(msg.text)
                        .build()
                        .map_err(|e| {
                            AppError::external_dependency_failed(
                                "OpenAI",
                                format!("Failed to build assistant message: {}", e),
                            )
                        })?;
                request_messages.push(ChatCompletionRequestMessage::Assistant(assistant_message));
            } else {
                // 添加用户消息（支持文本和图片）
                let mut content_parts: Vec<ChatCompletionRequestUserMessageContentPart> =
                    Vec::new();

                // 添加文本内容
                if !msg.text.is_empty() {
                    content_parts.push(ChatCompletionRequestUserMessageContentPart::Text(
                        async_openai::types::ChatCompletionRequestMessageContentPartText {
                            text: msg.text,
                        },
                    ));
                }

                // 添加图片内容
                for image in msg.images {
                    let image_url = match image {
                        crate::entities::ai::MessageImage::Url { data } => ImageUrl {
                            url: data,
                            detail: Some(ImageDetail::Auto),
                        },
                        crate::entities::ai::MessageImage::Base64 { data } => ImageUrl {
                            url: data,
                            detail: Some(ImageDetail::Auto),
                        },
                    };
                    content_parts.push(ChatCompletionRequestUserMessageContentPart::ImageUrl(
                        async_openai::types::ChatCompletionRequestMessageContentPartImage {
                            image_url,
                        },
                    ));
                }

                let user_message = ChatCompletionRequestUserMessageArgs::default()
                    .content(content_parts)
                    .build()
                    .map_err(|e| {
                        AppError::external_dependency_failed(
                            "OpenAI",
                            format!("Failed to build user message: {}", e),
                        )
                    })?;

                request_messages.push(ChatCompletionRequestMessage::User(user_message));
            }
        }

        // 如果需要禁用思维链，使用直接的 HTTP 请求
        let response = if disable_thinking {
            // 构建请求体（包含 thinking.type: disabled）
            let request_body = serde_json::json!({
                "model": DEFAULT_MODEL,
                "messages": request_messages.iter().map(|msg| {
                    match msg {
                        ChatCompletionRequestMessage::System(m) => {
                            serde_json::json!({
                                "role": "system",
                                "content": m.content
                            })
                        },
                        ChatCompletionRequestMessage::User(m) => {
                            // 处理用户消息（可能包含文本和图片）
                            serde_json::json!({
                                "role": "user",
                                "content": m.content
                            })
                        },
                        ChatCompletionRequestMessage::Assistant(m) => {
                            serde_json::json!({
                                "role": "assistant",
                                "content": m.content
                            })
                        },
                        _ => serde_json::json!({})
                    }
                }).collect::<Vec<_>>(),
                "max_tokens": max_tokens.unwrap_or(DEFAULT_MAX_TOKENS),
                "thinking": {
                    "type": "disabled"
                }
            });

            tracing::debug!(
                target: "AI:CLIENT",
                "Sending request with thinking disabled"
            );

            // 使用 reqwest 直接发送请求
            let client = reqwest::Client::new();
            let api_response = tokio::time::timeout(
                std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS),
                client
                    .post(format!("{}/chat/completions", OPENAI_BASE_URL))
                    .header("Authorization", format!("Bearer {}", OPENAI_API_KEY))
                    .header("Content-Type", "application/json")
                    .json(&request_body)
                    .send(),
            )
            .await
            .map_err(|_| AppError::external_dependency_failed("OpenAI", "Request timeout"))?
            .map_err(|e| {
                AppError::external_dependency_failed("OpenAI", format!("HTTP error: {}", e))
            })?;

            let response_json: serde_json::Value = api_response.json().await.map_err(|e| {
                AppError::external_dependency_failed("OpenAI", format!("JSON parse error: {}", e))
            })?;

            // 手动解析响应
            let reply = response_json["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("No response from AI")
                .to_string();

            let usage = response_json["usage"].as_object().ok_or_else(|| {
                AppError::external_dependency_failed("OpenAI", "No usage info in response")
            })?;

            let elapsed = start_time.elapsed();
            let response_time_ms = elapsed.as_millis() as u64;

            return Ok(crate::entities::ai::AiChatResponse {
                reply,
                usage: crate::entities::ai::TokenUsage {
                    prompt_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                    completion_tokens: usage["completion_tokens"].as_u64().unwrap_or(0) as u32,
                    total_tokens: usage["total_tokens"].as_u64().unwrap_or(0) as u32,
                },
                model: response_json["model"]
                    .as_str()
                    .unwrap_or(DEFAULT_MODEL)
                    .to_string(),
                response_time_ms,
            });
        } else {
            // 使用标准的 async-openai 客户端
            let request = CreateChatCompletionRequestArgs::default()
                .model(DEFAULT_MODEL)
                .messages(request_messages)
                .max_tokens(max_tokens.unwrap_or(DEFAULT_MAX_TOKENS))
                .build()
                .map_err(|e| {
                    AppError::external_dependency_failed(
                        "OpenAI",
                        format!("Failed to build chat request: {}", e),
                    )
                })?;

            tokio::time::timeout(
                std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS),
                self.client.chat().create(request),
            )
            .await
            .map_err(|_| AppError::external_dependency_failed("OpenAI", "Request timeout"))?
            .map_err(|e| {
                AppError::external_dependency_failed("OpenAI", format!("API error: {}", e))
            })?
        };

        // 提取响应内容（仅用于标准 async-openai 响应）
        let reply = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .unwrap_or_else(|| "No response from AI".to_string());

        // 提取 token 使用情况
        let usage = response.usage.ok_or_else(|| {
            AppError::external_dependency_failed("OpenAI", "No usage info in response")
        })?;

        // 计算耗时
        let elapsed = start_time.elapsed();
        let response_time_ms = elapsed.as_millis() as u64;

        Ok(crate::entities::ai::AiChatResponse {
            reply,
            usage: crate::entities::ai::TokenUsage {
                prompt_tokens: usage.prompt_tokens,
                completion_tokens: usage.completion_tokens,
                total_tokens: usage.total_tokens,
            },
            model: response.model,
            response_time_ms,
        })
    }
}

impl Default for OpenAIClient {
    fn default() -> Self {
        Self::new()
    }
}
