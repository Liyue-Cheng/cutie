/**
 * UserSettings Store（用户设置存储）
 *
 * RTL Hardware Pattern:
 * - core.ts: 寄存器 + 导线 + 多路复用器
 * - event-handlers.ts: 中断处理器
 * - index.ts: 对外暴露接口
 */

import { defineStore } from 'pinia'
import { createUserSettingsCore } from './core'
import { createEventHandlers } from './event-handlers'

export const useUserSettingsStore = defineStore('user-settings', () => {
  // 创建核心状态
  const core = createUserSettingsCore()

  // 创建事件处理器
  const eventHandlers = createEventHandlers(core)

  return {
    // State
    settings: core.settings,
    isLoading: core.isLoading,
    error: core.error,

    // Getters
    allSettings: core.allSettings,
    settingsByCategory: core.settingsByCategory,
    appearanceSettings: core.appearanceSettings,
    behaviorSettings: core.behaviorSettings,
    dataSettings: core.dataSettings,
    accountSettings: core.accountSettings,
    debugSettings: core.debugSettings,
    aiSettings: core.aiSettings,

    // Mux
    getSettingValue: core.getSettingValue,
    getSetting_Mux: core.getSetting_Mux,

    // 快捷访问器
    language: core.language,
    displayScale: core.displayScale,
    theme: core.theme,
    defaultTaskDuration: core.defaultTaskDuration,
    workHoursStart: core.workHoursStart,
    workHoursEnd: core.workHoursEnd,
    autoArchiveDays: core.autoArchiveDays,
    userName: core.userName,
    userEmail: core.userEmail,
    showLogs: core.showLogs,
    logLevel: core.logLevel,

    // Mutations
    addOrUpdateSetting_mut: core.addOrUpdateSetting_mut,
    addOrUpdateBatch_mut: core.addOrUpdateBatch_mut,
    clearAll_mut: core.clearAll_mut,
    replaceAll_mut: core.replaceAll_mut,

    // Event handlers
    initEventSubscriptions: eventHandlers.initEventSubscriptions,
  }
})

