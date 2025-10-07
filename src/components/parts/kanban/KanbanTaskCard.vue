<script setup lang="ts">
import { computed, ref } from 'vue'
import type { TaskCard } from '@/types/dtos'
import type { ViewMetadata, DateViewConfig } from '@/types/drag'
import { useTaskStore } from '@/stores/task'
import { useAreaStore } from '@/stores/area'
import { useTaskOperations } from '@/composables/useTaskOperations'
import { useContextMenu } from '@/composables/useContextMenu'
import KanbanTaskCardMenu from './KanbanTaskCardMenu.vue'
import CuteCard from '@/components/templates/CuteCard.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import AreaTag from '@/components/parts/AreaTag.vue'
import TimeDurationPicker from '@/components/parts/TimeDurationPicker.vue'

const props = defineProps<{
  task: TaskCard
  viewMetadata?: ViewMetadata
}>()

const taskStore = useTaskStore()
const areaStore = useAreaStore()
const taskOps = useTaskOperations()
const emit = defineEmits<{
  openEditor: []
  taskCompleted: [taskId: string]
}>()

const contextMenu = useContextMenu()

// ✅ 防误触状态：刚点击过在场按钮
const justToggledPresence = ref(false)

// ✅ 时间选择器状态
const showTimePicker = ref(false)

// 使用任务的subtasks字段替代checkpoints
const subtasks = computed(() => props.task.subtasks || [])

// ✅ 通过 area_id 从 store 获取完整 area 信息
const area = computed(() => {
  return props.task.area_id ? areaStore.getAreaById(props.task.area_id) : null
})

// ✅ 判断是否为日期看板（daily::*）
const isDateKanban = computed(() => {
  return props.viewMetadata?.type === 'date'
})

// ✅ 获取当日日期 (YYYY-MM-DD) - 使用本地时区
const todayDate = computed(() => {
  const today = new Date()
  const year = today.getFullYear()
  const month = (today.getMonth() + 1).toString().padStart(2, '0')
  const day = today.getDate().toString().padStart(2, '0')
  return `${year}-${month}-${day}`
})

// ✅ 判断当前看板的日期类型
const kanbanDateType = computed(() => {
  if (!isDateKanban.value || !props.viewMetadata?.config) return null

  const config = props.viewMetadata.config as DateViewConfig
  const kanbanDate = config.date
  const today = todayDate.value

  if (!today) return null

  if (kanbanDate === today) return 'today'
  if (kanbanDate < today) return 'past'
  if (kanbanDate > today) return 'future'
  return null
})

// ✅ 判断该日期之后是否有排期记录
const hasScheduleAfterDate = computed(() => {
  if (!isDateKanban.value || !props.viewMetadata?.config) return false
  if (!props.task.schedules) return false

  const config = props.viewMetadata.config as DateViewConfig
  const kanbanDate = config.date

  // 检查是否有任何排期在当前看板日期之后
  return props.task.schedules.some((schedule) => schedule.scheduled_day > kanbanDate)
})

// ✅ 完成按钮显示逻辑
const shouldShowCompleteButton = computed(() => {
  if (!isDateKanban.value) {
    // 非日期看板（如暂存区）：始终显示完成按钮
    return true
  }

  const dateType = kanbanDateType.value
  if (dateType === 'today' || dateType === 'future') {
    // 今日看板或未来看板：显示
    return true
  }

  if (dateType === 'past') {
    // 过去看板：条件显示 - 只有该天之后无其他排期时才显示
    return !hasScheduleAfterDate.value
  }

  return false
})

// ✅ 在场按钮显示逻辑
const shouldShowPresenceButton = computed(() => {
  if (!isDateKanban.value) {
    // 非日期看板：不显示在场按钮
    return false
  }

  // 已完成任务：不显示在场按钮
  if (props.task.is_completed) {
    return false
  }

  const dateType = kanbanDateType.value
  if (dateType === 'future') {
    // 未来看板：不显示在场按钮
    return false
  }

  if (dateType === 'today' || dateType === 'past') {
    // 今日看板或过去看板：显示（任务未完成时）
    return true
  }

  return false
})

