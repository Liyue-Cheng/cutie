<!--
  HomeCalendarPanel - 主页日历面板

  🎯 功能：
  - 整合多种日历视图（日历、时间线、Staging、Upcoming、Templates）
  - 管理日历的缩放、筛选等控制
  - 集成时间块创建对话框（TimeBlockCreateDialog）

  🎨 布局结构：
  - 上栏：控制栏（年月显示、缩放按钮、筛选菜单）
  - 下栏：内容区（根据 currentRightPaneView 切换不同视图）

  🔑 支持的视图：
  - calendar：完整的 FullCalendar 日历（CuteCalendar）
  - timeline：双行时间线视图（DoubleRowTimeline）
  - staging：Staging 任务列表
  - upcoming：即将到期任务列表
  - templates：任务模板列表

  🚀 框选创建流程：
  1. 用户在 CuteCalendar 中框选时间段
  2. CuteCalendar 调用 handlers.handleTimeGridSelection
  3. 本组件显示 TimeBlockCreateDialog
  4. 用户选择 Task/Event 并填写标题
  5. handleTimeBlockCreate 创建真实的任务或时间块
  6. clearCalendarSelectionAndPreview 清理预览
-->
<template>
  <div class="home-calendar-panel">
    <TwoRowLayout>
      <template #top>
        <div class="calendar-controls">
          <!-- 左侧：年月显示 -->
          <div v-if="props.currentRightPaneView === 'calendar'" class="calendar-year-month">
            {{ calendarYearMonth }}
          </div>

          <!-- 中间：占位 -->
          <div class="spacer"></div>

          <!-- 右侧控制组 -->
          <div class="controls-right">
            <!-- 缩放按钮（仅日历视图显示） -->
            <button
              v-if="props.currentRightPaneView === 'calendar'"
              class="zoom-btn"
              @click="cycleZoom"
              title="切换缩放"
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
        </div>
      </template>

      <template #bottom>
        <!-- 日历视图 -->
        <div v-if="props.currentRightPaneView === 'calendar'" class="calendar-wrapper">
          <CuteCalendar
            ref="calendarRef"
            :current-date="currentCalendarDate"
            :view-type="effectiveCalendarViewType"
            :zoom="calendarZoom"
            :days="calendarDays"
            :month-view-filters="monthViewFilters"
          />
        </div>
        <!-- 时间线视图 -->
        <DoubleRowTimeline
          v-else-if="props.currentRightPaneView === 'timeline'"
          :current-month="currentCalendarDate.slice(0, 7)"
          :month-view-filters="monthViewFilters"
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
import { getTodayDateString } from '@/infra/utils/dateUtils'
import { useUIStore } from '@/stores/ui'
import { pipeline } from '@/cpu'

// Props
interface Props {
  currentCalendarDate?: string
  calendarDays?: 1 | 3 | 5 | 7
  leftViewType?: 'recent' | 'staging' | 'projects'
  currentRightPaneView?: 'calendar' | 'staging' | 'upcoming' | 'templates' | 'timeline'
}

const props = withDefaults(defineProps<Props>(), {
  currentCalendarDate: () => getTodayDateString(),
  calendarDays: 3,
  leftViewType: 'recent',
  currentRightPaneView: 'calendar',
})

// Emits
const emit = defineEmits<{
  'calendar-size-update': []
}>()

// ==================== Stores ====================
const uiStore = useUIStore()

// ==================== 日历状态 ====================
const calendarRef = ref<InstanceType<typeof CuteCalendar> | null>(null) // 日历组件引用
const calendarZoom = ref<1 | 2 | 3>(1) // 缩放等级（1x/2x/3x）

/**
 * 创建对话框位置
 *
 * 🎯 根据 UI Store 中的锚点信息计算对话框显示位置
 *
 * 📌 坐标来源：
 * - CuteCalendar.handleTimeGridMouseUp 计算选区锚点
 * - useCalendarHandlers.handleTimeGridSelection 传递给 uiStore
 *
 * 📍 定位策略：
 * - top：锚点的 Y 坐标（选区中心）
 * - left：锚点的 X 坐标（选区左边界）
 * - TimeBlockCreateDialog 通过 transform: translate(-100%, -50%) 贴在左侧
 */
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

