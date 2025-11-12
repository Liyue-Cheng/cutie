<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import TimelineDayCell from './TimelineDayCell.vue'
import { useTaskStore } from '@/stores/task'
import { useTimeBlockStore } from '@/stores/timeblock'
import type { TaskCard, TimeBlockView } from '@/types/dtos'
import { getTodayDateString } from '@/infra/utils/dateUtils'
import type { MonthViewFilters } from '@/composables/calendar/useCalendarEvents'

interface Props {
  currentMonth?: string // YYYY-MM，默认当前月
  monthViewFilters?: MonthViewFilters
}

const props = withDefaults(defineProps<Props>(), {
  currentMonth: () => {
    const today = new Date()
    const year = today.getFullYear()
    const month = String(today.getMonth() + 1).padStart(2, '0')
    return `${year}-${month}`
  },
  monthViewFilters: () => ({
    showRecurringTasks: true,
    showScheduledTasks: true,
    showDueDates: true,
    showAllDayEvents: true,
  }),
})

const taskStore = useTaskStore()
const timeBlockStore = useTimeBlockStore()

interface DayData {
  date: string
  dayNumber: number
  tasks: TaskCard[]
  dueDates: TaskCard[]
  allDayEvents: TimeBlockView[]
  isToday: boolean
  isWeekend: boolean
}

const leftColumn = ref<DayData[]>([])
const rightColumn = ref<DayData[]>([])

function getDaysInMonth(yearMonth: string): number {
  const [yearStr, monthStr] = yearMonth.split('-')
  const year = Number(yearStr)
  const month = Number(monthStr)
  return new Date(year, month, 0).getDate()
}

function isWeekend(dateStr: string): boolean {
  const date = new Date(dateStr)
  const day = date.getDay()
  return day === 0 || day === 6
}

function buildTimelineData() {
  const [yearStr, monthStr] = props.currentMonth.split('-')
  const year = Number(yearStr)
  const month = Number(monthStr)
  const daysInMonth = getDaysInMonth(props.currentMonth)
  const today = getTodayDateString()

  const leftDays: DayData[] = []
  const rightDays: DayData[] = []

  // 构建左列（1-15日）
  for (let day = 1; day <= 15; day++) {
    const dateStr = `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`
    leftDays.push(buildDayData(dateStr, day, today))
  }

  // 构建右列（16-月末）
  for (let day = 16; day <= daysInMonth; day++) {
    const dateStr = `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`
    rightDays.push(buildDayData(dateStr, day, today))
  }

  leftColumn.value = leftDays
  rightColumn.value = rightDays
}

function buildDayData(dateStr: string, dayNumber: number, today: string): DayData {
  const filters = props.monthViewFilters

  // 收集任务
  const tasks: TaskCard[] = []
  const dueDates: TaskCard[] = []

  // 用于去重的 Set
  const taskIdsWithSchedule = new Set<string>()
  const taskIdsWithDueDate = new Set<string>()

  taskStore.allTasks.forEach((task) => {
    if (task.is_deleted) return

    // 已排期任务
    const hasScheduleOnDate = task.schedules?.some((s) => s.scheduled_day === dateStr)
    if (hasScheduleOnDate) {
      const isRecurring = !!task.recurrence_id
      if (
        (isRecurring && filters.showRecurringTasks) ||
        (!isRecurring && filters.showScheduledTasks)
      ) {
        tasks.push(task)
        taskIdsWithSchedule.add(task.id)
      }
    }

    // 截止日期
    if (filters.showDueDates && task.due_date) {
      const dueDateDay = task.due_date.date?.slice(0, 10)
      if (dueDateDay === dateStr && !task.is_completed && !task.is_archived) {
        // 如果当天已有排期，不重复显示截止日期
        if (!taskIdsWithSchedule.has(task.id)) {
          dueDates.push(task)
          taskIdsWithDueDate.add(task.id)
        }
      }
    }
  })

  // 收集全天事件
  const allDayEvents: TimeBlockView[] = []
  if (filters.showAllDayEvents) {
    timeBlockStore.allTimeBlocks.forEach((tb) => {
      if (!tb.is_all_day) return

      const startDate = new Date(tb.start_time).toISOString().split('T')[0]
      if (startDate === dateStr) {
        allDayEvents.push(tb)
      }
    })
  }

  return {
    date: dateStr,
    dayNumber,
    tasks,
    dueDates,
    allDayEvents,
    isToday: dateStr === today,
    isWeekend: isWeekend(dateStr),
  }
}

// 监听月份变化
watch(() => props.currentMonth, buildTimelineData, { immediate: true })
watch(() => props.monthViewFilters, buildTimelineData, { deep: true })

// 监听 store 变化
watch(() => [taskStore.allTasks, timeBlockStore.allTimeBlocks], buildTimelineData, { deep: false })

onMounted(() => {
  buildTimelineData()
})
</script>

<template>
  <div class="double-row-timeline">
    <div class="timeline-container">
      <div class="timeline-column left-column">
        <TimelineDayCell
          v-for="day in leftColumn"
          :key="day.date"
          :date="day.date"
          :day-number="day.dayNumber"
          :tasks="day.tasks"
          :due-dates="day.dueDates"
          :all-day-events="day.allDayEvents"
          :is-today="day.isToday"
          :is-weekend="day.isWeekend"
        />
      </div>

      <div class="timeline-column right-column">
        <TimelineDayCell
          v-for="day in rightColumn"
          :key="day.date"
          :date="day.date"
          :day-number="day.dayNumber"
          :tasks="day.tasks"
          :due-dates="day.dueDates"
          :all-day-events="day.allDayEvents"
          :is-today="day.isToday"
          :is-weekend="day.isWeekend"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.double-row-timeline {
  width: 100%;
  height: 100%;
  overflow: auto;
  padding: 1rem;
  background: var(--color-background-tertiary);
}

.timeline-container {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
  height: 100%;
}

.timeline-column {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}
</style>
