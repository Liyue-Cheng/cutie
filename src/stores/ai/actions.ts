import { sendChatMessage } from '@/services/ai'
import type { UserMessage, ChatMessage } from '@/types/ai'
import { addMessage, setLoading, setError, resetError, allMessages } from './core'

/**
 * 发送用户消息并获取 AI 回复（支持上下文）
 */
export async function sendMessage(userMessage: UserMessage): Promise<void> {
  // 添加用户消息到历史
  addMessage(userMessage)

  // 重置错误状态
  resetError()

  // 设置加载状态
  setLoading(true)

  try {
    // 构建完整的对话历史（包括用户和助手的所有消息）
    const conversationHistory: ChatMessage[] = []

    // 遍历所有历史消息（包括 user 和 assistant）
    for (const msg of allMessages.value) {
      const chatMsg: ChatMessage = {
        role: msg.role,
        text: msg.text,
        images: msg.role === 'user' ? msg.images : undefined, // 只有用户消息有图片
      }
      conversationHistory.push(chatMsg)
    }

    // 调用 API（发送完整对话上下文）
    const response = await sendChatMessage({
      messages: conversationHistory, // ✅ 发送完整对话历史（user + assistant）
      system: '你是一个乐于助人的 AI 助手。',
      max_tokens: 2000,
    })

    // 添加 AI 回复到历史（包含计时和模型信息）
    addMessage({
      role: 'assistant',
      text: response.reply,
      response_time_ms: response.response_time_ms,
      model: response.model,
      usage: response.usage,
    })
  } catch (err) {
    const errorMessage = err instanceof Error ? err.message : String(err)
    setError(errorMessage)
    throw err
  } finally {
    setLoading(false)
  }
}
