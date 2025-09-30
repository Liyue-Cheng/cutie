<script setup lang="ts">
import { ref } from 'vue'
import type { TaskCard } from '@/types/dtos'
import CutePane from '@/components/alias/CutePane.vue'
import KanbanTaskCard from './KanbanTaskCard.vue'

defineProps<{
  title: string
  subtitle?: string
  tasks: TaskCard[]
  showAddInput?: boolean
}>()

const emit = defineEmits<{
  openEditor: [task: TaskCard]
  addTask: [title: string]
}>()

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
    console.error(`[SimpleKanbanColumn] Task creation failed:`, error)
    newTaskTitle.value = originalTitle
  } finally {
    isCreatingTask.value = false
  }
}

// 处理原生拖拽开始事件
function handleDragStart(event: DragEvent, task: TaskCard) {
  if (!event.dataTransfer) return

  event.dataTransfer.setData(
    'application/json',
    JSON.stringify({
      type: 'task',
      task: task,
    })
  )
  event.dataTransfer.effectAllowed = 'copyMove'
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

    <div class="task-list-scroll-area">
      <div
        v-for="task in tasks"
        :key="task.id"
        class="task-card-wrapper"
        :data-task-id="task.id"
        draggable="true"
        @dragstart="handleDragStart($event, task)"
      >
        <KanbanTaskCard
          :task="task"
          class="kanban-task-card"
          @open-editor="emit('openEditor', task)"
        />
      </div>

      <div v-if="tasks.length === 0" class="empty-state">暂无任务</div>
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
}

.task-card-wrapper:active {
  cursor: grabbing;
}

.kanban-task-card {
  cursor: grab;
}

.kanban-task-card:active {
  cursor: grabbing;
}
</style>
