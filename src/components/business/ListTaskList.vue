<script setup lang="ts">
import { ref } from 'vue'
import { v4 as uuidv4 } from 'uuid'
import type { Task } from '@/types/models'
import { useTaskStore } from '@/stores/task'
import CutePane from '@/components/ui/CutePane.vue'
import CuteCard from '@/components/ui/CuteCard.vue'
import ListTaskCard from './ListTaskCard.vue'

defineProps<{
  title: string
  tasks: Task[]
}>()

const taskStore = useTaskStore()
const newTaskTitle = ref('')

async function handleAddTask() {
  const title = newTaskTitle.value.trim()
  if (!title) return
  console.log(`[TaskList] User initiated task creation with title: "${title}"`)

  // Use a UUID for the sort key to ensure uniqueness.
  // Note: For chronological sorting, a time-based UUID (v1 or v7) might be
  // preferable in a real application, but v4 is sufficient for this example.
  const sortKey = uuidv4()

  await taskStore.createTask({
    title,
    sort_key: sortKey,
  })

  // Clear the input after creation
  if (!taskStore.error) {
    console.log(`[TaskList] Task creation successful, clearing input.`)
    newTaskTitle.value = ''
  } else {
    console.error(`[TaskList] Task creation failed. Error from store:`, taskStore.error)
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
        @keydown.enter="handleAddTask"
      />
    </div>

    <div class="task-list-scroll-area">
      <ListTaskCard v-for="task in tasks" :key="task.id" :task="task" />
    </div>
  </CutePane>
</template>

<style scoped>
.task-list-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--color-background-content);
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

.task-list-scroll-area {
  flex-grow: 1;
  overflow-y: auto;
  padding: 0 1rem;
}
</style>
