import { ref, computed } from 'vue'
import type { AssistantToolCall, ChatMessage } from '@/types/ai'

// ==================== State ====================
export const messages = ref<ChatMessage[]>([])
export const isLoading = ref(false)
export const error = ref<string | null>(null)

// ==================== Getters ====================
export const allMessages = computed(() => messages.value)
export const hasMessages = computed(() => messages.value.length > 0)
export const lastMessage = computed(() => messages.value[messages.value.length - 1])

// ==================== Mutations ====================
export function addMessage(message: ChatMessage) {
  messages.value = [...messages.value, message]
}

export function clearMessages() {
  messages.value = []
}

export function setLoading(loading: boolean) {
  isLoading.value = loading
}

export function setError(err: string | null) {
  error.value = err
}

export function resetError() {
  error.value = null
}

// 追加内容到最后一条 assistant 消息（用于流式输出）
export function appendToLastAssistantMessage(text: string) {
  if (!text) return

  const list = messages.value
  const last = list[list.length - 1]

  if (!last || last.role !== 'assistant') {
    const newMsg: ChatMessage = {
      role: 'assistant',
      text,
      tool_calls: [],
    }
    messages.value = [...list, newMsg]
    return
  }

  const updated: ChatMessage = {
    ...last,
    text: (last.text || '') + text,
  }

  messages.value = [...list.slice(0, list.length - 1), updated]
}

export function appendToolCallToAssistant(toolCall: AssistantToolCall) {
  const list = messages.value
  const last = list[list.length - 1]

  if (!last || last.role !== 'assistant') {
    const newMsg: ChatMessage = {
      role: 'assistant',
      text: '',
      tool_calls: [toolCall],
    }
    messages.value = [...list, newMsg]
    return
  }

  const updated: ChatMessage = {
    ...last,
    tool_calls: [...(last.tool_calls || []), toolCall],
  }

  messages.value = [...list.slice(0, list.length - 1), updated]
}
