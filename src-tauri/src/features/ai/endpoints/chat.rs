/// AI 聊天端点 - 单文件组件
///
/// ⚠️ 开发前必读:
/// 1. 使用 async-openai SDK 调用 OpenAI API
/// 2. 支持多模态输入（文本 + 图片）
/// 3. 配置在 shared/config.rs 中写死（仅开发用）
// ==================== CABC 文档 ====================
/*
CABC for `ai_chat`

## 1. 端点签名
POST /api/ai/chat

## 2. 预期行为简介

### 2.1 用户故事
> 作为用户，我想要与 AI 聊天，可以发送文本和图片，以便获得智能回复

### 2.2 核心业务逻辑
接收用户消息（支持文本和图片），调用 OpenAI API，返回 AI 回复和 token 使用情况

## 3. 输入输出规范

### 3.1 请求 (Request)
{
  "messages": [
    {
      "role": "user",
      "text": "帮我写一首诗",
      "images": [
        { "kind": "url", "data": "https://example.com/image.png" },
        { "kind": "base64", "data": "data:image/png;base64,..." }
      ]
    }
  ],
  "system": "你是一个乐于助人的助手",
  "max_tokens": 500
}

### 3.2 响应 (Responses)
**200 OK:**
{
  "data": {
    "reply": "这是 AI 的回复内容",
    "usage": {
      "prompt_tokens": 123,
      "completion_tokens": 45,
      "total_tokens": 168
    },
    "model": "gpt-4o-mini"
  },
  "timestamp": "2025-10-10T10:00:00Z",
  "request_id": null
}

## 4. 验证规则
- messages: 必须，非空数组
- messages[].text: 必须，非空字符串

## 5. 业务逻辑详解
1. 验证输入
2. 创建 OpenAI 客户端
3. 调用 OpenAI API
4. 返回结果

## 6. 边界情况
- messages 为空: 返回 422
- OpenAI API 超时: 返回 503
- OpenAI API 错误: 返回 502

## 7. 预期副作用
无数据库操作，无 SSE 事件

## 8. 契约
### 前置条件:
- 配置了有效的 OpenAI API Key

### 后置条件:
- 返回 AI 回复和 token 使用情况

### 不变量:
- 不修改任何持久化数据
*/
// ==================== 依赖引入 ====================
use axum::{extract::State, response::IntoResponse, Json};

use crate::{
    entities::ai::{AiChatRequest, AiChatResponse},
    infra::{
        core::{AppError, AppResult, ValidationError},
        http::error_handler::success_response,
    },
    startup::AppState,
};

use super::super::shared::{load_conversation_model_config, OpenAIClient};

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<AiChatRequest>,
) -> impl IntoResponse {
    match logic::execute(&app_state, request).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate_request(request: &AiChatRequest) -> AppResult<()> {
        let mut errors = Vec::new();

        // 验证 messages
        if request.messages.is_empty() {
            errors.push(ValidationError::new(
                "messages",
                "messages cannot be empty",
                "MESSAGES_EMPTY",
            ));
        }

        // 验证每条消息的文本
        for (i, msg) in request.messages.iter().enumerate() {
            if msg.text.trim().is_empty() {
                errors.push(ValidationError::new(
                    format!("messages[{}].text", i),
                    "text cannot be empty",
                    "TEXT_EMPTY",
                ));
            }
        }

        if !errors.is_empty() {
            return Err(AppError::ValidationFailed(errors));
        }

        Ok(())
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        request: AiChatRequest,
    ) -> AppResult<AiChatResponse> {
        // 1. 验证
        validation::validate_request(&request)?;

        // 2. 加载对话模型配置
        let model_config = load_conversation_model_config(app_state.db_pool())
            .await?
            .require_complete("AI conversation model")?;

        // 3. 创建 OpenAI 客户端
        let client = OpenAIClient::new(model_config);

        // 4. 调用 OpenAI API
        let response = client
            .chat(request.messages, request.system, request.max_tokens)
            .await?;

        Ok(response)
    }
}
