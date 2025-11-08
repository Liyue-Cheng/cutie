<template>
  <div
    class="task-strip"
    :class="{ completed: task.is_completed }"
    @mousedown="onMouseDown"
    @click="handleClick"
    @contextmenu="showContextMenu"
  >
    <!-- 顶部：完成按钮 + 标题 + 预期时间 -->
    <div class="task-header">
      <CuteDualModeCheckbox
        class="main-checkbox"
        :state="checkboxState"
        size="large"
        @update:state="handleCheckboxStateChange"
        @click.stop
      />
      <div class="task-title" :class="{ completed: task.is_completed }">
        {{ task.title || '新任务' }}
      </div>

      <!-- 所属 Area 标签（移动到标题旁） -->
      <AreaTag
        v-if="area"
        class="area-tag-inline"
        :name="area.name"
        :color="area.color"
        size="normal"
      />

      <!-- 预期时间显示 -->
      <div class="estimated-duration-wrapper">
        <button class="estimated-duration" @click.stop="toggleTimePicker">
          {{ formattedDuration }}
        </button>

        <!-- 时间选择器弹窗 -->
        <div v-if="showTimePicker" class="time-picker-popup">
          <TimeDurationPicker
            :model-value="task.estimated_duration"
            @update:model-value="updateEstimatedDuration"
            @close="showTimePicker = false"
          />
        </div>
      </div>

      <!-- 时间块显示（如果有） -->
      <div v-if="todayTimeBlocks.length > 0" class="time-blocks-inline">
        <span v-for="block in todayTimeBlocks.slice(0, 3)" :key="block.id" class="time-tag">
          <span class="time-tag-dot" :style="{ backgroundColor: area?.color || '#999' }"></span>
          {{ formatTimeBlockStart(block) }}
        </span>
        <span v-if="todayTimeBlocks.length > 3" class="time-tag-more"
          >+{{ todayTimeBlocks.length - 3 }}</span
        >
      </div>
    </div>

    <!-- 概览笔记 -->
    <div v-if="task.glance_note" class="task-note">
      <span class="icon-wrapper">
        <CuteIcon name="FileText" size="1.4rem" />
      </span>
      <span class="note-text">{{ task.glance_note }}</span>
    </div>

    <!-- 截止日期显示 -->
    <div v-if="task.due_date" class="due-date-section">
      <span class="icon-wrapper">
        <!-- 硬截止：使用旗子图标，过期为红色，未过期为灰色 -->
        <CuteIcon
          v-if="task.due_date.type === 'HARD'"
          name="Flag"
          size="1.4rem"
          :color="task.due_date.is_overdue ? '#f44336' : '#999'"
        />
        <!-- 软截止：使用波浪号 -->
        <span v-else class="soft-deadline-icon">~</span>
      </span>

      <span
        class="due-date-text"
        :class="{
          overdue: task.due_date.is_overdue && task.due_date.type === 'HARD',
          'hard-deadline': task.due_date.type === 'HARD',
        }"
      >
        {{ formatDueDate(task.due_date.date) }}
      </span>
    </div>

    <!-- 子任务显示区 -->
    <div v-if="task.subtasks && task.subtasks.length > 0" class="subtasks-section">
      <div v-for="subtask in task.subtasks" :key="subtask.id" class="subtask-item">
        <CuteCheckbox
          :checked="subtask.is_completed"
          size="1.4rem"
          @update:checked="() => toggleSubtask(subtask.id)"
          @click.stop
        />
        <span class="subtask-title" :class="{ completed: subtask.is_completed }">
          {{ subtask.title }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { TaskCard } from '@/types/dtos'
import { useAreaStore } from '@/stores/area'
import { useUIStore } from '@/stores/ui'
import { useContextMenu } from '@/composables/useContextMenu'
import { getTodayDateString } from '@/infra/utils/dateUtils'
import { logger, LogTags } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'
import CuteIcon from './CuteIcon.vue'
import CuteCheckbox from './CuteCheckbox.vue'
import CuteDualModeCheckbox from './CuteDualModeCheckbox.vue'
import AreaTag from './AreaTag.vue'
import TimeDurationPicker from './TimeDurationPicker.vue'
import KanbanTaskCardMenu from './kanban/KanbanTaskCardMenu.vue'

// Props
interface Props {
  task: TaskCard
  viewKey?: string
}

const props = defineProps<Props>()

// Emits
const emit = defineEmits<{
  'toggle-subtask': [subtaskId: string]
}>()

// Stores
const areaStore = useAreaStore()
const uiStore = useUIStore()
const contextMenu = useContextMenu()

// 防误触：拖动后抑制一次点击
const suppressClickOnce = ref(false)
let mouseDownAt: { x: number; y: number } | null = null
const CLICK_SUPPRESS_DISTANCE = 4 // px

// 时间选择器状态
const showTimePicker = ref(false)

// 通过 area_id 从 store 获取完整 area 信息
const area = computed(() => {
  return props.task.area_id ? areaStore.getAreaById(props.task.area_id) : null
})

// 获取当前视图的日期 (YYYY-MM-DD)
// 如果 viewKey 是 daily::YYYY-MM-DD 格式，提取日期；否则使用今天
const currentDate = computed(() => {
  if (props.viewKey && props.viewKey.startsWith('daily::')) {
    return props.viewKey.split('::')[1]
  }
  return getTodayDateString()
})

// 获取当前日期的时间块（按开始时间排序）
const todayTimeBlocks = computed(() => {
  if (!props.task.schedules) return []

  const targetDate = currentDate.value
  const schedule = props.task.schedules.find((s) => s.scheduled_day === targetDate)

  if (!schedule || !schedule.time_blocks) {
    return []
  }

  // 按开始时间排序
  return [...schedule.time_blocks].sort((a, b) => {
    return a.start_time.localeCompare(b.start_time)
  })
})

// 计算今日时间块的总时长（分钟）
const todayTimeBlocksTotalDuration = computed(() => {
  if (todayTimeBlocks.value.length === 0) return 0

  let totalMinutes = 0
  for (const block of todayTimeBlocks.value) {
    const start = new Date(block.start_time)
    const end = new Date(block.end_time)
    const durationMs = end.getTime() - start.getTime()
    const durationMinutes = Math.round(durationMs / (1000 * 60))
    totalMinutes += durationMinutes
  }

  return totalMinutes
})

// 计算双模式复选框的状态
type CheckboxState = null | 'completed' | 'present'
const checkboxState = computed<CheckboxState>(() => {
  // 优先级1：如果任务已完成，显示完成状态
  if (props.task.is_completed) {
    return 'completed'
  }

  // 优先级2：检查当前日期的outcome是否为presence_logged
  // 从schedules数组中查找当前日期的schedule
  if (props.task.schedules) {
    const currentSchedule = props.task.schedules.find((s) => s.scheduled_day === currentDate.value)

    if (currentSchedule && currentSchedule.outcome === 'presence_logged') {
      return 'present'
    }
  }

  // 默认：未选中
  return null
})

// 格式化时间显示
const formattedDuration = computed(() => {
  // 如果有今日时间块，显示时间块总和
  if (todayTimeBlocks.value.length > 0) {
    const minutes = todayTimeBlocksTotalDuration.value

    if (minutes === 0) return 'tiny'

    const hours = Math.floor(minutes / 60)
    const mins = minutes % 60

    return `${hours}:${mins.toString().padStart(2, '0')}`
  }

  // 没有时间块时，显示预期时间
  if (props.task.estimated_duration === null || props.task.estimated_duration === 0) {
    return 'tiny'
  }

  const minutes = props.task.estimated_duration
  const hours = Math.floor(minutes / 60)
  const mins = minutes % 60

  return `${hours}:${mins.toString().padStart(2, '0')}`
})

// 格式化时间块的开始时间（HH:mm）
function formatTimeBlockStart(timeBlock: any): string {
  // 如果是浮动时间且有本地时间，使用本地时间
  if (timeBlock.time_type === 'FLOATING' && timeBlock.start_time_local) {
    return timeBlock.start_time_local.substring(0, 5) // HH:MM
  }

  // 否则使用UTC时间转换为本地时间显示
  const date = new Date(timeBlock.start_time)
  const hours = date.getHours().toString().padStart(2, '0')
  const minutes = date.getMinutes().toString().padStart(2, '0')
  return `${hours}:${minutes}`
}

// 格式化截止日期
function formatDueDate(isoString: string): string {
  const date = new Date(isoString)
  const month = (date.getMonth() + 1).toString().padStart(2, '0')
  const day = date.getDate().toString().padStart(2, '0')
  return `${month}/${day}`
}

// Methods
function toggleSubtask(subtaskId: string) {
  emit('toggle-subtask', subtaskId)
}

// 处理双模式复选框状态变化
async function handleCheckboxStateChange(newState: CheckboxState) {
  if (newState === 'completed') {
    // 完成任务
    await pipeline.dispatch('task.complete', { id: props.task.id })
  } else if (newState === 'present') {
    // 标记在场 - 更新当前日期的schedule outcome（后端API使用大写）
    await pipeline.dispatch('schedule.update', {
      task_id: props.task.id,
      scheduled_day: currentDate.value,
      updates: { outcome: 'PRESENCE_LOGGED' },
    })
  } else {
    // 取消状态
    if (props.task.is_completed) {
      // 重新打开任务
      await pipeline.dispatch('task.reopen', { id: props.task.id })
    } else {
      // 检查当前日期是否标记为在场
      const currentSchedule = props.task.schedules?.find(
        (s) => s.scheduled_day === currentDate.value
      )
      if (currentSchedule && currentSchedule.outcome === 'presence_logged') {
        // 取消在场标记，改回planned（后端API使用大写）
        await pipeline.dispatch('schedule.update', {
          task_id: props.task.id,
          scheduled_day: currentDate.value,
          updates: { outcome: 'PLANNED' },
        })
      }
    }
  }
}

// 切换时间选择器显示
function toggleTimePicker() {
  showTimePicker.value = !showTimePicker.value
}

// 更新预期时间
async function updateEstimatedDuration(duration: number | null) {
  try {
    await pipeline.dispatch('task.update', {
      id: props.task.id,
      updates: { estimated_duration: duration },
    })
    showTimePicker.value = false
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_TASK_BAR,
      'Error updating estimated duration',
      error instanceof Error ? error : new Error(String(error))
    )
  }
}

