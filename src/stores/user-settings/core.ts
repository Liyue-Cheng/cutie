import { ref, computed } from 'vue'
import type { UserSettingDto, SettingCategory } from '@/types/user-settings'
import { createLoadingState } from '@/stores/shared'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * UserSettings Store 核心状态管理 (RTL Hardware Pattern)
 *
 * 职责：
 * - 管理用户设置数据的单一数据源
 * - 提供基础的状态操作方法（寄存器写端口）
 * - 提供计算属性和过滤器（多路复用器和导线）
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
   * 根据 category 分组的设置
   */
  const settingsByCategory = computed(() => {
    const grouped = new Map<SettingCategory, UserSettingDto[]>()

    for (const setting of allSettingsArray.value) {
      const category = setting.category
      if (!grouped.has(category)) {
        grouped.set(category, [])
      }
      grouped.get(category)!.push(setting)
    }

    return grouped
  })

  /**
   * 获取 appearance 设置
   */
  const appearanceSettings = computed(() => {
    return settingsByCategory.value.get('appearance') || []
  })

  /**
   * 获取 behavior 设置
   */
  const behaviorSettings = computed(() => {
    return settingsByCategory.value.get('behavior') || []
  })

  /**
   * 获取 data 设置
   */
  const dataSettings = computed(() => {
    return settingsByCategory.value.get('data') || []
  })

  /**
   * 获取 account 设置
   */
  const accountSettings = computed(() => {
    return settingsByCategory.value.get('account') || []
  })

  /**
   * 获取 debug 设置
   */
  const debugSettings = computed(() => {
    return settingsByCategory.value.get('debug') || []
  })

  /**
   * 获取 AI 设置
   */
  const aiSettings = computed(() => {
    return settingsByCategory.value.get('ai') || []
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
  const theme = computed(() => getSettingValue('appearance.theme', 'auto'))

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
    settingsByCategory,
    appearanceSettings,
    behaviorSettings,
    dataSettings,
    accountSettings,
    debugSettings,
    aiSettings,

    // Mux
    getSettingValue,
    getSetting_Mux,

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

    // Mutations (write ports)
    addOrUpdateSetting_mut,
    addOrUpdateBatch_mut,
    clearAll_mut,
    replaceAll_mut,
  }
}