/**
 * 清除日历选区和预览
 *
 * 🧹 清理内容：
 * - resetSelectionState：清除自定义框选状态（isSelecting、起止时间、锚点等）
 * - clearPreview：清除预览事件（drag.previewEvent.value = null）
 *
 * 🔄 调用时机：
 * - 用户点击对话框外部取消创建
 * - 用户点击确认完成创建
 * - 切换日历日期（避免残留）
 *
 * 📌 注意：
 * - 已移除 calendarApi.unselect()，因为不再使用 FullCalendar 自带的 select
 */
function clearCalendarSelectionAndPreview() {
  const calendarComponent = calendarRef.value as any
  if (typeof calendarComponent?.resetSelectionState === 'function') {
    calendarComponent.resetSelectionState()
  }
  if (typeof calendarComponent?.clearPreview === 'function') {
    calendarComponent.clearPreview()
  }
}

/**
 * 处理创建对话框取消
 *
 * 🎯 流程：
 * 1. 关闭对话框（uiStore.closeTimeBlockCreateDialog）
 * 2. 清除日历上的预览卡片和选区状态
 *
 * 📌 用户体验：
 * - 点击对话框外部 → 触发此函数
 * - 点击"取消"按钮 → 触发此函数
 * - 按 Esc 键 → TimeBlockCreateDialog 内部处理，最终也触发此函数
 */
function handleTimeBlockDialogCancel() {
  uiStore.closeTimeBlockCreateDialog()
  clearCalendarSelectionAndPreview()
}

// ==================== 右栏视图状态 ====================
// 移除内部状态管理，使用从父组件传入的 currentRightPaneView

// 月视图筛选状态
const monthViewFilters = ref({
  showRecurringTasks: true,
  showScheduledTasks: true,
  showDueDates: true,
  showAllDayEvents: true,
})

// 根据天数计算视图类型：7天显示本周视图，其他显示多天视图
const calendarViewType = computed(() => {
  return props.calendarDays === 7 ? 'week' : 'day'
})

// 最终的日历视图类型：Staging 视图强制使用月视图，Projects 视图使用周视图
const effectiveCalendarViewType = computed(() => {
  if (props.leftViewType === 'staging') {
    return 'month'
  }
  if (props.leftViewType === 'projects') {
    return 'week'
  }
  return calendarViewType.value
})

// 格式化日历年月显示
const calendarYearMonth = computed(() => {
  const dateStr = props.currentCalendarDate
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

/**
 * ==================== 时间块创建逻辑 ====================
 *
 * 🎯 核心功能：
 * 根据用户在 TimeBlockCreateDialog 中的选择，创建 Task 或 Event
 *
 * 🔄 创建流程：
 * - Task：先创建任务 → 再用 time_block.create_from_task 关联时间块
 * - Event：直接用 time_block.create 创建独立时间块
 *
 * 📌 重要：
 * - Task 会在日历上显示为"带复选框的时间块"
 * - Event 会在日历上显示为"纯时间块（无复选框）"
 */
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
      // 📋 创建任务并关联时间块
      // 第一步：创建任务（返回 TaskCard）
      // ⚠️ 注意：预览卡片保持显示，避免网络延迟期间的空白
      const taskCard = await pipeline.dispatch('task.create', {
        title: data.title,
        estimated_duration: 60, // 默认 60 分钟（可在编辑器中修改）
      })

      // 🔥 任务创建成功后，立即清理预览和对话框
      // 时机：恰好在 time_block.create_from_task 的乐观更新之前
      // 效果：预览卡片 → 乐观更新临时时间块，无缝切换
      clearCalendarSelectionAndPreview()
      uiStore.closeTimeBlockCreateDialog()

      // 第二步：创建时间块并关联到任务（带乐观更新）
      // 🔑 使用 time_block.create_from_task 一次性完成：
      // - 创建时间块
      // - 建立任务 ↔ 时间块链接
      // - 创建 task_schedule 记录
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
      // 📅 创建独立事件（不关联任务）
      // 🔥 Event 不需要先创建任务，直接清理预览后立即创建
      clearCalendarSelectionAndPreview()
      uiStore.closeTimeBlockCreateDialog()

      // 使用 time_block.create 创建纯时间块（暂未启用乐观更新）
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

    // ✅ 创建成功（预览已在各自分支中清理）
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
  min-width: 5.6rem;
}

.zoom-btn:hover {
  background-color: var(--color-background-hover, #e8e8e8);
  border-color: var(--color-border-hover);
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
