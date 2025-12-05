<template>
  <div class="home-calendar-panel">
    <TwoRowLayout>
      <template #top>
        <div class="calendar-controls">
          <!-- ========== 日历模式控制栏 ========== -->
          <template v-if="props.isCalendarMode">
            <!-- 左侧：退出按钮 + 年月显示 + 导航 -->
            <div class="calendar-left-controls">
              <button
                class="calendar-mode-btn"
                title="退出日历模式"
                @click="emit('exit-calendar-mode')"
              >
                <CuteIcon name="ChevronsRight" :size="18" />
              </button>

              <!-- 年月显示 -->
              <div class="calendar-year-month">
                {{ calendarYearMonth }}
              </div>

              <!-- 左右导航按钮 -->
              <button class="control-btn nav-btn" title="上一周/月" @click="navigatePrevious">
                <CuteIcon name="ChevronLeft" :size="16" />
              </button>
              <button class="control-btn nav-btn" title="下一周/月" @click="navigateNext">
                <CuteIcon name="ChevronRight" :size="16" />
              </button>

              <!-- 本周按钮 + 日历选择器（仅周视图） -->
              <div v-if="calendarModeViewType === 'week'" class="combined-btn-wrapper">
                <button class="combined-btn-left" @click="goToThisWeek" title="回到本周">
                  <span>本周</span>
                </button>
                <button class="combined-btn-right" @click="toggleDatePicker" title="选择日期">
                  <CuteIcon name="CalendarDays" :size="16" />
                </button>
                <input
                  ref="dateInputRef"
                  type="date"
                  v-model="calendarModeCurrentDate"
                  class="date-input-hidden"
                  @change="onDatePickerChange"
                />
              </div>

              <!-- 本月按钮 + 日历选择器（仅月视图） -->
              <div v-if="calendarModeViewType === 'month'" class="combined-btn-wrapper">
                <button class="combined-btn-left" @click="goToThisMonth" title="回到本月">
                  <span>本月</span>
                </button>
                <button class="combined-btn-right" @click="toggleDatePicker" title="选择日期">
                  <CuteIcon name="CalendarDays" :size="16" />
                </button>
                <input
                  ref="dateInputRef"
                  type="date"
                  v-model="calendarModeCurrentDate"
                  class="date-input-hidden"
                  @change="onDatePickerChange"
                />
              </div>
            </div>

            <!-- 中间：占位 -->
            <div class="spacer"></div>

            <!-- 右侧控制组 -->
            <div class="controls-right">
              <!-- 缩放按钮（仅周视图） -->
              <button
                v-if="calendarModeViewType === 'week'"
                class="zoom-btn"
                title="切换缩放"
                @click="cycleZoom"
              >
                {{ calendarZoom }}x
              </button>

              <!-- 月视图筛选菜单 -->
              <CuteDropdown
                v-if="calendarModeViewType === 'month'"
                :close-on-select="false"
              >
                <template #trigger>
                  <button class="filter-btn">
                    <span>筛选</span>
                    <CuteIcon name="ChevronDown" :size="14" />
                  </button>
                </template>
                <CuteDropdownItem @click.prevent>
                  <label class="filter-option">
                    <CuteCheckbox
                      :checked="monthViewFilters.showRecurringTasks"
                      size="small"
                      @update:checked="(val) => (monthViewFilters.showRecurringTasks = val)"
                    />
                    <span>循环任务</span>
                  </label>
                </CuteDropdownItem>
                <CuteDropdownItem @click.prevent>
                  <label class="filter-option">
                    <CuteCheckbox
                      :checked="monthViewFilters.showScheduledTasks"
                      size="small"
                      @update:checked="(val) => (monthViewFilters.showScheduledTasks = val)"
                    />
                    <span>已排期任务</span>
                  </label>
                </CuteDropdownItem>
                <CuteDropdownItem @click.prevent>
                  <label class="filter-option">
                    <CuteCheckbox
                      :checked="monthViewFilters.showDueDates"
                      size="small"
                      @update:checked="(val) => (monthViewFilters.showDueDates = val)"
                    />
                    <span>截止日期</span>
                  </label>
                </CuteDropdownItem>
                <CuteDropdownItem @click.prevent>
                  <label class="filter-option">
                    <CuteCheckbox
                      :checked="monthViewFilters.showAllDayEvents"
                      size="small"
                      @update:checked="(val) => (monthViewFilters.showAllDayEvents = val)"
                    />
                    <span>全天事件</span>
                  </label>
                </CuteDropdownItem>
              </CuteDropdown>

              <!-- 周视图/月视图选择器 -->
              <CuteDropdown
                :model-value="calendarModeViewType"
                :options="calendarModeViewOptions"
                @update:model-value="onCalendarModeViewChange"
              >
                <template #trigger>
                  <button class="day-count-trigger">
                    <span>{{ calendarModeViewType === 'week' ? '周视图' : '月视图' }}</span>
                    <CuteIcon name="ChevronDown" :size="14" />
                  </button>
                </template>
              </CuteDropdown>
            </div>
          </template>

          <!-- ========== 普通模式控制栏 ========== -->
          <template v-else>
            <!-- 左侧：进入日历模式按钮 + 年月显示（日历视图） -->
            <div v-if="props.currentRightPaneView === 'calendar'" class="calendar-left-controls">
              <button
                class="calendar-mode-btn"
                title="进入日历模式"
                @click="emit('enter-calendar-mode')"
              >
                <CuteIcon name="ChevronsLeft" :size="18" />
              </button>
              <div class="calendar-year-month">
                {{ calendarYearMonth }}
              </div>
            </div>

            <!-- 左侧：年月显示（时间线视图） -->
            <div v-else-if="props.currentRightPaneView === 'timeline'" class="calendar-left-controls">
              <div class="calendar-year-month">
                {{ calendarYearMonth }}
              </div>
            </div>

            <!-- 中间：占位 -->
            <div class="spacer"></div>

            <!-- 右侧控制组 -->
            <div class="controls-right">
              <!-- 缩放按钮（仅日历视图显示） -->
              <button
                v-if="props.currentRightPaneView === 'calendar'"
                class="zoom-btn"
                title="切换缩放"
                @click="cycleZoom"
              >
                {{ calendarZoom }}x
              </button>

              <!-- 月视图筛选菜单 -->
              <CuteDropdown
                v-if="
                  props.currentRightPaneView === 'calendar' && effectiveCalendarViewType === 'month'
                "
                :close-on-select="false"
              >
                <template #trigger>
                  <button class="filter-btn">
                    <span>筛选</span>
                    <CuteIcon name="ChevronDown" :size="14" />
                  </button>
                </template>
                <CuteDropdownItem @click.prevent>
                  <label class="filter-option">
                    <CuteCheckbox
                      :checked="monthViewFilters.showRecurringTasks"
                      size="small"
                      @update:checked="(val) => (monthViewFilters.showRecurringTasks = val)"
                    />
                    <span>循环任务</span>
                  </label>
                </CuteDropdownItem>
                <CuteDropdownItem @click.prevent>
                  <label class="filter-option">
                    <CuteCheckbox
                      :checked="monthViewFilters.showScheduledTasks"
                      size="small"
                      @update:checked="(val) => (monthViewFilters.showScheduledTasks = val)"
                    />
                    <span>已排期任务</span>
                  </label>
                </CuteDropdownItem>
                <CuteDropdownItem @click.prevent>
                  <label class="filter-option">
                    <CuteCheckbox
                      :checked="monthViewFilters.showDueDates"
                      size="small"
                      @update:checked="(val) => (monthViewFilters.showDueDates = val)"
                    />
                    <span>截止日期</span>
                  </label>
                </CuteDropdownItem>
                <CuteDropdownItem @click.prevent>
                  <label class="filter-option">
                    <CuteCheckbox
                      :checked="monthViewFilters.showAllDayEvents"
                      size="small"
                      @update:checked="(val) => (monthViewFilters.showAllDayEvents = val)"
                    />
                    <span>全天事件</span>
                  </label>
                </CuteDropdownItem>
              </CuteDropdown>
            </div>
          </template>
        </div>
      </template>

      <template #bottom>
        <!-- 日历视图 -->
        <div v-if="props.currentRightPaneView === 'calendar'" class="calendar-wrapper">
          <CuteCalendar
            ref="calendarRef"
            :current-date="effectiveCurrentDate"
            :view-type="effectiveCalendarViewType"
            :zoom="calendarZoom"
            :days="props.calendarDays"
            :month-view-filters="monthViewFilters"
            @month-date-click="onMonthDateClick"
          />
        </div>
        <!-- 时间线视图 -->
        <DoubleRowTimeline
          v-else-if="props.currentRightPaneView === 'timeline'"
          :current-month="currentCalendarDate.slice(0, 7)"
          :month-view-filters="monthViewFilters"
          :layout-mode="props.leftViewType === 'projects' ? 'single' : 'auto'"
        />
        <!-- Staging 视图 -->
        <StagingList v-else-if="props.currentRightPaneView === 'staging'" />
        <!-- Upcoming 视图 -->
        <UpcomingPanel v-else-if="props.currentRightPaneView === 'upcoming'" />
        <!-- Templates 视图 -->
        <TemplateList v-else-if="props.currentRightPaneView === 'templates'" />
      </template>
    </TwoRowLayout>

    <!-- 时间块创建对话框（贴靠时间片左侧的浮动面板） -->
    <TimeBlockCreateDialog
      :show="uiStore.isTimeBlockCreateDialogOpen"
      :position="timeBlockDialogPosition"
      @confirm="handleTimeBlockCreate"
      @cancel="handleTimeBlockDialogCancel"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import CuteCalendar from '@/components/assembles/calender/CuteCalendar.vue'
