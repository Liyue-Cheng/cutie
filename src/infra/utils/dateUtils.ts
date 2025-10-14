/**
 * 日期与时间工具函数
 * 
 * TIME_REFACTOR_RFC 黄金标准：
 * - 瞬时刻（Instant）：存储与传输使用 UTC ISO-8601（含 Z）
 * - 日历日期（Calendar Date）：使用 YYYY-MM-DD 纯字符串（无时区）
 * - 前端仅在显示层转换时区，业务逻辑保持原始格式
 */

/**
 * 获取今天的日期字符串（用户本地时区）
 * @returns YYYY-MM-DD 格式的日期字符串（如 "2025-10-08"）
 */
export function getTodayDateString(): string {
  return new Date().toLocaleDateString('en-CA') // en-CA locale 保证 YYYY-MM-DD 格式
}

/**
 * 将 Date 对象转换为 UTC ISO 字符串
 * @param date - Date 对象
 * @returns UTC ISO-8601 字符串（如 "2025-10-08T14:30:00.000Z"）
 */
export function toUtcIsoString(date: Date): string {
  return date.toISOString()
}

/**
 * 将 Date 对象转换为日历日期字符串（YYYY-MM-DD）
 * @param date - Date 对象
 * @returns YYYY-MM-DD 格式的日期字符串
 */
export function toDateString(date: Date): string {
  return date.toLocaleDateString('en-CA')
}

/**
 * 解析 YYYY-MM-DD 字符串为 Date 对象（本地时区零点）
 * @param dateStr - YYYY-MM-DD 格式的日期字符串
 * @returns Date 对象（本地时区的当天零点）
 */
export function parseDateString(dateStr: string): Date {
  // 不带时间部分，避免时区转换歧义
  const [year, month, day] = dateStr.split('-').map(Number)
  return new Date(year, month - 1, day)
}

/**
 * 比较两个日期字符串是否表示同一天
 * @param dateStr1 - YYYY-MM-DD 格式的日期字符串
 * @param dateStr2 - YYYY-MM-DD 格式的日期字符串
 * @returns 是否同一天
 */
export function isSameDate(dateStr1: string, dateStr2: string): boolean {
  return dateStr1 === dateStr2
}

/**
 * 从 UTC 时间戳中提取日期字符串（用户本地时区）
 * @param utcIsoString - UTC ISO-8601 字符串
 * @returns YYYY-MM-DD 格式的日期字符串
 */
export function extractDateFromUtc(utcIsoString: string): string {
  return new Date(utcIsoString).toLocaleDateString('en-CA')
}

/**
 * 格式化 UTC 时间戳为本地可读字符串
 * @param utcIsoString - UTC ISO-8601 字符串
 * @returns 本地化的日期时间字符串（如 "2025年10月8日 14:30"）
 */
export function formatUtcToLocal(utcIsoString: string): string {
  return new Date(utcIsoString).toLocaleString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}

/**
 * 格式化相对时间（用于"几分钟前"/"几小时前"等）
 * @param utcIsoString - UTC ISO-8601 字符串
 * @returns 相对时间描述
 */
export function formatRelativeTime(utcIsoString: string): string {
  const date = new Date(utcIsoString)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)

  if (diffMins < 1) return '刚刚'
  if (diffMins < 60) return `${diffMins} 分钟前`

  const diffHours = Math.floor(diffMins / 60)
  if (diffHours < 24) return `${diffHours} 小时前`

  const diffDays = Math.floor(diffHours / 24)
  if (diffDays < 7) return `${diffDays} 天前`

  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  })
}

