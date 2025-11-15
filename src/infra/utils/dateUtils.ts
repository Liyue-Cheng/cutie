/**
 * 日期与时间工具函数
 *
 * ⚠️ 重要：项目时间模型规范（详见 docs/TIME_CONVENTION.md）
 *
 * 核心原则：
 * 1. 后端使用"本地时间模型"（无时区偏移）
 * 2. UTC 仅用于系统内部元数据（如 created_at, updated_at）
 * 3. 所有用户意图相关的时间（任务、日程、时间块）均使用本地时间
 *
 * 禁止行为：
 * - ❌ 不要对用户时间使用 Date.toISOString()（会转换为 UTC）
 * - ❌ 不要对日期判断使用 new Date().toISOString().split('T')[0]
 *
 * 推荐做法：
 * - ✅ 日期：使用 getTodayDateString() 或 toDateString()
 * - ✅ 时间：使用 formatLocalDateTime() 或 toLocalISOString()
 * - ✅ 显示：使用 formatTime() 或 formatDateTime()
 */

// ==================== 获取当前时间/日期 ====================

/**
 * 获取今天的日期字符串（用户本地时区）
 * @returns YYYY-MM-DD 格式的日期字符串（如 "2025-10-08"）
 */
export function getTodayDateString(): string {
  return new Date().toLocaleDateString('en-CA') // en-CA locale 保证 YYYY-MM-DD 格式
}

/**
 * 获取当前本地时间的 Date 对象
 * @returns Date 对象（当前本地时间）
 */
export function getNowLocal(): Date {
  return new Date()
}

/**
 * 获取当前用户的时区信息
 * @returns 时区字符串（如 "Asia/Shanghai", "America/New_York"）
 */
export function getLocalTimezone(): string {
  return Intl.DateTimeFormat().resolvedOptions().timeZone
}

/**
 * 获取当前用户的时区偏移（分钟）
 * @returns 时区偏移分钟数（如 -480 表示 UTC+8）
 */
export function getTimezoneOffset(): number {
  return new Date().getTimezoneOffset()
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
  const parts = dateStr.split('-').map(Number)
  const year = parts[0] ?? 0
  const month = parts[1] ?? 1
  const day = parts[2] ?? 1
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
    minute: '2-digit',
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
    day: 'numeric',
  })
}

// ==================== 本地时间处理函数（用户意图相关） ====================

/**
 * 将 Date 对象转换为本地 ISO 字符串（无时区偏移）
 *
 * ⚠️ 用于与后端通信时，传递用户意图的时间（任务、日程、时间块）
 *
 * @param date - Date 对象
 * @returns 本地时间的 ISO 格式字符串（如 "2025-10-08T14:30:00"，无 Z 后缀）
 *
 * @example
 * const now = new Date('2025-10-08T14:30:00+08:00')
 * toLocalISOString(now) // "2025-10-08T14:30:00"
 */
export function toLocalISOString(date: Date): string {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const hours = String(date.getHours()).padStart(2, '0')
  const minutes = String(date.getMinutes()).padStart(2, '0')
  const seconds = String(date.getSeconds()).padStart(2, '0')

  return `${year}-${month}-${day}T${hours}:${minutes}:${seconds}`
}

/**
 * 格式化本地日期时间为可读字符串
 *
 * @param date - Date 对象或 ISO 字符串
 * @returns 格式化的日期时间字符串（如 "2025年10月8日 14:30"）
 *
 * @example
 * formatDateTime(new Date()) // "2025年10月8日 14:30"
 */
export function formatDateTime(date: Date | string): string {
  const d = typeof date === 'string' ? new Date(date) : date
  return d.toLocaleString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

/**
 * 格式化本地时间为 12 小时制（如 "9:30 AM"）
 *
 * @param date - Date 对象或 ISO 字符串
 * @returns 格式化的时间字符串
 *
 * @example
 * formatTime(new Date('2025-10-08T09:30:00')) // "9:30 AM"
 * formatTime(new Date('2025-10-08T14:30:00')) // "2:30 PM"
 */
export function formatTime(date: Date | string): string {
  const d = typeof date === 'string' ? new Date(date) : date
  let hours = d.getHours()
  const minutes = d.getMinutes()
  const period = hours >= 12 ? 'PM' : 'AM'
  hours = hours % 12 || 12
  const minutesStr = String(minutes).padStart(2, '0')
  return `${hours}:${minutesStr} ${period}`
}

/**
 * 格式化本地时间为 24 小时制（如 "09:30"）
 *
 * @param date - Date 对象或 ISO 字符串
 * @returns 格式化的时间字符串
 *
 * @example
 * formatTime24(new Date('2025-10-08T09:30:00')) // "09:30"
 * formatTime24(new Date('2025-10-08T14:30:00')) // "14:30"
 */
export function formatTime24(date: Date | string): string {
  const d = typeof date === 'string' ? new Date(date) : date
  const hours = String(d.getHours()).padStart(2, '0')
  const minutes = String(d.getMinutes()).padStart(2, '0')
  return `${hours}:${minutes}`
}

/**
 * 获取当前时间的本地 ISO 字符串
 *
 * ⚠️ 用于创建新任务、时间块时的默认时间
 *
 * @returns 当前本地时间的 ISO 格式字符串（无 Z 后缀）
 *
 * @example
 * getNowLocalISOString() // "2025-10-08T14:30:00"
 */
export function getNowLocalISOString(): string {
  return toLocalISOString(new Date())
}

/**
 * 解析本地 ISO 字符串为 Date 对象
 *
 * ⚠️ 用于解析后端返回的本地时间字符串
 *
 * @param localIsoString - 本地时间的 ISO 格式字符串（如 "2025-10-08T14:30:00"）
 * @returns Date 对象
 *
 * @example
 * parseLocalISOString("2025-10-08T14:30:00") // Date 对象（本地时区）
 */
export function parseLocalISOString(localIsoString: string): Date {
  // 如果字符串已经包含时区信息（Z 或 +/-），直接解析
  if (localIsoString.includes('Z') || /[+-]\d{2}:\d{2}$/.test(localIsoString)) {
    return new Date(localIsoString)
  }

  // 否则，当作本地时间解析
  // 注意：new Date(string) 对于没有时区的 ISO 字符串会当作 UTC
  // 所以我们需要手动解析
  const [datePart, timePart] = localIsoString.split('T')
  if (!datePart) {
    return new Date() // 如果解析失败，返回当前时间
  }

  const dateParts = datePart.split('-').map(Number)
  const year = dateParts[0] ?? 0
  const month = dateParts[1] ?? 1
  const day = dateParts[2] ?? 1

  if (!timePart) {
    // 只有日期，返回当天零点
    return new Date(year, month - 1, day)
  }

  const timeParts = timePart.split(':').map(Number)
  const hours = timeParts[0] ?? 0
  const minutes = timeParts[1] ?? 0
  const seconds = timeParts[2] ?? 0
  return new Date(year, month - 1, day, hours, minutes, seconds)
}

/**
 * 获取昨天的日期字符串
 *
 * @returns YYYY-MM-DD 格式的日期字符串
 */
export function getYesterdayDateString(): string {
  const yesterday = new Date()
  yesterday.setDate(yesterday.getDate() - 1)
  return toDateString(yesterday)
}

/**
 * 获取明天的日期字符串
 *
 * @returns YYYY-MM-DD 格式的日期字符串
 */
export function getTomorrowDateString(): string {
  const tomorrow = new Date()
  tomorrow.setDate(tomorrow.getDate() + 1)
  return toDateString(tomorrow)
}
