import { ref, computed } from 'vue'
import type { UserSettingDto } from '@/types/user-settings'
import { createLoadingState } from '@/stores/shared'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * UserSettings Store 核心状态管理 (RTL Hardware Pattern)
 *
 * 职责：
 * - 管理用户设置数据的单一数据源
 * - 提供基础的状态操作方法（寄存器写端口）
 * - 提供计算属性和过滤器（多路复用器和导线）
 *
 * Key 格式: {category}.{group?}.{name}
 * 示例: appearance.theme, ai.conversation.api_key, debug.test_string
 */

/**
 * 创建用户设置核心状态
 */
export function createUserSettingsCore() {
  // ============================================================
  // STATE - 寄存器（Registers）
  // ============================================================

  /**
   * 设置映射表 (单一数据源)
   * key: setting_key
   * value: UserSettingDto
   */
  const settings = ref(new Map<string, UserSettingDto>())

  /**
   * 加载状态管理
   */
  const { isLoading, error, withLoading } = createLoadingState()

  // ============================================================
  // GETTERS - 导线 + 多路复用器（Wires + Mux）
  // ============================================================

  /**
   * 基础数组缓存层（性能优化）
   * ✅ 只转换一次 Map → Array，所有其他 getter 复用此数组
   */
  const allSettingsArray = computed(() => {
    return Array.from(settings.value.values())
  })

  /**
   * 获取所有设置（数组形式）
   */
  const allSettings = computed(() => {
    return allSettingsArray.value
  })

  /**
   * Mux: 根据 key 获取设置值（多路复用器）
   * 自动解析 JSON 并返回实际值
   */
  function getSettingValue<T = any>(key: string, defaultValue: T): T {
    const setting = settings.value.get(key)
    if (!setting) {
      return defaultValue
    }

    try {
      return JSON.parse(setting.setting_value) as T
    } catch (error) {
      logger.warn(LogTags.STORE_USER_SETTINGS, 'Failed to parse setting value', {
        key,
        value: setting.setting_value,
        error,
      })
      return defaultValue
    }
  }

  /**
   * Mux: 获取原始设置 DTO
   */
  function getSetting_Mux(key: string): UserSettingDto | undefined {
    return settings.value.get(key)
  }

  /**
   * 根据 key 前缀获取设置（按分类筛选）
   * @param prefix key 前缀，如 'appearance', 'ai', 'debug'
   */
  function getSettingsByPrefix(prefix: string): UserSettingDto[] {
    return allSettingsArray.value.filter((s) => s.setting_key.startsWith(prefix + '.'))
  }

  // ============================================================
  // 快捷访问器 - 常用设置的专用 wire
  // ============================================================

  /**
   * 语言设置
   */
  const language = computed(() => getSettingValue('appearance.language', 'en'))

  /**
   * 显示缩放
   */
  const displayScale = computed(() => getSettingValue('appearance.display_scale', 100))

  /**
   * 主题
   */
  const theme = computed(() => getSettingValue('appearance.theme', 'business'))

  /**
   * 默认任务时长（分钟）
   */
  const defaultTaskDuration = computed(() => getSettingValue('behavior.default_task_duration', 30))

  /**
   * 工作时间开始
   */
  const workHoursStart = computed(() => getSettingValue('behavior.work_hours_start', '09:00'))

  /**
   * 工作时间结束
   */
  const workHoursEnd = computed(() => getSettingValue('behavior.work_hours_end', '18:00'))

  /**
   * 自动归档天数
   */
  const autoArchiveDays = computed(() => getSettingValue('data.auto_archive_days', 30))

  /**
   * 用户名称
   */
  const userName = computed(() => getSettingValue('account.user_name', ''))

  /**
   * 用户邮箱
   */
  const userEmail = computed(() => getSettingValue('account.user_email', ''))

  /**
   * 显示日志
   */
  const showLogs = computed(() => getSettingValue('debug.show_logs', false))

  /**
   * 日志级别
   */
  const logLevel = computed(() => getSettingValue('debug.log_level', 'info'))

  // ============================================================
  // Internal Settings - 隐藏设置（不在设置面板显示）
  // ============================================================

  // CalendarPanel 设置 - 被 HomeView 和 CalendarView 共享
  const internalCalendarDefaultViewType = computed(() =>
    getSettingValue<'week' | 'month'>('internal.calendar.default_view_type', 'month')
  )
  const internalCalendarDefaultZoom = computed(() =>
    getSettingValue<1 | 2 | 3>('internal.calendar.default_zoom', 1)
  )
  const internalCalendarMonthFilterRecurring = computed(() =>
    getSettingValue('internal.calendar.month_filter.recurring', true)
  )
  const internalCalendarMonthFilterScheduled = computed(() =>
    getSettingValue('internal.calendar.month_filter.scheduled', true)
  )
  const internalCalendarMonthFilterDueDates = computed(() =>
    getSettingValue('internal.calendar.month_filter.due_dates', true)
  )
  const internalCalendarMonthFilterAllDay = computed(() =>
    getSettingValue('internal.calendar.month_filter.all_day', true)
  )

  // Home - RecentTaskPanel 设置
  const internalHomeRecentDefaultDays = computed(() =>
    getSettingValue<1 | 3 | 5>('internal.home.recent.default_days', 3)
  )
  const internalHomeRecentShowCompleted = computed(() =>
    getSettingValue('internal.home.recent.show_completed', true)
  )
  const internalHomeRecentShowDailyRecurring = computed(() =>
    getSettingValue('internal.home.recent.show_daily_recurring', true)
  )

  // ============================================================
  // MUTATIONS - 写端口（Write Ports）
  // ============================================================

  /**
   * 添加或更新单个设置（写端口）
   */
  function addOrUpdateSetting_mut(setting: UserSettingDto) {
    settings.value.set(setting.setting_key, setting)
    logger.debug(LogTags.STORE_USER_SETTINGS, 'Setting updated in store', {
      key: setting.setting_key,
      value: setting.setting_value,
    })
  }

  /**
   * 批量添加或更新设置（写端口）
   */
  function addOrUpdateBatch_mut(settingsList: UserSettingDto[]) {
    for (const setting of settingsList) {
      settings.value.set(setting.setting_key, setting)
    }
    logger.debug(LogTags.STORE_USER_SETTINGS, 'Batch settings updated in store', {
      count: settingsList.length,
    })
  }

  /**
   * 清空所有设置（写端口）
   */
  function clearAll_mut() {
    settings.value.clear()
    logger.debug(LogTags.STORE_USER_SETTINGS, 'All settings cleared from store')
  }

  /**
   * 替换所有设置（写端口）
   * 用于重置或初始化
   */
  function replaceAll_mut(newSettings: UserSettingDto[]) {
    settings.value = new Map(newSettings.map((s) => [s.setting_key, s]))
    logger.info(LogTags.STORE_USER_SETTINGS, 'All settings replaced in store', {
      count: newSettings.length,
    })
  }

  return {
    // State (registers)
    settings,
    isLoading,
    error,
    withLoading,

    // Getters (wires + mux)
    allSettings,
    allSettingsArray,

    // Mux
    getSettingValue,
    getSetting_Mux,
    getSettingsByPrefix,

    // 快捷访问器
    language,
    displayScale,
    theme,
    defaultTaskDuration,
    workHoursStart,
    workHoursEnd,
    autoArchiveDays,
    userName,
    userEmail,
    showLogs,
    logLevel,

    // Internal Settings（隐藏设置）
    internalCalendarDefaultViewType,
    internalCalendarDefaultZoom,
    internalCalendarMonthFilterRecurring,
    internalCalendarMonthFilterScheduled,
    internalCalendarMonthFilterDueDates,
    internalCalendarMonthFilterAllDay,
    internalHomeRecentDefaultDays,
    internalHomeRecentShowCompleted,
    internalHomeRecentShowDailyRecurring,

    // Mutations (write ports)
    addOrUpdateSetting_mut,
    addOrUpdateBatch_mut,
    clearAll_mut,
    replaceAll_mut,
  }
}
