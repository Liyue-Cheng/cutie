<script setup lang="ts">
/**
 * TimelineDayCell - 时间线日期单元格组件
 *
 * 🎯 设计理念：
 * 采用上中下三栏结构：
 * - 上栏：标题（日期数字、月日、星期、今天徽章）
 * - 中栏：虚线分隔（与 TaskList 一致的视觉效果）
 * - 下栏：内容区（任务、截止日期、全天事件）
 *
 * 🔑 VIEW_CONTEXT_KEY 规范支持：
 * 完整支持 VIEW_CONTEXT_KEY_SPEC.md 中定义的所有视图类型：
 * - misc::all, misc::staging, misc::planned, etc.
 * - daily::{YYYY-MM-DD}
 * - area::{area_uuid}
 * - project::{project_uuid}
 *
 * 默认行为：当不传 viewKey 时，自动使用 `daily::${date}`
 *
 * 📦 功能：
 * - 使用 CuteDualModeCheckbox 进行任务状态切换
 * - 支持拖放操作（接收任务拖放到此日期）
 * - 右键菜单支持
 * - 点击打开任务编辑器
 * - 字体大小与 TaskStrip 保持一致（1.5rem）
 */
import { computed, ref } from 'vue'
import type { TaskCard, TimeBlockView } from '@/types/dtos'
import type { ViewMetadata } from '@/types/drag'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CellItemTask from './CellItemTask.vue'
import CellItemDeadline from './CellItemDeadline.vue'
import QuickAddTaskModal from '@/components/organisms/QuickAddTaskModal.vue'

// Checkbox状态类型
type CheckboxState = null | 'completed' | 'present'
import { useUIStore } from '@/stores/ui'
import { useContextMenu } from '@/composables/useContextMenu'
import KanbanTaskCardMenu from '@/components/assembles/tasks/kanban/KanbanTaskCardMenu.vue'
import CalendarEventMenu from '@/components/assembles/ContextMenu/CalendarEventMenu.vue'
import { useInteractDrag } from '@/composables/drag/useInteractDrag'
import { useDragStrategy } from '@/composables/drag/useDragStrategy'
import { dragPreviewState } from '@/infra/drag-interact'
import { deriveViewMetadata } from '@/services/viewAdapter'
import { useViewTasks } from '@/composables/useViewTasks'
import { logger, LogTags } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'
import { getTodayDateString } from '@/infra/utils/dateUtils'

interface Props {
  date: string // YYYY-MM-DD
  dayNumber: number
  tasks: TaskCard[]
  dueDates: TaskCard[]
  allDayEvents: TimeBlockView[]
  isToday: boolean
  isWeekend: boolean
  viewKey?: string // 🔥 支持完整的 VIEW_CONTEXT_KEY 规范，默认为 daily::date
}

const props = defineProps<Props>()

// 计算有效的 viewKey
const effectiveViewKey = computed(() => {
  return props.viewKey || `daily::${props.date}`
})

const uiStore = useUIStore()
// 🔥 使用 useViewTasks 获取带排序的任务，保证与 TaskList 一致的持久化顺序
const { tasks: sortedViewTasks } = useViewTasks(effectiveViewKey.value)

// 如果 viewTasks 还未加载完成，则退回到 props.tasks
const resolvedTasks = computed(() => {
  return sortedViewTasks.value.length > 0 ? sortedViewTasks.value : props.tasks
})

const contextMenu = useContextMenu()
const dragStrategy = useDragStrategy()

const cellRef = ref<HTMLElement | null>(null)
const showQuickAddDialog = ref(false)

const hasContent = computed(() => {
  return props.tasks.length > 0 || props.dueDates.length > 0 || props.allDayEvents.length > 0
})

// 格式化星期显示
const weekdayText = computed(() => {
  const date = new Date(props.date)
  const weekdays = ['周日', '周一', '周二', '周三', '周四', '周五', '周六']
  return weekdays[date.getDay()]
})

