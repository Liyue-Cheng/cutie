/**
 * useUpcomingTasks.ts - Upcoming 视图共享逻辑
 *
 * 提供 Upcoming 视图（横向和竖向）共享的任务过滤、分组和时间范围计算逻辑
 */
import { computed, ref, onMounted } from 'vue'
import { useTaskStore } from '@/stores/task'
import { useRecurrenceStore } from '@/stores/recurrence'
import { getTodayDateString } from '@/infra/utils/dateUtils'
import { logger, LogTags } from '@/infra/logging/logger'
import type { TaskCard } from '@/types/dtos'

// ==================== 类型定义 ====================
export const TIME_RANGES = ['overdue', 'today', 'thisWeek', 'nextWeek', 'thisMonth', 'later'] as const
export type TimeRange = (typeof TIME_RANGES)[number]
export type TaskType = 'dueDate' | 'recurrence' | 'scheduled'

export const TIME_RANGE_LABELS: Record<TimeRange, string> = {
  overdue: '逾期',
  today: '今日',
  thisWeek: '本周',
  nextWeek: '下周',
  thisMonth: '本月',
  later: '更远',
}

// ==================== 工具函数 ====================

/**
 * 对循环任务进行去重，每个循环规则只保留最近的未完成任务
 */
function deduplicateRecurringTasks(tasks: TaskCard[]): TaskCard[] {
  const recurrenceGroups = new Map<string, TaskCard[]>()
  const nonRecurringTasks: TaskCard[] = []

  for (const task of tasks) {
    if (task.recurrence_id) {
      const group = recurrenceGroups.get(task.recurrence_id) || []
      group.push(task)
      recurrenceGroups.set(task.recurrence_id, group)
    } else {
      nonRecurringTasks.push(task)
    }
  }

  const filteredRecurringTasks: TaskCard[] = []
  for (const [, groupTasks] of recurrenceGroups) {
    const incompleteTasks = groupTasks.filter((t) => !t.is_completed)
    if (incompleteTasks.length > 0) {
      incompleteTasks.sort((a, b) => {
        const dateA = a.recurrence_original_date || ''
        const dateB = b.recurrence_original_date || ''
        return dateA.localeCompare(dateB)
      })
      filteredRecurringTasks.push(incompleteTasks[0]!)
    }
  }

  return [...nonRecurringTasks, ...filteredRecurringTasks]
}

// ==================== Composable ====================