import DoubleRowTimeline from '@/components/parts/timeline/DoubleRowTimeline.vue'
import StagingList from '@/components/assembles/tasks/list/StagingList.vue'
import UpcomingPanel from '@/components/assembles/tasks/list/UpcomingPanel.vue'
import TemplateList from '@/components/assembles/template/TemplateList.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import CuteDropdown from '@/components/parts/CuteDropdown.vue'
import CuteDropdownItem from '@/components/parts/CuteDropdownItem.vue'
import TimeBlockCreateDialog from '@/components/organisms/TimeBlockCreateDialog.vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { getTodayDateString, toDateString } from '@/infra/utils/dateUtils'
import { useUIStore } from '@/stores/ui'
import { pipeline } from '@/cpu'

// Props
interface Props {
  currentCalendarDate?: string
  calendarDays?: 1 | 3 | 5
  leftViewType?: 'recent' | 'staging' | 'projects'
  currentRightPaneView?: 'calendar' | 'staging' | 'upcoming' | 'templates' | 'timeline'
  isCalendarMode?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  currentCalendarDate: () => getTodayDateString(),
  calendarDays: 3,
  leftViewType: 'recent',
  currentRightPaneView: 'calendar',
  isCalendarMode: false,
})

// Emits
const emit = defineEmits<{
  'calendar-size-update': []
  'enter-calendar-mode': []
  'exit-calendar-mode': []
  'date-click': [date: string]
}>()

