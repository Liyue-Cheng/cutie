<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import InfiniteDailyKanban from '@/components/templates/InfiniteDailyKanban.vue'
import KanbanTaskEditorModal from '@/components/parts/kanban/KanbanTaskEditorModal.vue'
import GlobalRecurrenceEditDialog from '@/components/parts/recurrence/GlobalRecurrenceEditDialog.vue'
import CuteCalendar from '@/components/parts/CuteCalendar.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import StagingColumn from '@/components/parts/kanban/StagingColumn.vue'
import ArchiveColumn from '@/components/parts/kanban/ArchiveColumn.vue'
import UpcomingColumn from '@/components/parts/kanban/UpcomingColumn.vue'
import TemplateKanbanColumn from '@/components/parts/template/TemplateKanbanColumn.vue'
import UnderConstruction from '@/components/parts/UnderConstruction.vue'
import TrashView from '@/views/TrashView.vue'
import AiChatDialog from '@/components/parts/ai/AiChatDialog.vue'
import { useTaskStore } from '@/stores/task'
import { useUIStore } from '@/stores/ui'
import { logger, LogTags } from '@/services/logger'

// ==================== 视图类型 ====================
type RightPaneView =
  | 'calendar'
  | 'staging'
  | 'upcoming'
  | 'templates'
  | 'projects'
  | 'polling'
  | 'completed'
  | 'archive'
  | 'deleted'

// ==================== Stores ====================
const taskStore = useTaskStore()
const uiStore = useUIStore()

// ==================== 初始化 ====================
onMounted(async () => {
  logger.info(LogTags.VIEW_HOME, 'Initializing, loading all tasks...')
  await taskStore.fetchAllTasks()
  logger.info(LogTags.VIEW_HOME, 'Loaded tasks', { count: taskStore.allTasks.length })
})

// ==================== 状态 ====================
// 🗑️ 移除本地状态 - 由 UI Store 管理
// const isEditorOpen = ref(false)
// const selectedTaskId = ref<string | null>(null)
const kanbanRef = ref<InstanceType<typeof InfiniteDailyKanban> | null>(null)
const currentVisibleDate = ref<string | null>(null) // 当前可见日期
const currentRightPaneView = ref<RightPaneView>('calendar') // 右侧面板当前视图
const calendarZoom = ref<1 | 2 | 3>(1) // 日历缩放倍率
const isAiChatOpen = ref(false) // AI 聊天对话框状态

// 获取看板数量
const kanbanCount = computed(() => kanbanRef.value?.kanbanCount ?? 0)

// 视图配置
const viewConfig = {
  calendar: { icon: 'Calendar', label: '日历' },
  staging: { icon: 'Layers', label: 'Staging' },
  upcoming: { icon: 'Clock', label: '即将到期' },
  templates: { icon: 'FileText', label: '模板' },
  projects: { icon: 'FolderKanban', label: '项目' },
  polling: { icon: 'ListChecks', label: '轮询' },
  completed: { icon: 'CheckCheck', label: '已完成' },
  archive: { icon: 'Archive', label: '归档' },
  deleted: { icon: 'Trash2', label: '最近删除' },
} as const

// ==================== 事件处理 ====================
// 🗑️ 移除 handleOpenEditor - 由 KanbanTaskCard 直接调用 UI Store

async function handleAddTask(title: string, date: string) {
  logger.info(LogTags.VIEW_HOME, 'Add task with schedule', { title, date })

  try {
    // ✅ 使用新的合并端点，一次请求完成创建任务并添加日程
    const newTask = await taskStore.createTaskWithSchedule({
      title,
      scheduled_day: date,
      estimated_duration: 60, // ✅ 默认1小时
    })
    if (!newTask) {
      logger.error(
        LogTags.VIEW_HOME,
        'Failed to create task with schedule',
        new Error('Task creation returned null')
      )
      return
    }

    logger.info(LogTags.VIEW_HOME, 'Task created with schedule', {
      taskId: newTask.id,
      date,
    })

    // ✅ 无需手动刷新！TaskStore 已更新，Vue 响应式系统会自动更新 UI
  } catch (error) {
    logger.error(
      LogTags.VIEW_HOME,
      'Error adding task with schedule',
      error instanceof Error ? error : new Error(String(error))
    )
  }
}

function handleVisibleDateChange(date: string) {
  logger.debug(LogTags.VIEW_HOME, 'Visible date changed', { date })
  currentVisibleDate.value = date
  // 日历会自动通过 :current-date prop 更新显示
}

function switchRightPaneView(view: RightPaneView) {
  logger.debug(LogTags.VIEW_HOME, 'Switching right pane view', { view })
  currentRightPaneView.value = view
}

