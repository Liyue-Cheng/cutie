/**
 * UserSettings Store（用户设置存储）
 *
 * RTL Hardware Pattern:
 * - core.ts: 寄存器 + 导线 + 多路复用器
 * - event-handlers.ts: 中断处理器
 * - index.ts: 对外暴露接口
 *
 * Key 格式: {category}.{group?}.{name}
 * 示例: appearance.theme, ai.conversation.api_key, debug.test_string
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

    // Mux
    getSettingValue: core.getSettingValue,
    getSetting_Mux: core.getSetting_Mux,
    getSettingsByPrefix: core.getSettingsByPrefix,

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

    // Internal Settings（隐藏设置）
    internalCalendarDefaultViewType: core.internalCalendarDefaultViewType,
    internalCalendarDefaultZoom: core.internalCalendarDefaultZoom,
    internalCalendarMonthFilterRecurring: core.internalCalendarMonthFilterRecurring,
    internalCalendarMonthFilterScheduled: core.internalCalendarMonthFilterScheduled,
    internalCalendarMonthFilterDueDates: core.internalCalendarMonthFilterDueDates,
    internalCalendarMonthFilterAllDay: core.internalCalendarMonthFilterAllDay,
    internalHomeRecentDefaultDays: core.internalHomeRecentDefaultDays,
    internalHomeRecentShowCompleted: core.internalHomeRecentShowCompleted,
    internalHomeRecentShowDailyRecurring: core.internalHomeRecentShowDailyRecurring,

    // Mutations
    addOrUpdateSetting_mut: core.addOrUpdateSetting_mut,
    addOrUpdateBatch_mut: core.addOrUpdateBatch_mut,
    clearAll_mut: core.clearAll_mut,
    replaceAll_mut: core.replaceAll_mut,

    // Event handlers
    initEventSubscriptions: eventHandlers.initEventSubscriptions,
  }
})