// ==================== Stores ====================
const uiStore = useUIStore()

// 创建对话框位置（根据 UI Store 中的锚点信息计算）
const timeBlockDialogPosition = computed(() => {
  const context = uiStore.timeBlockCreateContext as {
    anchorTop?: number
    anchorLeft?: number
  } | null

  if (!context || context.anchorTop == null || context.anchorLeft == null) {
    return undefined
  }

  return {
    top: context.anchorTop,
    left: context.anchorLeft,
  }
})

function handleTimeBlockDialogCancel() {
  // 关闭对话框
  uiStore.closeTimeBlockCreateDialog()

  // 同时清理日历中的选区高亮
  const calendarComponent = calendarRef.value
  if (calendarComponent?.calendarRef) {
    const calendarApi = calendarComponent.calendarRef.getApi()
    calendarApi?.unselect()
  }
}

// ==================== 右栏视图状态 ====================
// 移除内部状态管理，使用从父组件传入的 currentRightPaneView

// ==================== 日历模式状态 ====================
const calendarModeViewType = ref<'week' | 'month'>('month') // 日历模式默认显示月视图
const calendarModeCurrentDate = ref<string>(getTodayDateString()) // 日历模式的当前日期
const dateInputRef = ref<HTMLInputElement | null>(null) // 日期输入框引用

