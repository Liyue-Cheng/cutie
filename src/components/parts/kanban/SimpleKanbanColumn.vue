<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import type { TaskCard } from '@/types/dtos'
import type { ViewMetadata } from '@/types/drag'
import { useViewStore } from '@/stores/view'
import {
  useCrossViewDrag,
  useDragTransfer,
  useSameViewDrag,
  useCrossViewDragTarget,
} from '@/composables/drag'
import CutePane from '@/components/alias/CutePane.vue'
import KanbanTaskCard from './KanbanTaskCard.vue'

const props = defineProps<{
  title: string
  subtitle?: string
  tasks: TaskCard[]
  showAddInput?: boolean
  viewKey?: string
  viewMetadata: ViewMetadata
}>()

const emit = defineEmits<{
  openEditor: [task: TaskCard]
  addTask: [title: string]
  reorderTasks: [newOrder: string[]]
  crossViewDrop: [taskId: string, targetViewId: string]
}>()

const viewStore = useViewStore()

// ==================== Composables ====================

// 跨看板拖放（全局）
const crossViewDrag = useCrossViewDrag()
const dragTransfer = useDragTransfer()

// 同看板拖放
const sameViewDrag = useSameViewDrag(() => props.tasks)

// 跨看板拖放目标
const crossViewTarget = useCrossViewDragTarget(props.viewMetadata)

// ==================== 任务创建 ====================

const newTaskTitle = ref('')
const isCreatingTask = ref(false)

async function handleAddTask() {
  const title = newTaskTitle.value.trim()
  if (!title || isCreatingTask.value) return

  isCreatingTask.value = true
  const originalTitle = newTaskTitle.value
  newTaskTitle.value = ''

  try {
    emit('addTask', title)
  } catch (error) {
    console.error('[SimpleKanbanColumn] Task creation failed:', error)
    newTaskTitle.value = originalTitle
  } finally {
    isCreatingTask.value = false
  }
}

// ==================== 任务完成后重新排序 ====================