// ✅ 获取当前日期的 schedule outcome
const currentScheduleOutcome = computed(() => {
  if (!isDateKanban.value || !props.viewMetadata?.config) return null
  if (!props.task.schedules) return null

  const config = props.viewMetadata.config as DateViewConfig
  const kanbanDate = config.date

  const schedule = props.task.schedules.find((s) => s.scheduled_day === kanbanDate)
  return schedule?.outcome || null
})

// ✅ 在场按钮的选中状态
const isPresenceLogged = computed(() => {
  return currentScheduleOutcome.value === 'presence_logged'
})

// ✅ 按钮布局模式
// 默认模式：完成按钮始终显示，在场按钮悬浮显示
// 在场激活模式：在场按钮始终显示，完成按钮悬浮显示
const buttonLayoutMode = computed(() => {
  // 如果在场按钮被选中，切换到"在场激活模式"
  if (isPresenceLogged.value && shouldShowPresenceButton.value) {
    return 'presence-active'
  }
  return 'default'
})

function showContextMenu(event: MouseEvent) {
  contextMenu.show(KanbanTaskCardMenu, { task: props.task }, event)
}

async function handleStatusChange(isChecked: boolean) {
  if (isChecked) {
    // ✅ 完成任务
    await taskOps.completeTask(props.task.id)
    // 通知父组件任务已完成，以便重新排序
    emit('taskCompleted', props.task.id)
  } else {
    // ✅ 重新打开任务
    await taskOps.reopenTask(props.task.id)
  }
}

async function handlePresenceToggle(newCheckedValue: boolean) {
  if (!isDateKanban.value || !props.viewMetadata?.config) return

  const config = props.viewMetadata.config as DateViewConfig
  const kanbanDate = config.date

  try {
    // 根据新的选中状态设置 outcome
    // checked = true: 在场 (PRESENCE_LOGGED)
    // checked = false: 仅计划 (PLANNED)
    const newOutcome = newCheckedValue ? 'PRESENCE_LOGGED' : 'PLANNED'
    console.log(
      `[KanbanTaskCard] Toggle presence for task ${props.task.id} on ${kanbanDate}: ${newOutcome} (checked=${newCheckedValue})`
    )

    // ✅ 标记刚点击过在场按钮，防止完成按钮立即出现在同一位置
    justToggledPresence.value = true

    // 调用后端 API 更新 schedule 的 outcome
    await taskStore.updateSchedule(props.task.id, kanbanDate, { outcome: newOutcome })

    console.log('[KanbanTaskCard] Presence toggled successfully')
  } catch (error) {
    console.error('[KanbanTaskCard] Error toggling presence:', error)
  }
}

// ✅ 鼠标离开卡片时重置防误触状态
function handleMouseLeave() {
  justToggledPresence.value = false
}

// ✅ 获取今天的时间块（按开始时间排序）
const todayTimeBlocks = computed(() => {
  if (!props.task.schedules) return []

  const today = todayDate.value
  const todaySchedule = props.task.schedules.find((s) => s.scheduled_day === today)

  if (!todaySchedule || !todaySchedule.time_blocks) {
    return []
  }

  // 按开始时间排序
  return [...todaySchedule.time_blocks].sort((a, b) => {
    return a.start_time.localeCompare(b.start_time)
  })
})

// ✅ 判断今天是否有时间块
const hasTodayTimeBlocks = computed(() => {
  return todayTimeBlocks.value.length > 0
})

