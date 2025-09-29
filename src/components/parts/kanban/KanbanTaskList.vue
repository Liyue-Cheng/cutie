<script setup lang="ts">
import { ref } from 'vue'
import type { Task } from '@/types/models'
import { useTaskStore } from '@/stores/task'
import CutePane from '@/components/alias/CutePane.vue'
import KanbanTaskCard from './KanbanTaskCard.vue'

defineProps<{
  title: string
  tasks: Task[]
}>()

const emit = defineEmits(['openEditor'])

const taskStore = useTaskStore()
const newTaskTitle = ref('')
const isCreatingTask = ref(false)

async function handleAddTask() {
  const title = newTaskTitle.value.trim()
  if (!title || isCreatingTask.value) return

  console.log(`[TaskList] User initiated task creation with title: "${title}"`)

  isCreatingTask.value = true
  // 立即清空输入框，提供即时反馈
  const originalTitle = newTaskTitle.value
  newTaskTitle.value = ''

  try {
    await taskStore.createTask({
      title,
      context: {
        context_type: 'MISC',
        context_id: 'floating',
      },
    })
    console.log(`[TaskList] Task creation successful.`)
  } catch (error) {
    console.error(`[TaskList] Task creation failed:`, error)
    // 如果创建失败，恢复输入框内容
    newTaskTitle.value = originalTitle
  } finally {
    isCreatingTask.value = false
  }
}
</script>

<template>
  <CutePane class="task-list-container">
    <div class="header">
      <h2>{{ title }}</h2>
      <!-- Optional progress bar -->
      <!-- <div class="progress-bar-placeholder"></div> -->
    </div>

    <div class="add-task-wrapper">
      <input
        v-model="newTaskTitle"
        type="text"
        placeholder="+ Add task"
        class="add-task-input"
        :disabled="isCreatingTask"
        @keydown.enter="handleAddTask"
      />
      <div v-if="isCreatingTask" class="creating-indicator">Creating...</div>
    </div>

    <div class="task-list-scroll-area">
      <KanbanTaskCard
        v-for="task in tasks"
        :key="task.id"
        :task="task"
        @open-editor="emit('openEditor', task)"
      />
    </div>
  </CutePane>
</template>

<style scoped>
.task-list-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--color-background-content);
  width: 21rem;
  flex-shrink: 0;
}

.header {
  padding: 1rem 1rem 0.5rem;
}

.header h2 {
  font-size: 2.2rem;
  font-weight: 600;
  margin: 0;
}

.progress-bar-placeholder {
  height: 4px;
  background-color: var(--color-border-default);
  border-radius: 2px;
  margin-top: 0.75rem;
}

.add-task-wrapper {
  padding: 0 1rem;
  margin-bottom: 1rem;
}

.add-task-input {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid var(--color-border-default);
  border-radius: 8px; /* Assuming a medium radius, as --radius-m is not defined */
  background-color: var(--color-card-available);
  color: var(--color-text-primary);
  font-size: 1.5rem;
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
  padding: 0 1rem;
}
</style>