// 格式化月日显示
const monthDayText = computed(() => {
  const date = new Date(props.date)
  const month = date.getMonth() + 1
  return `${month}月${props.dayNumber}日`
})

// 判断日期是否已过期（使用本地时间）
const isPastDate = computed(() => {
  const today = getTodayDateString()
  return props.date < today
})

// ==================== ViewMetadata 推导 ====================
const effectiveViewMetadata = computed<ViewMetadata>(() => {
  const derived = deriveViewMetadata(effectiveViewKey.value)
  if (derived) {
    return derived
  }

  // 兜底：提供最小可用元数据
  return {
    id: effectiveViewKey.value,
    type: 'custom',
    label: `${monthDayText.value} ${weekdayText.value}`,
    config: {},
  } as ViewMetadata
})

// ==================== 拖放系统集成 ====================
// 标准化 viewKey 作为 CSS class（:: 替换为 --）
const normalizedViewKey = computed(() => effectiveViewKey.value.replace(/::/g, '--'))

const { displayItems } = useInteractDrag({
  viewMetadata: effectiveViewMetadata,
  items: resolvedTasks,
  containerRef: cellRef,
  draggableSelector: `.cell-task-wrapper-${normalizedViewKey.value}`,
  objectType: 'task',
  getObjectId: (task) => task.id,
  onDrop: async (session) => {
    logger.debug(LogTags.COMPONENT_CALENDAR, 'Timeline cell drop event', {
      session,
      targetViewKey: effectiveViewKey.value,
      displayItems: displayItems.value.length,
      dropIndex: dragPreviewState.value?.computed.dropIndex,
    })

    // 🎯 执行拖放策略
    const result = await dragStrategy.executeDrop(session, effectiveViewKey.value, {
      sourceContext: (session.metadata?.sourceContext as Record<string, any>) || {},
      targetContext: {
        taskIds: displayItems.value.map((t) => t.id),
        displayTasks: displayItems.value,
        dropIndex: dragPreviewState.value?.computed.dropIndex,
        viewKey: effectiveViewKey.value,
      },
    })

    if (!result.success) {
      const errorMessage = result.message || result.error || 'Unknown error'
      logger.error(
        LogTags.COMPONENT_CALENDAR,
        'Timeline cell drop failed',
        new Error(errorMessage),
        {
          result,
          session,
        }
      )
    } else {
      logger.info(LogTags.COMPONENT_CALENDAR, 'Timeline cell drop succeeded', {
        taskId: session.object.id,
        targetViewKey: effectiveViewKey.value,
      })
    }
  },
})

// 计算任务的checkbox状态
function getTaskCheckboxState(task: TaskCard): CheckboxState {
  if (task.is_completed) {
    return 'completed'
  }

  // 检查当前日期的outcome
  if (task.schedules) {
    const schedule = task.schedules.find((s) => s.scheduled_day === props.date)
    if (schedule && schedule.outcome === 'presence_logged') {
      return 'present'
    }
  }

  return null
}

// 处理checkbox状态变化
async function handleCheckboxStateChange(task: TaskCard, newState: CheckboxState) {
  try {
    if (newState === 'completed') {
      // 完成任务
      await pipeline.dispatch('task.complete', { id: task.id })
    } else if (newState === 'present') {
      // 记录presence
      await pipeline.dispatch('task.log_presence', {
        id: task.id,
        scheduled_day: props.date,
      })
    } else if (newState === null) {
      // 重新打开任务或取消presence
      const currentState = getTaskCheckboxState(task)
      if (currentState === 'completed') {
        await pipeline.dispatch('task.reopen', { id: task.id })
      } else if (currentState === 'present') {
        await pipeline.dispatch('task.cancel_presence', {
          id: task.id,
          scheduled_day: props.date,
        })
      }
    }
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_CALENDAR,
      'Failed to update task checkbox state',
      error instanceof Error ? error : new Error(String(error)),
      { taskId: task.id, newState }
    )
  }
}