export function useUpcomingTasks() {
  const taskStore = useTaskStore()
  const recurrenceStore = useRecurrenceStore()

  // ==================== 乐观更新 ====================
  const completingTaskIds = ref<Set<string>>(new Set())

  function handleTaskCompleting(taskId: string) {
    completingTaskIds.value.add(taskId)
    setTimeout(() => {
      completingTaskIds.value.delete(taskId)
    }, 300)
  }

  // ==================== 初始化 ====================
  async function initialize() {
    logger.info(LogTags.VIEW_UPCOMING, 'Initializing upcoming tasks...')
    await Promise.all([
      taskStore.fetchAllIncompleteTasks_DMA(),
      recurrenceStore.fetchAllRecurrences(),
    ])
    logger.info(LogTags.VIEW_UPCOMING, 'Loaded tasks and recurrences', {
      taskCount: taskStore.incompleteTasks.length,
      recurrenceCount: recurrenceStore.allRecurrences.length,
    })
  }

  // ==================== 时间范围边界计算 ====================
  const timeRangeBoundaries = computed(() => {
    const today = getTodayDateString()
    const todayDate = new Date(today + 'T00:00:00')

    // 本周末（周日）
    const endOfWeek = new Date(todayDate)
    const currentDayOfWeek = todayDate.getDay()
    const daysUntilSunday = currentDayOfWeek === 0 ? 7 : 7 - currentDayOfWeek
    endOfWeek.setDate(endOfWeek.getDate() + daysUntilSunday)
    const endOfWeekStr = endOfWeek.toLocaleDateString('en-CA')

    // 下周末（下个周日）
    const endOfNextWeek = new Date(endOfWeek)
    endOfNextWeek.setDate(endOfNextWeek.getDate() + 7)
    const endOfNextWeekStr = endOfNextWeek.toLocaleDateString('en-CA')

    // 本月末
    const endOfMonth = new Date(todayDate.getFullYear(), todayDate.getMonth() + 1, 0)
    const endOfMonthStr = endOfMonth.toLocaleDateString('en-CA')

    return {
      today,
      endOfWeek: endOfWeekStr,
      endOfNextWeek: endOfNextWeekStr,
      endOfMonth: endOfMonthStr,
    }
  })

  // ==================== 活跃任务 ====================
  // ⚠️ 过滤 EXPIRE 过期任务并去重循环任务
  const activeTasks = computed(() => {
    const today = getTodayDateString()

    // 1. 基础过滤
    const filtered = taskStore.allTasks.filter((task) => {
      if (completingTaskIds.value.has(task.id)) return true
      if (task.is_completed || task.is_deleted || task.is_archived) return false
      // 2. 过滤 EXPIRE 过期任务
      if (taskStore.isExpiredRecurringTask(task, today)) return false
      return true
    })

    // 3. 去重循环任务
    return deduplicateRecurringTasks(filtered)
  })

  // ==================== 日期范围判断 ====================
  function isDateInRange(dateStr: string, timeRange: TimeRange): boolean {
    const { today, endOfWeek, endOfNextWeek, endOfMonth } = timeRangeBoundaries.value

    switch (timeRange) {
      case 'overdue':
        return dateStr < today
      case 'today':
        return dateStr === today
      case 'thisWeek':
        return dateStr > today && dateStr <= endOfWeek
      case 'nextWeek':
        return dateStr > endOfWeek && dateStr <= endOfNextWeek
      case 'thisMonth':
        if (endOfNextWeek > endOfMonth) {
          return dateStr > endOfWeek && dateStr <= endOfMonth
        } else {
          return dateStr > endOfNextWeek && dateStr <= endOfMonth
        }
      case 'later':
        return dateStr > endOfMonth
      default:
        return false
    }
  }

  // ==================== 获取任务日期 ====================
  function getTaskDate(task: TaskCard): string | null {
    const today = getTodayDateString()

    // 优先级：截止日期 > 循环日期 > 排期日期
    if (task.due_date?.date) {
      return task.due_date.date
    }

    if (task.recurrence_id && task.recurrence_original_date) {
      return task.recurrence_original_date
    }

    if (task.schedules && task.schedules.length > 0) {
      const sortedSchedules = task.schedules.map((s) => s.scheduled_day).sort()
      const futureSchedule = sortedSchedules.find((date) => date >= today)
      if (futureSchedule) {
        return futureSchedule
      }
      return sortedSchedules[sortedSchedules.length - 1] ?? null
    }

    return null
  }

  // ==================== 按时间范围获取任务（竖排视图用） ====================
  function getTasksForRange(timeRange: TimeRange): TaskCard[] {
    return activeTasks.value.filter((task) => {
      // 逾期只筛选有截止日期的任务
      if (timeRange === 'overdue') {
        if (!task.due_date?.date) return false
        return task.due_date.date < timeRangeBoundaries.value.today
      }

      const dateToCheck = getTaskDate(task)
      if (!dateToCheck) return false
      return isDateInRange(dateToCheck, timeRange)
    })
  }

  // ==================== 按任务类型和时间范围获取任务（横排视图用） ====================
  function getTasksForCell(timeRange: TimeRange, taskType: TaskType): TaskCard[] {
    const today = getTodayDateString()

    // 逾期栏只显示截止日期类型的任务
    if (timeRange === 'overdue' && taskType !== 'dueDate') {
      return []
    }

    return activeTasks.value.filter((task) => {
      let matchesType = false
      let dateToCheck: string | null = null

      switch (taskType) {
        case 'dueDate':
          if (task.due_date?.date) {
            matchesType = true
            dateToCheck = task.due_date.date
          }
          break

        case 'recurrence':
          if (task.recurrence_id && task.recurrence_original_date && !task.due_date) {
            matchesType = true
            dateToCheck = task.recurrence_original_date
          }
          break

        case 'scheduled':
          if (task.schedules && task.schedules.length > 0 && !task.due_date && !task.recurrence_id) {
            matchesType = true
            const sortedSchedules = task.schedules.map((s) => s.scheduled_day).sort()
            const futureSchedule = sortedSchedules.find((date) => date >= today)
            if (futureSchedule) {
              dateToCheck = futureSchedule
            } else {
              dateToCheck = sortedSchedules[sortedSchedules.length - 1] ?? null
            }
          }
          break
      }

      if (!matchesType || !dateToCheck) return false
      return isDateInRange(dateToCheck, timeRange)
    })
  }

  // ==================== 统计函数 ====================
  function getColumnTotalCount(timeRange: TimeRange): number {
    let count = 0
    for (const taskType of ['dueDate', 'recurrence', 'scheduled'] as TaskType[]) {
      count += getTasksForCell(timeRange, taskType).length
    }
    return count
  }

  const totalTaskCount = computed(() => {
    let count = 0
    for (const timeRange of TIME_RANGES) {
      count += getColumnTotalCount(timeRange)
    }
    return count
  })

  // ==================== 横排视图 columns 数据 ====================
  const columnsData = computed(() => {
    return TIME_RANGES.map((key) => ({
      key,
      title: TIME_RANGE_LABELS[key],
      dueDate: getTasksForCell(key, 'dueDate'),
      recurrence: getTasksForCell(key, 'recurrence'),
      scheduled: getTasksForCell(key, 'scheduled'),
    }))
  })

  // ==================== 竖排视图 sections 数据 ====================
  const sectionsData = computed(() => {
    return TIME_RANGES.map((key) => {
      const tasks = getTasksForRange(key)
      return {
        key,
        title: TIME_RANGE_LABELS[key],
        tasks,
        totalCount: tasks.length,
      }
    })
  })

  return {
    // 状态
    completingTaskIds,
    activeTasks,
    timeRangeBoundaries,
    totalTaskCount,

    // 数据
    columnsData,
    sectionsData,

    // 方法
    initialize,
    handleTaskCompleting,
    isDateInRange,
    getTaskDate,
    getTasksForRange,
    getTasksForCell,
    getColumnTotalCount,

    // 常量
    TIME_RANGES,
    TIME_RANGE_LABELS,
  }
}