// ✅ 计算今天时间片的总时长（分钟）
const todayTimeBlocksTotalDuration = computed(() => {
  if (!hasTodayTimeBlocks.value) return 0

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

// ✅ 格式化时间块的开始时间（HH:mm）
function formatTimeBlockStart(isoString: string): string {
  const date = new Date(isoString)
  const hours = date.getHours().toString().padStart(2, '0')
  const minutes = date.getMinutes().toString().padStart(2, '0')
  return `${hours}:${minutes}`
}

// ✅ 格式化截止日期
function formatDueDate(isoString: string): string {
  const date = new Date(isoString)
  const month = (date.getMonth() + 1).toString().padStart(2, '0')
  const day = date.getDate().toString().padStart(2, '0')
  return `${month}/${day}`
}

// ✅ 格式化时间显示（根据是否有时间片显示不同内容）
const formattedDuration = computed(() => {
  // 如果有今天的时间片，显示时间片总和
  if (hasTodayTimeBlocks.value) {
    const minutes = todayTimeBlocksTotalDuration.value

    if (minutes === 0) return 'tiny'

    const hours = Math.floor(minutes / 60)
    const mins = minutes % 60

    if (hours > 0 && mins > 0) {
      return `${hours}:${mins.toString().padStart(2, '0')}`
    } else if (hours > 0) {
      return `${hours}:00`
    } else {
      return `${mins} min`
    }
  }

  // 没有时间片时，显示预期时间
  if (props.task.estimated_duration === null || props.task.estimated_duration === 0) {
    return 'tiny'
  }

  const minutes = props.task.estimated_duration
  const hours = Math.floor(minutes / 60)
  const mins = minutes % 60

  if (hours > 0 && mins > 0) {
    return `${hours}:${mins.toString().padStart(2, '0')}`
  } else if (hours > 0) {
    return `${hours}:00`
  } else {
    return `${mins} min`
  }
})

// ✅ 切换时间选择器显示
function toggleTimePicker(event: Event) {
  event.stopPropagation()
  showTimePicker.value = !showTimePicker.value
}

// ✅ 更新预期时间
async function updateEstimatedDuration(duration: number | null) {
  try {
    await taskStore.updateTask(props.task.id, {
      estimated_duration: duration,
    } as any)
    showTimePicker.value = false
  } catch (error) {
    console.error('[KanbanTaskCard] Error updating estimated duration:', error)
  }
}

async function handleSubtaskStatusChange(subtaskId: string, isCompleted: boolean) {
  // 更新subtask状态
  const updatedSubtasks = subtasks.value.map((subtask) =>
    subtask.id === subtaskId ? { ...subtask, is_completed: isCompleted } : subtask
  )

  // ✅ 更新任务的subtasks（仍然使用 taskStore，因为这是简单的更新操作）
  await taskStore.updateTask(props.task.id, {
    subtasks: updatedSubtasks,
  })
}
</script>

<template>
  <CuteCard
    class="task-card"
    :data-completed="task.is_completed"
    @click="emit('openEditor')"
    @contextmenu="showContextMenu"
    @mouseleave="handleMouseLeave"
  >
    <div class="main-content">
      <!-- 时间指示器栏（有时间块时显示） -->
      <div v-if="hasTodayTimeBlocks" class="time-indicator-bar">
        <div class="time-tags">
          <span
            v-for="block in todayTimeBlocks.slice(0, 2)"
            :key="block.id"
            class="time-tag"
            :style="{ backgroundColor: area?.color || '#ccc' }"
          >
            {{ formatTimeBlockStart(block.start_time) }}
          </span>
          <span v-if="todayTimeBlocks.length > 2" class="time-tag-more"
            >+{{ todayTimeBlocks.length - 2 }}</span
          >
        </div>

        <!-- 时间片总和显示（不可点击） -->
        <div class="estimated-duration-wrapper">
          <span class="estimated-duration readonly">
            {{ formattedDuration }}
          </span>
        </div>
      </div>

      <!-- 第一行：标题 + 预期时间（无时间块时显示） -->
      <div v-if="!hasTodayTimeBlocks" class="card-header">
        <span class="title">{{ task.title }}</span>

        <!-- 预期时间显示 -->
        <div class="estimated-duration-wrapper">
          <button class="estimated-duration" @click="toggleTimePicker">
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

      <!-- 标题行（有时间块时显示） -->
      <div v-if="hasTodayTimeBlocks" class="card-title-row">
        <span class="title">{{ task.title }}</span>
      </div>

      <div v-if="task.glance_note" class="notes-section">
        <CuteIcon name="CornerDownRight" :size="14" />
        <span class="note-text">{{ task.glance_note }}</span>
      </div>

      <!-- 截止时间显示 -->
      <div v-if="task.due_date" class="due-date-section">
        <CuteIcon
          name="Flag"
          :size="14"
          :color="task.due_date.type === 'HARD' ? '#f44336' : '#999'"
        />
        <span
          class="due-date-text"
          :class="{
            overdue: task.due_date.is_overdue,
            'hard-deadline': task.due_date.type === 'HARD',
          }"
        >
          {{ formatDueDate(task.due_date.date) }}
        </span>
      </div>

      <div v-if="subtasks.length > 0" class="subtasks-section">
        <div v-for="subtask in subtasks" :key="subtask.id" class="subtask-item">
          <CuteCheckbox
            :checked="subtask.is_completed"
            size="small"
            @update:checked="
              (isChecked: boolean) => handleSubtaskStatusChange(subtask.id, isChecked)
            "
            @click.stop
          />
          <span class="subtask-title">{{ subtask.title }}</span>
        </div>
      </div>

      <!-- 第二行：完成/在场按钮 + Area标签 -->
      <div class="card-footer">
        <div
          class="main-checkbox-wrapper"
          :class="{ 'presence-active-mode': buttonLayoutMode === 'presence-active' }"
        >
          <!-- 默认模式：完成按钮在左，在场按钮在右（悬浮） -->
          <template v-if="buttonLayoutMode === 'default'">
            <!-- 完成按钮：始终显示（左边） -->
            <CuteCheckbox
              v-if="shouldShowCompleteButton"
              class="main-checkbox always-visible"
              :checked="task.is_completed"
              size="large"
              @update:checked="handleStatusChange"
              @click.stop
            ></CuteCheckbox>

            <!-- 占位符：过去日期且后续有记录时保留空间 -->
            <div v-else-if="kanbanDateType === 'past'" class="main-checkbox-placeholder"></div>

            <!-- 在场按钮：悬浮显示（右边） -->
            <CuteCheckbox
              v-if="shouldShowPresenceButton"
              class="star-checkbox hover-visible"
              variant="star"
              size="large"
              :checked="isPresenceLogged"
              @update:checked="handlePresenceToggle"
              @click.stop
            ></CuteCheckbox>
          </template>

          <!-- 在场激活模式：在场按钮在左，完成按钮在右（悬浮） -->
          <template v-else-if="buttonLayoutMode === 'presence-active'">
            <!-- 在场按钮：始终显示（左边） -->
            <CuteCheckbox
              v-if="shouldShowPresenceButton"
              class="star-checkbox always-visible"
              variant="star"
              size="large"
              :checked="isPresenceLogged"
              @update:checked="handlePresenceToggle"
              @click.stop
            ></CuteCheckbox>

            <!-- 完成按钮：悬浮显示（右边），但刚点击在场按钮后暂时不显示 -->
            <CuteCheckbox
              v-if="shouldShowCompleteButton && !justToggledPresence"
              class="main-checkbox hover-visible"
              :checked="task.is_completed"
              size="large"
              @update:checked="handleStatusChange"
              @click.stop
            ></CuteCheckbox>
          </template>
        </div>

        <AreaTag v-if="area" :name="area.name" :color="area.color" size="normal" />
      </div>
    </div>
  </CuteCard>
</template>

<style scoped>
.task-card {
  display: flex;
  flex-direction: column;
  padding: 1rem;
  margin-bottom: 0.75rem;
  border: 1px solid var(--color-border-default);
  background-color: var(--color-card-available);
  border-radius: 0.4rem;
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
  cursor: pointer;
}

.task-card:hover {
  border-color: var(--color-border-hover);
  box-shadow: 0 4px 12px rgb(0 0 0 / 10%);
}

.main-content {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

/* 时间指示器栏 */
.time-indicator-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
}

.time-tags {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex: 1;
  overflow: hidden;
}

.time-tag {
  display: inline-block;
  padding: 0.2rem 0.6rem;
  font-size: 1.1rem;
  font-weight: 500;
  color: #fff;
  white-space: nowrap;
  border-radius: 0.8rem;
  text-shadow: 0 1px 2px rgb(0 0 0 / 20%);
  box-shadow: 0 1px 3px rgb(0 0 0 / 15%);
}

.time-tag-more {
  display: inline-block;
  padding: 0.2rem 0.6rem;
  font-size: 1.1rem;
  font-weight: 500;
  color: #666;
  white-space: nowrap;
  border-radius: 0.8rem;
  background-color: #f0f0f0;
}

/* 有时间块时的标题行 */
.card-title-row {
  display: flex;
  align-items: flex-start;
}

/* 第一行：标题 + 预期时间 */
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 1rem;
}

