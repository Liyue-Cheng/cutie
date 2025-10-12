<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import SimpleKanbanColumn from '@/components/parts/kanban/SimpleKanbanColumn.vue'
import CuteCalendar from '@/components/parts/CuteCalendar.vue'
import KanbanTaskEditorModal from '@/components/parts/kanban/KanbanTaskEditorModal.vue'
import { useTaskStore } from '@/stores/task'
import { useUIStore } from '@/stores/ui'
import { logger, LogTags } from '@/services/logger'
import { getTodayDateString } from '@/utils/dateUtils'

// ==================== Stores ====================
const taskStore = useTaskStore()
const uiStore = useUIStore()

// ==================== 状态 ====================
const today = ref(getTodayDateString())

// ==================== 计算属性 ====================
// 今天的任务列表
const todayTasks = computed(() => {
  return taskStore.getTasksByDate(today.value)
})

// ==================== 初始化 ====================
onMounted(async () => {
  logger.info(LogTags.VIEW_HOME, 'Daily Planning: Initializing...')
  await taskStore.fetchAllTasks()
  logger.info(LogTags.VIEW_HOME, 'Daily Planning: Loaded tasks', {
    today: today.value,
    count: todayTasks.value.length,
  })
})

// ==================== 任务编辑器 ====================
function handleOpenTaskEditor(taskId: string) {
  uiStore.openEditor(taskId, 'daily-planning')
}

function handleCloseTaskEditor() {
  uiStore.closeEditor()
}

// ==================== 日历交互 ====================
function handleCalendarDateChange(date: string) {
  today.value = date
  logger.debug(LogTags.VIEW_HOME, 'Daily Planning: Date changed', { date })
}
</script>

<template>
  <div class="daily-planning-container">
    <!-- 左侧：Today 看板 -->
    <div class="today-pane">
      <TwoRowLayout>
        <template #top>
          <div class="pane-header">
            <h2>Today</h2>
            <span class="task-count">{{ todayTasks.length }} 个任务</span>
          </div>
        </template>
        <template #bottom>
          <SimpleKanbanColumn
            title=""
            :subtitle="today"
            :tasks="todayTasks"
            view-key="daily-planning::today"
            drop-mode="schedule"
            @open-task-editor="handleOpenTaskEditor"
          />
        </template>
      </TwoRowLayout>
    </div>

    <!-- 右侧：日历 -->
    <div class="calendar-pane">
      <TwoRowLayout>
        <template #top>
          <div class="pane-header">
            <h3>Calendar</h3>
          </div>
        </template>
        <template #bottom>
          <CuteCalendar :initial-date="today" @date-change="handleCalendarDateChange" />
        </template>
      </TwoRowLayout>
    </div>
  </div>

  <!-- 任务编辑器弹窗 -->
  <KanbanTaskEditorModal
    v-if="uiStore.isEditorOpen"
    :task-id="uiStore.editorTaskId"
    @close="handleCloseTaskEditor"
  />
</template>

<style scoped>
.daily-planning-container {
  display: flex;
  height: 100%;
  width: 100%;
  background-color: var(--color-background-content);
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
}

.today-pane {
  flex: 1;
  min-width: 0;
  border-right: 1px solid var(--color-border-default);
  box-shadow: inset -4px 0 12px -2px rgb(0 0 0 / 5%);
  position: relative;
}

.calendar-pane {
  width: 28rem;
  min-width: 0;
}

.pane-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  gap: 1rem;
}

.pane-header h2,
.pane-header h3 {
  margin: 0;
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.pane-header h3 {
  font-size: 1.6rem;
  flex: 1;
  text-align: center;
}

.task-count {
  font-size: 1.3rem;
  color: var(--color-text-tertiary);
}
</style>
