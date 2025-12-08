<script setup lang="ts">
import { computed, ref } from 'vue'
import type { TaskCard } from '@/types/dtos'
import type { ViewMetadata, DateViewConfig } from '@/types/drag'
import { useTaskStore } from '@/stores/task'
import { useAreaStore } from '@/stores/area'
import { useUIStore } from '@/stores/ui'
import { useContextMenu } from '@/composables/useContextMenu'
import { logger, LogTags } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'
import KanbanTaskCardMenu from './KanbanTaskCardMenu.vue'
import CuteCard from '@/components/templates/CuteCard.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import CuteDualModeCheckbox from '@/components/parts/CuteDualModeCheckbox.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import AreaTag from '@/components/parts/AreaTag.vue'
import TimeDurationPicker from '@/components/parts/TimeDurationPicker.vue'

const props = defineProps<{
  task: TaskCard
  viewMetadata?: ViewMetadata
}>()

const taskStore = useTaskStore()
const areaStore = useAreaStore()
const uiStore = useUIStore()
const emit = defineEmits<{
  taskCompleted: [taskId: string]
}>()

const contextMenu = useContextMenu()

const currentViewKey = computed(() => {
  return props.viewMetadata?.id ?? ''
})

// ✅ 从 viewMetadata 生成 view_context（而不是从路由获取）
const viewContext = computed(() => {
  if (!props.viewMetadata) {
    return 'misc::staging' // 降级默认值
  }

  const metadata = props.viewMetadata

  // 根据 viewMetadata 的类型生成 view_context
  if (metadata.type === 'date') {
    // 日期看板：daily::2025-10-01
    const config = metadata.config as DateViewConfig
    return `daily::${config.date}`
  } else if (metadata.type === 'misc') {
    // 杂项看板：misc::staging, misc::incomplete 等
    return metadata.id
  } else if (metadata.type === 'area') {
    // 区域看板：area::{uuid}
    return metadata.id
  } else if (metadata.type === 'project') {
    // 项目看板：project::{uuid}
    return metadata.id
  }

  // 降级默认值
  return 'misc::staging'
})

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

import { getTodayDateString } from '@/infra/utils/dateUtils'
// ✅ 获取当日日期 (YYYY-MM-DD) - 使用本地时区
const todayDate = computed(() => getTodayDateString())

// ✅ 获取看板的日期（对于日期看板）或今天的日期（对于非日期看板）
const kanbanDate = computed(() => {
  if (isDateKanban.value && props.viewMetadata?.config) {
    const config = props.viewMetadata.config as DateViewConfig
    return config.date
  }
  // 对于非日期看板（如暂存区），使用今天的日期
  return todayDate.value
})