const calendarModeViewOptions = [
  { value: 'week', label: '周视图' },
  { value: 'month', label: '月视图' },
]

function onCalendarModeViewChange(value: string) {
  calendarModeViewType.value = value as 'week' | 'month'
  logger.debug(LogTags.COMPONENT_CALENDAR, 'Calendar mode view changed', { view: value })
}

// 切换日期选择器
function toggleDatePicker() {
  if (dateInputRef.value) {
    dateInputRef.value.showPicker()
  }
}

// 日期选择器变化
function onDatePickerChange() {
  logger.debug(LogTags.COMPONENT_CALENDAR, 'Date picker changed', { date: calendarModeCurrentDate.value })
}

// 日历模式导航：上一周/月
function navigatePrevious() {
  const date = new Date(calendarModeCurrentDate.value)
  if (calendarModeViewType.value === 'week') {
    date.setDate(date.getDate() - 7)
  } else {
    date.setMonth(date.getMonth() - 1)
  }
  calendarModeCurrentDate.value = toDateString(date)
  logger.debug(LogTags.COMPONENT_CALENDAR, 'Navigate previous', { date: calendarModeCurrentDate.value })
}

// 日历模式导航：下一周/月
function navigateNext() {
  const date = new Date(calendarModeCurrentDate.value)
  if (calendarModeViewType.value === 'week') {
    date.setDate(date.getDate() + 7)
  } else {
    date.setMonth(date.getMonth() + 1)
  }
  calendarModeCurrentDate.value = toDateString(date)
  logger.debug(LogTags.COMPONENT_CALENDAR, 'Navigate next', { date: calendarModeCurrentDate.value })
}

// 跳转到本周
function goToThisWeek() {
  calendarModeCurrentDate.value = getTodayDateString()
  logger.debug(LogTags.COMPONENT_CALENDAR, 'Go to this week')
}

// 跳转到本月
function goToThisMonth() {
  calendarModeCurrentDate.value = getTodayDateString()
  logger.debug(LogTags.COMPONENT_CALENDAR, 'Go to this month')
}

// 月视图日期点击处理
function onMonthDateClick(date: string) {
  emit('date-click', date)
  logger.debug(LogTags.COMPONENT_CALENDAR, 'Month date clicked', { date })
}

// ==================== 日历状态 ====================
const calendarRef = ref<InstanceType<typeof CuteCalendar> | null>(null)
const calendarZoom = ref<1 | 2 | 3>(1)

// 月视图筛选状态
const monthViewFilters = ref({
  showRecurringTasks: true,
  showScheduledTasks: true,
  showDueDates: true,
  showAllDayEvents: true,
})

// 根据天数计算视图类型：总是显示多天视图
const calendarViewType = computed(() => {
  return 'day'
})

// 最终的日历视图类型
const effectiveCalendarViewType = computed(() => {
  // 日历模式：使用日历模式的视图类型
  if (props.isCalendarMode) {
    return calendarModeViewType.value
  }
  // Staging 视图强制使用月视图
  if (props.leftViewType === 'staging') {
    return 'month'
  }
  // Projects 视图使用周视图
  if (props.leftViewType === 'projects') {
    return 'week'
  }
  return calendarViewType.value
})

// 最终的当前日期
const effectiveCurrentDate = computed(() => {
  if (props.isCalendarMode) {
    return calendarModeCurrentDate.value
  }
  return props.currentCalendarDate
})

// 格式化日历年月显示
const calendarYearMonth = computed(() => {
  const dateStr = props.isCalendarMode ? calendarModeCurrentDate.value : props.currentCalendarDate
  if (!dateStr) return ''

  const date = new Date(dateStr)
  const year = date.getFullYear()
  const month = date.getMonth() + 1

  return `${year}年${month}月`
})