function openAiChat() {
  logger.debug(LogTags.VIEW_HOME, 'Opening AI chat dialog')
  isAiChatOpen.value = true
}

// ==================== 调试功能 ====================
const isDeletingAll = ref(false)
const isLoadingAll = ref(false)

async function handleDeleteAllTasks() {
  const confirmed = confirm('⚠️ 确定要删除所有任务吗？此操作不可撤销！')
  if (!confirmed) return

  isDeletingAll.value = true
  logger.warn(LogTags.VIEW_HOME, 'Starting to delete all tasks')

  try {
    const allTasks = taskStore.allTasks
    const totalCount = allTasks.length
    logger.warn(LogTags.VIEW_HOME, 'Deleting tasks', { totalCount })

    // 批量删除所有任务（添加延迟避免数据库锁冲突）
    let successCount = 0
    let failCount = 0

    for (const task of allTasks) {
      try {
        await taskStore.deleteTask(task.id)
        successCount++
        logger.debug(LogTags.VIEW_HOME, 'Deleted task', {
          successCount,
          totalCount,
          taskTitle: task.title,
        })
      } catch (error) {
        failCount++
        logger.error(
          LogTags.VIEW_HOME,
          'Failed to delete task',
          error instanceof Error ? error : new Error(String(error)),
          { taskTitle: task.title }
        )
      }
    }

    logger.info(LogTags.VIEW_HOME, 'Delete completed', { successCount, failCount })
    alert(`删除完成！成功：${successCount}，失败：${failCount}`)
  } catch (error) {
    logger.error(
      LogTags.VIEW_HOME,
      'Error during batch delete',
      error instanceof Error ? error : new Error(String(error))
    )
    alert('删除过程中出现错误')
  } finally {
    isDeletingAll.value = false
  }
}

async function handleLoadAllTasks() {
  isLoadingAll.value = true
  logger.info(LogTags.VIEW_HOME, 'Loading all tasks...')

  try {
    await taskStore.fetchAllTasks()
    const taskCount = taskStore.allTasks.length
    const archivedCount = taskStore.archivedTasks.length
    logger.info(LogTags.VIEW_HOME, 'Loaded tasks', { taskCount, archivedCount })
    alert(`加载完成！总任务数：${taskCount}，归档任务：${archivedCount}`)
  } catch (error) {
    logger.error(
      LogTags.VIEW_HOME,
      'Error loading tasks',
      error instanceof Error ? error : new Error(String(error))
    )
    alert('加载任务失败')
  } finally {
    isLoadingAll.value = false
  }
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
            <div class="debug-buttons">
              <button
                class="debug-btn load-btn"
                :disabled="isLoadingAll"
                @click="handleLoadAllTasks"
                title="重新加载所有任务（调试用）"
              >
                {{ isLoadingAll ? '加载中...' : '🔄 加载全部' }}
              </button>
              <button
                class="debug-btn delete-btn"
                :disabled="isDeletingAll || taskStore.allTasks.length === 0"
                @click="handleDeleteAllTasks"
                title="删除所有任务（调试用）"
              >
                {{ isDeletingAll ? '删除中...' : '🗑️ 删除全部' }}
              </button>
            </div>
          </div>
        </template>
        <template #bottom>
          <InfiniteDailyKanban
            ref="kanbanRef"
            @add-task="handleAddTask"
            @visible-date-change="handleVisibleDateChange"
          />
        </template>
      </TwoRowLayout>
    </div>
    <div class="calendar-pane">
      <TwoRowLayout>
        <template #top>
          <div class="calendar-pane-header">
            <h3>{{ viewConfig[currentRightPaneView].label }}</h3>
            <!-- 日历缩放按钮 -->
            <div v-if="currentRightPaneView === 'calendar'" class="calendar-zoom-controls">
              <button
                v-for="scale in [1, 2, 3] as const"
                :key="scale"
                :class="['zoom-btn', { active: calendarZoom === scale }]"
                @click="calendarZoom = scale as 1 | 2 | 3"
              >
                {{ scale }}x
              </button>
            </div>
          </div>
        </template>
        <template #bottom>
          <!-- 日历视图 -->
          <CuteCalendar
            v-if="currentRightPaneView === 'calendar'"
            :current-date="currentVisibleDate || undefined"
            :zoom="calendarZoom"
          />
          <!-- Staging 视图 -->
          <StagingColumn v-else-if="currentRightPaneView === 'staging'" />
          <!-- Upcoming 视图 -->
          <UpcomingColumn v-else-if="currentRightPaneView === 'upcoming'" />
          <!-- 模板视图 -->
          <TemplateKanbanColumn v-else-if="currentRightPaneView === 'templates'" />
          <!-- 其他视图（开发中） -->
          <UnderConstruction
            v-else-if="currentRightPaneView === 'projects'"
            title="项目管理"
            description="管理你的项目和任务分类"
          />
          <UnderConstruction
            v-else-if="currentRightPaneView === 'polling'"
            title="轮询清单"
            description="需要定期检查的阻碍点和检查清单"
          />
          <UnderConstruction
            v-else-if="currentRightPaneView === 'completed'"
            title="已完成任务"
            description="查看已完成的任务历史"
          />
          <!-- 归档视图 -->
          <ArchiveColumn v-else-if="currentRightPaneView === 'archive'" />
          <!-- 回收站视图 -->
          <TrashView v-else-if="currentRightPaneView === 'deleted'" />
        </template>
      </TwoRowLayout>
    </div>
    <div class="toolbar-pane">
      <div class="toolbar-content">
        <!-- AI 聊天按钮 (置顶) -->
        <button class="toolbar-button ai-button" title="AI 助手" @click="openAiChat">
          <CuteIcon name="Sparkles" :size="24" />
        </button>
        <div class="toolbar-divider"></div>
        <!-- 其他视图切换按钮 -->
        <button
          v-for="(config, viewKey) in viewConfig"
          :key="viewKey"
          class="toolbar-button"
          :class="{ active: currentRightPaneView === viewKey }"
          :title="config.label"
          @click="switchRightPaneView(viewKey as RightPaneView)"
        >
          <CuteIcon :name="config.icon" :size="24" />
        </button>
      </div>
    </div>
    <KanbanTaskEditorModal
      v-if="uiStore.isEditorOpen"
      :task-id="uiStore.editorTaskId"
      :view-key="uiStore.editorViewKey ?? undefined"
      @close="uiStore.closeEditor"
    />
    <GlobalRecurrenceEditDialog />
    <AiChatDialog v-if="isAiChatOpen" @close="isAiChatOpen = false" />
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
  width: 28rem;
  min-width: 0;
  border-right: 1px solid var(--color-border-default);
}