// ✅ 判断当前看板的日期类型
const kanbanDateType = computed(() => {
  if (!isDateKanban.value || !props.viewMetadata?.config) return null

  const config = props.viewMetadata.config as DateViewConfig
  const kanbanDateValue = config.date
  const today = todayDate.value

  if (!today) return null

  if (kanbanDateValue === today) return 'today'
  if (kanbanDateValue < today) return 'past'
  if (kanbanDateValue > today) return 'future'
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

// ✅ 主按钮状态（双模式 checkbox）
// 优先级：已完成 > 在场 > 未选中
const mainCheckboxState = computed<'completed' | 'present' | null>(() => {
  if (props.task.is_completed) {
    return 'completed'
  }
  if (isPresenceLogged.value && shouldShowPresenceButton.value) {
    return 'present'
  }
  return null
})

// ✅ 是否显示主按钮
const shouldShowMainCheckbox = computed(() => {
  return shouldShowCompleteButton.value || shouldShowPresenceButton.value
})

function showContextMenu(event: MouseEvent) {
  contextMenu.show(
    KanbanTaskCardMenu,
    {
      task: props.task,
      viewKey: currentViewKey.value,
    },
    event
  )
}

async function handleStatusChange(isChecked: boolean) {
  if (isChecked) {
    // ✅ 完成任务 - 传递视图上下文
    await pipeline.dispatch('task.complete', {
      id: props.task.id,
      view_context: viewContext.value,
    })
    // 通知父组件任务已完成，以便重新排序
    emit('taskCompleted', props.task.id)
  } else {
    // ✅ 重新打开任务 - 自动追踪！
    await pipeline.dispatch('task.reopen', { id: props.task.id })
  }
}

// ✅ 处理主按钮状态变化（双模式）
async function handleMainCheckboxChange(newState: 'completed' | 'present' | null) {
  logger.debug(LogTags.COMPONENT_KANBAN, 'Main checkbox state changed', {
    taskId: props.task.id,
    oldState: mainCheckboxState.value,
    newState,
  })

  // 完成状态变化
  if (newState === 'completed') {
    // 标记为完成
    await pipeline.dispatch('task.complete', {
      id: props.task.id,
      view_context: viewContext.value,
    })
    emit('taskCompleted', props.task.id)
  } else if (newState === 'present') {
    // 标记在场（长按）
    if (!isDateKanban.value || !props.viewMetadata?.config) return

    const config = props.viewMetadata.config as DateViewConfig
    const kanbanDate = config.date

    // 如果任务已完成，先重新打开
    if (props.task.is_completed) {
      await pipeline.dispatch('task.reopen', { id: props.task.id })
    }

    // 更新 schedule outcome 为在场
    await pipeline.dispatch('schedule.update', {
      task_id: props.task.id,
      scheduled_day: kanbanDate,
      updates: { outcome: 'PRESENCE_LOGGED' },
    })
  } else {
    // newState === null，取消选中
    const currentState = mainCheckboxState.value

    if (currentState === 'completed') {
      // 从完成状态恢复：重新打开任务
      await pipeline.dispatch('task.reopen', { id: props.task.id })
    } else if (currentState === 'present') {
      // 从在场状态恢复：设置为仅计划
      if (!isDateKanban.value || !props.viewMetadata?.config) return

      const config = props.viewMetadata.config as DateViewConfig
      const kanbanDate = config.date

      await pipeline.dispatch('schedule.update', {
        task_id: props.task.id,
        scheduled_day: kanbanDate,
        updates: { outcome: 'PLANNED' },
      })
    }
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
    logger.debug(LogTags.COMPONENT_KANBAN, 'Toggle presence for task', {
      taskId: props.task.id,
      kanbanDate,
      newOutcome,
      checked: newCheckedValue,
    })

    // ✅ 标记刚点击过在场按钮，防止完成按钮立即出现在同一位置
    justToggledPresence.value = true

    // 调用后端 API 更新 schedule 的 outcome - 自动追踪！
    await pipeline.dispatch('schedule.update', {
      task_id: props.task.id,
      scheduled_day: kanbanDate,
      updates: { outcome: newOutcome },
    })

    logger.debug(LogTags.COMPONENT_KANBAN, 'Presence toggled successfully')
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_KANBAN,
      'Error toggling presence',
      error instanceof Error ? error : new Error(String(error))
    )
  }
}

// ✅ 鼠标离开卡片时重置防误触状态
function handleMouseLeave() {
  justToggledPresence.value = false
}

// ✅ 获取看板日期的时间块（按开始时间排序）
const todayTimeBlocks = computed(() => {
  if (!props.task.schedules) return []

  const dateToShow = kanbanDate.value
  const schedule = props.task.schedules.find((s) => s.scheduled_day === dateToShow)

  if (!schedule || !schedule.time_blocks) {
    return []
  }

  // 按开始时间排序
  return [...schedule.time_blocks].sort((a, b) => {
    return a.start_time.localeCompare(b.start_time)
  })
})

// ✅ 判断看板日期是否有时间块
const hasTodayTimeBlocks = computed(() => {
  return todayTimeBlocks.value.length > 0
})

// ✅ 计算看板日期时间片的总时长（分钟）
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

// ✅ 格式化时间块的开始时间（HH:mm） - 支持浮动时间
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

// ✅ 格式化截止日期
// ✅ due_date.date 现在是 YYYY-MM-DD 格式
function formatDueDate(dateString: string): string {
  const [year, month, day] = dateString.split('-')
  return `${month}/${day}`
}

// ✅ 格式化时间显示（根据是否有时间片显示不同内容）
const formattedDuration = computed(() => {
  // 如果有看板日期的时间片，显示时间片总和
  if (hasTodayTimeBlocks.value) {
    const minutes = todayTimeBlocksTotalDuration.value

    if (minutes === 0) return 'tiny'

    const hours = Math.floor(minutes / 60)
    const mins = minutes % 60

    // ✅ 统一格式为 x:xx
    return `${hours}:${mins.toString().padStart(2, '0')}`
  }

  // 没有时间片时，显示预期时间
  if (props.task.estimated_duration === null || props.task.estimated_duration === 0) {
    return 'tiny'
  }

  const minutes = props.task.estimated_duration
  const hours = Math.floor(minutes / 60)
  const mins = minutes % 60

  // ✅ 统一格式为 x:xx
  return `${hours}:${mins.toString().padStart(2, '0')}`
})

// ✅ 切换时间选择器显示
function toggleTimePicker(event: Event) {
  event.stopPropagation()
  showTimePicker.value = !showTimePicker.value
}

// ✅ 更新预期时间
async function updateEstimatedDuration(duration: number | null) {
  try {
    await pipeline.dispatch('task.update', {
      id: props.task.id,
      updates: { estimated_duration: duration },
    })
    showTimePicker.value = false
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_KANBAN,
      'Error updating estimated duration',
      error instanceof Error ? error : new Error(String(error))
    )
  }
}

async function handleSubtaskStatusChange(subtaskId: string, isCompleted: boolean) {
  // 更新subtask状态
  const updatedSubtasks = subtasks.value.map((subtask) =>
    subtask.id === subtaskId ? { ...subtask, is_completed: isCompleted } : subtask
  )

  // ✅ 更新任务的subtasks（使用 CPU Pipeline）
  await pipeline.dispatch('task.update', {
    id: props.task.id,
    updates: { subtasks: updatedSubtasks },
  })
}
</script>

