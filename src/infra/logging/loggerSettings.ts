/**
 * 日志系统设置 - 直接配置，无需环境变量
 * 基于黑白名单系统，方便调试
 */

import { LogLevel } from './logger'

// ==================== 基础配置 ====================

/**
 * 全局日志级别
 * 可以随时修改这个值来调整整体日志输出
 */
export const GLOBAL_LOG_LEVEL = LogLevel.DEBUG

/**
 * 是否启用控制台输出
 */
export const ENABLE_CONSOLE = true

/**
 * 是否启用远程上报
 */
export const ENABLE_REMOTE = false

/**
 * 远程上报端点
 */
export const REMOTE_ENDPOINT = '/api/logs'

// ==================== 黑白名单系统 ====================

/**
 * 标签白名单 - 只显示这些标签的日志
 * 如果为空数组，则显示所有标签
 * 如果有内容，则只显示匹配的标签
 */
export const TAG_WHITELIST: string[] = [
  // 取消注释下面的标签来启用白名单过滤
  // 'Component:InfiniteDailyKanban',
  // 'Component:Kanban:Column',
  // 'Drag:Strategy',
  // 'API:ViewAdapter',
  // 'Store:Tasks',
]

/**
 * 标签黑名单 - 隐藏这些标签的日志
 * 即使在白名单中，黑名单也会生效
 */
export const TAG_BLACKLIST: string[] = [
  // 默认隐藏一些噪音较大的标签
  'Store:View',
  'Drag:Strategy',
  'Drag:CrossView',
  'Drag:Context',
  // 'Component:Kanban:Column', // 如果看板列日志太多可以取消注释
]

/**
 * 级别白名单 - 只显示这些级别的日志
 * 如果为空数组，则显示所有级别
 */
export const LEVEL_WHITELIST: LogLevel[] = [
  // 取消注释下面的级别来启用级别过滤
  // LogLevel.INFO,
  // LogLevel.WARN,
  // LogLevel.ERROR,
]

/**
 * 级别黑名单 - 隐藏这些级别的日志
 */
export const LEVEL_BLACKLIST: LogLevel[] = [
  // 默认不隐藏任何级别
  // LogLevel.DEBUG, // 如果调试日志太多可以取消注释
]

// ==================== 采样配置 ====================

/**
 * 采样率配置 - 控制各级别日志的显示比例
 * 1.0 = 100% 显示，0.5 = 50% 显示，0.0 = 不显示
 */
export const SAMPLING_RATES = {
  [LogLevel.DEBUG]: 1.0, // 调试日志 100% 显示
  [LogLevel.INFO]: 1.0, // 信息日志 100% 显示
  [LogLevel.WARN]: 1.0, // 警告日志 100% 显示
  [LogLevel.ERROR]: 1.0, // 错误日志 100% 显示
  [LogLevel.SILENT]: 0.0, // 静默级别不显示
}

// ==================== 特殊场景配置 ====================

/**
 * 调试模式 - 快速切换到调试配置
 * 设为 true 时会覆盖部分设置，便于调试特定功能
 */
export const DEBUG_MODE = false

/**
 * 调试模式下的特殊配置
 */
export const DEBUG_CONFIG = {
  // 调试模式下只关注这些标签
  focusTags: ['Drag:Strategy', 'Drag:CrossView', 'Component:InfiniteDailyKanban'],
  // 调试模式下的日志级别
  level: LogLevel.DEBUG,
  // 调试模式下是否显示所有上下文信息
  showFullContext: true,
}

// ==================== 性能配置 ====================

/**
 * 用户轨迹缓冲区大小
 */
export const RING_BUFFER_SIZE = 50

/**
 * 远程上报批量大小
 */
export const REMOTE_BATCH_SIZE = 10

/**
 * 远程上报间隔（毫秒）
 */
export const REMOTE_FLUSH_INTERVAL = 2000

/**
 * 最大重试次数
 */
export const MAX_RETRIES = 3

// ==================== 快速配置预设 ====================

/**
 * 预设配置 - 可以快速切换到不同的调试场景
 */
export const PRESETS = {
  // 默认配置 - 显示所有日志
  default: {
    level: LogLevel.DEBUG,
    tagWhitelist: [],
    tagBlacklist: [],
  },

  // 只看错误和警告
  errorsOnly: {
    level: LogLevel.WARN,
    tagWhitelist: [],
    tagBlacklist: [],
  },

  // 只看拖拽相关
  dragOnly: {
    level: LogLevel.DEBUG,
    tagWhitelist: ['Drag:Strategy', 'Drag:CrossView', 'Drag:Context'],
    tagBlacklist: [],
  },

  // 只看API相关
  apiOnly: {
    level: LogLevel.DEBUG,
    tagWhitelist: ['API:ViewAdapter', 'API:Tasks', 'API:Timeblock'],
    tagBlacklist: [],
  },

  // 只看组件相关
  componentsOnly: {
    level: LogLevel.DEBUG,
    tagWhitelist: [
      'Component:InfiniteDailyKanban',
      'Component:Kanban:Column',
      'Component:CuteCalendar',
    ],
    tagBlacklist: [],
  },

  // 性能调试 - 只看关键信息
  performance: {
    level: LogLevel.INFO,
    tagWhitelist: ['Perf', 'System:Init'],
    tagBlacklist: [],
  },
}

// ==================== 辅助函数 ====================

/**
 * 应用预设配置
 * 在浏览器控制台中调用：applyLoggerPreset('dragOnly')
 */
export function applyPreset(presetName: keyof typeof PRESETS) {
  const preset = PRESETS[presetName]
  if (!preset) {
    console.warn(`Unknown preset: ${presetName}`)
    return
  }

  // 这里可以动态修改配置
  console.log(`Applied logger preset: ${presetName}`, preset)
  return preset
}

/**
 * 检查标签是否应该被过滤
 */
export function shouldFilterTag(tag: string): boolean {
  // 防御性检查：如果 tag 是 undefined 或空，不过滤
  if (!tag) {
    return false
  }

  // 黑名单优先
  if (TAG_BLACKLIST.some((blackTag) => tag.includes(blackTag))) {
    return true
  }

  // 如果有白名单，检查是否在白名单中
  if (TAG_WHITELIST.length > 0) {
    return !TAG_WHITELIST.some((whiteTag) => tag.includes(whiteTag))
  }

  return false
}

/**
 * 检查级别是否应该被过滤
 */
export function shouldFilterLevel(level: LogLevel): boolean {
  // 防御性检查：如果 level 是 undefined，不过滤
  if (level === undefined || level === null) {
    return false
  }

  // 黑名单优先
  if (LEVEL_BLACKLIST.includes(level)) {
    return true
  }

  // 如果有白名单，检查是否在白名单中
  if (LEVEL_WHITELIST.length > 0) {
    return !LEVEL_WHITELIST.includes(level)
  }

  return false
}

/**
 * 获取采样率
 */
export function getSamplingRate(level: LogLevel): number {
  return SAMPLING_RATES[level] || 1.0
}
