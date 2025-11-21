<!--
  UpcomingView.vue - 即将到来的任务视图

  功能：
  - 按时间范围（逾期、今日、本周、下周、本月、更远）展示任务
  - 任务分类（截止日期、循环任务、一般排期）
  - 支持任务拖拽、完成、编辑等操作

  Bug 修复记录 (2025-01-21):
  1. Bug 1 (周日边界): 修复周日当天 "本周" 栏为空的问题
  2. Bug 2 (循环频率): 移除基于频率的过滤，改为基于实际实例日期判断
  3. Bug 3 (归档过滤): 添加 is_archived 检查，防止已归档任务显示
  4. Bug 4 (thisMonth 逻辑): 简化跨月边界逻辑，避免与本周/下周重叠
  5. Bug 6 (scheduled 日期): 取未来最近的 schedule 而不是最早的

  数据加载：
  - onMounted 时加载所有未完成任务 (fetchAllIncompleteTasks_DMA)
  - 后端会自动实例化循环任务的未来实例
-->
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
import { computed, ref, onMounted } from 'vue'
import { TransitionGroup } from 'vue'
import { useTaskStore } from '@/stores/task'
import { useUIStore } from '@/stores/ui'
import { useRecurrenceStore } from '@/stores/recurrence'
import TaskStrip from '@/components/assembles/tasks/list/TaskStrip.vue'
import TaskEditorModal from '@/components/assembles/tasks/TaskEditorModal.vue'
import { getTodayDateString } from '@/infra/utils/dateUtils'
import type { TaskCard } from '@/types/dtos'
import { logger, LogTags } from '@/infra/logging/logger'

const taskStore = useTaskStore()
const uiStore = useUIStore()
const recurrenceStore = useRecurrenceStore()

// ==================== 生命周期 ====================
onMounted(async () => {
  logger.info(LogTags.VIEW_UPCOMING, 'Initializing upcoming view...')
  await Promise.all([
    taskStore.fetchAllIncompleteTasks_DMA(),
    recurrenceStore.fetchAllRecurrences(),
  ])
  logger.info(LogTags.VIEW_UPCOMING, 'Loaded tasks and recurrences', {
    taskCount: taskStore.incompleteTasks.length,
    recurrenceCount: recurrenceStore.allRecurrences.length,
  })
})

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

// ==================== 时间范围定义 ====================
const timeRanges = ['overdue', 'today', 'thisWeek', 'nextWeek', 'thisMonth', 'later'] as const
type TimeRange = (typeof timeRanges)[number]
type TaskType = 'dueDate' | 'recurrence' | 'scheduled'

/**
 * 计算时间范围的边界日期
 *
 * 边界定义：
 * - today: 今天的日期（YYYY-MM-DD）
 * - endOfWeek: 本周末（周日）的日期
 * - endOfNextWeek: 下周末（下个周日）的日期
 * - endOfMonth: 本月最后一天的日期
 *
 * Bug 修复：
 * - 修复周日边界问题：如果今天是周日，endOfWeek 指向下周日而不是今天
 */
