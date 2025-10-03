<script setup lang="ts">
import { ref, computed } from 'vue'
import type { TaskCard } from '@/types/dtos'
import InfiniteDailyKanban from '@/components/templates/InfiniteDailyKanban.vue'
import KanbanTaskEditorModal from '@/components/parts/kanban/KanbanTaskEditorModal.vue'
import CuteCalendar from '@/components/parts/CuteCalendar.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CuteButton from '@/components/parts/CuteButton.vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import { useTaskStore } from '@/stores/task'

// ==================== Stores ====================
const taskStore = useTaskStore()

// ==================== 状态 ====================
const isEditorOpen = ref(false)
const selectedTaskId = ref<string | null>(null)
const kanbanRef = ref<InstanceType<typeof InfiniteDailyKanban> | null>(null)
const currentVisibleDate = ref<string | null>(null) // 当前可见日期

// 获取看板数量
const kanbanCount = computed(() => kanbanRef.value?.kanbanCount ?? 0)

// ==================== 事件处理 ====================
function handleOpenEditor(task: TaskCard) {
  selectedTaskId.value = task.id
  isEditorOpen.value = true
  console.log('[HomeView] 📝 Opening editor for task:', task.id)
}

async function handleAddTask(title: string, date: string) {
  console.log('[HomeView] ➕ Add task:', { title, date })

  try {
    // 1. 创建任务
    const newTask = await taskStore.createTask({ title })
    if (!newTask) {
      console.error('[HomeView] ❌ Failed to create task')
      return
    }

    console.log('[HomeView] ✅ Task created:', newTask.id)

    // 2. 立即为任务添加日程
    const updatedTask = await taskStore.addSchedule(newTask.id, date)
    if (!updatedTask) {
      console.error('[HomeView] ❌ Failed to add schedule')
      return
    }

    console.log('[HomeView] ✅ Schedule added for task:', updatedTask.id, 'on', date)
    
    // ✅ 无需手动刷新！TaskStore 已更新，Vue 响应式系统会自动更新 UI
  } catch (error) {
    console.error('[HomeView] ❌ Error adding task with schedule:', error)
  }
}

function handleVisibleDateChange(date: string) {
  console.log('[HomeView] 📅 Visible date changed:', date)
  currentVisibleDate.value = date
  // 日历会自动通过 :current-date prop 更新显示
}
</script>

<template>
  <div class="home-view-container">
    <div class="main-content-pane">
      <TwoRowLayout>
        <template #top>
          <div class="kanban-header">
            <h2>日程看板</h2>
            <span class="kanban-count">{{ kanbanCount }} 个看板</span>
          </div>
        </template>
        <template #bottom>
          <InfiniteDailyKanban
            ref="kanbanRef"
            @open-editor="handleOpenEditor"
            @add-task="handleAddTask"
            @visible-date-change="handleVisibleDateChange"
          />
        </template>
      </TwoRowLayout>
    </div>
    <div class="calendar-pane">
      <TwoRowLayout>
        <template #top>
          <CuteButton>Test Button 2</CuteButton>
        </template>
        <template #bottom>
          <CuteCalendar :current-date="currentVisibleDate || undefined" />
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
  width: 30rem;
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

/* ==================== 看板标题栏 ==================== */
.kanban-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 0 1rem; /* 减少padding，因为top-row已经有padding了 */
  gap: 1rem;
}

.kanban-header h2 {
  margin: 0;
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.kanban-count {
  font-size: 1.3rem;
  color: var(--color-text-tertiary);
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
