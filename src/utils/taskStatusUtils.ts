/**
 * 任务状态计算工具函数
 *
 * 根据任务的 schedules 数组实时计算状态，不再依赖 schedule_status 字段
 */

import type { TaskCard } from '@/types/dtos'

/**
 * 计算任务的调度状态
 *
 * 规则：
 * - 如果有今天或未来的日程 → 'scheduled'
 * - 否则 → 'staging'
 */
export function getTaskScheduleStatus(task: TaskCard): 'scheduled' | 'staging' {
  if (!task.schedules || task.schedules.length === 0) {
    return 'staging'
  }

  const today = new Date().toISOString().split('T')[0]!
  const hasFutureOrTodaySchedule = task.schedules.some(
    (schedule) => schedule.scheduled_day >= today
  )

  return hasFutureOrTodaySchedule ? 'scheduled' : 'staging'
}

/**
 * 判断任务是否为 staging 状态
 */
export function isTaskStaging(task: TaskCard): boolean {
  return getTaskScheduleStatus(task) === 'staging'
}

/**
 * 判断任务是否为 scheduled 状态
 */
export function isTaskScheduled(task: TaskCard): boolean {
  return getTaskScheduleStatus(task) === 'scheduled'
}

/**
 * 判断任务在指定日期是否有日程
 */
export function hasScheduleOnDate(task: TaskCard, date: string): boolean {
  if (!task.schedules) return false
  return task.schedules.some((schedule) => schedule.scheduled_day === date)
}

/**
 * 获取任务的所有日程日期（排序后）
 */
export function getScheduleDates(task: TaskCard): string[] {
  if (!task.schedules) return []
  return task.schedules.map((s) => s.scheduled_day).sort()
}
