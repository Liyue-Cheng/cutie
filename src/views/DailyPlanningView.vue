<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import SimpleKanbanColumn from '@/components/parts/kanban/SimpleKanbanColumn.vue'
import StagingColumn from '@/components/parts/kanban/StagingColumn.vue'
import CuteCalendar from '@/components/parts/CuteCalendar.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
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

// 计算明天的日期
const tomorrow = computed(() => {
  const todayDate = new Date(today.value)
  todayDate.setDate(todayDate.getDate() + 1)
  return todayDate.toLocaleDateString('en-CA') // YYYY-MM-DD
})

// 当前右侧面板视图
const currentRightView = ref<'tomorrow' | 'upcoming'>('tomorrow')

// ==================== 计算属性 ====================
// 今天的任务列表
const todayTasks = computed(() => {
  return taskStore.getTasksByDate(today.value)
})

// 明天的任务列表
const tomorrowTasks = computed(() => {
  return taskStore.getTasksByDate(tomorrow.value)
})

// 即将到期的任务（未来7天内有截止日期的未完成任务）
const upcomingTasks = computed(() => {
  const now = new Date()
  const sevenDaysLater = new Date(now)
  sevenDaysLater.setDate(now.getDate() + 7)

  return Array.from(taskStore.tasks.values()).filter((task) => {
    if (task.is_completed || task.is_archived || task.is_deleted) return false
    if (!task.due_date) return false

    const dueDate = new Date(task.due_date.date)
    return dueDate >= now && dueDate <= sevenDaysLater
  })
})

// ==================== 初始化 ====================
onMounted(async () => {
  logger.info(LogTags.VIEW_HOME, 'Daily Planning: Initializing...')
  await taskStore.fetchAllTasks()
  logger.info(LogTags.VIEW_HOME, 'Daily Planning: Loaded tasks', {
    today: today.value,
    todayCount: todayTasks.value.length,
    tomorrowCount: tomorrowTasks.value.length,
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

// ==================== 右侧视图切换 ====================
function switchRightView(view: 'tomorrow' | 'upcoming') {
  currentRightView.value = view
  logger.debug(LogTags.VIEW_HOME, 'Switching right view', { view })
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

      <!-- 下栏：Staging + Today + 日历/明天 + 工具栏 -->
      <template #bottom>
        <div class="content-container">
          <!-- 左侧：Staging -->
          <div class="staging-wrapper">
            <StagingColumn />
          </div>

          <!-- 中间：Today 看板 -->
          <div class="kanban-wrapper">
            <SimpleKanbanColumn
              title="Today"
              :subtitle="today"
              :tasks="todayTasks"
              :view-key="`daily::${today}`"
              drop-mode="schedule"
              :show-add-input="true"
              @open-task-editor="handleOpenTaskEditor"
            />
          </div>

          <!-- 日历（始终显示） -->
          <div class="calendar-pane">
            <CuteCalendar :initial-date="today" @date-change="handleCalendarDateChange" />
          </div>

          <!-- 右侧：明天或即将到期 -->
          <div class="right-pane">
            <!-- 明天看板 -->
            <SimpleKanbanColumn
              v-if="currentRightView === 'tomorrow'"
              title="Tomorrow"
              :subtitle="tomorrow"
              :tasks="tomorrowTasks"
              :view-key="`daily::${tomorrow}`"
              drop-mode="schedule"
              :show-add-input="true"
              @open-task-editor="handleOpenTaskEditor"
            />
            <!-- 即将到期看板 -->
            <SimpleKanbanColumn
              v-else-if="currentRightView === 'upcoming'"
              title="Upcoming"
              subtitle="Due in 7 days"
              :tasks="upcomingTasks"
              view-key="misc::upcoming"
              drop-mode="none"
              @open-task-editor="handleOpenTaskEditor"
            />
          </div>

          <!-- 工具栏 -->
          <div class="toolbar-pane">
            <div class="toolbar-content">
              <button
                :class="['toolbar-button', { active: currentRightView === 'tomorrow' }]"
                title="Tomorrow"
                @click="switchRightView('tomorrow')"
              >
                <CuteIcon name="CalendarDays" :size="20" />
              </button>
              <button
                :class="['toolbar-button', { active: currentRightView === 'upcoming' }]"
                title="Upcoming (Due in 7 days)"
                @click="switchRightView('upcoming')"
              >
                <CuteIcon name="Clock" :size="20" />
              </button>
            </div>
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
  height: 100%;
  width: 100%;
  overflow: hidden;
}

/* ==================== Staging 容器 ==================== */
.staging-wrapper {
  width: 28rem;
  min-width: 28rem;
  height: 100%;
  border-right: 1px solid var(--color-border-default);
  overflow: auto;
}

/* ==================== Today 看板容器 ==================== */
.kanban-wrapper {
  width: 28rem;
  min-width: 28rem;
  height: 100%;
  border-right: 1px solid var(--color-border-default);
  overflow: auto;
}

/* ==================== 日历面板 ==================== */
.calendar-pane {
  width: 28rem;
  min-width: 28rem;
  height: 100%;
  border-right: 1px solid var(--color-border-default);
  overflow: auto;
}

/* ==================== 右侧面板 ==================== */
.right-pane {
  width: 28rem;
  min-width: 28rem;
  height: 100%;
  border-right: 1px solid var(--color-border-default);
  overflow: auto;
}

/* ==================== 工具栏 ==================== */
.toolbar-pane {
  width: 6rem;
  min-width: 6rem;
  display: flex;
  flex-direction: column;
}

.toolbar-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 1rem 0;
  gap: 0.5rem;
  height: 100%;
}

.toolbar-button {
  width: 4.8rem;
  height: 4.8rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: transparent;
  border: none;
  border-radius: 0.8rem;
  cursor: pointer;
  transition: all 0.2s ease;
  color: var(--color-text-tertiary);
  position: relative;
}

.toolbar-button:hover {
  background-color: var(--color-background-hover, rgb(0 0 0 / 5%));
  color: var(--color-text-secondary);
}

.toolbar-button.active {
  background-color: var(--color-button-primary, #4a90e2);
  color: white;
}

.toolbar-button.active::before {
  content: '';
  position: absolute;
  left: -0.5rem;
  top: 50%;
  transform: translateY(-50%);
  width: 0.3rem;
  height: 2.4rem;
  background-color: var(--color-button-primary, #4a90e2);
  border-radius: 0 0.2rem 0.2rem 0;
}

.toolbar-button:active {
  transform: scale(0.95);
}
</style>
