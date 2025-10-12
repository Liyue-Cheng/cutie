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
const calendarRef = ref<InstanceType<typeof CuteCalendar> | null>(null)

// ==================== 计算属性 ====================
// 今天的任务列表
const todayTasks = computed(() => {
  return taskStore.getTasksByDate(today.value)
})

// ==================== 初始化 ====================
onMounted(async () => {
  logger.info(LogTags.VIEW_HOME, 'Daily Planning: Initializing...')
  await taskStore.fetchAllTasks()
  await taskStore.fetchTasksForDate(today.value)
  logger.info(LogTags.VIEW_HOME, 'Daily Planning: Loaded tasks', {
    today: today.value,
    count: todayTasks.value.length,
  })
})

// ==================== 任务编辑器 ====================
function handleOpenTaskEditor(taskId: string) {
  uiStore.openTaskEditor(taskId)
}

function handleCloseTaskEditor() {
  uiStore.closeTaskEditor()
}

// ==================== 日历交互 ====================
function handleCalendarDateChange(date: string) {
  today.value = date
  logger.debug(LogTags.VIEW_HOME, 'Daily Planning: Date changed', { date })
}
</script>

<template>
  <TwoRowLayout>
    <!-- 上栏：占位符 -->
    <template #top-row>
      <div class="top-placeholder">
        <div class="placeholder-content">
          <h2 class="placeholder-title">Daily Planning</h2>
          <p class="placeholder-subtitle">Plan your day, achieve your goals</p>
        </div>
      </div>
    </template>

    <!-- 下栏：Today 看板 + 日历 -->
    <template #bottom-row>
      <div class="daily-planning-container">
        <!-- 左侧：Today 看板 -->
        <div class="today-kanban-section">
          <SimpleKanbanColumn
            title="Today"
            :subtitle="today"
            :tasks="todayTasks"
            view-key="daily-planning::today"
            drop-mode="schedule"
            @open-task-editor="handleOpenTaskEditor"
          />
        </div>

        <!-- 右侧：日历 -->
        <div class="calendar-section">
          <CuteCalendar
            ref="calendarRef"
            :initial-date="today"
            @date-change="handleCalendarDateChange"
          />
        </div>
      </div>
    </template>
  </TwoRowLayout>

  <!-- 任务编辑器弹窗 -->
  <KanbanTaskEditorModal
    v-if="uiStore.isTaskEditorOpen"
    :task-id="uiStore.currentEditingTaskId"
    @close="handleCloseTaskEditor"
  />
</template>

<style scoped>
/* ==================== 上栏占位符 ==================== */
.top-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, var(--color-primary-soft, #e8f4ff) 0%, var(--color-background, #ffffff) 100%);
  border-bottom: 1px solid var(--color-border-default, #e0e0e0);
}

.placeholder-content {
  text-align: center;
}

.placeholder-title {
  font-size: 3.2rem;
  font-weight: 700;
  color: var(--color-text-primary, #333333);
  margin: 0 0 0.8rem 0;
  letter-spacing: -0.02em;
}

.placeholder-subtitle {
  font-size: 1.6rem;
  color: var(--color-text-secondary, #666666);
  margin: 0;
}

/* ==================== 下栏容器 ==================== */
.daily-planning-container {
  width: 100%;
  height: 100%;
  display: flex;
  gap: 2rem;
  padding: 2rem;
  overflow: hidden;
}

/* ==================== 左侧 Today 看板 ==================== */
.today-kanban-section {
  flex: 1;
  min-width: 0; /* 防止 flex item 溢出 */
  height: 100%;
  overflow: hidden;
}

/* ==================== 右侧日历 ==================== */
.calendar-section {
  flex: 1;
  min-width: 0; /* 防止 flex item 溢出 */
  height: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* ==================== 响应式调整 ==================== */
@media (max-width: 1200px) {
  .daily-planning-container {
    flex-direction: column;
    gap: 1.5rem;
  }

  .today-kanban-section,
  .calendar-section {
    flex: none;
    height: 50%;
  }
}
</style>

