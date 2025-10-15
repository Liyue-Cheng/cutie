<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'
import type { TaskCard } from '@/types/dtos'
import type { ViewMetadata } from '@/types/drag'
import { useViewStore } from '@/stores/view'
import {
  useCrossViewDrag,
  useDragTransfer,
  useSameViewDrag,
  useCrossViewDragTarget,
  useTemplateDrop,
} from '@/composables/drag'
import { useViewTasks } from '@/composables/useViewTasks'
import { deriveViewMetadata } from '@/services/viewAdapter'
import CutePane from '@/components/alias/CutePane.vue'
import KanbanTaskCard from './KanbanTaskCard.vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { commandBus } from '@/commandBus'

const props = defineProps<{
  title: string
  subtitle?: string
  showAddInput?: boolean
  viewKey: string // 🔥 必需：所有看板都必须提供 viewKey
  viewMetadata?: ViewMetadata // 可选：可自动推导
}>()

// 🗑️ 移除所有 emit 定义 - 所有操作都内部处理或通过 store

const viewStore = useViewStore()

// ==================== 数据源管理 ====================

// 🔥 统一数据模式：所有看板都通过 viewKey 获取数据
const { tasks: effectiveTasks } = useViewTasks(props.viewKey)

// ✅ 统一的 ViewMetadata：优先使用父传的，否则自动推导
const effectiveViewMetadata = computed<ViewMetadata>(() => {
  if (props.viewMetadata) {
    return props.viewMetadata
  }

  const derived = deriveViewMetadata(props.viewKey)
  if (derived) {
    return derived
  }

  // 兜底：提供最小可用元数据
  return {
    id: props.viewKey,
    type: 'custom', // 使用 ViewType 中的有效值
    label: props.title,
    config: {}, // 提供空配置对象
  } as ViewMetadata
})

// ==================== Composables ====================

// 跨看板拖放（全局）
const crossViewDrag = useCrossViewDrag()
const dragTransfer = useDragTransfer()

// 同看板拖放
const sameViewDrag = useSameViewDrag(() => effectiveTasks.value)

// 跨看板拖放目标
// 注意：这里使用初始值，如果 viewMetadata 在运行时变化，可能需要重新考虑
const initialViewMetadata = effectiveViewMetadata.value
const crossViewTarget = useCrossViewDragTarget(initialViewMetadata)

// 模板拖放处理
const templateDrop = useTemplateDrop()

// ==================== 任务创建 ====================

const newTaskTitle = ref('')
const isCreatingTask = ref(false)
const addTaskInputRef = ref<HTMLInputElement | null>(null)

async function handleAddTask() {
  const title = newTaskTitle.value.trim()
  if (!title || isCreatingTask.value) return

  isCreatingTask.value = true
  const originalTitle = newTaskTitle.value
  newTaskTitle.value = ''

  try {
    // 检查是否是日期视图（daily::YYYY-MM-DD）
    const viewMetadata = effectiveViewMetadata.value
    const isDateView = viewMetadata.type === 'date'

    if (isDateView) {
      // 日期视图：使用合并端点一次性创建任务并添加日程
      const dateConfig = viewMetadata.config as import('@/types/drag').DateViewConfig
      const date = dateConfig.date // YYYY-MM-DD

      await commandBus.emit('task.create_with_schedule', {
        title,
        scheduled_day: date,
      })

      logger.info(LogTags.COMPONENT_KANBAN_COLUMN, 'Task created with schedule', {
        title,
        date,
        viewKey: props.viewKey,
      })
    } else {
      // 非日期视图：只创建任务
      await commandBus.emit('task.create', {
        title,
      })
      logger.info(LogTags.COMPONENT_KANBAN_COLUMN, 'Task created', {
        title,
        viewKey: props.viewKey,
      })
    }
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_KANBAN_COLUMN,
      'Task creation failed',
      error instanceof Error ? error : new Error(String(error)),
      { title, viewKey: props.viewKey }
    )
    newTaskTitle.value = originalTitle
  } finally {
    isCreatingTask.value = false
    // 重新聚焦到输入框，方便连续添加任务
    nextTick(() => {
      if (addTaskInputRef.value) {
        addTaskInputRef.value.focus()
      }
    })
  }
}

// ==================== 任务完成后重新排序 ====================

