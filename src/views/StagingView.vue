<template>
  <div class="staging-view">
    <div class="header">
      <h1>Staging Area</h1>
      <p>未安排的任务将在这里显示</p>
    </div>

    <div class="content">
      <div v-if="taskStore.isLoading" class="loading">
        <p>加载中...</p>
      </div>

      <div v-else-if="taskStore.error" class="error">
        <p>错误: {{ taskStore.error }}</p>
        <button @click="taskStore.fetchStagingTasks()">重试</button>
      </div>

      <div v-else class="task-container">
        <KanbanTaskList
          title="未安排任务"
          :tasks="taskStore.stagingTasks"
          @open-editor="handleOpenEditor"
        />
      </div>
    </div>

    <KanbanTaskEditorModal
      v-if="isEditorOpen"
      :task-id="selectedTaskId"
      @close="isEditorOpen = false"
    />
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import type { TaskCard } from '@/types/dtos'
import { useTaskStore } from '@/stores/task'
import KanbanTaskList from '@/components/parts/kanban/KanbanTaskList.vue'
import KanbanTaskEditorModal from '@/components/parts/kanban/KanbanTaskEditorModal.vue'

const taskStore = useTaskStore()
const isEditorOpen = ref(false)
const selectedTaskId = ref<string | null>(null)

function handleOpenEditor(task: TaskCard) {
  selectedTaskId.value = task.id
  isEditorOpen.value = true
}

onMounted(async () => {
  // 加载未安排的任务（Staging）
  await taskStore.fetchStagingTasks()
})
</script>

<style scoped>
.staging-view {
  padding: 2rem;
  height: 100%;
  background-color: var(--color-background-content);
  display: flex;
  flex-direction: column;
}

.header {
  margin-bottom: 2rem;
}

.header h1 {
  color: var(--color-text-primary);
  margin-bottom: 0.5rem;
  font-size: 2.4rem;
  font-weight: 600;
}

.header p {
  color: var(--color-text-secondary);
  font-size: 1.4rem;
}

.content {
  flex-grow: 1;
  display: flex;
  flex-direction: column;
}

.loading,
.error {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex-grow: 1;
  gap: 1rem;
}

.loading p,
.error p {
  color: var(--color-text-secondary);
  font-size: 1.6rem;
}

.error button {
  padding: 0.8rem 1.6rem;
  background-color: var(--color-button-primary);
  color: white;
  border: none;
  border-radius: 0.6rem;
  cursor: pointer;
  font-size: 1.4rem;
}

.error button:hover {
  background-color: var(--color-button-primary-hover);
}

.task-container {
  flex-grow: 1;
  display: flex;
  justify-content: center;
}
</style>