<template>
  <CuteCard
    class="task-card"
    :data-completed="task.is_completed"
    @click="uiStore.openEditor(task.id, viewMetadata?.id)"
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
            :style="{ backgroundColor: area?.color || 'var(--color-tag-background)' }"
          >
            {{ formatTimeBlockStart(block) }}
          </span>
          <span v-if="todayTimeBlocks.length > 2" class="time-tag-more"
            >+{{ todayTimeBlocks.length - 2 }}</span
          >
        </div>

        <!-- 时间片总和显示（不可点击） -->
        <div class="estimated-duration-wrapper">
          <button class="estimated-duration" disabled>
            {{ formattedDuration }}
          </button>
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
        <span class="icon-wrapper">
          <CuteIcon name="CornerDownRight" size="1.4rem" />
        </span>
        <span class="note-text">{{ task.glance_note }}</span>
      </div>

      <!-- 截止时间显示 -->
      <div v-if="task.due_date" class="due-date-section">
        <span class="icon-wrapper">
          <!-- 硬截止：使用旗子图标，过期为红色，未过期为灰色 -->
          <CuteIcon
            v-if="task.due_date.type === 'HARD'"
            name="Flag"
            size="1.4rem"
            :color="
              task.due_date.is_overdue
                ? 'var(--color-deadline-overdue)'
                : 'var(--color-text-tertiary)'
            "
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

      <div v-if="subtasks.length > 0" class="subtasks-section">
        <div v-for="subtask in subtasks" :key="subtask.id" class="subtask-item">
          <span class="checkbox-wrapper">
            <CuteCheckbox
              :checked="subtask.is_completed"
              size="1.4rem"
              @update:checked="
                (isChecked: boolean) => handleSubtaskStatusChange(subtask.id, isChecked)
              "
              @click.stop
            />
          </span>
          <span class="subtask-title">{{ subtask.title }}</span>
        </div>
      </div>

      <!-- 第二行：双模式按钮 + Area标签 -->
      <div class="card-footer">
        <div class="main-checkbox-wrapper">
          <!-- 双模式按钮：单击完成，长按在场 -->
          <CuteDualModeCheckbox
            v-if="shouldShowMainCheckbox"
            class="main-checkbox"
            :state="mainCheckboxState"
            size="large"
            @update:state="handleMainCheckboxChange"
            @click.stop
          />
          <!-- 占位符：过去日期且后续有记录时保留空间 -->
          <div
            v-else-if="kanbanDateType === 'past' && !shouldShowCompleteButton"
            class="main-checkbox-placeholder"
          ></div>
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
  transition: box-shadow 0.2s;
  cursor: pointer;
}

.task-card:hover {
  box-shadow: var(--shadow-md);
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
  padding: 0.18rem 0.54rem;
  font-size: 1.08rem;
  font-weight: 500;
  color: var(--color-text-on-accent);
  white-space: nowrap;
  border-radius: 0.36rem;
  text-shadow: var(--shadow-text, #f0f);
  box-shadow: var(--shadow-sm, #f0f);
  line-height: 1.4;
}

.time-tag-more {
  display: inline-block;
  padding: 0.18rem 0.54rem;
  font-size: 1.08rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  white-space: nowrap;
  border-radius: 0.27rem;
  background-color: var(--color-background-secondary);
  line-height: 1.4;
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
  align-items: center;
  gap: 1rem;
}

.title {
  flex: 1;
  font-size: 1.5rem;
  font-weight: 500;
  color: var(--color-text-primary);
  line-height: 1.4;
}

/* ✅ 统一的图标/选择框容器 - 确保文字对齐 */
.icon-wrapper,
.checkbox-wrapper {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.4rem;
  height: 1.4rem;
  flex-shrink: 0;
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
  line-height: 1.4;
}

/* 截止时间区 */
.due-date-section {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.soft-deadline-icon {
  font-size: 1.4rem;
  color: var(--color-text-tertiary);
  font-weight: 400;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.due-date-text {
  font-size: 1.3rem;
  color: var(--color-text-tertiary);
  font-weight: 500;
  line-height: 1.4;
}

/* 只有硬截止且逾期时才显示红色 */
.due-date-text.overdue.hard-deadline {
  color: var(--color-danger);
  font-weight: 600;
}

.subtasks-section {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}

.subtask-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.subtask-title {
  font-size: 1.4rem;
  color: var(--color-text-primary);
  line-height: 1.4;
}

/* 第二行：完成/在场按钮 + Area标签 */
.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
}

.estimated-duration-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.estimated-duration {
  padding: 0.18rem 0.54rem;
  background-color: var(--color-background-secondary);
  border: none;
  border-radius: 0.36rem;
  font-size: 1.08rem;
  font-weight: 500;
  color: var(--color-text-primary);
  cursor: pointer;
  transition: all 0.15s;
  line-height: 1.4;
}

/* 禁用状态：不可点击，光标不变 */
.estimated-duration:disabled {
  cursor: default;
  background-color: var(--color-background-secondary);
  color: var(--color-text-primary);
  opacity: 1;
}

/* 可点击状态：hover时变暗 */
.estimated-duration:not(:disabled):hover {
  background-color: var(--color-background-hover);
  color: var(--color-text-primary);
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
