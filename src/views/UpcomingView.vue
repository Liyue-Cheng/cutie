<template>
  <div class="upcoming-view">
    <!-- 顶部标题栏 -->
    <div class="upcoming-header">
      <div class="header-left">
        <h2 class="header-title">Upcoming</h2>
        <span class="task-count">{{ totalTaskCount }} 个任务</span>
      </div>
      <div class="header-right">
        <span class="header-subtitle">按时间和类型查看即将到来的任务</span>
      </div>
    </div>

    <!-- 6 栏布局容器 -->
    <div
      ref="scrollContainer"
      class="columns-container"
      @mousedown="handleMouseDown"
      @mousemove="handleMouseMove"
      @mouseup="handleMouseUp"
      @mouseleave="handleMouseLeave"
    >
      <!-- 逾期栏 -->
      <div class="time-column">
        <div class="column-header">
          <h3 class="column-title">逾期</h3>
          <span class="column-count">{{ getColumnTotalCount('overdue') }}</span>
        </div>
        <div class="column-content">
          <!-- 截止日期任务组 -->
          <div v-if="getTasksForCell('overdue', 'dueDate').length > 0" class="task-group">
            <div class="group-header">
              <span>截止日期</span>
              <span class="group-count">{{ getTasksForCell('overdue', 'dueDate').length }}</span>
            </div>
            <TransitionGroup name="task-list" tag="div" class="group-tasks">
              <TaskStrip
                v-for="task in getTasksForCell('overdue', 'dueDate')"
                :key="task.id"
                :task="task"
                :view-key="`upcoming::overdue::dueDate`"
                @completing="handleTaskCompleting"
              />
            </TransitionGroup>
          </div>

          <!-- 循环任务组 -->
          <div v-if="getTasksForCell('overdue', 'recurrence').length > 0" class="task-group">
            <div class="group-header">
              <span>循环任务</span>
              <span class="group-count">{{ getTasksForCell('overdue', 'recurrence').length }}</span>
            </div>
            <TransitionGroup name="task-list" tag="div" class="group-tasks">
              <TaskStrip
                v-for="task in getTasksForCell('overdue', 'recurrence')"
                :key="task.id"
                :task="task"
                :view-key="`upcoming::overdue::recurrence`"
                @completing="handleTaskCompleting"
              />
            </TransitionGroup>
          </div>

          <!-- 排期任务组 -->
          <div v-if="getTasksForCell('overdue', 'scheduled').length > 0" class="task-group">
            <div class="group-header">
              <span>排期任务</span>
              <span class="group-count">{{ getTasksForCell('overdue', 'scheduled').length }}</span>
            </div>
            <TransitionGroup name="task-list" tag="div" class="group-tasks">
              <TaskStrip
                v-for="task in getTasksForCell('overdue', 'scheduled')"
                :key="task.id"
                :task="task"
                :view-key="`upcoming::overdue::scheduled`"
                @completing="handleTaskCompleting"
              />
            </TransitionGroup>
          </div>

          <!-- 空状态 -->
          <div v-if="getColumnTotalCount('overdue') === 0" class="empty-state">
            <p>无逾期任务</p>
          </div>
        </div>
      </div>

      <!-- 今日栏 -->
      <div class="time-column">
        <div class="column-header">
          <h3 class="column-title">今日</h3>
          <span class="column-count">{{ getColumnTotalCount('today') }}</span>
        </div>
        <div class="column-content">
          <div v-if="getTasksForCell('today', 'dueDate').length > 0" class="task-group">
            <div class="group-header">
              <span>截止日期</span>
              <span class="group-count">{{ getTasksForCell('today', 'dueDate').length }}</span>
            </div>
            <TransitionGroup name="task-list" tag="div" class="group-tasks">
              <TaskStrip
                v-for="task in getTasksForCell('today', 'dueDate')"
                :key="task.id"
                :task="task"
                :view-key="`upcoming::today::dueDate`"
                @completing="handleTaskCompleting"
              />
            </TransitionGroup>
          </div>

          <div v-if="getTasksForCell('today', 'recurrence').length > 0" class="task-group">
            <div class="group-header">
              <span>循环任务</span>
              <span class="group-count">{{ getTasksForCell('today', 'recurrence').length }}</span>
            </div>
            <TransitionGroup name="task-list" tag="div" class="group-tasks">
              <TaskStrip
                v-for="task in getTasksForCell('today', 'recurrence')"
                :key="task.id"
                :task="task"
                :view-key="`upcoming::today::recurrence`"
                @completing="handleTaskCompleting"
              />
            </TransitionGroup>
          </div>

          <div v-if="getTasksForCell('today', 'scheduled').length > 0" class="task-group">
            <div class="group-header">
              <span>排期任务</span>
              <span class="group-count">{{ getTasksForCell('today', 'scheduled').length }}</span>
            </div>
            <TransitionGroup name="task-list" tag="div" class="group-tasks">
              <TaskStrip
                v-for="task in getTasksForCell('today', 'scheduled')"
                :key="task.id"
                :task="task"
                :view-key="`upcoming::today::scheduled`"
                @completing="handleTaskCompleting"
              />
            </TransitionGroup>
          </div>

          <div v-if="getColumnTotalCount('today') === 0" class="empty-state">
            <p>今日无任务</p>
          </div>
        </div>
      </div>

      <!-- 本周栏 -->
      <div class="time-column">
        <div class="column-header">
          <h3 class="column-title">本周</h3>
          <span class="column-count">{{ getColumnTotalCount('thisWeek') }}</span>
        </div>
        <div class="column-content">
          <div v-if="getTasksForCell('thisWeek', 'dueDate').length > 0" class="task-group">
            <div class="group-header">
              <span>截止日期</span>
              <span class="group-count">{{ getTasksForCell('thisWeek', 'dueDate').length }}</span>
            </div>
            <TransitionGroup name="task-list" tag="div" class="group-tasks">
              <TaskStrip
                v-for="task in getTasksForCell('thisWeek', 'dueDate')"
                :key="task.id"
                :task="task"
                :view-key="`upcoming::thisWeek::dueDate`"
                @completing="handleTaskCompleting"
              />
            </TransitionGroup>
          </div>

          <div v-if="getTasksForCell('thisWeek', 'recurrence').length > 0" class="task-group">
            <div class="group-header">
              <span>循环任务</span>
              <span class="group-count">{{
                getTasksForCell('thisWeek', 'recurrence').length
              }}</span>
            </div>
            <TransitionGroup name="task-list" tag="div" class="group-tasks">
              <TaskStrip
                v-for="task in getTasksForCell('thisWeek', 'recurrence')"
                :key="task.id"
                :task="task"
                :view-key="`upcoming::thisWeek::recurrence`"
                @completing="handleTaskCompleting"
              />
            </TransitionGroup>
          </div>

          <div v-if="getTasksForCell('thisWeek', 'scheduled').length > 0" class="task-group">
            <div class="group-header">
              <span>排期任务</span>
              <span class="group-count">{{ getTasksForCell('thisWeek', 'scheduled').length }}</span>
            </div>
            <TransitionGroup name="task-list" tag="div" class="group-tasks">
              <TaskStrip
                v-for="task in getTasksForCell('thisWeek', 'scheduled')"
                :key="task.id"
                :task="task"
                :view-key="`upcoming::thisWeek::scheduled`"
                @completing="handleTaskCompleting"
              />
            </TransitionGroup>
          </div>

          <div v-if="getColumnTotalCount('thisWeek') === 0" class="empty-state">
            <p>本周无任务</p>
          </div>
        </div>
      </div>

      <!-- 下周栏 -->
      <div class="time-column">
        <div class="column-header">
          <h3 class="column-title">下周</h3>
          <span class="column-count">{{ getColumnTotalCount('nextWeek') }}</span>
        </div>
        <div class="column-content">
          <div v-if="getTasksForCell('nextWeek', 'dueDate').length > 0" class="task-group">
            <div class="group-header">
              <span>截止日期</span>
              <span class="group-count">{{ getTasksForCell('nextWeek', 'dueDate').length }}</span>
            </div>
            <TransitionGroup name="task-list" tag="div" class="group-tasks">
              <TaskStrip
                v-for="task in getTasksForCell('nextWeek', 'dueDate')"
                :key="task.id"
                :task="task"
                :view-key="`upcoming::nextWeek::dueDate`"
                @completing="handleTaskCompleting"
              />
            </TransitionGroup>
          </div>

          <div v-if="getTasksForCell('nextWeek', 'recurrence').length > 0" class="task-group">
            <div class="group-header">
              <span>循环任务</span>
              <span class="group-count">{{
                getTasksForCell('nextWeek', 'recurrence').length
              }}</span>
            </div>
            <TransitionGroup name="task-list" tag="div" class="group-tasks">
              <TaskStrip
                v-for="task in getTasksForCell('nextWeek', 'recurrence')"
                :key="task.id"
                :task="task"
                :view-key="`upcoming::nextWeek::recurrence`"
                @completing="handleTaskCompleting"
              />
            </TransitionGroup>
          </div>

          <div v-if="getTasksForCell('nextWeek', 'scheduled').length > 0" class="task-group">
            <div class="group-header">
              <span>排期任务</span>
              <span class="group-count">{{ getTasksForCell('nextWeek', 'scheduled').length }}</span>
            </div>
            <TransitionGroup name="task-list" tag="div" class="group-tasks">
              <TaskStrip
                v-for="task in getTasksForCell('nextWeek', 'scheduled')"
                :key="task.id"
                :task="task"
                :view-key="`upcoming::nextWeek::scheduled`"
                @completing="handleTaskCompleting"
              />
            </TransitionGroup>
          </div>

          <div v-if="getColumnTotalCount('nextWeek') === 0" class="empty-state">
            <p>下周无任务</p>
          </div>
        </div>
      </div>

      <!-- 本月栏 -->
      <div class="time-column">
        <div class="column-header">
          <h3 class="column-title">本月</h3>
          <span class="column-count">{{ getColumnTotalCount('thisMonth') }}</span>
        </div>
        <div class="column-content">
          <div v-if="getTasksForCell('thisMonth', 'dueDate').length > 0" class="task-group">
            <div class="group-header">
              <span>截止日期</span>
              <span class="group-count">{{ getTasksForCell('thisMonth', 'dueDate').length }}</span>
            </div>
            <TransitionGroup name="task-list" tag="div" class="group-tasks">
              <TaskStrip
                v-for="task in getTasksForCell('thisMonth', 'dueDate')"
                :key="task.id"
                :task="task"
                :view-key="`upcoming::thisMonth::dueDate`"
                @completing="handleTaskCompleting"
              />
            </TransitionGroup>
          </div>

          <div v-if="getTasksForCell('thisMonth', 'recurrence').length > 0" class="task-group">
            <div class="group-header">
              <span>循环任务</span>
              <span class="group-count">{{
                getTasksForCell('thisMonth', 'recurrence').length
              }}</span>
            </div>
            <TransitionGroup name="task-list" tag="div" class="group-tasks">
              <TaskStrip
                v-for="task in getTasksForCell('thisMonth', 'recurrence')"
                :key="task.id"
                :task="task"
                :view-key="`upcoming::thisMonth::recurrence`"
                @completing="handleTaskCompleting"
              />
            </TransitionGroup>
          </div>

          <div v-if="getTasksForCell('thisMonth', 'scheduled').length > 0" class="task-group">
            <div class="group-header">
              <span>排期任务</span>
              <span class="group-count">{{
                getTasksForCell('thisMonth', 'scheduled').length
              }}</span>
            </div>
            <TransitionGroup name="task-list" tag="div" class="group-tasks">
              <TaskStrip
                v-for="task in getTasksForCell('thisMonth', 'scheduled')"
                :key="task.id"
                :task="task"
                :view-key="`upcoming::thisMonth::scheduled`"
                @completing="handleTaskCompleting"
              />
            </TransitionGroup>
          </div>

          <div v-if="getColumnTotalCount('thisMonth') === 0" class="empty-state">
            <p>本月无任务</p>
          </div>
        </div>
      </div>

      <!-- 更远栏 -->
      <div class="time-column">
        <div class="column-header">
          <h3 class="column-title">更远</h3>
          <span class="column-count">{{ getColumnTotalCount('later') }}</span>
        </div>
        <div class="column-content">
          <div v-if="getTasksForCell('later', 'dueDate').length > 0" class="task-group">
            <div class="group-header">
              <span>截止日期</span>
              <span class="group-count">{{ getTasksForCell('later', 'dueDate').length }}</span>
            </div>
            <TransitionGroup name="task-list" tag="div" class="group-tasks">
              <TaskStrip
                v-for="task in getTasksForCell('later', 'dueDate')"
                :key="task.id"
                :task="task"
                :view-key="`upcoming::later::dueDate`"
                @completing="handleTaskCompleting"
              />
            </TransitionGroup>
          </div>

          <div v-if="getTasksForCell('later', 'recurrence').length > 0" class="task-group">
            <div class="group-header">
              <span>循环任务</span>
              <span class="group-count">{{ getTasksForCell('later', 'recurrence').length }}</span>
            </div>
            <TransitionGroup name="task-list" tag="div" class="group-tasks">
              <TaskStrip
                v-for="task in getTasksForCell('later', 'recurrence')"
                :key="task.id"
                :task="task"
                :view-key="`upcoming::later::recurrence`"
                @completing="handleTaskCompleting"
              />
            </TransitionGroup>
          </div>

          <div v-if="getTasksForCell('later', 'scheduled').length > 0" class="task-group">
            <div class="group-header">
              <span>排期任务</span>
              <span class="group-count">{{ getTasksForCell('later', 'scheduled').length }}</span>
            </div>
            <TransitionGroup name="task-list" tag="div" class="group-tasks">
              <TaskStrip
                v-for="task in getTasksForCell('later', 'scheduled')"
                :key="task.id"
                :task="task"
                :view-key="`upcoming::later::scheduled`"
                @completing="handleTaskCompleting"
              />
            </TransitionGroup>
          </div>

          <div v-if="getColumnTotalCount('later') === 0" class="empty-state">
            <p>暂无更远的任务</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 任务编辑器模态框 -->
    <TaskEditorModal
      v-if="uiStore.isEditorOpen"
      :task-id="uiStore.editorTaskId"
      :view-key="uiStore.editorViewKey ?? undefined"
      @close="uiStore.closeEditor"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { TransitionGroup } from 'vue'
