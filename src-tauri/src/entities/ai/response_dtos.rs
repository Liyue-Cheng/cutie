use serde::Serialize;

/// Token 使用统计
#[derive(Debug, Serialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// AI 聊天响应
#[derive(Debug, Serialize)]
pub struct AiChatResponse {
    pub reply: String,
    pub usage: TokenUsage,
    pub model: String,
    /// 响应时间（毫秒）
    pub response_time_ms: u64,
}
