import type { UserMessage, ChatMessage } from '@/types/ai'
import {
  addMessage,
  setLoading,
  setError,
  resetError,
  allMessages,
  appendToLastAssistantMessage,
  appendToolCallToAssistant,
} from './core'
import { sendChatWithTools } from '@/services/ai/client/AiClient'
import type { AssistantToolCall } from '@/types/ai'

/**
 * 使用前端 OpenAI + XML 工具流水线发送用户消息并获取 AI 回复（支持上下文与流式输出）
 */
export async function sendMessage(userMessage: UserMessage): Promise<void> {
  // 添加用户消息到历史
  addMessage(userMessage)

  // 重置错误状态
  resetError()

  // 设置加载状态
  setLoading(true)

  try {
    // 构建完整的对话历史（包括 user 和 assistant）
    const conversationHistory: ChatMessage[] = []

    for (const msg of allMessages.value) {
      const chatMsg: ChatMessage = {
        role: msg.role,
        text: msg.text,
        images: msg.role === 'user' ? msg.images : undefined,
      }
      conversationHistory.push(chatMsg)
    }

    // 启动一次新的 assistant 消息（后续通过流式增量填充）
    appendToLastAssistantMessage('')

    const result = await sendChatWithTools(conversationHistory, {
      onTextChunk: (chunk) => {
        appendToLastAssistantMessage(chunk)
      },
      onToolCallCompleted: ({ tool, result }) => {
        const toolCall: AssistantToolCall = {
          id: `${tool.name}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
          tool_name: tool.name,
          params: tool.params,
          status: result.success ? 'success' : 'error',
          message: result.message,
          data: result.data,
        }
        appendToolCallToAssistant(toolCall)
      },
    })

    // 在最后一条 assistant 消息上补充响应时间信息
    if (allMessages.value.length > 0) {
      const last = allMessages.value[allMessages.value.length - 1]
      if (last.role === 'assistant') {
        allMessages.value[allMessages.value.length - 1] = {
          ...last,
          response_time_ms: result.response_time_ms,
          model: result.model,
        }
      }
    }
  } catch (err) {
    const errorMessage = err instanceof Error ? err.message : String(err)
    setError(errorMessage)
    throw err
  } finally {
    setLoading(false)
  }
}