import { useTaskStore } from '@/stores/task'
import { useUIStore } from '@/stores/ui'
import { useRecurrenceStore } from '@/stores/recurrence'
import TaskStrip from '@/components/assembles/tasks/list/TaskStrip.vue'
import TaskEditorModal from '@/components/assembles/tasks/TaskEditorModal.vue'
import { getTodayDateString } from '@/infra/utils/dateUtils'
import type { TaskCard } from '@/types/dtos'

const taskStore = useTaskStore()
const uiStore = useUIStore()
const recurrenceStore = useRecurrenceStore()

// ==================== 拖动滚动状态 ====================
const scrollContainer = ref<HTMLElement | null>(null)
const isDragging = ref(false)
const dragStartX = ref(0)
const dragStartScrollLeft = ref(0)

// ==================== 乐观更新：正在完成的任务ID集合 ====================
const completingTaskIds = ref<Set<string>>(new Set())

// 处理任务开始完成
function handleTaskCompleting(taskId: string) {
  completingTaskIds.value.add(taskId)

  // 300ms 后允许任务消失
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

  // 本周末（周日）
  const endOfWeek = new Date(todayDate)
  const daysUntilSunday = todayDate.getDay() === 0 ? 0 : 7 - todayDate.getDay()
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

// 获取所有未完成且未删除的任务（包括正在完成中的任务）
const activeTasks = computed(() => {
  return taskStore.allTasks.filter((task) => {
    // 如果任务正在完成中，强制保留
    if (completingTaskIds.value.has(task.id)) {
      return true
    }
    // 否则按原逻辑：未完成且未删除
    return !task.is_completed && !task.is_deleted
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
      // 本周：今天之后到本周末
      return dateStr > today && dateStr <= endOfWeek
    case 'nextWeek':
      // 下周：本周末之后到下周末
      return dateStr > endOfWeek && dateStr <= endOfNextWeek
    case 'thisMonth':
      // 本月：今天之后到本月末（排除本周和下周）
      // 如果下周末已经超过本月末，则本月栏应该为空或只包含下周末后到本月末的部分
      if (endOfNextWeek > endOfMonth) {
        // 下周已经跨月了，本月栏显示本周末之后到本月末之间的任务
        return dateStr > endOfWeek && dateStr <= endOfMonth
      } else {
        // 下周还在本月内，本月栏显示下周末之后到本月末之间的任务
        return dateStr > endOfNextWeek && dateStr <= endOfMonth
      }
    case 'later':
      return dateStr > endOfMonth
    default:
      return false
  }
}

// 判断循环任务的频率是否匹配时间范围
// 规则：短期可以容纳长期
function isRecurrenceFrequencyMatch(task: TaskCard, timeRange: TimeRange): boolean {
  // 逾期栏：显示所有循环任务
  if (timeRange === 'overdue') {
    return true
  }

  // 获取循环规则
  if (!task.recurrence_id) return false
  const recurrence = recurrenceStore.getRecurrenceById(task.recurrence_id)
  if (!recurrence) return false

  // 解析 RRULE 的 FREQ 字段
  const rule = recurrence.rule.toUpperCase()
  const freqMatch = rule.match(/FREQ=(DAILY|WEEKLY|MONTHLY|YEARLY)/)
  if (!freqMatch) return true // 无法解析时默认显示

  const freq = freqMatch[1]

  // 根据时间范围匹配循环频率（短期容纳长期）
  switch (timeRange) {
    case 'today':
      // 今日栏：显示每日、每周、每月循环
      return freq === 'DAILY' || freq === 'WEEKLY' || freq === 'MONTHLY'
    case 'thisWeek':
    case 'nextWeek':
      // 本周/下周栏：显示每周、每月循环
      return freq === 'WEEKLY' || freq === 'MONTHLY'
    case 'thisMonth':
      // 本月栏：只显示每月循环
      return freq === 'MONTHLY'
    case 'later':
      // 更远栏：只显示每年循环
      return freq === 'YEARLY'
    default:
      return true
  }
}

// 获取指定单元格的任务列表
function getTasksForCell(timeRange: TimeRange, taskType: TaskType): TaskCard[] {
  return activeTasks.value.filter((task) => {
    // 首先判断任务类型
    let matchesType = false
    let dateToCheck: string | null = null

    switch (taskType) {
      case 'dueDate':
        // 带截止日期的任务（优先级最高）
        if (task.due_date?.date) {
          matchesType = true
          dateToCheck = task.due_date.date
        }
        break

      case 'recurrence':
        // 循环任务（排除已经有截止日期的）
        if (task.recurrence_id && task.recurrence_original_date && !task.due_date) {
          // 检查循环频率是否匹配时间范围
          if (!isRecurrenceFrequencyMatch(task, timeRange)) {
            return false
          }
          matchesType = true
          dateToCheck = task.recurrence_original_date
        }
        break

      case 'scheduled':
        // 一般排期任务（排除有截止日期或循环任务的）
        if (task.schedules && task.schedules.length > 0 && !task.due_date && !task.recurrence_id) {
          matchesType = true
          // 取最早的排期日期
          const earliestSchedule = task.schedules.map((s) => s.scheduled_day).sort()[0]
          dateToCheck = earliestSchedule ?? null
        }
        break
    }

    // 如果类型不匹配或没有日期，直接返回 false
    if (!matchesType || !dateToCheck) {
      return false
    }

    // 判断日期是否在时间范围内
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

// ==================== 拖动滚动逻辑 ====================
function handleMouseDown(event: MouseEvent) {
  // 只在空白区域拖动（排除任务条和按钮等）
  const target = event.target as HTMLElement

  // 检查是否点击在允许拖动的区域（空白区域）
  const isAllowedArea =
    target.classList.contains('columns-container') ||
    target.classList.contains('column-content') ||
    target.classList.contains('empty-state') ||
    target.classList.contains('time-column')

  if (!isAllowedArea || !scrollContainer.value) return

  event.preventDefault()
  isDragging.value = true
  dragStartX.value = event.pageX
  dragStartScrollLeft.value = scrollContainer.value.scrollLeft

  // 改变光标样式
  if (scrollContainer.value) {
    scrollContainer.value.style.cursor = 'default'
    scrollContainer.value.style.userSelect = 'none'
  }
}

function handleMouseMove(event: MouseEvent) {
  if (!isDragging.value || !scrollContainer.value) return

  event.preventDefault()

  const deltaX = event.pageX - dragStartX.value
  scrollContainer.value.scrollLeft = dragStartScrollLeft.value - deltaX
}

function handleMouseUp() {
  if (!isDragging.value) return

  isDragging.value = false

  // 恢复光标样式
  if (scrollContainer.value) {
    scrollContainer.value.style.cursor = 'default'
    scrollContainer.value.style.userSelect = ''
  }
}

function handleMouseLeave() {
  if (isDragging.value) {
    handleMouseUp()
  }
}
</script>

<style scoped>
.upcoming-view {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: var(--color-background-content);
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
  overflow: hidden;
}

/* 顶部标题栏 */
.upcoming-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.6rem 2rem;
  background-color: var(--color-background-content);
  border-bottom: 1px solid var(--color-border-default);
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 1.2rem;
}

.header-title {
  font-size: 2rem;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.task-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0.4rem 1rem;
  font-size: 1.3rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  background-color: var(--color-background-secondary, #f0f0f0);
  border-radius: 1.2rem;
}

.header-right {
  display: flex;
  align-items: center;
}

.header-subtitle {
  font-size: 1.4rem;
  color: var(--color-text-tertiary);
}

/* 6 栏容器 */
.columns-container {
  flex: 1;
  display: flex;
  gap: 0;
  overflow: auto hidden;
  min-height: 0;

  /* 拖动滚动样式 */
  cursor: default;
  user-select: none;
}

/* 时间栏 */
.time-column {
  flex: 1;
  min-width: 30rem;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--color-border-default);
  background-color: var(--color-background-content);
}

.time-column:last-child {
  border-right: none;
}

/* 栏标题 */
.column-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.2rem 1.6rem;
  background-color: var(--color-background-content);
  border-bottom: 1px solid var(--color-border-default);
  flex-shrink: 0;
}

.column-title {
  font-size: 1.6rem;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.column-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2.4rem;
  height: 2.4rem;
  padding: 0 0.6rem;
  font-size: 1.2rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  background-color: var(--color-background-secondary, #f0f0f0);
  border-radius: 1.2rem;
}

/* 栏内容区 */
.column-content {
  flex: 1;
  overflow: hidden auto;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

/* 任务组 */
.task-group {
  display: flex;
  flex-direction: column;
}

/* 组标题 */
.group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.2rem 1.6rem 0.8rem;
  font-size: 1.3rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  background-color: var(--color-background-content);
  position: sticky;
  top: 0;
  z-index: 10;
  border-bottom: 1px solid var(--color-border-light, rgb(0 0 0 / 5%));
}

.group-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2rem;
  height: 2rem;
  padding: 0 0.5rem;
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--color-text-tertiary);
  background-color: var(--color-background-hover, rgb(0 0 0 / 3%));
  border-radius: 1rem;
}

/* 任务列表 */
.group-tasks {
  display: flex;
  flex-direction: column;
}

/* 空状态 */
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4rem 2rem;
  color: var(--color-text-secondary);
  font-size: 1.4rem;
}

.empty-state p {
  margin: 0;
}

/* 任务列表动画 */
/* 进入和退出的过渡，以及移动动画 */
.task-list-move,
.task-list-enter-active,
.task-list-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

/* 进入动画：从右侧淡入 */
.task-list-enter-from {
  opacity: 0;
  transform: translateX(20px);
}

/* 退出动画：向右淡出 + 缩小 + 高度压缩 */
.task-list-leave-to {
  opacity: 0;
  transform: translateX(30px) scale(0.95);
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
  margin-bottom: 0;
  overflow: hidden;
}

/* 滚动条样式 */
.column-content::-webkit-scrollbar {
  width: 0.6rem;
}

.column-content::-webkit-scrollbar-track {
  background: transparent;
}

.column-content::-webkit-scrollbar-thumb {
  background-color: rgb(0 0 0 / 10%);
  border-radius: 0.3rem;
}

.column-content::-webkit-scrollbar-thumb:hover {
  background-color: rgb(0 0 0 / 20%);
}

.columns-container::-webkit-scrollbar {
  height: 0.6rem;
}

.columns-container::-webkit-scrollbar-track {
  background: transparent;
}

.columns-container::-webkit-scrollbar-thumb {
  background-color: rgb(0 0 0 / 10%);
  border-radius: 0.3rem;
}

.columns-container::-webkit-scrollbar-thumb:hover {
  background-color: rgb(0 0 0 / 20%);
}
</style>