// 点击打开编辑器
function handleClick() {
  if (suppressClickOnce.value) {
    suppressClickOnce.value = false
    return
  }
  uiStore.openEditor(props.task.id, props.viewKey)
}

// 右键菜单
function showContextMenu(event: MouseEvent) {
  event.preventDefault()
  contextMenu.show(KanbanTaskCardMenu, { task: props.task, viewKey: props.viewKey }, event)
}

function onMouseDown(event: MouseEvent) {
  mouseDownAt = { x: event.clientX, y: event.clientY }
  const onUp = (ev: MouseEvent) => {
    if (mouseDownAt) {
      const dx = ev.clientX - mouseDownAt.x
      const dy = ev.clientY - mouseDownAt.y
      const dist = Math.sqrt(dx * dx + dy * dy)
      if (dist >= CLICK_SUPPRESS_DISTANCE) {
        suppressClickOnce.value = true
      }
    }
    mouseDownAt = null
    document.removeEventListener('mouseup', onUp)
  }
  document.addEventListener('mouseup', onUp)
}
</script>

<style scoped>
.task-strip {
  background-color: var(--color-background-content);
  border: none;
  border-radius: 0;
  padding: 0.8rem 1.6rem;
  margin-bottom: 0;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  justify-content: center;
  min-height: 4.8rem; /* 最小高度，确保有足够空间 */

  /* 防止动画过程中出现发丝线 */
  outline: 1px solid transparent;
  background-clip: padding-box;
  transition:
    background-color 0.2s ease,
    opacity 0.2s ease;
}

