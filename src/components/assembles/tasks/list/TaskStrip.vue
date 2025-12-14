<template>
  <div
    class="task-strip"
    :class="{
      completed: task.is_completed,
      'hover-disabled': isGlobalDragActive && (task as any)._isPreview !== true,
      'is-preview': (task as any)._isPreview === true,
    }"
    @mousedown="onMouseDown"
    @click="handleClick"
    @contextmenu="showContextMenu"
  >
    <!-- 顶部：完成按钮 + 标题 + 元信息 -->
    <div class="task-header">
      <!-- 复选框 + 标题（不可分割） -->
      <div class="task-main">
        <div class="main-checkbox-wrapper" :class="{ readonly: props.readOnly }">
          <CuteDualModeCheckbox
            class="main-checkbox"
            :state="checkboxState"
            size="large"
            :disable-long-press="props.readOnly"
            v-on="props.readOnly ? {} : { 'update:state': handleCheckboxStateChange }"
            @click.stop
          />
        </div>
        <div class="task-title" :class="{ completed: task.is_completed }">
          {{ task.title || '新任务' }}
        </div>

        <!-- 简单模式：信息指示器 -->
        <div v-if="displayMode === 'simple'" class="task-indicators">
          <CuteIcon
            v-if="task.glance_note"
            name="FileText"
            size="1.4rem"
            class="indicator-icon"
            title="有笔记"
          />
          <CuteIcon
            v-if="hasSubtasks"
            name="ListChecks"
            size="1.4rem"
            class="indicator-icon"
            title="有子任务"
          />
        </div>
      </div>

      <!-- 元信息区域（会自动换行） -->
      <div class="task-meta">
        <!-- 所属项目（优先）或 Area 标签 -->
        <!-- 在项目视图内部不显示项目标签 -->
        <span v-if="project && !isInProjectView" class="meta-tag project-tag">
          <CuteIcon
            name="FolderKanban"
            size="1.5rem"
            :color="area?.color || 'var(--color-text-tertiary)'"
          />
          <span class="meta-tag-text">{{ project.name }}</span>
        </span>
        <!-- 在项目视图中，只有当任务 Area 与项目 Area 不一致时才显示 -->
        <span v-else-if="area && shouldShowAreaTag" class="meta-tag area-tag">
          <CuteIcon name="Hash" size="1.5rem" :color="area.color" />
          <span class="meta-tag-text">{{ area.name }}</span>
        </span>

        <!-- 截止日期 -->
        <span
          v-if="task.due_date"
          class="meta-tag due-date-tag"
          :class="{ danger: task.due_date.is_overdue && task.due_date.type === 'HARD' }"
        >
          <CuteIcon
            v-if="task.due_date.type === 'HARD'"
            name="Flag"
            size="1.5rem"
            :color="
              task.due_date.is_overdue
                ? 'var(--color-deadline-overdue)'
                : 'var(--color-text-tertiary)'
            "
          />
          <span v-else class="soft-deadline-icon">~</span>
          <span class="meta-tag-text">{{ formatDueDate(task.due_date.date) }}</span>
        </span>

        <!-- 最近排期（仅在项目和区域视图显示） -->
        <span v-if="shouldShowNextSchedule && nextScheduleDate" class="meta-tag schedule-tag">
          <CuteIcon name="CalendarDays" size="1.5rem" />
          <span class="meta-tag-text">{{ formatScheduleDate(nextScheduleDate) }}</span>
        </span>

        <!-- Daily view：时间块显示（如果有） -->
        <span v-if="isInDailyView && todayTimeBlocks.length > 0" class="meta-tag time-block-tag">
          <CuteIcon name="Clock" size="1.5rem" />
          <span class="meta-tag-text">{{ timeBlocksDisplayText }}</span>
        </span>
      </div>
    </div>

    <!-- 概览笔记（仅完整模式显示） -->
    <div v-if="displayMode === 'full' && task.glance_note" class="task-row">
      <div class="row-head">
        <CuteIcon name="FileText" size="1.4rem" class="row-icon" />
      </div>
      <span class="note-text">{{ task.glance_note }}</span>
    </div>

    <!-- 子任务显示区（仅完整模式显示） -->
    <div v-if="displayMode === 'full' && hasSubtasks" class="subtasks-section">
      <div v-for="subtask in task.subtasks" :key="subtask.id" class="task-row">
        <div class="row-head">
          <CuteCheckbox
            :checked="subtask.is_completed"
            size="1.4rem"
            v-on="props.readOnly ? {} : { 'update:checked': () => toggleSubtask(subtask.id) }"
            @click.stop
          />
        </div>
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
import { useProjectStore } from '@/stores/project'
import { useUIStore } from '@/stores/ui'
import { useRegisterStore } from '@/stores/register'
import { useContextMenu } from '@/composables/useContextMenu'
import { useViewContext } from '@/composables/useViewContext'
import { getTodayDateString } from '@/infra/utils/dateUtils'
import { logger, LogTags } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import CuteDualModeCheckbox from '@/components/parts/CuteDualModeCheckbox.vue'
import KanbanTaskCardMenu from '@/components/assembles/tasks/kanban/KanbanTaskCardMenu.vue'