// 循环切换缩放等级
function cycleZoom() {
  if (calendarZoom.value === 1) {
    calendarZoom.value = 2
  } else if (calendarZoom.value === 2) {
    calendarZoom.value = 3
  } else {
    calendarZoom.value = 1
  }
  logger.debug(LogTags.COMPONENT_KANBAN_COLUMN, 'Calendar zoom cycled', {
    zoom: calendarZoom.value,
  })
}

// 通知父组件需要更新日历尺寸
function notifyCalendarSizeUpdate() {
  emit('calendar-size-update')
}

// 监听右栏视图变化，通知父组件更新日历尺寸
watch(
  () => props.currentRightPaneView,
  () => {
    notifyCalendarSizeUpdate()
  }
)

// ==================== 时间块创建逻辑 ====================
async function handleTimeBlockCreate(data: { type: 'task' | 'event'; title: string }) {
  const context = uiStore.timeBlockCreateContext
  if (!context) {
    logger.error(
      LogTags.COMPONENT_CALENDAR,
      'No context available for time block creation',
      new Error('Context is null')
    )
    return
  }

  try {
    if (data.type === 'task') {
      // 创建任务并关联时间块：
      // 1. 先创建任务（返回 TaskCard）
      const taskCard = await pipeline.dispatch('task.create', {
        title: data.title,
        estimated_duration: 60, // 默认 60 分钟
      })

      // 2. 使用 time_block.create_from_task 创建时间块并关联
      await pipeline.dispatch('time_block.create_from_task', {
        task_id: taskCard.id,
        start_time: context.startISO,
        end_time: context.endISO,
        start_time_local: context.startTimeLocal,
        end_time_local: context.endTimeLocal,
        time_type: 'FLOATING', // 默认使用浮动时间
        creation_timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
        is_all_day: context.isAllDay,
      })

      logger.info(LogTags.COMPONENT_CALENDAR, 'Created task with time block from calendar', {
        title: data.title,
        taskId: taskCard.id,
        startISO: context.startISO,
        endISO: context.endISO,
      })
    } else {
      // 创建事件：使用 time_block.create
      await pipeline.dispatch('time_block.create', {
        title: data.title,
        start_time: context.startISO,
        end_time: context.endISO,
        start_time_local: context.startTimeLocal,
        end_time_local: context.endTimeLocal,
        time_type: 'FLOATING', // 默认使用浮动时间
        creation_timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
        is_all_day: context.isAllDay,
      })

      logger.info(LogTags.COMPONENT_CALENDAR, 'Created time block from calendar', {
        title: data.title,
        startISO: context.startISO,
        endISO: context.endISO,
        isAllDay: context.isAllDay,
      })
    }

    // 关闭对话框
    uiStore.closeTimeBlockCreateDialog()

    // 清理日历中的选区高亮
    const calendarComponent = calendarRef.value
    if (calendarComponent?.calendarRef) {
      const calendarApi = calendarComponent.calendarRef.getApi()
      calendarApi?.unselect()
    }
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_CALENDAR,
      'Failed to create from calendar',
      error instanceof Error ? error : new Error(String(error)),
      { type: data.type, title: data.title }
    )

    // 显示错误信息
    let errorMessage = '创建失败，请重试'
    if (error instanceof Error) {
      errorMessage = error.message
    } else if (typeof error === 'string') {
      errorMessage = error
    }
    alert(`创建失败: ${errorMessage}`)
  }
}

// 暴露方法给父组件
defineExpose({
  calendarRef,
})
</script>

<style scoped>
.home-calendar-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ==================== 控制栏 ==================== */
.calendar-controls {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1.2rem;
  padding: 1.2rem 1.6rem;
  background-color: transparent;
}

.controls-right {
  display: flex;
  align-items: center;
  gap: 1.2rem;
}

/* 年月显示 */
.calendar-year-month {
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary);
  white-space: nowrap;
}