function handleTaskCompleted(completedTaskId: string) {
  // ✅ 使用 effectiveTasks 替代 props.tasks
  const tasks = effectiveTasks.value

  // 找到已完成任务的当前索引
  const currentIndex = tasks.findIndex((t) => t.id === completedTaskId)
  if (currentIndex === -1) return

  // 找到最后一个未完成任务的索引
  let lastIncompleteIndex = -1
  for (let i = tasks.length - 1; i >= 0; i--) {
    const task = tasks[i]
    if (task && !task.is_completed && task.id !== completedTaskId) {
      lastIncompleteIndex = i
      break
    }
  }

  // 如果没有其他未完成的任务，或者已完成任务已经在正确位置，则不需要移动
  if (lastIncompleteIndex === -1 || currentIndex === lastIncompleteIndex + 1) {
    return
  }

  // 创建新的任务顺序
  const newOrder = [...tasks.map((t) => t.id)]
  // 移除已完成的任务
  newOrder.splice(currentIndex, 1)

  // 计算插入位置（移除元素后索引会变化）
  // 如果被完成的任务原本在最后一个未完成任务之前，移除后 lastIncompleteIndex 需要减 1
  const insertPosition =
    currentIndex < lastIncompleteIndex ? lastIncompleteIndex : lastIncompleteIndex + 1

  // 插入到最后一个未完成任务的后面
  newOrder.splice(insertPosition, 0, completedTaskId)

  // 🔥 使用 Command Bus 更新排序（乐观更新）
  const originalOrder = viewStore.getSortedTaskIds(props.viewKey, effectiveTasks.value)
  commandBus
    .emit('view.update_sorting', {
      view_key: props.viewKey,
      sorted_task_ids: newOrder,
      original_sorted_task_ids: originalOrder, // 用于失败回滚
    })
    .catch((error) => {
      logger.error(
        LogTags.COMPONENT_KANBAN_COLUMN,
        'Failed to persist completed task reorder',
        error,
        { viewKey: props.viewKey }
      )
    })
}

// ==================== 排序配置管理 ====================

const previousTaskIds = ref<Set<string>>(new Set())

onMounted(async () => {
  // 🔥 简化：所有看板都有 viewKey，直接加载排序配置
  const alreadyLoaded = viewStore.sortWeights.has(props.viewKey)
  if (!alreadyLoaded) {
    await viewStore.fetchViewPreference(props.viewKey)
  }
  // ✅ 移除 sortingConfigLoaded 状态，避免闪烁
  // 🆕 注册 daily 视图
  const parts = props.viewKey.split('::')
  if (parts.length >= 2 && parts[0] === 'daily' && parts[1]) {
    viewStore.registerDailyView(parts[1])
  }
})

onBeforeUnmount(() => {
  const parts = props.viewKey.split('::')
  if (parts.length >= 2 && parts[0] === 'daily' && parts[1]) {
    viewStore.unregisterDailyView(parts[1])
  }
})

// ✅ 自动检测任务列表变化并持久化（使用 effectiveTasks）
watch(
  () => effectiveTasks.value,
  (newTasks) => {
    // ✅ 移除 sortingConfigLoaded 检查，避免闪烁
    if (sameViewDrag.isDragging.value) {
      previousTaskIds.value = new Set(newTasks.map((t) => t.id))
      return
    }

    const currentTaskIds = new Set(newTasks.map((t) => t.id))
    const hasChanges =
      currentTaskIds.size !== previousTaskIds.value.size ||
      !Array.from(currentTaskIds).every((id) => previousTaskIds.value.has(id))

    if (hasChanges) {
      previousTaskIds.value = currentTaskIds
      const currentOrder = newTasks.map((t) => t.id)

      // 🔥 使用 Command Bus 自动持久化排序（乐观更新）
      const originalOrder = viewStore.getSortedTaskIds(props.viewKey, effectiveTasks.value)
      commandBus
        .emit('view.update_sorting', {
          view_key: props.viewKey,
          sorted_task_ids: currentOrder,
          original_sorted_task_ids: originalOrder,
        })
        .catch((error) => {
          logger.error(
            LogTags.COMPONENT_KANBAN_COLUMN,
            'Failed to auto-persist view tasks',
            error,
            {
              viewKey: props.viewKey,
            }
          )
        })
    } else {
      previousTaskIds.value = currentTaskIds
    }
  },
  { deep: false, immediate: true }
)

// ==================== 显示任务列表 ====================

