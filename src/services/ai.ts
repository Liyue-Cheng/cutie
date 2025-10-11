import { apiBaseUrl } from '@/composables/useApiConfig'
import type { AiChatRequest, AiChatResponse } from '@/types/ai'

/**
 * 发送 AI 聊天请求
 */
export async function sendChatMessage(request: AiChatRequest): Promise<AiChatResponse> {
  const response = await fetch(`${apiBaseUrl.value}/ai/chat`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(request),
  })

  if (!response.ok) {
    const error = await response.text()
    throw new Error(`AI 请求失败: ${error}`)
  }

  // ✅ 正确：提取 .data 字段（遵循 ApiResponse 包装格式）
  const responseData = await response.json()
  const data: AiChatResponse = responseData.data

  return data
}