.task-strip:hover {
  background-color: var(--color-background-hover, rgb(0 0 0 / 2%));
}

.task-strip.completed {
  opacity: 0.7;
}

/* 顶部：完成按钮 + 标题 + 预期时间 */
.task-header {
  display: flex;
  align-items: center;
  gap: 1rem;
}

/* 当有其他内容时，标题栏需要底部间距 */
.task-strip:has(.task-note, .due-date-section, .subtasks-section) .task-header {
  margin-bottom: 0.8rem;
}

/* 主要完成复选框 */
.main-checkbox {
  flex-shrink: 0;
}

.task-title {
  flex: 1;
  font-size: 1.5rem;
  font-weight: 500;
  color: var(--color-text-primary);
  line-height: 1.4;
  overflow-wrap: break-word;
}

.task-title.completed {
  color: var(--color-text-secondary);
  text-decoration: line-through;
}

/* 预期时间显示 */
.estimated-duration-wrapper {
  position: relative;
  flex-shrink: 0;
}

.estimated-duration {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.3rem 0.8rem;
  font-family: inherit;
  font-size: 1.2rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  background-color: var(--color-background-hover, rgb(0 0 0 / 5%));
  border: 1.5px solid rgb(0 0 0 / 15%);
  border-radius: 1.2rem;
  line-height: 1.4;
  cursor: pointer;
  transition: all 0.2s ease;
}

