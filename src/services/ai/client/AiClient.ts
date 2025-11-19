import { generateSystemPrompt } from '../prompts/system'
import { AssistantMessageParser } from '../parser/AssistantMessageParser'
import { executeToolCall } from '../executor/ToolExecutor'
import type { ChatMessage } from '@/types/ai'
import { useUserSettingsStore } from '@/stores/user-settings'
import type { ToolUse, ToolResult } from '../shared/cutie-tools'
import { logger, LogTags } from '@/infra/logging'

export interface AiStreamCallbacks {
  onTextChunk: (text: string) => void
  onToolCallCompleted?: (payload: { tool: ToolUse; result: ToolResult }) => void
}

interface OpenAIChatMessage {
  role: 'system' | 'user' | 'assistant'
  content:
    | string
    | Array<
        | { type: 'text'; text: string }
        | { type: 'image_url'; image_url: { url: string; detail?: 'auto' | 'low' | 'high' } }
      >
}

function buildOpenAIMessages(chatMessages: ChatMessage[]): OpenAIChatMessage[] {
  const systemPrompt = generateSystemPrompt()
  const result: OpenAIChatMessage[] = [
    {
      role: 'system',
      content: systemPrompt,
    },
  ]

  for (const msg of chatMessages) {
    if (msg.role === 'user') {
      if (msg.images && msg.images.length > 0) {
        const parts: OpenAIChatMessage['content'] = []
        if (msg.text.trim().length > 0) {
          parts.push({ type: 'text', text: msg.text })
        }
        for (const img of msg.images) {
          parts.push({
            type: 'image_url',
            image_url: {
              url: img.data,
              detail: 'auto',
            },
          })
        }
        result.push({
          role: 'user',
          content: parts,
        })
      } else {
        result.push({
          role: 'user',
          content: msg.text,
        })
      }
    } else {
      // assistant 历史消息仅保留文本
      result.push({
        role: 'assistant',
        content: msg.text,
      })
    }
  }

  return result
}

async function* streamOpenAIChat(
  apiBaseUrl: string,
  apiKey: string,
  model: string,
  messages: OpenAIChatMessage[]
): AsyncGenerator<string> {
  const url = `${apiBaseUrl.replace(/\/$/, '')}/chat/completions`

  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${apiKey}`,
    },
    body: JSON.stringify({
      model,
      messages,
      stream: true,
    }),
  })

  if (!response.ok || !response.body) {
    const text = await response.text()
    throw new Error(`AI 请求失败：${text}`)
  }

  const reader = response.body.getReader()
  const decoder = new TextDecoder('utf-8')
  let buffer = ''

  while (true) {
    const { done, value } = await reader.read()
    if (done) break
    buffer += decoder.decode(value, { stream: true })

    const lines = buffer.split('\n')
    buffer = lines.pop() ?? ''

    for (const line of lines) {
      const trimmed = line.trim()
      if (!trimmed.startsWith('data:')) continue
      const data = trimmed.replace(/^data:\s*/, '')
      if (data === '[DONE]') return
      try {
        const parsed = JSON.parse(data)
        const delta = parsed.choices?.[0]?.delta
        const content: string | undefined = delta?.content
        if (content) {
          yield content
        }
      } catch {
        // 忽略无法解析的行
      }
    }
  }
}

/**
 * 使用 Kilocode 风格的 AssistantMessageParser 解析流式响应并执行工具调用
 */
export async function sendChatWithTools(
  chatMessages: ChatMessage[],
  callbacks: AiStreamCallbacks
): Promise<{ response_time_ms: number; model?: string }> {
  const settingsStore = useUserSettingsStore()
  const apiBaseUrl = settingsStore.getSettingValue('ai.conversation.api_base_url', '')
  const apiKey = settingsStore.getSettingValue('ai.conversation.api_key', '')
  const model = settingsStore.getSettingValue('ai.conversation.model', '')

  if (!apiBaseUrl || !apiKey || !model) {
    throw new Error('请先在设置中配置 AI 对话模型的 API Base URL / API Key / Model')
  }

  const openAiMessages = buildOpenAIMessages(chatMessages)
  const parser = new AssistantMessageParser()
  const start = performance.now()

  let lastProcessedToolCount = 0
  let streamedTextLength = 0
  const assistantChunks: string[] = []

  const emitChunk = (chunk: string) => {
    if (!chunk) return
    callbacks.onTextChunk(chunk)
    assistantChunks.push(chunk)
  }

  logger.info(LogTags.AI_CLIENT, '开始 AI 请求', { model, messageCount: chatMessages.length })

  for await (const textChunk of streamOpenAIChat(apiBaseUrl, apiKey, model, openAiMessages)) {
    // 使用 Kilocode 的 AssistantMessageParser 解析文本流
    const contentBlocks = parser.processChunk(textChunk)

    // 遍历解析出的内容块
    for (const block of contentBlocks) {
      if (block.type === 'text') {
        const content = block.content ?? ''
        if (content.length > streamedTextLength) {
          const delta = content.slice(streamedTextLength)
          if (delta.trim().length > 0) {
            emitChunk(delta)
          }
          streamedTextLength = content.length
        } else if (content.length < streamedTextLength) {
          streamedTextLength = content.length
        }
      } else if (block.type === 'tool_use') {
        // 工具调用完成（!partial）才执行
        if (!block.partial) {
          const toolIndex = contentBlocks.filter((b) => b.type === 'tool_use').indexOf(block)
          if (toolIndex >= lastProcessedToolCount) {
            logger.info(LogTags.AI_CLIENT, '执行工具调用', {
              toolName: block.name,
              params: block.params,
            })
            const result = await executeToolCall(block as ToolUse)
            logger.info(LogTags.AI_CLIENT, '工具执行结果', {
              toolName: block.name,
              success: result.success,
              message: result.message,
            })
            callbacks.onToolCallCompleted?.({ tool: block as ToolUse, result })
            lastProcessedToolCount = toolIndex + 1
          }
        }
      }
    }
  }

  // 流结束后最终化所有块
  parser.finalizeContentBlocks()

  const responseTime = Math.round(performance.now() - start)
  const responseText = assistantChunks.join('').trim()

  logger.info(LogTags.AI_CLIENT, 'AI 请求完成', {
    responseTimeMs: responseTime,
    responseText,
  })

  return {
    response_time_ms: responseTime,
    model,
  }
}