const displayTasks = computed(() => {
  // ✅ 使用 effectiveTasks 替代 props.tasks
  let taskList = [...effectiveTasks.value]

  // 1. 如果是源看板，且任务正在被拖到其他看板，隐藏幽灵元素
  const context = crossViewDrag.currentContext.value
  const targetView = crossViewDrag.targetViewId.value
  const viewMetadata = effectiveViewMetadata.value

  if (context && context.sourceView.id === viewMetadata.id) {
    if (targetView && targetView !== viewMetadata.id) {
      taskList = taskList.filter((t) => t.id !== context.task.id)
    }
  }

  // 2. 如果正在接收跨看板拖放，添加幽灵元素
  taskList = crossViewTarget.getTasksWithGhost(taskList)

  // 3. 同看板内重排序预览
  // 仅当未发生跨看板（或目标仍为本列）时才返回同列预览
  const isCrossViewActive = !!context && !!targetView && targetView !== viewMetadata.id
  if (
    sameViewDrag.isDragging.value &&
    !isCrossViewActive &&
    !crossViewTarget.isReceivingDrag.value
  ) {
    return sameViewDrag.reorderedTasks.value
  }

  return taskList
})

// ==================== 拖放事件处理 ====================

const taskListRef = ref<HTMLElement | null>(null)

/**
 * 拖动开始
 */
function handleDragStart(event: DragEvent, task: TaskCard) {
  if (!event.dataTransfer) return

  // 启动同看板拖放
  sameViewDrag.startDrag(task.id)

  // 启动跨看板拖放
  crossViewDrag.startNormalDrag(task, effectiveViewMetadata.value)

  // 设置拖拽数据
  dragTransfer.setDragData(event, {
    type: 'task',
    task,
    sourceView: effectiveViewMetadata.value,
    dragMode: { mode: 'normal' },
  })

  // 设置拖拽效果
  if (event.target instanceof HTMLElement) {
    event.target.style.opacity = '0.5'
  }
}

/**
 * 拖动结束
 */
function handleDragEnd(event: DragEvent) {
  // 恢复样式
  if (event.target instanceof HTMLElement) {
    event.target.style.opacity = '1'
  }

  // 检查是否有跨看板拖放正在执行
  const context = crossViewDrag.currentContext.value
  const isDropExecuting = crossViewDrag.isDropInProgress.value

  // 如果 drop 正在执行，延迟清理以避免闪烁
  if (isDropExecuting) {
    logger.debug(LogTags.COMPONENT_KANBAN_COLUMN, 'Dragend: Drop in progress, delaying cleanup', {
      viewKey: props.viewKey,
    })
    // drop 会在完成后自动清理上下文，这里只清理本地状态
    sameViewDrag.cancelDrag()
    crossViewTarget.clearReceivingState()
    return
  }

  // 清理同看板拖放状态
  sameViewDrag.cancelDrag()

  // 清理跨看板拖放状态
  crossViewTarget.clearReceivingState()

  // 如果 drop 被拒绝（dropEffect === 'none'），清理全局上下文
  if (context && event.dataTransfer?.dropEffect === 'none') {
    logger.debug(LogTags.COMPONENT_KANBAN_COLUMN, 'Dragend: Drop rejected, clearing context', {
      viewKey: props.viewKey,
    })
    crossViewDrag.cancelDrag()
  }

  crossViewDrag.setTargetViewId(null)
}

/**
 * 拖动经过卡片
 */
function handleDragOver(event: DragEvent, targetIndex: number) {
  event.preventDefault()

  // 跨看板拖放：交给 crossViewTarget 处理
  const context = crossViewDrag.currentContext.value
  if (context && context.sourceView.id !== effectiveViewMetadata.value.id) {
    return
  }

  // 同看板拖放
  sameViewDrag.dragOver(targetIndex)
}

/**
 * 容器级 dragover（用于跨看板拖放）
 */
function handleContainerDragOver(event: DragEvent) {
  if (!crossViewTarget.isReceivingDrag.value) return

  event.preventDefault()

  const container = taskListRef.value
  if (!container) return

  const wrappers = Array.from(container.querySelectorAll<HTMLElement>('.task-card-wrapper'))
  crossViewTarget.handleContainerDragOver(event, wrappers)
}

/**
 * 容器级 dragleave（用于同看板拖放的顺序恢复）
 */