.calendar-pane-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  gap: 1rem;
}

.calendar-pane-header h3 {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 600;
  color: var(--color-text-primary);
  flex: 1;
  text-align: center;
}

.calendar-zoom-controls {
  display: flex;
  gap: 0.4rem;
  margin-left: auto;
}

.zoom-btn {
  padding: 0.4rem 0.8rem;
  font-size: 1.2rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  background-color: var(--color-background-content);
  border: 1px solid var(--color-border-default);
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.2s ease;
  min-width: 3.2rem;
}

.zoom-btn:hover {
  color: var(--color-text-primary);
  background-color: var(--color-background-hover);
  border-color: var(--color-border-hover);
}

.zoom-btn.active {
  color: var(--color-primary);
  background-color: var(--color-primary-bg);
  border-color: var(--color-primary);
  font-weight: 600;
}

.toolbar-pane {
  width: 6rem; /* 96px */
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

.toolbar-button.ai-button {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  animation: ai-pulse 2s ease-in-out infinite;
}

.toolbar-button.ai-button:hover {
  background: linear-gradient(135deg, #5568d3 0%, #6a3f8f 100%);
  transform: scale(1.05);
}

@keyframes ai-pulse {
  0%,
  100% {
    box-shadow: 0 0 0 0 rgb(102 126 234 / 50%);
  }

  50% {
    box-shadow: 0 0 0 8px rgb(102 126 234 / 0%);
  }
}

.toolbar-divider {
  width: 80%;
  height: 1px;
  background-color: var(--color-border-default);
  margin: 0.5rem auto;
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

/* ==================== 调试按钮 ==================== */
.debug-buttons {
  display: flex;
  gap: 0.5rem;
}

.debug-btn {
  padding: 0.5rem 1rem;
  font-size: 1.3rem;
  font-weight: 500;
  color: #fff;
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.debug-btn:disabled {
  background-color: #ccc;
  color: #666;
  cursor: not-allowed;
  opacity: 0.6;
}

.debug-btn:hover:not(:disabled) {
  transform: translateY(-1px);
}

.debug-btn:active:not(:disabled) {
  transform: translateY(0);
}

.load-btn {
  background-color: #4a90e2;
}

.load-btn:hover:not(:disabled) {
  background-color: #357abd;
  box-shadow: 0 2px 8px rgb(74 144 226 / 30%);
}

.delete-btn {
  background-color: #ff4d4f;
}

.delete-btn:hover:not(:disabled) {
  background-color: #d9363e;
  box-shadow: 0 2px 8px rgb(255 77 79 / 30%);
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
