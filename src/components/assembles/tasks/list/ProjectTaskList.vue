<template>
  <div class="task-bar" :class="taskBarClasses" ref="taskBarRef">
    <!-- 标题栏（可隐藏） -->
    <div
      v-if="!props.hideHeader"
      ref="headerRef"
      class="task-bar-header"
      :class="{ 'non-collapsible': !props.collapsible }"
      @click="props.collapsible ? toggleCollapse() : undefined"
    >
      <div class="header-left">
        <h3 class="task-bar-title" :style="titleStyle">{{ title }}</h3>
        <span class="task-count">{{ displayItems.length }}</span>
      </div>
      <div class="header-right">
        <!-- Section 操作按钮（仅当有 sectionId 时显示） -->
        <template v-if="sectionId">
          <span
            class="header-icon drag-icon"
            draggable="true"
            @dragstart="handleDragStart"
            @mousedown.stop
          >
            <CuteIcon name="GripVertical" :size="16" />
          </span>
          <span class="header-icon edit-icon" @click.stop="handleEditSection">
            <CuteIcon name="Pencil" :size="16" />
          </span>
        </template>
        <CuteIcon
          v-if="props.collapsible"
          name="ChevronDown"
          :size="16"
          class="header-icon collapse-icon"
          :class="{ rotated: isCollapsed }"
        />
      </div>
    </div>

    <!-- 节段描述（可折叠时跟随折叠） -->
    <div v-if="!isCollapsed && sectionDescription" class="section-description">
      {{ sectionDescription }}
    </div>

    <!-- 内容区（可折叠） -->
    <div v-if="!isCollapsed" class="task-bar-content">
      <!-- 任务输入框 -->
      <div v-if="showAddInput" class="task-input-wrapper" :class="{ focused: isInputFocused }">
        <input
          ref="taskInputRef"
          v-model="newTaskTitle"
          type="text"
          class="task-input"
          :placeholder="$t('task.action.addNewTask')"
          :disabled="isCreatingTask"
          @keydown.enter="addTask"
          @focus="isInputFocused = true"
          @blur="isInputFocused = false"
        />
        <button v-if="newTaskTitle && !isCreatingTask" class="add-task-btn" @click="addTask">
          <CuteIcon name="Plus" :size="16" />
        </button>
      </div>

      <!-- 任务纸条列表 -->
      <div ref="taskListRef" class="task-list-container">
        <TransitionGroup name="task-list" tag="div" class="task-list">
          <div
            v-for="task in displayItems"
            :key="task.id"
            :class="[
              'task-draggable',
              `task-draggable-${normalizedViewKey}`,
              {
                'is-preview': (task as any)._isPreview === true,
                'drag-compact': (task as any)._dragCompact === true,
                'fading-out': fadingTasks.has(task.id),
              },
            ]"
            :data-task-id="task.id"
          >
            <TaskStrip
              :task="task"
              :view-key="viewKey"
              display-mode="full"
              :show-estimated-duration="true"
              @toggle-complete="toggleTaskComplete(task.id)"
              @toggle-subtask="(subtaskId) => toggleSubtask(task.id, subtaskId)"
              @completing="onTaskCompleting"
            />
          </div>
          <div v-if="displayItems.length === 0" key="empty-state" class="empty-state">
            <p>{{ $t('task.label.noTasks') }}</p>
          </div>
        </TransitionGroup>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { ViewMetadata } from '@/types/drag'
import type { TaskCard } from '@/types/dtos'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import TaskStrip from './TaskStrip.vue'
import { useViewTasks } from '@/composables/useViewTasks'
import { useInteractDrag } from '@/composables/drag/useInteractDrag'
import { useDragStrategy } from '@/composables/drag/useDragStrategy'
import { dragPreviewState } from '@/infra/drag-interact'
import { deriveViewMetadata } from '@/services/viewAdapter'
import { pipeline } from '@/cpu'
import { logger, LogTags } from '@/infra/logging/logger'
import { useRecurrenceStore } from '@/stores/recurrence'

interface Props {
  title: string
  viewKey: string // 🔥 必需：遵循 VIEW_CONTEXT_KEY_SPEC 规范（project::${projectId}::section::${sectionId}）
  sectionId?: string // Section ID（有值时显示拖拽和编辑按钮）
  sectionDescription?: string // Section 描述
  defaultCollapsed?: boolean
  showAddInput?: boolean // 是否显示添加任务输入框
  collapsible?: boolean // 是否可折叠
  titleColor?: string // 标题颜色（CSS 颜色值或 CSS 变量）
  hideHeader?: boolean // 是否隐藏标题栏（用于外部自定义标题）
}