function handleTaskClick(taskId: string) {
  uiStore.openEditor(taskId, effectiveViewKey.value)
}

function handleTaskContextMenu(event: MouseEvent, task: TaskCard) {
  event.preventDefault()
  event.stopPropagation()
  contextMenu.show(KanbanTaskCardMenu, { task, viewKey: effectiveViewKey.value }, event)
}

function handleDueDateClick(taskId: string) {
  uiStore.openEditor(taskId, effectiveViewKey.value)
}

function handleEventContextMenu(event: MouseEvent, timeBlock: TimeBlockView) {
  event.preventDefault()
  event.stopPropagation()
  contextMenu.show(CalendarEventMenu, { event: { id: timeBlock.id } }, event)
}
</script>

<template>
  <div
    ref="cellRef"
    class="timeline-day-cell"
    :class="{
      'is-today': isToday,
      'is-weekend': isWeekend,
      'has-content': hasContent,
      'is-past': isPastDate,
    }"
    :data-date="date"
  >
    <!-- 上栏：标题 -->
    <div class="cell-header">
      <div class="header-content">
        <span class="day-number">{{ dayNumber }}</span>
        <div class="date-info">
          <span class="month-day">{{ monthDayText }}</span>
          <span class="weekday">{{ weekdayText }}</span>
        </div>
      </div>
      <div class="header-actions">
        <div v-if="isToday" class="today-badge">今天</div>
        <button class="quick-add-button" @click.stop="showQuickAddDialog = true" title="添加任务">
          <CuteIcon name="Plus" :size="16" />
        </button>
      </div>
    </div>

    <!-- 中栏：虚线分隔 -->
    <div class="cell-divider"></div>

    <!-- 下栏：内容区 -->
    <div class="cell-content">
      <!-- 上部：截止日期区 -->
      <div v-if="dueDates.length > 0" class="deadline-area">
        <CellItemDeadline
          v-for="dueTask in dueDates"
          :key="`due-${dueTask.id}`"
          :task="dueTask"
          @click="handleDueDateClick(dueTask.id)"
        />
      </div>

      <!-- 下部：任务区（支持拖放排序） -->
      <div class="task-area">
        <div
          v-for="task in displayItems"
          :key="`task-${task.id}-${date}`"
          :class="[
            'task-card-wrapper',
            'cell-task-wrapper',
            `cell-task-wrapper-${normalizedViewKey}`,
            {
              'is-preview': (task as any)._isPreview === true,
              'drag-compact': (task as any)._dragCompact === true,
            },
          ]"
          :data-task-id="task.id"
        >
          <CellItemTask
            :task="task"
            :schedule-day="date"
            @click="handleTaskClick(task.id)"
            @contextmenu="handleTaskContextMenu($event, task)"
            @checkbox-change="(newState) => handleCheckboxStateChange(task, newState)"
          />
        </div>

        <!-- 空状态 -->
        <div v-if="displayItems.length === 0 && dueDates.length === 0" class="empty-state">
          <span>暂无内容</span>
        </div>
      </div>

      <!-- 全天事件列表 -->
      <div v-if="allDayEvents.length > 0" class="events-area">
        <div
          v-for="event in allDayEvents"
          :key="`event-${event.id}`"
          class="timeline-event"
          @contextmenu="handleEventContextMenu($event, event)"
        >
          <div class="event-icon">
            <CuteIcon name="Clock" :size="16" />
          </div>
          <div class="event-content">
            <div class="event-title">{{ event.title || 'Time Block' }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- 快速添加任务对话框 -->
    <QuickAddTaskModal
      :show="showQuickAddDialog"
      :view-key="effectiveViewKey"
      @close="showQuickAddDialog = false"
    />
  </div>
</template>

<style scoped>
.timeline-day-cell {
  display: flex;
  flex-direction: column;
  background: transparent;
  transition: all 0.15s ease;
}

/* ==================== 上栏：标题 ==================== */
.cell-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.6rem;
  background: transparent;
}