function handleContainerDragLeave(event: DragEvent) {
  const context = crossViewDrag.currentContext.value

  // 只处理源看板的同看板拖放
  if (!context || context.sourceView.id !== effectiveViewMetadata.value.id) return
  if (!sameViewDrag.isDragging.value) return

  // 检查是否真的离开了容器
  const container = event.currentTarget as HTMLElement
  const rect = container.getBoundingClientRect()
  const x = event.clientX
  const y = event.clientY
  const reallyLeft = x < rect.left || x > rect.right || y < rect.top || y > rect.bottom

  if (reallyLeft) {
    logger.debug(LogTags.COMPONENT_KANBAN_COLUMN, 'Drag left column, resetting order', {
      viewKey: props.viewKey,
    })
    sameViewDrag.resetDragOverIndex()
  }
}

/**
 * 放置
 */
async function handleDrop(event: DragEvent) {
  event.preventDefault()

  // 0. 优先处理模板拖放
  const templateResult = await templateDrop.handleTemplateDrop(event, effectiveViewMetadata.value)
  if (templateResult.handled) {
    if (!templateResult.success) {
      logger.error(
        LogTags.COMPONENT_KANBAN_COLUMN,
        'Template drop failed',
        new Error(templateResult.error || 'Unknown error')
      )
      if (templateResult.error) {
        alert(templateResult.error)
      }
    }
    return // 模板拖放已处理，直接返回
  }

  // 1. 尝试跨看板拖放
  // 预先记录当前预览的插入索引（目标 composable 在 handleDrop 内会清理状态）
  const plannedInsertIndex =
    crossViewTarget.targetIndex.value !== null
      ? (crossViewTarget.targetIndex.value as number)
      : effectiveTasks.value.length

  const crossViewResult = await crossViewTarget.handleDrop(event)

  if (crossViewResult.isHandled) {
    if (crossViewResult.success) {
      // 🔥 跨视图拖放成功（不再发出事件）
      logger.info(LogTags.COMPONENT_KANBAN_COLUMN, 'Cross-view drop successful', {
        taskId: crossViewResult.taskId,
        viewKey: props.viewKey,
      })

      // 固化跨列插入位置到 ViewStore，避免回到底部
      if (props.viewKey && crossViewResult.taskId) {
        const incomingId = crossViewResult.taskId
        // ✅ 基于当前列任务构建排序，移除可能已存在的该任务ID
        const baseOrder = effectiveTasks.value.map((t) => t.id).filter((id) => id !== incomingId)
        const safeIndex = Math.max(0, Math.min(plannedInsertIndex, baseOrder.length))
        baseOrder.splice(safeIndex, 0, incomingId)

        // 🔥 使用 Command Bus 更新排序（乐观更新）
        const originalOrder = viewStore.getSortedTaskIds(props.viewKey, effectiveTasks.value)
        commandBus
          .emit('view.update_sorting', {
            view_key: props.viewKey,
            sorted_task_ids: baseOrder,
            original_sorted_task_ids: originalOrder,
          })
          .catch((err) =>
            logger.error(
              LogTags.COMPONENT_KANBAN_COLUMN,
              'Failed to persist cross-view sort',
              err,
              { viewKey: props.viewKey }
            )
          )
      }
    } else {
      logger.error(
        LogTags.COMPONENT_KANBAN_COLUMN,
        'Cross-view drop failed',
        crossViewResult.error
          ? new Error(crossViewResult.error)
          : new Error('Unknown cross-view drop error'),
        { viewKey: props.viewKey }
      )
    }
    sameViewDrag.cancelDrag()
    return
  }

  // 2. 同看板拖放
  const finalOrder = sameViewDrag.finishDrag()
  if (finalOrder) {
    // 🔥 使用 Command Bus 更新排序（乐观更新）
    const originalOrder = viewStore.getSortedTaskIds(props.viewKey, effectiveTasks.value)
    commandBus
      .emit('view.update_sorting', {
        view_key: props.viewKey,
        sorted_task_ids: finalOrder,
        original_sorted_task_ids: originalOrder,
      })
      .catch((error) => {
        logger.error(
          LogTags.COMPONENT_KANBAN_COLUMN,
          'Failed to persist same-view reorder',
          error,
          {
            viewKey: props.viewKey,
          }
        )
      })
  }
}
</script>

