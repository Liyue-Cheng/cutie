<script setup lang="ts">
import { onMounted, computed, ref } from 'vue'
import type { TaskCard } from '@/types/dtos'
import SimpleKanbanColumn from '@/components/parts/kanban/SimpleKanbanColumn.vue'
import KanbanTaskEditorModal from '@/components/parts/kanban/KanbanTaskEditorModal.vue'
import CuteCalendar from '@/components/parts/CuteCalendar.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CuteButton from '@/components/parts/CuteButton.vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import { useTaskStore } from '@/stores/task'

const taskStore = useTaskStore()
const isEditorOpen = ref(false)
const selectedTaskId = ref<string | null>(null)

// 获取不同状态的任务
const allTasks = computed(() => {
  return taskStore.allTasks.filter((task) => !task.is_completed)
})

const stagingTasks = computed(() => {
  return taskStore.stagingTasks
})

const plannedTasks = computed(() => {
  return taskStore.scheduledTasks.filter((task) => !task.is_completed)
})

function handleOpenEditor(task: TaskCard) {
  selectedTaskId.value = task.id
  isEditorOpen.value = true
}

async function handleAddTask(title: string) {
  await taskStore.createTask({ title })
  console.log('[HomeView] Task created:', title)
}

onMounted(async () => {
  // 加载所有视图的任务数据
  try {
    await Promise.all([
      taskStore.fetchAllIncompleteTasks(), // All 列
      taskStore.fetchPlannedTasks(),       // Planned 列
      taskStore.fetchStagingTasks(),       // Staging 列
    ])
    console.log('[HomeView] Loaded all task views')
  } catch (error) {
    console.error('[HomeView] Failed to fetch tasks:', error)
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
            <SimpleKanbanColumn
              title="All"
              subtitle="所有未完成任务"
              :tasks="allTasks"
              @open-editor="handleOpenEditor"
            />
            <SimpleKanbanColumn
              title="Staging"
              subtitle="未排期"
              :tasks="stagingTasks"
              :show-add-input="true"
              @open-editor="handleOpenEditor"
              @add-task="handleAddTask"
            />
            <SimpleKanbanColumn
              title="Planned"
              subtitle="已排期"
              :tasks="plannedTasks"
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
