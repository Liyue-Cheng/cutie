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
  <div class="daily-planning-view">
    <TwoRowLayout>
      <!-- 上栏：标题 -->
      <template #top>
        <div class="header">
          <h2>Daily Planning</h2>
          <span class="task-count">{{ todayTasks.length }} tasks today</span>
        </div>
      </template>

      <!-- 下栏：Today 看板 + 日历（横向排列） -->
      <template #bottom>
        <div class="content-container">
          <!-- Today 看板 -->
          <div class="kanban-wrapper">
            <SimpleKanbanColumn
              title="Today"
              :subtitle="today"
              :tasks="todayTasks"
              view-key="daily-planning::today"
              drop-mode="schedule"
              @open-task-editor="handleOpenTaskEditor"
            />
          </div>

          <!-- 日历 -->
          <div class="calendar-wrapper">
            <CuteCalendar :initial-date="today" @date-change="handleCalendarDateChange" />
          </div>
        </div>
      </template>
    </TwoRowLayout>
  </div>

  <!-- 任务编辑器弹窗 -->
  <KanbanTaskEditorModal
    v-if="uiStore.isEditorOpen"
    :task-id="uiStore.editorTaskId"
    @close="handleCloseTaskEditor"
  />
</template>

<style scoped>
/* ==================== 视图容器 ==================== */
.daily-planning-view {
  height: 100%;
  width: 100%;
  background-color: var(--color-background-content);
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
}

/* ==================== 上栏标题 ==================== */
.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 0 1rem;
  gap: 1rem;
}

.header h2 {
  margin: 0;
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.task-count {
  font-size: 1.3rem;
  color: var(--color-text-tertiary);
}

/* ==================== 下栏容器 ==================== */
.content-container {
  display: flex;
  justify-content: center;
  align-items: flex-start;
  gap: 2rem;
  height: 100%;
  width: 100%;
  padding: 2rem;
  overflow: auto;
}

/* ==================== 看板容器 ==================== */
.kanban-wrapper {
  width: 28rem;
  max-width: 28rem;
  height: 100%;
  min-height: 0;
  flex-shrink: 0;
}

/* ==================== 日历容器 ==================== */
.calendar-wrapper {
  width: 28rem;
  max-width: 28rem;
  height: 100%;
  min-height: 0;
  flex-shrink: 0;
}
</style>
