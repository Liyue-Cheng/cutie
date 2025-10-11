import { ref, computed } from 'vue'
import type { ChatMessage } from '@/types/ai'

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

