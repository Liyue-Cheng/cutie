use serde::Deserialize;

/// 消息图片类型
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum MessageImage {
    /// URL 图片
    #[serde(rename = "url")]
    Url { data: String },
    /// Base64 图片
    #[serde(rename = "base64")]
    Base64 { data: String },
}

/// 聊天消息（支持用户和助手）
#[derive(Debug, Clone, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user" 或 "assistant"
    pub text: String,
    #[serde(default)]
    pub images: Vec<MessageImage>,
}

/// 用户消息（向后兼容）
pub type UserMessage = ChatMessage;

/// AI 聊天请求
#[derive(Debug, Deserialize)]
pub struct AiChatRequest {
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub system: Option<String>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
}