// Props
interface Props {
  task: TaskCard
  viewKey?: string
  displayMode?: 'simple' | 'full'
  readOnly?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  displayMode: 'full',
  readOnly: false,
})

// Emits
const emit = defineEmits<{
  'toggle-subtask': [subtaskId: string]
  completing: [taskId: string]
}>()

// Stores
const areaStore = useAreaStore()
const projectStore = useProjectStore()
const uiStore = useUIStore()
const registerStore = useRegisterStore()
const contextMenu = useContextMenu()

// ✅ 视图上下文：直接使用 viewKey prop（已经是正确的格式）
// 如果没有 viewKey，降级到路由获取
const { viewContext: routeViewContext } = useViewContext()
const viewContext = computed(() => {
  return props.viewKey || routeViewContext.value
})

// 防误触：拖动后抑制一次点击
const suppressClickOnce = ref(false)
let mouseDownAt: { x: number; y: number } | null = null
const CLICK_SUPPRESS_DISTANCE = 4 // px

// 乐观更新：正在完成中的状态
const isCompleting = ref(false)

// 全局拖拽进行中时关闭 hover 效果
const isGlobalDragActive = computed(() =>
  registerStore.hasRegister(registerStore.RegisterKeys.GLOBAL_DRAG_ACTIVE)
)

// 通过 area_id 从 store 获取完整 area 信息
const area = computed(() => {
  return props.task.area_id ? areaStore.getAreaById(props.task.area_id) : null
})

// 通过 project_id 从 store 获取完整 project 信息
const project = computed(() => {
  return props.task.project_id ? projectStore.getProjectById(props.task.project_id) : null
})

// 判断是否在即将到期列表中
const isInUpcomingList = computed(() => {
  return props.viewKey === 'misc::deadline' || props.viewKey?.startsWith('upcoming::')
})

// 是否有子任务
const hasSubtasks = computed(() => {
  return props.task.subtasks && props.task.subtasks.length > 0
})

// 判断是否在 daily view 中
const isInDailyView = computed(() => {
  return props.viewKey?.startsWith('daily::')
})

// 判断是否在项目视图中（viewKey 包含 project 就认为是项目视图）
const isInProjectView = computed(() => {
  return props.viewKey?.includes('project') ?? false
})

// 判断是否在区域视图中（viewKey 包含区域 ID 就认为是区域视图）
const isInAreaView = computed(() => {
  if (!props.viewKey) return false
  // area::{areaId} - 标准区域视图
  if (props.viewKey.startsWith('area::')) return true
  // misc::staging::{areaId}::... - staging 中指定区域的视图（排除特殊值）
  if (props.viewKey.startsWith('misc::staging::')) {
    const parts = props.viewKey.split('::')
    const thirdPart = parts[2]
    // 排除特殊值：no-area, no-project, recent-carryover, project
    if (thirdPart && !['no-area', 'no-project', 'recent-carryover', 'project'].includes(thirdPart)) {
      return true
    }
  }
  return false
})

// 是否应该显示最近排期（仅在项目和区域视图）
const shouldShowNextSchedule = computed(() => {
  return isInProjectView.value || isInAreaView.value
})