function handleTaskCompleted(completedTaskId: string) {
  // 找到已完成任务的当前索引
  const currentIndex = props.tasks.findIndex((t) => t.id === completedTaskId)
  if (currentIndex === -1) return

  // 找到最后一个未完成任务的索引
  let lastIncompleteIndex = -1
  for (let i = props.tasks.length - 1; i >= 0; i--) {
    const task = props.tasks[i]
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
  const newOrder = [...props.tasks.map((t) => t.id)]
  // 移除已完成的任务
  newOrder.splice(currentIndex, 1)

  // 计算插入位置（移除元素后索引会变化）
  // 如果被完成的任务原本在最后一个未完成任务之前，移除后 lastIncompleteIndex 需要减 1
  const insertPosition =
    currentIndex < lastIncompleteIndex ? lastIncompleteIndex : lastIncompleteIndex + 1

  // 插入到最后一个未完成任务的后面
  newOrder.splice(insertPosition, 0, completedTaskId)

  // 触发重新排序
  emit('reorderTasks', newOrder)
}

// ==================== 排序配置管理 ====================

const sortingConfigLoaded = ref(false)
const previousTaskIds = ref<Set<string>>(new Set())

onMounted(async () => {
  if (props.viewKey) {
    const alreadyLoaded = viewStore.sortWeights.has(props.viewKey)
    if (!alreadyLoaded) {
      await viewStore.fetchViewPreference(props.viewKey)
    }
    sortingConfigLoaded.value = true
  } else {
    sortingConfigLoaded.value = true
  }
})

// 自动检测任务列表变化并持久化
watch(
  () => props.tasks,
  (newTasks) => {
    if (!sortingConfigLoaded.value || !props.viewKey || sameViewDrag.isDragging.value) {
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

      viewStore.updateSorting(props.viewKey, currentOrder).catch((error) => {
        console.error(`[SimpleKanbanColumn] Failed to auto-persist for "${props.viewKey}":`, error)
      })
    } else {
      previousTaskIds.value = currentTaskIds
    }
  },
  { deep: false, immediate: true }
)

// ==================== 显示任务列表 ====================

const displayTasks = computed(() => {
  let taskList = [...props.tasks]

  // 1. 如果是源看板，且任务正在被拖到其他看板，隐藏幽灵元素
  const context = crossViewDrag.currentContext.value
  const targetView = crossViewDrag.targetViewId.value

  if (context && context.sourceView.id === props.viewMetadata.id) {
    if (targetView && targetView !== props.viewMetadata.id) {
      taskList = taskList.filter((t) => t.id !== context.task.id)
    }
  }

  // 2. 如果正在接收跨看板拖放，添加幽灵元素
  taskList = crossViewTarget.getTasksWithGhost(taskList)

  // 3. 同看板内重排序预览
  // 仅当未发生跨看板（或目标仍为本列）时才返回同列预览
  const isCrossViewActive = !!context && !!targetView && targetView !== props.viewMetadata.id
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
  crossViewDrag.startNormalDrag(task, props.viewMetadata)

  // 设置拖拽数据
  dragTransfer.setDragData(event, {
    type: 'task',
    task,
    sourceView: props.viewMetadata,
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
    console.log('[SimpleKanbanColumn] dragend: Drop in progress, delaying cleanup')
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
    console.log('[SimpleKanbanColumn] dragend: Drop rejected, clearing context')
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
  if (context && context.sourceView.id !== props.viewMetadata.id) {
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
  if (!context || context.sourceView.id !== props.viewMetadata.id) return
  if (!sameViewDrag.isDragging.value) return

  // 检查是否真的离开了容器
  const container = event.currentTarget as HTMLElement
  const rect = container.getBoundingClientRect()
  const x = event.clientX
  const y = event.clientY
  const reallyLeft = x < rect.left || x > rect.right || y < rect.top || y > rect.bottom

  if (reallyLeft) {
    console.log('[SimpleKanbanColumn] 🚪 Drag left column, resetting order')
    sameViewDrag.resetDragOverIndex()
  }
}

/**
 * 放置
 */
async function handleDrop(event: DragEvent) {
  event.preventDefault()

  // 1. 尝试跨看板拖放
  // 预先记录当前预览的插入索引（目标 composable 在 handleDrop 内会清理状态）
  const plannedInsertIndex =
    crossViewTarget.targetIndex.value !== null
      ? (crossViewTarget.targetIndex.value as number)
      : props.tasks.length

  const crossViewResult = await crossViewTarget.handleDrop(event)

  if (crossViewResult.isHandled) {
    if (crossViewResult.success) {
      emit('crossViewDrop', crossViewResult.taskId!, props.viewMetadata.id)

      // 固化跨列插入位置到 ViewStore，避免回到底部
      if (props.viewKey && crossViewResult.taskId) {
        const incomingId = crossViewResult.taskId
        // 基于当前列任务构建排序，移除可能已存在的该任务ID
        const baseOrder = props.tasks.map((t) => t.id).filter((id) => id !== incomingId)
        const safeIndex = Math.max(0, Math.min(plannedInsertIndex, baseOrder.length))
        baseOrder.splice(safeIndex, 0, incomingId)
        viewStore
          .updateSorting(props.viewKey, baseOrder)
          .catch((err) => console.error('[SimpleKanbanColumn] Failed to persist cross-view sort:', err))
      }
    } else {
      console.error('❌ Cross-view drop failed:', crossViewResult.error)
    }
    sameViewDrag.cancelDrag()
    return
  }

  // 2. 同看板拖放
  const finalOrder = sameViewDrag.finishDrag()
  if (finalOrder) {
    emit('reorderTasks', finalOrder)
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
        <span class="count">{{ tasks.length }}</span>
      </div>
    </div>

    <div v-if="showAddInput" class="add-task-wrapper">
      <input
        v-model="newTaskTitle"
        type="text"
        placeholder="+ 添加任务"
        class="add-task-input"
        :disabled="isCreatingTask"
        @keydown.enter="handleAddTask"
      />
      <div v-if="isCreatingTask" class="creating-indicator">创建中...</div>
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
          class="kanban-task-card"
          @open-editor="emit('openEditor', task)"
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

.creating-indicator {
  font-size: 1.2rem;
  color: var(--color-text-secondary);
  padding: 0.5rem 0.75rem;
  font-style: italic;
}

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