.header-content {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.day-number {
  font-size: 2.4rem;
  font-weight: 700;
  color: var(--color-text-primary);
  line-height: 1;
  min-width: 3rem;
}

.timeline-day-cell.is-today .day-number {
  color: var(--color-primary);
}

.date-info {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}

.month-day {
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary);
  line-height: 1.2;
}

.weekday {
  font-size: 1.2rem;
  color: var(--color-text-secondary);
  line-height: 1.2;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.quick-add-button {
  all: unset;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.4rem;
  height: 2.4rem;
  border-radius: 0.4rem;
  cursor: pointer;
  color: var(--color-text-secondary);
  background: var(--color-background-secondary);
  transition: all 0.15s ease;
  opacity: 0;
  pointer-events: none;
}

.quick-add-button:hover {
  background: var(--color-background-hover);
  color: var(--color-text-primary);
}

.timeline-day-cell:hover .quick-add-button {
  opacity: 1;
  pointer-events: auto;
}

.today-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-primary, #6366f1);
  color: var(--color-button-primary-text, #fff);
  padding: 0.4rem 0.8rem;
  border-radius: 1rem;
  font-size: 1.2rem;
  font-weight: 500;
  line-height: 1;
  flex-shrink: 0;
}

/* ==================== 过期日期遮罩 ==================== */
.timeline-day-cell.is-past {
  position: relative;
}

.timeline-day-cell.is-past::before {
  content: '';
  position: absolute;
  inset: 0;
  background: rgb(255 255 255 / 60%);
  pointer-events: none;
  z-index: 1;
  border-radius: inherit;
}

/* ==================== 中栏：虚线分隔 ==================== */
.cell-divider {
  height: 0;
  border-bottom: 2px dashed rgb(0 0 0 / 15%);
  margin: 0;
}

/* ==================== 下栏：内容区 ==================== */
.cell-content {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  padding: 1.2rem 1.6rem;
  flex: 1;
  min-height: 8rem;
}

/* 截止日期区 */
.deadline-area {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

/* 任务区（支持拖放） */
.task-area {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  flex: 1;
  min-height: 0;
  position: relative;
}

.cell-task-wrapper {
  transition: transform 0.15s cubic-bezier(0.4, 0, 0.2, 1);
  will-change: transform;
  backface-visibility: hidden;
  contain: paint;
}

/* 拖放预览样式 */
.cell-task-wrapper.is-preview {
  opacity: 0.6;
}

.cell-task-wrapper.drag-compact {
  opacity: 0.3;
  transform: scale(0.95);
}

/* 全天事件区 */
.events-area {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.timeline-event {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  padding: 0.8rem;
  border-radius: 0.6rem;
  transition: background-color 0.15s ease;
  cursor: pointer;
  background: var(--color-background-secondary);
}

.timeline-event:hover {
  background: var(--color-background-hover);
}

.event-icon {
  flex-shrink: 0;
  color: var(--color-text-secondary);
  display: flex;
  align-items: center;
}

.event-content {
  flex: 1;
  min-width: 0;
}

.event-title {
  font-size: 1.5rem;
  font-weight: 500;
  color: var(--color-text-primary);
  line-height: 1.4;
  overflow-wrap: break-word;
}

/* 空状态 */
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  color: var(--color-text-tertiary);
  font-size: 1.3rem;
}

/* 特殊状态 */
.timeline-day-cell.is-today {
  background: transparent;
}

/* 拖放接收状态 */
.timeline-day-cell[data-zone-receiving='true'] {
  background: var(--color-primary-bg, rgb(74 144 226 / 10%));
}

.timeline-day-cell[data-zone-receiving='true'] .cell-divider {
  border-color: var(--color-primary);
}
</style>