.title {
  flex: 1;
  font-size: 1.5rem;
  font-weight: 500;
  color: var(--color-text-primary);
  line-height: 1.4;
}

.notes-section {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--color-text-primary);
}

.note-text {
  font-size: 1.3rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 截止时间区 */
.due-date-section {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.due-date-text {
  font-size: 1.3rem;
  color: #999;
  font-weight: 500;
}

.due-date-text.hard-deadline {
  color: #f44336;
}

.due-date-text.overdue {
  font-weight: 600;
  text-decoration: underline;
}

.subtasks-section {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}

.subtask-item {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.subtask-title {
  font-size: 1.4rem;
  color: var(--color-text-primary);
}

/* 第二行：完成/在场按钮 + Area标签 */
.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
  margin-top: 0.5rem;
}

.estimated-duration-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.estimated-duration {
  padding: 0.4rem 0.8rem;
  background-color: var(--color-bg-secondary, #f5f5f5);
  border: none;
  border-radius: 0.4rem;
  font-size: 1.2rem;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.estimated-duration:hover {
  background-color: var(--color-bg-hover, #e0e0e0);
  color: var(--color-text-primary);
}

/* 时间指示器栏中的预期时间按钮 */
.time-indicator-bar .estimated-duration {
  background-color: #f5f5f5;
  color: #666;
  font-weight: 500;
}

.time-indicator-bar .estimated-duration:hover {
  background-color: #e8e8e8;
  color: #333;
}

.time-picker-popup {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 0.4rem;
  z-index: 1000;
}

.main-checkbox-wrapper {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  align-self: flex-start;
}

.main-checkbox-placeholder {
  width: 2.4rem;
  height: 2.4rem;

  /* 保留占位空间，避免布局跳动 */
}

/* ✅ 按钮显示模式 */

/* 始终显示的按钮 */
.always-visible {
  opacity: 1;
}

/* 悬浮时显示的按钮 */
.hover-visible {
  opacity: 0;
  transition: opacity 0.2s ease-in-out;
}

/* 卡片悬浮时，悬浮按钮变为可见 */
.task-card:hover .hover-visible {
  opacity: 1;
}

/* Area 标签位置调整 */
.card-footer :deep(.area-tag) {
  flex-shrink: 0;
}

/* 只有主复选框被选中时，主标题才划线 */
/* stylelint-disable-next-line selector-class-pattern */
.main-checkbox-wrapper:has(.n-checkbox--checked) ~ .title {
  text-decoration: line-through;
  color: var(--color-text-secondary);
}

/* 或者使用更直接的方式：检查 task.is_completed */
.task-card[data-completed='true'] .title {
  text-decoration: line-through;
  color: var(--color-text-secondary);
}

/* 子任务选中时，只划子任务的线 */
/* stylelint-disable-next-line selector-class-pattern */
.subtask-item:has(.n-checkbox--checked) .subtask-title {
  text-decoration: line-through;
  color: var(--color-text-secondary);
}
</style>