const props = withDefaults(defineProps<Props>(), {
  sectionId: undefined,
  sectionDescription: undefined,
  defaultCollapsed: false,
  showAddInput: true,
  collapsible: true,
  titleColor: '',
  hideHeader: false,
})

// Emits
const emit = defineEmits<{
  'add-task': [title: string]
  'edit-section': [sectionId: string]
  'drag-start': [event: DragEvent]
}>()

// 🔥 使用 useViewTasks 获取任务数据
const { tasks } = useViewTasks(props.viewKey)

// 获取循环规则 store
const recurrenceStore = useRecurrenceStore()

// 🔥 淡出任务缓存：用于在任务消失后仍能显示淡出动画
// 利用 sort_positions 来保持正确的排序位置
interface FadingTask {
  task: TaskCard // 任务快照（包含 sort_positions）
}
const fadingTasks = ref<Map<string, FadingTask>>(new Map())

// 过滤和排序任务
const filteredTasks = computed(() => {
  let result = [...tasks.value]

  // 1. 添加淡出任务（如果不在原始列表中）
  for (const [taskId, { task }] of fadingTasks.value) {
    if (!result.find((t) => t.id === taskId)) {
      result.push(task)
    }
  }

  // 2. 按 sort_positions 排序（利用现有排序系统保持位置）
  result.sort((a, b) => {
    const posA = a.sort_positions?.[props.viewKey] || ''
    const posB = b.sort_positions?.[props.viewKey] || ''
    return posA.localeCompare(posB)
  })

  // 3. 过滤已完成任务（但保留淡出中的任务）
  result = result.filter((task) => {
    if (!task.is_completed) return true
    if (fadingTasks.value.has(task.id)) return true
    return false
  })

  return result
})

// State
const isCollapsed = ref(props.defaultCollapsed)
const newTaskTitle = ref('')
const isCreatingTask = ref(false)
const taskBarRef = ref<HTMLElement | null>(null)
const taskListRef = ref<HTMLElement | null>(null)
const taskInputRef = ref<HTMLInputElement | null>(null)
const isInputFocused = ref(false)
const headerRef = ref<HTMLElement | null>(null)

// 暴露标题栏 ref 给父组件（用于 Section 拖拽）
defineExpose({
  headerRef,
  taskCount: computed(() => displayItems.value.length),
})

const taskBarClasses = computed(() => ({
  collapsed: isCollapsed.value,
}))

// 标题样式
const titleStyle = computed(() => {
  if (!props.titleColor) return {}
  return { color: props.titleColor }
})

// ==================== ViewMetadata 推导 ====================
const effectiveViewMetadata = computed<ViewMetadata>(() => {
  const derived = deriveViewMetadata(props.viewKey)
  if (derived) {
    return derived
  }

  // 兜底：提供最小可用元数据
  return {
    id: props.viewKey,
    type: 'custom',
    label: props.title,
    config: {},
  } as ViewMetadata
})

// ==================== 拖放系统集成 ====================
const dragStrategy = useDragStrategy()

// 标准化 viewKey 作为 CSS class（:: 替换为 --）
const normalizedViewKey = computed(() => props.viewKey.replace(/::/g, '--'))

const { displayItems } = useInteractDrag({
  viewMetadata: effectiveViewMetadata,
  items: filteredTasks,
  containerRef: taskBarRef,
  draggableSelector: `.task-draggable-${normalizedViewKey.value}`,
  objectType: 'task',
  getObjectId: (task) => task.id,
  onDrop: async (session) => {
    logger.debug(LogTags.COMPONENT_TASK_BAR, 'ProjectTaskList drop event', {
      session,
      targetViewKey: props.viewKey,
      displayItems: displayItems.value.length,
      dropIndex: dragPreviewState.value?.computed.dropIndex,
    })

    // 🎯 执行拖放策略
    const result = await dragStrategy.executeDrop(session, props.viewKey, {
      sourceContext: (session.metadata?.sourceContext as Record<string, any>) || {},
      targetContext: {
        taskIds: displayItems.value.map((t) => t.id),
        displayTasks: displayItems.value,
        dropIndex: dragPreviewState.value?.computed.dropIndex,
        viewKey: props.viewKey,
      },
    })

    if (!result.success) {
      const errorMessage = result.message || result.error || 'Unknown error'
      logger.error(
        LogTags.COMPONENT_TASK_BAR,
        'ProjectTaskList drop failed',
        new Error(errorMessage),
        {
          result,
          session,
        }
      )
    } else {
      logger.info(LogTags.COMPONENT_TASK_BAR, 'ProjectTaskList drop succeeded', {
        taskId: session.object.id,
        targetViewKey: props.viewKey,
      })
    }
  },
})

