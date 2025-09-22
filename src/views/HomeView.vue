<script setup lang="ts">
import { onMounted, computed, ref } from 'vue'
import type { Task } from '@/types/models'
import KanbanTaskList from '@/components/business/KanbanTaskList.vue'
import KanbanTaskEditorModal from '@/components/business/KanbanTaskEditorModal.vue'
import CuteCalendar from '@/components/ui/CuteCalendar.vue'
import { useTaskStore } from '@/stores/task'

const taskStore = useTaskStore()
const isEditorOpen = ref(false)
const selectedTaskId = ref<string | null>(null)

function handleOpenEditor(task: Task) {
  selectedTaskId.value = task.id
  isEditorOpen.value = true
}

// For now, we use all tasks for both lists as placeholder data.
// In the future, this would be filtered based on task properties.
const inboxTasks = computed(() => taskStore.allTasks)
const todayTasks = computed(() => taskStore.allTasks.filter((t) => t.status !== 'done'))

onMounted(() => {
  // Fetch tasks when the component mounts
  taskStore.fetchTasks()
})
</script>

<template>
  <div class="home-view-container">
    <div class="left-pane">
      <div class="task-view-pane">
        <KanbanTaskList title="Todo" :tasks="todayTasks" @open-editor="handleOpenEditor" />
        <KanbanTaskList title="In Progress" :tasks="inboxTasks" @open-editor="handleOpenEditor" />
      </div>
    </div>
    <div class="right-pane">
      <CuteCalendar />
    </div>
    <KanbanTaskEditorModal
      v-if="isEditorOpen"
      :task-id="selectedTaskId"
      @close="isEditorOpen = false"
    />
  </div>
</template>

<style scoped>
.home-view-container {
  display: flex;
  height: 100vh;
  width: 100%;
  background-color: var(--color-background-content);
}

.left-pane,
.right-pane {
  flex: 1;
  min-width: 0; /* Prevents flexbox overflow */
  padding: 1rem;
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
  box-shadow: 0 4px 12px rgb(0 0 0 / 5%);
}

.task-view-pane {
  display: flex;
  gap: 1rem;
  height: 100%;
}
</style>
