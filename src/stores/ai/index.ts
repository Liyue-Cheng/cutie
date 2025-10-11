import { defineStore } from 'pinia'
import * as core from './core'
import * as actions from './actions'

export const useAiStore = defineStore('ai', () => {
  return {
    // State & Getters
    messages: core.messages,
    isLoading: core.isLoading,
    error: core.error,
    allMessages: core.allMessages,
    hasMessages: core.hasMessages,
    lastMessage: core.lastMessage,

    // Mutations
    addMessage: core.addMessage,
    clearMessages: core.clearMessages,
    setLoading: core.setLoading,
    setError: core.setError,
    resetError: core.resetError,

    // Actions
    sendMessage: actions.sendMessage,
  }
})