// Methods
function toggleCollapse() {
  isCollapsed.value = !isCollapsed.value
}

// Section 操作
function handleEditSection() {
  if (props.sectionId) {
    emit('edit-section', props.sectionId)
  }
}

function handleDragStart(event: DragEvent) {
  emit('drag-start', event)
}

// 🔥 处理任务完成事件：缓存任务快照并延迟消失
function onTaskCompleting(taskId: string) {
  // 找到任务
  const task = tasks.value.find((t) => t.id === taskId)

  if (task) {
    // 缓存任务快照（包含 sort_positions，用于保持排序位置）
    const newMap = new Map(fadingTasks.value)
    newMap.set(taskId, {
      task: { ...task, is_completed: true },
    })
    fadingTasks.value = newMap
  }

  // 延迟后从缓存中移除，任务会自然消失
  setTimeout(() => {
    const newMap = new Map(fadingTasks.value)
    newMap.delete(taskId)
    fadingTasks.value = newMap
  }, 800)
}

async function addTask() {
  const title = newTaskTitle.value.trim()
  if (!title || isCreatingTask.value) return

  isCreatingTask.value = true
  newTaskTitle.value = ''

  try {
    // 解析 viewKey: project::${projectId}::section::${sectionId|all} 或 misc::no-project
    const parts = props.viewKey.split('::')
    const [type, identifier, thirdPart] = parts

    const taskData: any = {
      title,
      estimated_duration: 60, // 默认 60 分钟
    }

    if (type === 'project' && identifier) {
      // project::${projectId}::section::${sectionId} - 指定章节的任务
      // project::${projectId}::section::all - 项目无分类任务
      // project::${projectId} - 指定项目的任务
      taskData.project_id = identifier

      if (thirdPart === 'section' && parts[3]) {
        const sectionId = parts[3]
        if (sectionId !== 'all') {
          taskData.section_id = sectionId
        }
      }
    }
    // misc::no-project 不需要设置 project_id

    logger.info(LogTags.COMPONENT_TASK_BAR, 'Creating project task', {
      title,
      viewKey: props.viewKey,
      taskData,
    })

    await pipeline.dispatch('task.create', taskData)
    emit('add-task', title)
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_TASK_BAR,
      'Failed to create project task',
      error instanceof Error ? error : new Error(String(error)),
      { title, viewKey: props.viewKey }
    )
  } finally {
    isCreatingTask.value = false
    // 重新聚焦到输入框
    setTimeout(() => {
      taskInputRef.value?.focus()
    }, 0)
  }
}

async function toggleTaskComplete(taskId: string) {
  try {
    const task = displayItems.value.find((t) => t.id === taskId)
    if (!task) return

    logger.info(LogTags.COMPONENT_TASK_BAR, 'Toggling task completion', {
      taskId,
      currentStatus: task.is_completed,
      viewKey: props.viewKey,
    })

    if (task.is_completed) {
      await pipeline.dispatch('task.reopen', { id: taskId })
    } else {
      await pipeline.dispatch('task.complete', { id: taskId })
    }

    logger.info(LogTags.COMPONENT_TASK_BAR, 'Task completion toggled', {
      taskId,
      newStatus: !task.is_completed,
    })
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_TASK_BAR,
      'Failed to toggle task completion',
      error instanceof Error ? error : new Error(String(error)),
      { taskId, viewKey: props.viewKey }
    )
  }
}

async function toggleSubtask(taskId: string, subtaskId: string) {
  try {
    const task = displayItems.value.find((t) => t.id === taskId)
    if (!task || !task.subtasks) return

    const subtask = task.subtasks.find((st) => st.id === subtaskId)
    if (!subtask) return

    logger.info(LogTags.COMPONENT_TASK_BAR, 'Toggling subtask completion', {
      taskId,
      subtaskId,
      currentStatus: subtask.is_completed,
      viewKey: props.viewKey,
    })

    const updatedSubtasks = task.subtasks.map((st) =>
      st.id === subtaskId ? { ...st, is_completed: !st.is_completed } : st
    )

    await pipeline.dispatch('task.update', {
      id: taskId,
      updates: { subtasks: updatedSubtasks },
    })

    logger.info(LogTags.COMPONENT_TASK_BAR, 'Subtask completion toggled', {
      taskId,
      subtaskId,
      newStatus: !subtask.is_completed,
    })
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_TASK_BAR,
      'Failed to toggle subtask completion',
      error instanceof Error ? error : new Error(String(error)),
      { taskId, subtaskId, viewKey: props.viewKey }
    )
  }
}
</script>

