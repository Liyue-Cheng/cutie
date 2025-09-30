<script setup lang="ts">
import { onMounted, computed, ref } from 'vue'
import type { Task } from '@/types/models'
import DailyKanbanColumn from '@/components/parts/kanban/DailyKanbanColumn.vue'
import KanbanTaskEditorModal from '@/components/parts/kanban/KanbanTaskEditorModal.vue'
import CuteCalendar from '@/components/parts/CuteCalendar.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CuteButton from '@/components/parts/CuteButton.vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import { useTaskStore } from '@/stores/task'
import { useScheduleStore } from '@/stores/schedule'

const taskStore = useTaskStore()
const scheduleStore = useScheduleStore()
const isEditorOpen = ref(false)
const selectedTaskId = ref<string | null>(null)

// 创建今天、明天、后天的日期对象
const today = ref(new Date())
today.value.setHours(0, 0, 0, 0)

const tomorrow = computed(() => {
  const date = new Date(today.value)
  date.setDate(date.getDate() + 1)
  return date
})

const dayAfterTomorrow = computed(() => {
  const date = new Date(today.value)
  date.setDate(date.getDate() + 2)
  return date
})

// 获取每天的任务
const todayTasks = computed(() => {
  const dateStr = today.value.toISOString().split('T')[0]
  return getTasksForDate(dateStr)
})

const tomorrowTasks = computed(() => {
  const dateStr = tomorrow.value.toISOString().split('T')[0]
  return getTasksForDate(dateStr)
})

const dayAfterTomorrowTasks = computed(() => {
  const dateStr = dayAfterTomorrow.value.toISOString().split('T')[0]
  return getTasksForDate(dateStr)
})

// 根据日期从 schedules 和 tasks 中获取任务
function getTasksForDate(dateStr: string): Task[] {
  const schedulesForDate = Array.from(scheduleStore.schedules.values()).filter((schedule) => {
    const scheduleDate = new Date(schedule.scheduled_day).toISOString().split('T')[0]
    return scheduleDate === dateStr
  })

  const taskIds = schedulesForDate.map((s) => s.task_id)
  const tasks = Array.from(taskStore.tasks.values()).filter(
    (task) => taskIds.includes(task.id) && !task.is_deleted
  )

  return tasks
}

function handleOpenEditor(task: Task) {
  selectedTaskId.value = task.id
  isEditorOpen.value = true
}

onMounted(async () => {
  // 加载今天、明天、后天的任务数据
  try {
    const startDate = today.value.toISOString()
    const endDate = new Date(today.value)
    endDate.setDate(endDate.getDate() + 2)

    await scheduleStore.fetchSchedulesForRange(startDate, endDate.toISOString())
    console.log('[HomeView] Loaded schedules for 3 days')
  } catch (error) {
    console.error('[HomeView] Failed to fetch initial schedules:', error)
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
            <DailyKanbanColumn :date="today" :tasks="todayTasks" @open-editor="handleOpenEditor" />
            <DailyKanbanColumn
              :date="tomorrow"
              :tasks="tomorrowTasks"
              @open-editor="handleOpenEditor"
            />
            <DailyKanbanColumn
              :date="dayAfterTomorrow"
              :tasks="dayAfterTomorrowTasks"
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