const timeRangeBoundaries = computed(() => {
  const today = getTodayDateString()
  const todayDate = new Date(today + 'T00:00:00')

  // 本周末（周日）
  // 修复 Bug 1: 如果今天是周日（getDay() === 0），则本周末应该是今天，但为了避免
  // "thisWeek" 栏为空（因为 thisWeek 要求 dateStr > today），我们让周日也指向下周日
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

/**
 * 获取所有活跃任务（未完成、未删除、未归档）
 *
 * 过滤规则：
 * 1. 正在完成中的任务（乐观更新）：强制保留 300ms 用于动画
 * 2. 常规任务：必须满足 !is_completed && !is_deleted && !is_archived
 *
 * Bug 修复：
 * - 修复 Bug 3: 添加 !task.is_archived 检查，防止已归档任务出现在视图中
 */
const activeTasks = computed(() => {
  return taskStore.allTasks.filter((task) => {
    // 如果任务正在完成中，强制保留（用于动画效果）
    if (completingTaskIds.value.has(task.id)) {
      return true
    }
    // 常规过滤：未完成 && 未删除 && 未归档
    return !task.is_completed && !task.is_deleted && !task.is_archived
  })
})

/**
 * 判断日期字符串是否在指定时间范围内
 *
 * 时间范围定义：
 * - overdue: < 今天
 * - today: == 今天
 * - thisWeek: (今天, 本周末]
 * - nextWeek: (本周末, 下周末]
 * - thisMonth: (下周末, 本月末]（如果下周跨月，则为 (本周末, 本月末]）
 * - later: > 本月末
 *
 * Bug 修复：
 * - 修复 Bug 4: 简化 thisMonth 逻辑，当下周跨月时使用本周末作为起点
 *
 * @param dateStr 日期字符串（YYYY-MM-DD 格式）
 * @param timeRange 时间范围枚举
 * @returns 日期是否在该范围内
 */
function isDateInRange(dateStr: string, timeRange: TimeRange): boolean {
  const { today, endOfWeek, endOfNextWeek, endOfMonth } = timeRangeBoundaries.value

  switch (timeRange) {
    case 'overdue':
      // 逾期：所有小于今天的日期
      return dateStr < today

    case 'today':
      // 今日：完全等于今天
      return dateStr === today

    case 'thisWeek':
      // 本周：今天之后到本周末（不包括今天，包括周末）
      return dateStr > today && dateStr <= endOfWeek

    case 'nextWeek':
      // 下周：本周末之后到下周末（不包括本周末，包括下周末）
      return dateStr > endOfWeek && dateStr <= endOfNextWeek

    case 'thisMonth':
      // 本月：下周末之后到本月末（排除已被本周/下周覆盖的日期）
      // Bug 修复 (Bug 4): 简化跨月逻辑
      // - 如果下周末已经跨月（endOfNextWeek > endOfMonth），则本月栏为空或仅显示本周末到月末
      // - 否则，显示下周末之后到本月末的日期
      if (endOfNextWeek > endOfMonth) {
        // 下周跨月了，本月栏显示本周末之后到本月末
        return dateStr > endOfWeek && dateStr <= endOfMonth
      } else {
        // 下周还在本月内，本月栏显示下周末之后到本月末
        return dateStr > endOfNextWeek && dateStr <= endOfMonth
      }

    case 'later':
      // 更远：所有大于本月末的日期
      return dateStr > endOfMonth

    default:
      return false
  }
}

/**
 * 判断循环任务是否应该显示在指定时间范围
 *
 * 新策略（Bug 修复 Bug 2）：
 * - 不再基于循环频率类型（DAILY/WEEKLY/MONTHLY/YEARLY）判断
 * - 而是基于 recurrence_original_date（实际实例日期）判断
 * - 这样可以正确处理所有频率的循环任务（包括季度、年度等）
 *
 * 原有问题：
 * - 旧逻辑：年度循环只显示在 "later" 栏，即使生日在本月也不会显示
 * - 旧逻辑：每月循环会出现在 "today" 栏，逻辑混乱
 *
 * @param task 任务对象
 * @param timeRange 时间范围
 * @returns 是否应该显示（已废弃，现在直接用 isDateInRange 判断）
 */
function isRecurrenceFrequencyMatch(task: TaskCard, timeRange: TimeRange): boolean {
  // Bug 修复 (Bug 2): 移除基于频率的过滤逻辑
  // 现在循环任务的显示完全由 recurrence_original_date 和 isDateInRange 决定
  // 保留此函数是为了向后兼容，但始终返回 true
  return true
}

/**
 * 获取指定单元格的任务列表
 *
 * 任务分类优先级（从高到低）：
 * 1. dueDate: 有截止日期（due_date）的任务
 * 2. recurrence: 循环任务实例（有 recurrence_id 和 recurrence_original_date）
 * 3. scheduled: 一般排期任务（有 schedules）
 *
 * Bug 修复：
 * - Bug 6: scheduled 任务改为取"未来最近的"日期，而不是"最早的"日期
 * - Bug 2: 循环任务不再基于频率过滤，直接用实例日期判断
 *
 * @param timeRange 时间范围枚举
 * @param taskType 任务类型枚举
 * @returns 该单元格的任务列表
 */
function getTasksForCell(timeRange: TimeRange, taskType: TaskType): TaskCard[] {
  const today = getTodayDateString()

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
          matchesType = true
          dateToCheck = task.recurrence_original_date
          // Bug 修复 (Bug 2): 移除频率匹配逻辑，直接用日期判断
        }
        break

      case 'scheduled':
        // 一般排期任务（排除有截止日期或循环任务的）
        if (task.schedules && task.schedules.length > 0 && !task.due_date && !task.recurrence_id) {
          matchesType = true

          // Bug 修复 (Bug 6): 取"未来最近的" schedule，而不是"最早的"
          // 原逻辑：取最早的（可能是过去的日期）
          // 新逻辑：取 >= today 的最近日期；如果都是过去的，则取最新的过去日期
          const sortedSchedules = task.schedules.map((s) => s.scheduled_day).sort()

          // 找到第一个 >= today 的日期
          const futureSchedule = sortedSchedules.find((date) => date >= today)

          if (futureSchedule) {
            // 有未来的日期，使用它
            dateToCheck = futureSchedule
          } else {
            // 都是过去的日期，取最新的一个（用于显示在 overdue）
            dateToCheck = sortedSchedules[sortedSchedules.length - 1] ?? null
          }
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