<style scoped>
.task-bar {
  background-color: transparent;
  margin-bottom: 0;
  padding-bottom: 1.6rem;
}

/* 标题栏 */
.task-bar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.6rem;
  cursor: pointer;
  user-select: none;
  transition: background-color 0.2s;
  border-radius: 0.6rem;
}

.task-bar-header:hover {
  background-color: var(--color-overlay-light);
}

/* 不可折叠的标题栏 */
.task-bar-header.non-collapsible {
  cursor: default;
}

.task-bar-header.non-collapsible:hover {
  background-color: transparent;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

/* 标题栏图标统一样式 */
.header-icon {
  color: var(--color-text-secondary, #f0f);
  transition: all 0.2s ease;
}

.header-icon:hover {
  color: var(--color-text-primary, #f0f);
}

/* 拖拽把手 */
.drag-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: grab;
  opacity: 0.5;
}

.drag-icon:hover {
  opacity: 1;
}

.drag-icon:active {
  cursor: grabbing;
}

/* 编辑按钮 */
.edit-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  opacity: 0.5;
}

.edit-icon:hover {
  opacity: 1;
}

/* 折叠箭头 */
.collapse-icon.rotated {
  transform: rotate(-90deg);
}

/* 节段描述 */
.section-description {
  padding: 0 1.6rem 0.8rem;
  font-size: 1.3rem;
  color: var(--color-text-secondary, #f0f);
  line-height: 1.5;
  white-space: pre-wrap;
}

.task-bar-title {
  font-size: 1.6rem;
  font-weight: 500;
  color: var(--color-text-primary);
  margin: 0;
  line-height: 1.4;
}

.task-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2rem;
  height: 2rem;
  padding: 0 0.6rem;
  font-size: 1.2rem;
  font-weight: 400;
  line-height: 1;
  color: var(--color-text-secondary);
  background-color: var(--color-background-secondary);
  border-radius: 1rem;
}

/* 内容区 */
.task-bar-content {
  padding: 0;
}

/* 任务输入框 */
.task-input-wrapper {
  position: relative;
  margin: 0 1.6rem 1rem;
  border-bottom: 2px dashed var(--color-input-underline, #f0f);
}

.task-input {
  width: 100%;
  padding: 0.8rem 0;
  padding-right: 3.4rem;
  font-size: 1.5rem;
  line-height: 1.4;
  color: var(--color-text-primary, #f0f);
  background-color: transparent;
  border: none;
  border-radius: 0;
  outline: none;
  transition: all 0.2s ease;
  box-sizing: border-box;
}

.task-input::placeholder {
  color: var(--color-text-tertiary, #f0f);
}

.task-input:focus {
  background-color: var(--color-background-input-focus, #f0f);
}

.task-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.add-task-btn {
  position: absolute;
  right: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3rem;
  height: 3rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: transparent;
  color: var(--color-text-accent, #f0f);
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.add-task-btn:hover {
  background-color: var(--color-background-accent-light, #f0f);
}

.add-task-btn:active {
  transform: translateY(-50%) scale(0.95);
}

/* 任务列表容器 */
.task-list-container {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

/* 任务列表 */
.task-list {
  display: flex;
  flex-direction: column;
  position: relative;
}

/* 任务列表动画 */
.task-list-move {
  transition: transform 0.15s cubic-bezier(0.4, 0, 0.2, 1);
  will-change: transform;
  backface-visibility: hidden;
  contain: paint;
}

.task-list-enter-active {
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
}

.task-list-leave-active {
  display: none;
}

.task-list-enter-from {
  opacity: 0;
  transform: translateY(-10px);
}

/* 空状态 */
.empty-state {
  padding: 0.8rem 1.6rem;
  text-align: center;
}

.empty-state p {
  font-size: 1.4rem;
  color: var(--color-text-tertiary);
  margin: 0;
  line-height: 2.35;
}

/* 折叠状态 */
.task-bar.collapsed .task-bar-content {
  display: none;
}
</style>
