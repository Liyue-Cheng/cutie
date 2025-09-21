<script setup lang="ts">
import { onMounted, computed } from 'vue'
import TaskList from '@/components/business/TaskList.vue'
import CuteCalendar from '@/components/ui/CuteCalendar.vue'
import { useTaskStore } from '@/stores/task'

const taskStore = useTaskStore()

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
        <TaskList title="Inbox" :tasks="inboxTasks" />
      </div>
    </div>
    <div class="right-pane">
      <CuteCalendar />
    </div>
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
}

.task-view-pane {
  display: flex;
  gap: 1rem;
  height: 100%;
}
</style>