/* 左侧控制组 */
.calendar-left-controls {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

/* 日历模式按钮 */
.calendar-mode-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3.2rem;
  height: 3.2rem;
  padding: 0;
  color: var(--color-text-tertiary);
  background-color: transparent;
  border: 1px solid transparent;
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.calendar-mode-btn:hover {
  color: var(--color-text-primary);
  background-color: var(--color-background-hover, #e8e8e8);
  border-color: var(--color-border-default);
}

.calendar-mode-btn:active {
  transform: scale(0.95);
}

/* 控制按钮（导航、本周、本月） */
.control-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.6rem;
  height: 3.6rem;
  padding: 0 1.2rem;
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary);
  background-color: var(--color-background-secondary, #f5f5f5);
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.control-btn:hover {
  background-color: var(--color-background-hover, #e8e8e8);
  border-color: var(--color-border-hover);
}

.control-btn:active {
  transform: scale(0.98);
}

/* 导航按钮 */
.nav-btn {
  width: 3.6rem;
  padding: 0;
}

/* 合并按钮（本周/本月 + 日历） */
.combined-btn-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  height: 3.6rem;
}

.combined-btn-left,
.combined-btn-right {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 3.6rem;
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary);
  background-color: var(--color-background-secondary, #f5f5f5);
  border: 1px solid var(--color-border-default);
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.combined-btn-left {
  gap: 0.6rem;
  padding: 0 1.2rem;
  border-radius: 0.6rem 0 0 0.6rem;
  border-right: none;
}

.combined-btn-right {
  width: 3.6rem;
  padding: 0;
  border-radius: 0 0.6rem 0.6rem 0;
  border-left: 1px solid var(--color-border-default);
}

.combined-btn-left:hover,
.combined-btn-right:hover {
  background-color: var(--color-background-hover, #e8e8e8);
  border-color: var(--color-border-hover);
  z-index: 1;
}

.combined-btn-left:active,
.combined-btn-right:active {
  transform: scale(0.98);
}

.date-input-hidden {
  position: absolute;
  opacity: 0;
  pointer-events: none;
  width: 0;
  height: 0;
}

/* 天数选择器触发器 */
.day-count-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.6rem;
  height: 3.6rem;
  padding: 0 1.2rem;
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary);
  background-color: var(--color-background-secondary, #f5f5f5);
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
  outline: none;
  min-width: 7rem;
  white-space: nowrap;
}

.day-count-trigger:hover {
  background-color: var(--color-background-hover, #e8e8e8);
  border-color: var(--color-border-hover);
}

.day-count-trigger:active {
  transform: scale(0.98);
}

/* 占位 */
.spacer {
  flex: 1;
}

/* 缩放按钮 */
.zoom-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 3.6rem;
  padding: 0 1rem;
  font-size: 1.6rem;
  font-weight: 500;
  color: var(--color-text-secondary, #f0f);
  background-color: transparent;
  border: none;
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.zoom-btn:hover {
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-hover, #f0f);
}

.zoom-btn:active {
  transform: scale(0.98);
}

/* 筛选按钮 */
.filter-btn {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.6rem;
  height: 3.6rem;
  padding: 0 1.2rem;
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary);
  background-color: var(--color-background-secondary, #f5f5f5);
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
  min-width: 10rem;
}

.filter-btn:hover {
  background-color: var(--color-background-hover, #e8e8e8);
  border-color: var(--color-border-hover);
}

.filter-btn:active {
  transform: scale(0.98);
}

/* 筛选选项 */
.filter-option {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  width: 100%;
  font-size: 1.4rem;
  color: var(--color-text-primary);
  cursor: pointer;
  user-select: none;
}

.filter-option span {
  user-select: none;
}

/* 视图选择器按钮 */
.view-selector-btn {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.6rem;
  height: 3.6rem;
  padding: 0 1.2rem;
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary);
  background-color: var(--color-background-secondary, #f5f5f5);
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
  min-width: 10rem;
}

.view-selector-btn:hover {
  background-color: var(--color-background-hover, #e8e8e8);
  border-color: var(--color-border-hover);
}

.view-selector-btn:active {
  transform: scale(0.98);
}

/* 日历包装器 */
.calendar-wrapper {
  height: 100%;
  width: 100%;
  overflow: hidden;
}
</style>