.estimated-duration:hover {
  border-color: var(--color-primary, #4a90e2);
  background-color: var(--color-primary-bg, rgb(74 144 226 / 12%));
  color: var(--color-primary, #4a90e2);
}

.time-picker-popup {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 0.4rem;
  z-index: 100;
}

/* 时间块内联显示 */
.time-blocks-inline {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.time-tag {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.3rem 0.8rem;
  font-size: 1.2rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  background-color: var(--color-background-hover, rgb(0 0 0 / 5%));
  border: 1.5px solid rgb(0 0 0 / 15%);
  white-space: nowrap;
  border-radius: 1.2rem;
  line-height: 1.4;
}

.time-tag-dot {
  width: 0.6rem;
  height: 0.6rem;
  border-radius: 50%;
}

.time-tag-more {
  display: inline-flex;
  align-items: center;
  padding: 0.3rem 0.8rem;
  font-size: 1.2rem;
  font-weight: 500;
  color: var(--color-text-tertiary);
  background-color: var(--color-background-hover, rgb(0 0 0 / 5%));
  border: 1.5px solid rgb(0 0 0 / 15%);
  white-space: nowrap;
  border-radius: 1.2rem;
  line-height: 1.4;
}

/* 概览笔记 */
.task-note {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  margin-bottom: 0.8rem;
  padding-left: 3rem;
}

.icon-wrapper {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  color: var(--color-text-tertiary);
}

.note-text {
  flex: 1;
  font-size: 1.4rem;
  color: var(--color-text-secondary);
  line-height: 1.6;
  white-space: pre-wrap;
  overflow-wrap: break-word;
}

/* 截止日期显示 */
.due-date-section {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  margin-bottom: 0.8rem;
  padding-left: 3rem;
}

.soft-deadline-icon {
  font-size: 1.6rem;
  color: #999;
  font-weight: 600;
}

.due-date-text {
  font-size: 1.3rem;
  color: var(--color-text-secondary);
}

.due-date-text.hard-deadline {
  font-weight: 500;
}

.due-date-text.overdue {
  color: #f44336;
  font-weight: 600;
}

/* 子任务显示区 */
.subtasks-section {
  padding-left: 3rem;
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  margin-bottom: 0.8rem;
}

.subtask-item {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.subtask-title {
  font-size: 1.4rem;
  color: var(--color-text-secondary);
  line-height: 1.4;
}

.subtask-title.completed {
  color: var(--color-text-tertiary);
  text-decoration: line-through;
}

/* 标题行中的 Area 标签 */
.area-tag-inline {
  flex-shrink: 0;
}
</style>