<template>
  <CutePane
    class="simple-kanban-column"
    @dragenter="crossViewTarget.handleEnter"
    @dragleave="
      (e: DragEvent) => {
        crossViewTarget.handleLeave(e)
        handleContainerDragLeave(e)
      }
    "
    @drop="handleDrop"
    @dragover.prevent
  >
    <div class="header">
      <div class="title-section">
        <h2 class="title">{{ title }}</h2>
        <p v-if="subtitle" class="subtitle">{{ subtitle }}</p>
      </div>
      <div class="task-count">
        <span class="count">{{ effectiveTasks.length }}</span>
      </div>
    </div>

    <div v-if="showAddInput" class="add-task-wrapper">
      <input
        ref="addTaskInputRef"
        v-model="newTaskTitle"
        type="text"
        placeholder="+ 添加任务"
        class="add-task-input"
        :disabled="isCreatingTask"
        @keydown.enter="handleAddTask"
      />
      <!-- ✅ 移除"创建中..."提示，避免闪烁 -->
    </div>

    <div ref="taskListRef" class="task-list-scroll-area" @dragover="handleContainerDragOver">
      <div
        v-for="(task, index) in displayTasks"
        :key="task.id"
        class="task-card-wrapper"
        :data-task-id="task.id"
        :data-dragging="sameViewDrag.draggedTaskId.value === task.id"
        draggable="true"
        @dragstart="handleDragStart($event, task)"
        @dragend="handleDragEnd"
        @dragover="handleDragOver($event, index)"
      >
        <KanbanTaskCard
          :task="task"
          :view-metadata="effectiveViewMetadata"
          class="kanban-task-card"
          @task-completed="handleTaskCompleted"
        />
      </div>

      <div v-if="displayTasks.length === 0" class="empty-state">暂无任务</div>
    </div>
  </CutePane>
</template>

<style scoped>
.simple-kanban-column {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--color-background-content);
  width: 23rem;
  flex-shrink: 0;
  padding-left: 0.5rem;
  padding-right: 0.5rem;
}

.header {
  padding: 1rem 1rem 0.5rem;
  border-bottom: 1px solid var(--color-border-default);
}

.title-section {
  margin-bottom: 0.5rem;
}

.title {
  font-size: 2.2rem;
  font-weight: 600;
  margin: 0;
  color: var(--color-text-primary);
}

.subtitle {
  font-size: 1.2rem;
  color: var(--color-text-secondary);
  margin: 0.25rem 0 0;
}

.task-count {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 1.4rem;
  font-weight: 500;
}

.task-count .count {
  color: var(--color-text-secondary);
}

.add-task-wrapper {
  padding: 1rem 1rem 0.5rem;
}

.add-task-input {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid var(--color-border-default);
  border-radius: 8px;
  background-color: var(--color-card-available);
  color: var(--color-text-primary);
  font-size: 1.5rem;
  transition: all 0.2s ease;
}

.add-task-input:focus {
  outline: none;
  border-color: var(--color-primary, #4a90e2);
  box-shadow: 0 0 0 3px rgb(74 144 226 / 10%);
}

.add-task-input::placeholder {
  color: var(--color-text-secondary);
}

.add-task-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* ✅ 移除 .creating-indicator 样式 */

.task-list-scroll-area {
  flex-grow: 1;
  overflow-y: auto;
  padding: 0.5rem 1rem 1rem;
  min-height: 100px;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 10rem;
  color: var(--color-text-tertiary);
  font-size: 1.4rem;
}

/* 滚动条样式 */
.task-list-scroll-area::-webkit-scrollbar {
  width: 6px;
}

.task-list-scroll-area::-webkit-scrollbar-track {
  background: transparent;
}

.task-list-scroll-area::-webkit-scrollbar-thumb {
  background: var(--color-border-default);
  border-radius: 3px;
}

.task-list-scroll-area::-webkit-scrollbar-thumb:hover {
  background: var(--color-text-tertiary);
}

/* 拖拽相关样式 */
.task-card-wrapper {
  position: relative;
  cursor: grab;
  transition: transform 0.2s ease;
}

.task-card-wrapper:active {
  cursor: grabbing;
}

.task-card-wrapper[data-dragging='true'] {
  opacity: 0.5;
}

.kanban-task-card {
  cursor: grab;
  pointer-events: auto;
}

.kanban-task-card:active {
  cursor: grabbing;
}
</style>
