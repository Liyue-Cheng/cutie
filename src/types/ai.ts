// AI 功能相关类型定义

/**
 * 消息图片类型
 */
export type MessageImage = { kind: 'url'; data: string } | { kind: 'base64'; data: string }

/**
 * 用户消息
 */
export interface UserMessage {
  role: 'user'
  text: string
  images?: MessageImage[]
}

/**
 * AI 助手消息
 */
export interface AssistantMessage {
  role: 'assistant'
  text: string
  response_time_ms?: number // 响应时间（毫秒）
  model?: string // 使用的模型
  usage?: TokenUsage // Token 使用情况
  tool_calls?: AssistantToolCall[]
}

export interface AssistantToolCall {
  id: string
  tool_name: string
  params: Record<string, string>
  status: 'success' | 'error'
  message: string
  data?: any
}

/**
 * 聊天消息（用户或助手）
 */
export type ChatMessage = UserMessage | AssistantMessage

/**
 * AI 聊天请求
 */
export interface AiChatRequest {
  messages: ChatMessage[] // ✅ 支持完整对话历史
  system?: string
  max_tokens?: number
}

/**
 * Token 使用统计
 */
export interface TokenUsage {
  prompt_tokens: number
  completion_tokens: number
  total_tokens: number
}

/**
 * AI 聊天响应
 */
export interface AiChatResponse {
  reply: string
  usage: TokenUsage
  model: string
  response_time_ms: number
}
