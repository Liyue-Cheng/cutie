<template>
  <div class="calendar-panel">
    <TwoRowLayout>
      <template #top>
        <div class="calendar-controls">
          <!-- ========== 日历模式控制栏 ========== -->
          <template v-if="props.isCalendarMode">
            <!-- 左侧：年月标题（可点击展开导航面板） -->
            <div class="controls-left">
              <div class="date-title-wrapper">
                <div class="date-title" @click="toggleNavPanel">
                  <span class="date-text">{{ calendarYearMonth }}</span>
                </div>
              </div>

              <!-- 导航面板 -->
              <div v-if="showNavPanel" ref="navPanelRef" class="date-nav-panel">
                <div class="panel-header">
                  <span class="panel-title">导航</span>
                </div>
                <div class="panel-body">
                  <!-- 导航控制 -->
                  <div class="nav-row">
                    <button class="panel-nav-btn" title="上一周/月" @click="navigatePrevious">
                      <CuteIcon name="ChevronLeft" :size="16" />
                    </button>
                    <span class="current-range">{{ calendarYearMonth }}</span>
                    <button class="panel-nav-btn" title="下一周/月" @click="navigateNext">
                      <CuteIcon name="ChevronRight" :size="16" />
                    </button>
                  </div>

                  <!-- 日期输入 -->
                  <div class="date-input-row">
                    <label class="date-label">跳转到日期</label>
                    <input
                      type="date"
                      v-model="calendarModeCurrentDate"
                      class="date-input"
                      @change="onDatePickerChange"
                    />
                  </div>

                  <!-- 快捷按钮 -->
                  <button class="today-btn" @click="goToToday">
                    <CuteIcon name="Calendar" :size="16" />
                    <span>{{ calendarModeViewType === 'week' ? '回到本周' : '回到本月' }}</span>
                  </button>
                </div>
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

              <!-- 导航按钮组 -->
              <div class="nav-buttons">
                <button class="nav-btn" title="上一周/月" @click="navigatePrevious">
                  <CuteIcon name="ChevronLeft" :size="18" />
                </button>
                <button class="nav-btn" title="下一周/月" @click="navigateNext">
                  <CuteIcon name="ChevronRight" :size="18" />
                </button>
                <button class="nav-btn today-nav-btn" title="回到今天" @click="goToToday">
                  <CuteIcon name="Calendar" :size="18" />
                </button>
              </div>

              <!-- 统一菜单按钮 -->
              <CuteDropdown :close-on-select="false" :max-height="'none'" align-right>
                <template #trigger>
                  <button class="icon-btn menu-btn" title="设置">
                    <CuteIcon name="Menu" :size="18" />
                  </button>
                </template>

                <!-- 视图切换（横向按钮组） -->
                <div class="menu-row">
                  <span class="menu-section-label">视图</span>
                  <div class="view-type-buttons">
                    <button
                      class="view-type-btn"
                      :class="{ active: calendarModeViewType === 'week' }"
                      @click.stop="onCalendarModeViewChange('week')"
                    >
                      周
                    </button>
                    <button
                      class="view-type-btn"
                      :class="{ active: calendarModeViewType === 'month' }"
                      @click.stop="onCalendarModeViewChange('month')"
                    >
                      月
                    </button>
                  </div>
                </div>

                <!-- 分隔线（仅月视图显示筛选） -->
                <template v-if="calendarModeViewType === 'month'">
                  <div class="menu-divider"></div>

                  <!-- 筛选选项 -->
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
                </template>
              </CuteDropdown>
            </div>
          </template>

          <!-- ========== 普通模式控制栏 ========== -->
          <template v-else>
            <!-- 左侧：年月显示 -->
            <div class="controls-left">
              <div class="date-title-wrapper">
                <div class="date-title-static">
                  <span class="date-text">{{ calendarYearMonth }}</span>
                </div>
              </div>
            </div>

            <!-- 中间：占位 -->
            <div class="spacer"></div>

            <!-- 右侧控制组 -->
            <div class="controls-right">
              <!-- 缩放按钮 -->
              <button
                class="zoom-btn"
                title="切换缩放"
                @click="cycleZoom"
              >
                {{ calendarZoom }}x
              </button>

              <!-- 月视图筛选菜单 -->
              <CuteDropdown
                v-if="effectiveCalendarViewType === 'month'"
                :close-on-select="false"
                :max-height="'none'"
                align-right
              >
                <template #trigger>
                  <button class="icon-btn menu-btn" title="筛选">
                    <CuteIcon name="Menu" :size="18" />
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
        <div class="calendar-wrapper">
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
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import CuteCalendar from '@/components/assembles/calender/CuteCalendar.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import CuteDropdown from '@/components/parts/CuteDropdown.vue'
import CuteDropdownItem from '@/components/parts/CuteDropdownItem.vue'
import TimeBlockCreateDialog from '@/components/organisms/TimeBlockCreateDialog.vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { getTodayDateString, toDateString } from '@/infra/utils/dateUtils'
import { useUIStore } from '@/stores/ui'
import { useUserSettingsStore } from '@/stores/user-settings'
import { pipeline } from '@/cpu'
import { dialog } from '@/composables/useDialog'

