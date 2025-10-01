<script setup lang="ts">
import { ref, computed } from 'vue'
import type { TaskCard } from '@/types/dtos'
import CutePane from '@/components/alias/CutePane.vue'
import KanbanTaskCard from './KanbanTaskCard.vue'

const props = defineProps<{
  title: string
  subtitle?: string
  tasks: TaskCard[]
  showAddInput?: boolean
  viewKey?: string // 视图标识，用于保存排序
}>()

const emit = defineEmits<{
  openEditor: [task: TaskCard]
  addTask: [title: string]
  reorderTasks: [newOrder: string[]] // 新顺序的任务ID数组
}>()

const newTaskTitle = ref('')
const isCreatingTask = ref(false)

// 拖拽状态
const draggedTaskId = ref<string | null>(null)
const draggedOverIndex = ref<number | null>(null)

// ✅ 视觉预览：动态计算显示的任务顺序
const displayTasks = computed(() => {
  if (!draggedTaskId.value || draggedOverIndex.value === null) {
    return props.tasks
  }

  const draggedIndex = props.tasks.findIndex((t) => t.id === draggedTaskId.value)
  if (draggedIndex === -1 || draggedIndex === draggedOverIndex.value) {
    return props.tasks
  }

  // 实时重排（仅视觉）
  const newOrder = [...props.tasks]
  const [draggedTask] = newOrder.splice(draggedIndex, 1)
  if (draggedTask) {
    newOrder.splice(draggedOverIndex.value, 0, draggedTask)
  }
  return newOrder
})

async function handleAddTask() {
  const title = newTaskTitle.value.trim()
  if (!title || isCreatingTask.value) return

  isCreatingTask.value = true
  const originalTitle = newTaskTitle.value
  newTaskTitle.value = ''

  try {
    emit('addTask', title)
  } catch (error) {
    console.error(`[SimpleKanbanColumn] Task creation failed:`, error)
    newTaskTitle.value = originalTitle
  } finally {
    isCreatingTask.value = false
  }
}

// ==================== 拖拽排序逻辑 ====================

// 节流控制：防止过度频繁的DOM更新
let lastDragOverTime = 0
const DRAG_THROTTLE_MS = 50 // 50ms节流

/**
 * 拖动开始
 */
function handleDragStart(event: DragEvent, task: TaskCard) {
  if (!event.dataTransfer) return

  // 记录被拖动的任务
  draggedTaskId.value = task.id

  // 设置拖拽数据（供日历等其他组件使用）
  event.dataTransfer.setData(
    'application/json',
    JSON.stringify({
      type: 'task',
      task: task,
    })
  )
  event.dataTransfer.effectAllowed = 'copyMove'

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

  // 清理状态
  draggedTaskId.value = null
  draggedOverIndex.value = null
  lastDragOverTime = 0 // 重置节流时间戳
}

/**
 * 拖动经过其他卡片时（实时重排 + 优化）
 */
function handleDragOver(event: DragEvent, targetIndex: number) {
  event.preventDefault() // 必须调用，否则无法 drop

  // 只处理本列表内的拖拽
  if (!draggedTaskId.value) return

  // ✅ 节流：限制执行频率，减少闪烁
  const now = Date.now()
  if (now - lastDragOverTime < DRAG_THROTTLE_MS) {
    return
  }
  lastDragOverTime = now

  const draggedIndex = props.tasks.findIndex((t) => t.id === draggedTaskId.value)
  if (draggedIndex === -1) return // 被拖动的任务不在本列表

  // 避免不必要的重排：只阻止拖到自己
  if (draggedIndex === targetIndex) {
    return
  }

  draggedOverIndex.value = targetIndex

  // ⚠️ 注意：这里不调用后端！
  // dragover 只是视觉预览，真正的持久化在 drop 时进行
}

/**
 * 放置（持久化排序）
 */
function handleDrop(event: DragEvent) {
  event.preventDefault()

  if (!draggedTaskId.value) return

  // ✅ 使用 displayTasks（包含最新的拖拽结果）
  const finalOrder = displayTasks.value.map((t) => t.id)

  console.log('[SimpleKanbanColumn] Drop完成，最终顺序:', finalOrder)

  // ✅ 持久化到后端（通过父组件）
  emit('reorderTasks', finalOrder)

  // 清理状态
  draggedTaskId.value = null
  draggedOverIndex.value = null
}
</script>

<template>
  <CutePane class="simple-kanban-column">
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

    <div class="task-list-scroll-area" @drop="handleDrop" @dragover.prevent>
      <div
        v-for="(task, index) in displayTasks"
        :key="task.id"
        class="task-card-wrapper"
        :data-task-id="task.id"
        :data-dragging="draggedTaskId === task.id"
        draggable="true"
        @dragstart="handleDragStart($event, task)"
        @dragend="handleDragEnd"
        @dragover="handleDragOver($event, index)"
      >
        <KanbanTaskCard
          :task="task"
          class="kanban-task-card"
          @open-editor="emit('openEditor', task)"
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
  width: 21rem;
  flex-shrink: 0;
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

  /* scrollbar-gutter: stable; */
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