// 获取最近的未来排期（包括今天，不包括过去）
const nextScheduleDate = computed(() => {
  if (!props.task.schedules || props.task.schedules.length === 0) return null

  const today = getTodayDateString()

  // 筛选出今天及未来的排期，并排序
  const futureSchedules = props.task.schedules
    .filter(s => s.scheduled_day >= today)
    .sort((a, b) => a.scheduled_day.localeCompare(b.scheduled_day))

  return futureSchedules.length > 0 ? futureSchedules[0].scheduled_day : null
})

// 从 viewKey 提取当前项目视图的项目 ID
const currentViewProjectId = computed(() => {
  if (!props.viewKey?.startsWith('project::')) return null
  // viewKey 格式: project::{projectId} 或 project::{projectId}::section::...
  const parts = props.viewKey.split('::')
  return parts[1] || null
})

// 当前视图的项目
const currentViewProject = computed(() => {
  return currentViewProjectId.value ? projectStore.getProjectById(currentViewProjectId.value) : null
})

// 是否应该显示 Area 标签
// 区域视图：不显示
// 项目视图：只有当任务 Area 与项目 Area 不一致时才显示
// 其他视图：始终显示（如果没有项目标签）
const shouldShowAreaTag = computed(() => {
  // 在区域视图中，不显示区域标签
  if (isInAreaView.value) return false
  // 在项目视图中，检查任务 Area 是否与项目 Area 不一致
  if (isInProjectView.value) {
    return props.task.area_id !== currentViewProject.value?.area_id
  }
  return true
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

// 时间块显示文本：第一个时间 + 剩余数量
const timeBlocksDisplayText = computed(() => {
  if (todayTimeBlocks.value.length === 0) return ''

  const firstTime = formatTimeBlockStart(todayTimeBlocks.value[0])
  const remaining = todayTimeBlocks.value.length - 1

  return remaining > 0 ? `${firstTime} +${remaining}` : firstTime
})

// 计算双模式复选框的状态
type CheckboxState = null | 'completed' | 'present'
const checkboxState = computed<CheckboxState>(() => {
  // 优先级1：如果任务已完成，显示完成状态
  if (props.task.is_completed) {
    return 'completed'
  }

  // 优先级2：正在完成中（乐观更新），但任务实际未完成时才显示
  if (isCompleting.value && !props.task.is_completed) {
    return 'completed'
  }

  // 优先级3：检查当前日期的outcome是否为presence_logged
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
// ✅ due_date.date 现在是 YYYY-MM-DD 格式
function formatDueDate(dateString: string): string {
  const [year, month, day] = dateString.split('-')
  return `${month}/${day}`
}

// 格式化排期日期（显示友好名称：今天、明天、或 MM/DD）
function formatScheduleDate(dateString: string): string {
  const today = getTodayDateString()

  if (dateString === today) {
    return '今天'
  }

  // 计算明天
  const todayDate = new Date(today)
  todayDate.setDate(todayDate.getDate() + 1)
  const tomorrow = todayDate.toISOString().split('T')[0]

  if (dateString === tomorrow) {
    return '明天'
  }

  const [year, month, day] = dateString.split('-')
  return `${month}/${day}`
}

// Methods
function toggleSubtask(subtaskId: string) {
  emit('toggle-subtask', subtaskId)
}

// 处理双模式复选框状态变化
async function handleCheckboxStateChange(newState: CheckboxState) {
  if (newState === 'completed') {
    // 立即设置为完成中状态（乐观更新）
    isCompleting.value = true

    // 通知父组件，让它暂时保留此任务的可见性
    emit('completing', props.task.id)

    try {
      // 执行真实的完成操作
      await pipeline.dispatch('task.complete', {
        id: props.task.id,
        view_context: viewContext.value,
      })
    } finally {
      // 无论成功失败，都重置 isCompleting 状态
      isCompleting.value = false
    }
  } else if (newState === 'present') {
    // 标记在场 - 更新当前日期的schedule outcome（后端API使用大写）
    await pipeline.dispatch('schedule.update', {
      task_id: props.task.id,
      scheduled_day: currentDate.value,
      updates: { outcome: 'PRESENCE_LOGGED' },
    })
  } else {
    // 取消状态时，重置 isCompleting
    isCompleting.value = false

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
  background-color: transparent;
  border: none;
  border-radius: 0.8rem;
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
  background-color: var(--color-background-hover, #f0f);
}

.task-strip.is-preview,
.task-strip.is-preview:hover,
.task-strip.is-preview.hover-disabled,
.task-strip.is-preview.hover-disabled:hover {
  background-color: var(--color-background-hover, #f0f);
}

.task-strip.hover-disabled,
.task-strip.hover-disabled:hover {
  background-color: transparent;
}

.task-strip.completed {
  opacity: 0.7;
}

/* 顶部：完成按钮 + 标题 + 元信息 */
.task-header {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.6rem 1rem;
}

/* 复选框 + 标题：不可分割的整体 */
.task-main {
  display: flex;
  align-items: flex-start;
  gap: 1rem;
  flex: 1 1 auto;
  min-width: 0;
}

/* 元信息区域：空间不够时换行到第二行 */
.task-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.6rem;
  flex-shrink: 0;
}

/* 当有其他内容时，标题栏需要底部间距 */
.task-strip:has(.task-row) .task-header {
  margin-bottom: 0.8rem;
}

/* 主要完成复选框：与标题第一行中线对齐 */
.main-checkbox {
  flex-shrink: 0;
  /* 标题 line-height: 1.4, font-size: 1.5rem, 行高 = 2.1rem */
  /* 复选框高度约 2.1rem，与第一行中线对齐 */
  margin-top: 0.05rem;
}

.main-checkbox-wrapper.readonly {
  pointer-events: none;
  opacity: 0.9;
}

.task-title {
  flex: 1 1 auto;
  min-width: 0;
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

/* 通用子行结构：行头 + 内容 */
.task-row {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  margin-bottom: 0.6rem;
  padding-left: 0.4rem; /* 补偿与 task-header gap(1rem) 的差值，使文字对齐 */
}

.task-row:last-child {
  margin-bottom: 0;
}

/* 行头：固定宽度，内容居中，与主 checkbox 宽度一致 */
.row-head {
  flex-shrink: 0;
  width: 2.1rem; /* 与 CuteDualModeCheckbox large size 一致 */
  display: flex;
  align-items: center;
  justify-content: center;
}

.row-icon {
  color: var(--color-text-tertiary);
}

/* 概览笔记文本 */
.note-text {
  flex: 1;
  font-size: 1.4rem;
  color: var(--color-text-secondary);
  line-height: 1.4;
  white-space: pre-wrap;
  overflow-wrap: break-word;
}

/* 简单模式：信息指示器 */
.task-indicators {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  flex-shrink: 0;
}

.indicator-icon {
  color: var(--color-text-tertiary, #f0f);
  opacity: 0.7;
}

/* 子任务显示区 */
.subtasks-section {
  display: flex;
  flex-direction: column;
}

/* 子任务文本 */
.subtask-title {
  flex: 1;
  font-size: 1.4rem;
  color: var(--color-text-secondary);
  line-height: 1.4;
}

.subtask-title.completed {
  color: var(--color-text-tertiary);
  text-decoration: line-through;
}

/* 软截止日期波浪号图标 */
.soft-deadline-icon {
  font-size: 1.65rem;
  color: var(--color-text-tertiary, #f0f);
  font-weight: 400;
  line-height: 1;
}

/* ========== Meta Tags ========== */
.meta-tag {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 1.4rem;
  color: var(--color-text-tertiary, #f0f);
  line-height: 1.4;
}

.meta-tag-text {
  white-space: nowrap;
  font-variant-numeric: tabular-nums;
}

/* 项目标签 */
.project-tag {
  /* 可以后续添加特定样式 */
}

/* 区域标签 */
.area-tag {
  /* 可以后续添加特定样式 */
}

/* 截止日期标签 */
.due-date-tag {
  /* 可以后续添加特定样式 */
}

.due-date-tag.danger {
  color: var(--color-danger, #f0f);
}

/* 排期标签 */
.schedule-tag {
  /* 可以后续添加特定样式 */
}

/* 时间块标签 */
.time-block-tag {
  /* 可以后续添加特定样式 */
}
</style>