// Props
interface Props {
  currentCalendarDate?: string
  calendarDays?: 1 | 3 | 5
  leftViewType?: 'recent' | 'staging' | 'projects'
  isCalendarMode?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  currentCalendarDate: () => getTodayDateString(),
  calendarDays: 3,
  leftViewType: 'recent',
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
const userSettingsStore = useUserSettingsStore()

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

// ==================== 日历模式状态 ====================
// 从用户设置中读取默认值
const calendarModeViewType = ref<'week' | 'month'>(userSettingsStore.internalCalendarDefaultViewType)
const calendarModeCurrentDate = ref<string>(getTodayDateString()) // 日历模式的当前日期
const showNavPanel = ref(false) // 导航面板显示状态
const navPanelRef = ref<HTMLElement | null>(null) // 导航面板引用

function onCalendarModeViewChange(value: string) {
  calendarModeViewType.value = value as 'week' | 'month'
  logger.debug(LogTags.COMPONENT_CALENDAR, 'Calendar mode view changed', { view: value })
}

// 监听视图类型变化，自动保存到设置
watch(calendarModeViewType, (newValue) => {
  pipeline.dispatch('user_settings.update', {
    key: 'internal.calendar.default_view_type',
    value: newValue,
    value_type: 'string',
  })
})

// 切换导航面板
function toggleNavPanel() {
  showNavPanel.value = !showNavPanel.value
}

// 点击外部关闭面板
function handleClickOutside(event: MouseEvent) {
  if (navPanelRef.value && !navPanelRef.value.contains(event.target as Node)) {
    const trigger = (event.target as Element).closest('.date-title')
    if (!trigger) {
      showNavPanel.value = false
    }
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

// 跳转到今天（本周/本月）
function goToToday() {
  calendarModeCurrentDate.value = getTodayDateString()
  showNavPanel.value = false
  logger.debug(LogTags.COMPONENT_CALENDAR, 'Go to today')
}

// 月视图日期点击处理
function onMonthDateClick(date: string) {
  emit('date-click', date)
  logger.debug(LogTags.COMPONENT_CALENDAR, 'Month date clicked', { date })
}

// ==================== 日历状态 ====================
const calendarRef = ref<InstanceType<typeof CuteCalendar> | null>(null)
// 从用户设置中读取默认值
const calendarZoom = ref<1 | 2 | 3>(userSettingsStore.internalCalendarDefaultZoom)

// 月视图筛选状态 - 从用户设置中读取默认值
const monthViewFilters = ref({
  showRecurringTasks: userSettingsStore.internalCalendarMonthFilterRecurring,
  showScheduledTasks: userSettingsStore.internalCalendarMonthFilterScheduled,
  showDueDates: userSettingsStore.internalCalendarMonthFilterDueDates,
  showAllDayEvents: userSettingsStore.internalCalendarMonthFilterAllDay,
})

// 监听缩放变化，自动保存到设置
watch(calendarZoom, (newValue) => {
  pipeline.dispatch('user_settings.update', {
    key: 'internal.calendar.default_zoom',
    value: newValue,
    value_type: 'number',
  })
})

// 监听月视图筛选变化，自动保存到设置
watch(
  monthViewFilters,
  (newValue) => {
    pipeline.dispatch('user_settings.update_batch', {
      settings: [
        { key: 'internal.calendar.month_filter.recurring', value: newValue.showRecurringTasks, value_type: 'boolean' },
        { key: 'internal.calendar.month_filter.scheduled', value: newValue.showScheduledTasks, value_type: 'boolean' },
        { key: 'internal.calendar.month_filter.due_dates', value: newValue.showDueDates, value_type: 'boolean' },
        { key: 'internal.calendar.month_filter.all_day', value: newValue.showAllDayEvents, value_type: 'boolean' },
      ],
    })
  },
  { deep: true }
)

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

// 获取一年中的第几周（ISO 8601 标准）
function getWeekNumber(date: Date): number {
  const d = new Date(Date.UTC(date.getFullYear(), date.getMonth(), date.getDate()))
  // 设置为本周四（ISO 周从周一开始，周四决定周属于哪一年）
  const dayNum = d.getUTCDay() || 7
  d.setUTCDate(d.getUTCDate() + 4 - dayNum)
  // 获取年初第一天
  const yearStart = new Date(Date.UTC(d.getUTCFullYear(), 0, 1))
  // 计算周数
  const weekNo = Math.ceil(((d.getTime() - yearStart.getTime()) / 86400000 + 1) / 7)
  return weekNo
}

// 格式化日历年月显示
const calendarYearMonth = computed(() => {
  const dateStr = props.isCalendarMode ? calendarModeCurrentDate.value : props.currentCalendarDate
  if (!dateStr) return ''

  const date = new Date(dateStr)
  const year = date.getFullYear()
  const month = date.getMonth() + 1

  // 周视图时显示第几周
  if (props.isCalendarMode && calendarModeViewType.value === 'week') {
    const weekNumber = getWeekNumber(date)
    return `${year}年${month}月 · 第${weekNumber}周`
  }

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

// ==================== 时间块创建逻辑 ====================
async function handleTimeBlockCreate(data: { type: 'task' | 'event'; title: string; description?: string }) {
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
        glance_note: data.description || null,
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
        description: data.description || null,
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
    await dialog.alert(`创建失败: ${errorMessage}`)
  }
}

// ==================== 生命周期 ====================
onMounted(() => {
  // 添加点击外部关闭面板的监听器
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})

// 暴露方法给父组件
defineExpose({
  calendarRef,
})
</script>

<style scoped>
.calendar-panel {
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
  padding: 1.2rem 0.8rem 1.2rem 1.6rem;
  background-color: transparent;
}

.controls-left {
  position: relative;
  display: flex;
  align-items: center;
  gap: 1.2rem;
}

.controls-right {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

/* ==================== 日期标题样式 ==================== */
.date-title-wrapper {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.date-title {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  cursor: pointer;
  transition: opacity 0.2s ease;
}

.date-title:hover {
  opacity: 0.7;
}

.date-title:active {
  opacity: 0.5;
}

.date-title-static {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.date-text {
  font-size: 1.8rem;
  font-weight: 500;
  color: var(--color-text-primary, #f0f);
  line-height: 1.4;
  white-space: nowrap;
}

/* ==================== 导航面板 ==================== */
.date-nav-panel {
  position: absolute;
  top: calc(100% + 0.8rem);
  left: 0;
  z-index: 100;
  min-width: 24rem;
  background-color: var(--color-background-primary, #f0f);
  border: 1px solid var(--color-border-default, #f0f);
  border-radius: 0.8rem;
  box-shadow: var(--shadow-lg, #f0f);
  overflow: hidden;
}

.panel-header {
  padding: 1.2rem 1.6rem;
  border-bottom: 1px solid var(--color-border-light, #f0f);
}

.panel-title {
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary, #f0f);
  line-height: 1.4;
}

.panel-body {
  padding: 1.2rem 1.6rem;
  display: flex;
  flex-direction: column;
  gap: 1.2rem;
}

/* 导航行 */
.nav-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.panel-nav-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3.2rem;
  height: 3.2rem;
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-secondary, #f0f);
  border: 1px solid var(--color-border-default, #f0f);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.panel-nav-btn:hover {
  background-color: var(--color-background-hover, #f0f);
  border-color: var(--color-border-hover, #f0f);
}

.panel-nav-btn:active {
  transform: scale(0.95);
}

.current-range {
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary, #f0f);
  line-height: 1.4;
}

/* 日期输入行 */
.date-input-row {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.date-label {
  font-size: 1.2rem;
  font-weight: 500;
  color: var(--color-text-secondary, #f0f);
  line-height: 1.4;
}

.date-input {
  width: 100%;
  height: 3.6rem;
  padding: 0 1rem;
  font-size: 1.4rem;
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-secondary, #f0f);
  border: 1px solid var(--color-border-default, #f0f);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.date-input:hover {
  border-color: var(--color-border-hover, #f0f);
}

.date-input:focus {
  outline: none;
  border-color: var(--color-border-focus, #f0f);
}

/* 今天按钮 */
.today-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.6rem;
  height: 3.6rem;
  padding: 0 1.2rem;
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-secondary, #f0f);
  border: 1px solid var(--color-border-default, #f0f);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.today-btn:hover {
  background-color: var(--color-background-hover, #f0f);
  border-color: var(--color-border-hover, #f0f);
}

.today-btn:active {
  transform: scale(0.98);
}

/* ==================== 图标按钮 ==================== */
.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3.6rem;
  height: 3.6rem;
  padding: 0;
  color: var(--color-text-secondary, #f0f);
  background-color: transparent;
  border: 1px solid transparent;
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.icon-btn:hover {
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-hover, #f0f);
  border-color: var(--color-border-default, #f0f);
}

.icon-btn:active {
  transform: scale(0.95);
}

/* 菜单按钮 */
.menu-btn {
  margin-left: 0;
}

/* ==================== 导航按钮组 ==================== */
.nav-buttons {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.nav-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3.6rem;
  height: 3.6rem;
  padding: 0;
  color: var(--color-text-secondary, #f0f);
  background-color: transparent;
  border: 1px solid transparent;
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.nav-btn:hover {
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-hover, #f0f);
  border-color: var(--color-border-default, #f0f);
}

.nav-btn:active {
  transform: scale(0.95);
}

/* ==================== 菜单样式 ==================== */
/* 菜单分组标签 */
.menu-section-label {
  font-size: 1.2rem;
  font-weight: 600;
  color: var(--color-text-tertiary, #f0f);
}

/* 菜单行（用于视图选择等） */
.menu-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1.2rem;
  padding: 0.8rem 1.2rem;
}

/* 菜单分隔线 */
.menu-divider {
  height: 1px;
  background-color: var(--color-divider, #f0f);
  margin: 0.4rem 1.2rem;
}

/* 视图类型按钮组 */
.view-type-buttons {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.view-type-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3.2rem;
  height: 2.8rem;
  font-size: 1.3rem;
  font-weight: 500;
  color: var(--color-text-secondary, #f0f);
  background-color: transparent;
  border: 1px solid var(--color-border-default, #f0f);
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.15s ease;
}

.view-type-btn:hover {
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-hover, #f0f);
}

.view-type-btn.active {
  color: var(--color-text-on-accent, #f0f);
  background-color: var(--color-background-accent, #f0f);
  border-color: var(--color-background-accent, #f0f);
}

/* 筛选选项 */
.filter-option {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  width: 100%;
  font-size: 1.4rem;
  color: var(--color-text-primary, #f0f);
  cursor: pointer;
  user-select: none;
}

.filter-option span {
  user-select: none;
}

/* 占位 */
.spacer {
  flex: 1;
}

/* ==================== 缩放按钮 ==================== */
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

/* 日历包装器 */
.calendar-wrapper {
  height: 100%;
  width: 100%;
  overflow: hidden;
}
</style>
