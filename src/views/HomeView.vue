<script setup lang="ts">
import { onMounted, computed, ref } from 'vue'
import type { Task } from '@/types/models'
import KanbanTaskList from '@/components/parts/kanban/KanbanTaskList.vue'
import KanbanTaskEditorModal from '@/components/parts/kanban/KanbanTaskEditorModal.vue'
import CuteCalendar from '@/components/parts/CuteCalendar.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CuteButton from '@/components/parts/CuteButton.vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import { useTaskStore } from '@/stores/task'

const taskStore = useTaskStore()
const isEditorOpen = ref(false)
const selectedTaskId = ref<string | null>(null)

function handleOpenEditor(task: Task) {
  selectedTaskId.value = task.id
  isEditorOpen.value = true
}

// Use unscheduled tasks for the staging area
const inboxTasks = computed(() => taskStore.unscheduledTasks)
const todayTasks = computed(() => taskStore.unscheduledTasks)

onMounted(async () => {
  // 预先加载任务数据，避免UI跳动
  try {
    await taskStore.fetchUnscheduledTasks()
  } catch (error) {
    console.error('[HomeView] Failed to fetch initial tasks:', error)
  }
})
</script>

<template>
  <div class="home-view-container">
    <div class="main-content-pane">
      <TwoRowLayout>
        <template #top>
          <CuteButton>Test Button 1</CuteButton>
        </template>
        <template #bottom>
          <div class="task-view-pane">
            <KanbanTaskList title="Todo" :tasks="todayTasks" @open-editor="handleOpenEditor" />
            <KanbanTaskList
              title="In Progress"
              :tasks="inboxTasks"
              @open-editor="handleOpenEditor"
            />
          </div>
        </template>
      </TwoRowLayout>
    </div>
    <div class="calendar-pane">
      <TwoRowLayout>
        <template #top>
          <CuteButton>Test Button 2</CuteButton>
        </template>
        <template #bottom>
          <CuteCalendar />
        </template>
      </TwoRowLayout>
    </div>
    <div class="toolbar-pane">
      <TwoRowLayout>
        <template #top>
          <CuteButton>Test</CuteButton>
        </template>
        <template #bottom>
          <div class="toolbar-icons">
            <CuteIcon name="Calendar" :size="28" />
            <CuteIcon name="Theater" :size="28" />
          </div>
        </template>
      </TwoRowLayout>
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
  height: 100%;
  width: 100%;
  background-color: var(--color-background-content);
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
}

.main-content-pane {
  flex: 1;
  min-width: 0;
  border-right: 1px solid var(--color-border-default);
  box-shadow: inset -4px 0 12px -2px rgb(0 0 0 / 5%);
  position: relative;
}

.calendar-pane {
  flex: 1;
  min-width: 0;
  border-right: 1px solid var(--color-border-default);
}

.toolbar-pane {
  width: 6rem; /* 96px */
  min-width: 6rem;
}

.toolbar-icons {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  align-items: center;
  padding-top: 1rem;
}

.task-view-pane {
  display: flex;
  gap: 1rem;
  height: 100%;
}

:deep(.top-row .cute-button) {
  background-color: #4a90e2; /* A nice blue */
  color: #fff; /* White text */
  border-color: transparent;
}

:deep(.top-row .cute-button:hover) {
  background-color: #357abd; /* A darker blue for hover */
}
</style>
