<template>
  <div class="upcoming-panel">
    <UpcomingSixColumnLayout :columns="columnsData" @completing="handleTaskCompleting" />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { useTaskStore } from '@/stores/task'
import { useRecurrenceStore } from '@/stores/recurrence'
import UpcomingSixColumnLayout from '@/components/assembles/upcoming/UpcomingSixColumnLayout.vue'
import { getTodayDateString } from '@/infra/utils/dateUtils'
import type { TaskCard } from '@/types/dtos'
import { logger, LogTags } from '@/infra/logging/logger'

const taskStore = useTaskStore()
const recurrenceStore = useRecurrenceStore()

// 乐观更新：正在完成的任务ID集合
const completingTaskIds = ref<Set<string>>(new Set())

// 处理任务开始完成
function handleTaskCompleting(taskId: string) {
  completingTaskIds.value.add(taskId)
  setTimeout(() => {
    completingTaskIds.value.delete(taskId)
  }, 300)
}

// 时间范围定义
const timeRanges = ['overdue', 'today', 'thisWeek', 'nextWeek', 'thisMonth', 'later'] as const
type TimeRange = (typeof timeRanges)[number]
type TaskType = 'dueDate' | 'recurrence' | 'scheduled'

// 计算时间范围的边界
const timeRangeBoundaries = computed(() => {
  const today = getTodayDateString()
  const todayDate = new Date(today + 'T00:00:00')

  const endOfWeek = new Date(todayDate)
  const currentDayOfWeek = todayDate.getDay()
  const daysUntilSunday = currentDayOfWeek === 0 ? 7 : 7 - currentDayOfWeek
  endOfWeek.setDate(endOfWeek.getDate() + daysUntilSunday)
  const endOfWeekStr = endOfWeek.toLocaleDateString('en-CA')

  const endOfNextWeek = new Date(endOfWeek)
  endOfNextWeek.setDate(endOfNextWeek.getDate() + 7)
  const endOfNextWeekStr = endOfNextWeek.toLocaleDateString('en-CA')

  const endOfMonth = new Date(todayDate.getFullYear(), todayDate.getMonth() + 1, 0)
  const endOfMonthStr = endOfMonth.toLocaleDateString('en-CA')

  return {
    today,
    endOfWeek: endOfWeekStr,
    endOfNextWeek: endOfNextWeekStr,
    endOfMonth: endOfMonthStr,
  }
})

// 获取所有活跃任务
const activeTasks = computed(() => {
  return taskStore.allTasks.filter((task) => {
    if (completingTaskIds.value.has(task.id)) {
      return true
    }
    return !task.is_completed && !task.is_deleted && !task.is_archived
  })
})

// 判断日期字符串是否在指定时间范围内
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

// 获取指定单元格的任务列表
function getTasksForCell(timeRange: TimeRange, taskType: TaskType): TaskCard[] {
  const today = getTodayDateString()

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
          dateToCheck = futureSchedule ?? sortedSchedules[sortedSchedules.length - 1] ?? null
        }
        break
    }

    if (!matchesType || !dateToCheck) {
      return false
    }

    return isDateInRange(dateToCheck, timeRange)
  })
}

// 统计每栏的总任务数
function getColumnTotalCount(timeRange: TimeRange): number {
  let count = 0
  for (const taskType of ['dueDate', 'recurrence', 'scheduled'] as TaskType[]) {
    count += getTasksForCell(timeRange, taskType).length
  }
  return count
}

// 统计所有任务总数
const totalTaskCount = computed(() => {
  let count = 0
  for (const timeRange of timeRanges) {
    count += getColumnTotalCount(timeRange)
  }
  return count
})

// 准备6栏数据
const columnsData = computed(() => {
  return [
    {
      key: 'overdue',
      title: '逾期',
      dueDate: getTasksForCell('overdue', 'dueDate'),
      recurrence: getTasksForCell('overdue', 'recurrence'),
      scheduled: getTasksForCell('overdue', 'scheduled'),
    },
    {
      key: 'today',
      title: '今日',
      dueDate: getTasksForCell('today', 'dueDate'),
      recurrence: getTasksForCell('today', 'recurrence'),
      scheduled: getTasksForCell('today', 'scheduled'),
    },
    {
      key: 'thisWeek',
      title: '本周',
      dueDate: getTasksForCell('thisWeek', 'dueDate'),
      recurrence: getTasksForCell('thisWeek', 'recurrence'),
      scheduled: getTasksForCell('thisWeek', 'scheduled'),
    },
    {
      key: 'nextWeek',
      title: '下周',
      dueDate: getTasksForCell('nextWeek', 'dueDate'),
      recurrence: getTasksForCell('nextWeek', 'recurrence'),
      scheduled: getTasksForCell('nextWeek', 'scheduled'),
    },
    {
      key: 'thisMonth',
      title: '本月',
      dueDate: getTasksForCell('thisMonth', 'dueDate'),
      recurrence: getTasksForCell('thisMonth', 'recurrence'),
      scheduled: getTasksForCell('thisMonth', 'scheduled'),
    },
    {
      key: 'later',
      title: '更远',
      dueDate: getTasksForCell('later', 'dueDate'),
      recurrence: getTasksForCell('later', 'recurrence'),
      scheduled: getTasksForCell('later', 'scheduled'),
    },
  ]
})

// 初始化时加载数据
onMounted(async () => {
  logger.info(LogTags.COMPONENT_KANBAN_COLUMN, 'UpcomingPanel: Initializing...')
  await Promise.all([
    taskStore.fetchAllIncompleteTasks_DMA(),
    recurrenceStore.fetchAllRecurrences(),
  ])
})
</script>

<style scoped>
.upcoming-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: var(--color-background-content);
  overflow: hidden;
}
</style>
