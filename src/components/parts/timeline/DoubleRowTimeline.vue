<script setup lang="ts">
import { ref, watch, onMounted, nextTick, computed } from 'vue'
import TimelineDayCell from './TimelineDayCell.vue'
import { useTaskStore } from '@/stores/task'
import { useTimeBlockStore } from '@/stores/timeblock'
import type { TaskCard, TimeBlockView } from '@/types/dtos'
import { getTodayDateString, extractDateFromUtc } from '@/infra/utils/dateUtils'
import type { MonthViewFilters } from '@/composables/calendar/useCalendarEvents'

type LayoutMode = 'auto' | 'single' | 'double'

interface Props {
  currentMonth?: string // YYYY-MM，默认当前月
  monthViewFilters?: MonthViewFilters
  layoutMode?: LayoutMode // 布局模式：auto=自适应, single=单栏, double=双栏
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
  layoutMode: 'auto',
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

const dayCells = ref<DayData[]>([])
const timelineContainerRef = ref<HTMLElement | null>(null)
const hasAutoScrolledToToday = ref(false)

// 计算实际栏数（用于滚动逻辑）
const effectiveColumns = computed(() => {
  if (props.layoutMode === 'single') return 1
  if (props.layoutMode === 'double') return 2
  // auto模式下，通过container query确定
  return 'auto' as const
})

// CSS类名计算
const gridClass = computed(() => {
  if (props.layoutMode === 'single') return 'timeline-grid-single'
  if (props.layoutMode === 'double') return 'timeline-grid-double'
  return 'timeline-grid-auto' // 自适应
})

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

  const days: DayData[] = []

  for (let day = 1; day <= daysInMonth; day++) {
    const dateStr = `${year}-${String(month).padStart(2, '0')}-${String(day).padStart(2, '0')}`
    days.push(buildDayData(dateStr, day, today))
  }

  dayCells.value = days
  scrollTodayIntoView()
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
      // ✅ due_date.date 现在是 YYYY-MM-DD 格式的字符串，直接使用
      const dueDateDay = task.due_date.date
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

      // ⚠️ 使用 extractDateFromUtc() 从 UTC 时间提取本地日期，符合 TIME_CONVENTION.md
      const startDate = extractDateFromUtc(tb.start_time)
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

async function scrollTodayIntoView(force = false) {
  if (!force && hasAutoScrolledToToday.value) return

  await nextTick()
  const container = timelineContainerRef.value
  if (!container) return

  const todayDay = dayCells.value.find((day) => day.isToday)
  if (!todayDay) return

  const todayIndex = dayCells.value.findIndex((day) => day.isToday)
  if (todayIndex === -1) return

  // 根据不同的布局模式计算滚动位置
  const cols = effectiveColumns.value

  if (cols === 'auto') {
    // 自适应模式：尝试通过DOM检测实际列数
    const target = container.querySelector<HTMLElement>(
      `.timeline-day-cell[data-date="${todayDay.date}"]`
    )
    if (!target) return

    const containerRect = container.getBoundingClientRect()
    const targetRect = target.getBoundingClientRect()
    const nextTop = container.scrollTop + (targetRect.top - containerRect.top)
    container.scrollTo({ top: Math.max(nextTop - 20, 0), behavior: 'auto' })
  } else if (cols === 1) {
    // 单栏模式：直接滚动到目标元素
    const target = container.querySelector<HTMLElement>(
      `.timeline-day-cell[data-date="${todayDay.date}"]`
    )
    if (!target) return

    const containerRect = container.getBoundingClientRect()
    const targetRect = target.getBoundingClientRect()
    const nextTop = container.scrollTop + (targetRect.top - containerRect.top)
    container.scrollTo({ top: Math.max(nextTop - 20, 0), behavior: 'auto' })
  } else {
    // 双栏模式：计算今天所在的行，滚动到该行
    const rowIndex = Math.floor(todayIndex / 2)
    const allCells = container.querySelectorAll<HTMLElement>('.timeline-day-cell')
    const targetCell = allCells[rowIndex * 2] // 该行的第一个cell

    if (!targetCell) return

    const containerRect = container.getBoundingClientRect()
    const targetRect = targetCell.getBoundingClientRect()
    const nextTop = container.scrollTop + (targetRect.top - containerRect.top)
    container.scrollTo({ top: Math.max(nextTop - 20, 0), behavior: 'auto' })
  }

  hasAutoScrolledToToday.value = true
}
</script>

<template>
  <div ref="timelineContainerRef" class="double-row-timeline">
    <div :class="['timeline-grid', gridClass]">
      <TimelineDayCell
        v-for="day in dayCells"
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
</template>

<style scoped>
.double-row-timeline {
  width: 100%;
  height: 100%;
  overflow: auto;
  padding: 1rem;
  background: var(--color-background-tertiary);
  container-type: inline-size;
}

/* 自适应模式 - 根据container宽度自动切换 */
.timeline-grid-auto {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 1rem;
  height: 100%;
}

@container (width <= 50rem) {
  .timeline-grid-auto {
    grid-template-columns: 1fr;
  }
}

/* 单栏模式 - 强制单栏 */
.timeline-grid-single {
  display: grid;
  grid-template-columns: 1fr;
  gap: 1rem;
  height: 100%;
}

/* 双栏模式 - 强制双栏 */
.timeline-grid-double {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 1rem;
  height: 100%;
}
</style>
