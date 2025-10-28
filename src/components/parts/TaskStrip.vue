<template>
  <div
    class="task-strip"
    :class="{ completed: task.is_completed }"
    @click="handleClick"
    @contextmenu="showContextMenu"
  >
    <!-- 顶部：完成按钮 + 标题 + 预期时间 -->
    <div class="task-header">
      <button
        class="complete-btn"
        :class="{ completed: task.is_completed }"
        @click.stop="toggleComplete"
      >
        <CuteIcon v-if="task.is_completed" name="Check" :size="16" />
      </button>
      <div class="task-title" :class="{ completed: task.is_completed }">
        {{ task.title || '新任务' }}
      </div>

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
    </div>

    <!-- 时间块显示（如果有） -->
    <div v-if="todayTimeBlocks.length > 0" class="time-blocks-section">
      <span
        v-for="block in todayTimeBlocks.slice(0, 3)"
        :key="block.id"
        class="time-tag"
        :style="{ backgroundColor: area?.color || '#ccc' }"
      >
        {{ formatTimeBlockStart(block) }}
      </span>
      <span v-if="todayTimeBlocks.length > 3" class="time-tag-more"
        >+{{ todayTimeBlocks.length - 3 }}</span
      >
    </div>

    <!-- 概览笔记 -->
    <div v-if="task.glance_note" class="task-note">
      <span class="icon-wrapper">
        <CuteIcon name="CornerDownRight" size="1.4rem" />
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
        <button
          class="subtask-complete-btn"
          :class="{ completed: subtask.is_completed }"
          @click.stop="toggleSubtask(subtask.id)"
        >
          <CuteIcon v-if="subtask.is_completed" name="Check" :size="12" />
        </button>
        <span class="subtask-title" :class="{ completed: subtask.is_completed }">
          {{ subtask.title }}
        </span>
      </div>
    </div>

    <!-- 底部：Area 标签 -->
    <div v-if="area" class="task-footer">
      <AreaTag :name="area.name" :color="area.color" size="small" />
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
  'toggle-complete': []
  'toggle-subtask': [subtaskId: string]
}>()

// Stores
const areaStore = useAreaStore()
const uiStore = useUIStore()
const contextMenu = useContextMenu()

// 时间选择器状态
const showTimePicker = ref(false)

// 通过 area_id 从 store 获取完整 area 信息
const area = computed(() => {
  return props.task.area_id ? areaStore.getAreaById(props.task.area_id) : null
})

// 获取今日日期 (YYYY-MM-DD)
const todayDate = computed(() => getTodayDateString())

// 获取今日的时间块（按开始时间排序）
const todayTimeBlocks = computed(() => {
  if (!props.task.schedules) return []

  const today = todayDate.value
  const schedule = props.task.schedules.find((s) => s.scheduled_day === today)

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
function toggleComplete() {
  emit('toggle-complete')
}

function toggleSubtask(subtaskId: string) {
  emit('toggle-subtask', subtaskId)
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
  uiStore.openEditor(props.task.id)
}

// 右键菜单
function showContextMenu(event: MouseEvent) {
  event.preventDefault()
  contextMenu.show(KanbanTaskCardMenu, { task: props.task }, event)
}
</script>

<style scoped>
.task-strip {
  background-color: var(--color-background-content);
  border: none;
  border-bottom: 2px dashed rgb(0 0 0 / 15%);
  border-radius: 0;
  padding: 1.2rem 1.6rem;
  margin-bottom: 0;
  transition: all 0.2s ease;
  cursor: pointer;
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
  align-items: flex-start;
  gap: 1rem;
  margin-bottom: 0.8rem;
}

.complete-btn {
  flex-shrink: 0;
  width: 2rem;
  height: 2rem;
  border: 2px solid var(--color-border-default);
  border-radius: 0.4rem;
  background-color: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  color: transparent;
}

.complete-btn:hover {
  border-color: var(--color-primary, #4a90e2);
  background-color: var(--color-primary-bg, #e3f2fd);
}

.complete-btn.completed {
  border-color: var(--color-primary, #4a90e2);
  background-color: var(--color-primary, #4a90e2);
  color: white;
}

.task-title {
  flex: 1;
  font-size: 1.5rem;
  font-weight: 500;
  color: var(--color-text-primary);
  line-height: 1.4;
  overflow-wrap: break-word;
  margin-top: 0.1rem;
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
  padding: 0.3rem 0.6rem;
  font-size: 1.2rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  background-color: transparent;
  border: 1px solid var(--color-border-default);
  border-radius: 0.3rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.estimated-duration:hover {
  border-color: var(--color-primary, #4a90e2);
  background-color: var(--color-primary-bg, #e3f2fd);
}

.time-picker-popup {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 0.4rem;
  z-index: 100;
}

/* 时间块显示 */
.time-blocks-section {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.8rem;
  padding-left: 3rem;
  flex-wrap: wrap;
}

.time-tag {
  display: inline-block;
  padding: 0.18rem 0.54rem;
  font-size: 1.08rem;
  font-weight: 500;
  color: #fff;
  white-space: nowrap;
  border-radius: 0.36rem;
  text-shadow: 0 1px 2px rgb(0 0 0 / 20%);
  box-shadow: 0 1px 3px rgb(0 0 0 / 15%);
  line-height: 1.4;
}

.time-tag-more {
  display: inline-block;
  padding: 0.18rem 0.54rem;
  font-size: 1.08rem;
  font-weight: 500;
  color: #666;
  white-space: nowrap;
  border-radius: 0.27rem;
  background-color: #f0f0f0;
  line-height: 1.4;
}

/* 概览笔记 */
.task-note {
  display: flex;
  align-items: flex-start;
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

.subtask-complete-btn {
  flex-shrink: 0;
  width: 1.6rem;
  height: 1.6rem;
  border: 2px solid var(--color-border-default);
  border-radius: 0.3rem;
  background-color: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  color: transparent;
}

.subtask-complete-btn:hover {
  border-color: var(--color-primary, #4a90e2);
  background-color: var(--color-primary-bg, #e3f2fd);
}

.subtask-complete-btn.completed {
  border-color: var(--color-primary, #4a90e2);
  background-color: var(--color-primary, #4a90e2);
  color: white;
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

/* 底部：Area 标签 */
.task-footer {
  display: flex;
  justify-content: flex-end;
  margin-top: 0.4rem;
}
</style>
