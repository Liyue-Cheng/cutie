<script setup lang="ts">
import { ref, onMounted } from 'vue'
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
import { logger, LogTags } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'

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
  logger.info(LogTags.VIEW_HOME, 'Initializing, loading incomplete tasks...')
  // 🔥 替换：只加载未完成任务，避免循环任务导致的无限数据
  await taskStore.fetchAllIncompleteTasks_DMA()
  logger.info(LogTags.VIEW_HOME, 'Loaded incomplete tasks', {
    count: taskStore.incompleteTasks.length,
  })
})

// ==================== 状态 ====================
// 🗑️ 移除本地状态 - 由 UI Store 管理
// const isEditorOpen = ref(false)
// const selectedTaskId = ref<string | null>(null)
const kanbanRef = ref<InstanceType<typeof InfiniteDailyKanban> | null>(null)
const calendarRef = ref<InstanceType<typeof CuteCalendar> | null>(null)
const currentRightPaneView = ref<RightPaneView>('calendar') // 右侧面板当前视图
const calendarZoom = ref<1 | 2 | 3>(1) // 日历缩放倍率
const isAiChatOpen = ref(false) // AI 聊天对话框状态
const showDatePicker = ref(false) // 日期选择器显示状态
const selectedDate = ref('') // 选中的日期
const calendarDays = ref<1 | 3>(1) // 🆕 日历显示天数（1天 or 3天）
const isRightPaneCollapsed = ref(true) // 🆕 右边栏是否收起（默认收起）
// 🗑️ 移除 currentCalendarDate - 现在使用 register store

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
    // ✅ 使用 CPU Pipeline 创建任务并添加日程
    await pipeline.dispatch('task.create_with_schedule', {
      title,
      scheduled_day: date,
      estimated_duration: 60, // ✅ 默认1小时
    })

    logger.info(LogTags.VIEW_HOME, 'Task created with schedule', {
      title,
      date,
    })

    // ✅ 无需手动刷新！TaskStore 会通过 SSE 自动更新，Vue 响应式系统会自动更新 UI
  } catch (error) {
    logger.error(
      LogTags.VIEW_HOME,
      'Error adding task with schedule',
      error instanceof Error ? error : new Error(String(error))
    )
  }
}

// 🆕 强制刷新日历（动画期间持续重绘）
function forceCalendarRefresh() {
  const ANIMATION_DURATION = 300
  const startTime = performance.now()

  const resize = () => {
    const elapsed = performance.now() - startTime

    if (calendarRef.value?.calendarRef) {
      const calendarApi = calendarRef.value.calendarRef.getApi()
      if (calendarApi) {
        calendarApi.updateSize()
      }
    }

    if (elapsed < ANIMATION_DURATION) {
      requestAnimationFrame(resize)
    } else {
      // 最终再刷新一次
      if (calendarRef.value?.calendarRef) {
        calendarRef.value.calendarRef.getApi()?.updateSize()
      }
    }
  }

  requestAnimationFrame(resize)
}

function switchRightPaneView(view: RightPaneView) {
  logger.debug(LogTags.VIEW_HOME, 'Switching right pane view', { view })

  // 🆕 如果点击的是当前已选中的视图，则切换右边栏的展开/收起状态
  if (currentRightPaneView.value === view) {
    const willExpand = isRightPaneCollapsed.value
    isRightPaneCollapsed.value = !isRightPaneCollapsed.value

    // 如果是展开操作，触发日历刷新
    if (willExpand) {
      forceCalendarRefresh()
    }

    logger.info(LogTags.VIEW_HOME, 'Toggled right pane', {
      view,
      collapsed: isRightPaneCollapsed.value,
    })
    return
  }

  // 切换到新视图
  currentRightPaneView.value = view

  // 🆕 切换视图时展开右边栏并刷新日历
  const wasCollapsed = isRightPaneCollapsed.value
  isRightPaneCollapsed.value = false

  if (wasCollapsed) {
    forceCalendarRefresh()
  }

  // 🔥 切换到非日历视图时，强制将日历收窄回1天
  if (view !== 'calendar' && calendarDays.value === 3) {
    calendarDays.value = 1
    logger.info(LogTags.VIEW_HOME, 'Calendar auto-collapsed to 1 day', { view })
  }
}

function openAiChat() {
  logger.debug(LogTags.VIEW_HOME, 'Opening AI chat dialog')
  isAiChatOpen.value = true
}

// 循环切换日历缩放倍率
function cycleZoom() {
  if (calendarZoom.value === 1) {
    calendarZoom.value = 2
  } else if (calendarZoom.value === 2) {
    calendarZoom.value = 3
  } else {
    calendarZoom.value = 1
  }
}

// 🆕 切换日历显示天数
function toggleCalendarDays() {
  calendarDays.value = calendarDays.value === 1 ? 3 : 1
  logger.info(LogTags.VIEW_HOME, 'Calendar days toggled', { days: calendarDays.value })

  // 触发日历刷新
  forceCalendarRefresh()
}

// 跳转到今天
function goToToday() {
  const today = new Date()
  const todayStr = formatDateToYYYYMMDD(today)
  goToDate(todayStr)
}

// 跳转到指定日期
function goToDate(dateStr: string) {
  logger.info(LogTags.VIEW_HOME, 'Jumping to date', { date: dateStr })

  // 日历跳转
  if (calendarRef.value?.calendarRef) {
    const calendarApi = calendarRef.value.calendarRef.getApi()
    if (calendarApi) {
      calendarApi.gotoDate(dateStr)
      logger.debug(LogTags.VIEW_HOME, 'Calendar jumped to date', { dateStr })
    }
  }

  // 无限看板跳转
  if (kanbanRef.value?.goToDate) {
    kanbanRef.value.goToDate(dateStr)
    logger.debug(LogTags.VIEW_HOME, 'Kanban jumped to date', { dateStr })
  }

  showDatePicker.value = false
}

// 格式化日期为 YYYY-MM-DD
function formatDateToYYYYMMDD(date: Date): string {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

// 处理日期输入变化
function handleDateChange(event: Event) {
  const input = event.target as HTMLInputElement
  if (input.value) {
    goToDate(input.value)
  }
}

// 处理看板日期点击（跳转日历并展开右边栏）
function handleKanbanDateClick(dateStr: string) {
  logger.info(LogTags.VIEW_HOME, 'Kanban date clicked, jumping calendar and expanding pane', {
    date: dateStr,
  })

  // 🆕 切换到日历视图并展开
  const wasCollapsed = isRightPaneCollapsed.value
  currentRightPaneView.value = 'calendar'
  isRightPaneCollapsed.value = false

  // 如果之前是收起状态，触发刷新
  if (wasCollapsed) {
    forceCalendarRefresh()
  }

  // 跳转日历
  if (calendarRef.value?.calendarRef) {
    const calendarApi = calendarRef.value.calendarRef.getApi()
    if (calendarApi) {
      calendarApi.gotoDate(dateStr)
      logger.debug(LogTags.VIEW_HOME, 'Calendar jumped to date', { dateStr })
    }
  }
}

// 🗑️ 移除 handleCalendarDateChange - 日历现在直接写入 register store

// 🆕 处理日历日期可见性变化
function handleCalendarDateVisibilityChange(isVisible: boolean) {
  logger.debug(LogTags.VIEW_HOME, 'Calendar date visibility changed in kanban', { isVisible })

  // 当日历视图显示且日历当前显示的日期不可见时，自动收起右边栏
  if (!isVisible && currentRightPaneView.value === 'calendar' && !isRightPaneCollapsed.value) {
    isRightPaneCollapsed.value = true
    logger.info(LogTags.VIEW_HOME, 'Auto-collapsed right pane (calendar date not visible)')
  }
}

// ==================== 调试功能 ====================
// 🗑️ 已移除调试功能：handleDeleteAllTasks 和 handleLoadAllTasks
</script>

<template>
  <div class="home-view-container">
    <div class="main-content-pane">
      <TwoRowLayout>
        <template #top>
          <div class="kanban-header">
            <button class="filter-button" title="筛选">
              <CuteIcon name="ListFilter" :size="16" />
              <span>筛选</span>
            </button>
            <div class="date-navigation">
              <div class="today-group">
                <button class="today-button" @click="goToToday">今天</button>
                <button
                  class="expand-button"
                  :class="{ active: showDatePicker }"
                  @click="showDatePicker = !showDatePicker"
                >
                  <CuteIcon name="ChevronDown" :size="16" />
                </button>
              </div>
              <div v-if="showDatePicker" class="date-picker-dropdown">
                <input
                  type="date"
                  :value="selectedDate"
                  @change="handleDateChange"
                  class="date-input"
                />
              </div>
            </div>
          </div>
        </template>
        <template #bottom>
          <InfiniteDailyKanban
            ref="kanbanRef"
            @add-task="handleAddTask"
            @date-click="handleKanbanDateClick"
            @calendar-date-visibility-change="handleCalendarDateVisibilityChange"
          />
        </template>
      </TwoRowLayout>
    </div>
    <div
      class="calendar-pane"
      :class="{
        'calendar-pane-wide': calendarDays === 3,
        'calendar-pane-collapsed': isRightPaneCollapsed,
      }"
    >
      <TwoRowLayout>
        <template #top>
          <div class="calendar-pane-header">
            <!-- 日历天数切换按钮 -->
            <div v-if="currentRightPaneView === 'calendar'" class="calendar-days-toggle">
              <button
                class="days-toggle-btn"
                :class="{ active: calendarDays === 3 }"
                @click="toggleCalendarDays"
                :title="calendarDays === 1 ? '切换到3天视图' : '切换到1天视图'"
              >
                <CuteIcon name="Columns3" :size="16" />
              </button>
            </div>
            <!-- 日历导航按钮 -->
            <div v-if="currentRightPaneView === 'calendar'" class="calendar-nav-buttons">
              <button class="nav-btn" title="上一天">
                <CuteIcon name="ChevronLeft" :size="16" />
              </button>
              <button class="nav-btn" title="下一天">
                <CuteIcon name="ChevronRight" :size="16" />
              </button>
            </div>
            <!-- 日历缩放按钮 -->
            <div v-if="currentRightPaneView === 'calendar'" class="calendar-zoom-controls">
              <button class="zoom-toggle-btn" @click="cycleZoom">{{ calendarZoom }}x</button>
            </div>
            <h3 v-else>{{ viewConfig[currentRightPaneView].label }}</h3>
          </div>
        </template>
        <template #bottom>
          <!-- 日历视图 -->
          <CuteCalendar
            v-if="currentRightPaneView === 'calendar'"
            ref="calendarRef"
            :zoom="calendarZoom"
            :days="calendarDays"
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
        <!-- AI 聊天按钮 (置底) -->
        <button class="toolbar-button ai-button" title="AI 助手" @click="openAiChat">
          <CuteIcon name="Sparkles" :size="24" />
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
  transition: width 0.3s ease;
  overflow: hidden; /* 🆕 收起时隐藏内容 */
}

.calendar-pane.calendar-pane-wide {
  width: 48rem; /* 3天视图时宽度约480px，更加紧凑 */
}

.calendar-pane.calendar-pane-collapsed {
  width: 0; /* 🆕 收起时宽度为0 */
  border-right: none; /* 🆕 收起时不显示边框 */
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
}

.calendar-days-toggle {
  display: flex;
  gap: 0.5rem;
}

.days-toggle-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.8rem;
  height: 2.8rem;
  padding: 0;
  border-radius: 0.4rem;
  border: 1px solid var(--color-border-default);
  background-color: transparent;
  color: var(--color-text-primary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.days-toggle-btn:hover {
  background-color: var(--color-background-hover, rgb(0 0 0 / 5%));
  border-color: var(--rose-pine-foam, #56949f);
  color: var(--rose-pine-foam, #56949f);
}

.days-toggle-btn.active {
  background-color: var(--rose-pine-foam, #56949f);
  color: var(--rose-pine-base, #faf4ed);
  border-color: var(--rose-pine-foam, #56949f);
}

.calendar-nav-buttons {
  display: flex;
  gap: 0.5rem;
}

.nav-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.8rem;
  height: 2.8rem;
  padding: 0;
  border-radius: 0.4rem;
  border: 1px solid var(--color-border-default);
  background-color: transparent;
  color: var(--color-text-primary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.nav-btn:hover {
  background-color: var(--color-background-hover, rgb(0 0 0 / 5%));
  border-color: var(--rose-pine-foam, #56949f);
  color: var(--rose-pine-foam, #56949f);
}

.calendar-zoom-controls {
  margin-left: auto;
}

.zoom-toggle-btn {
  padding: 0.4rem 1rem;
  font-size: 1.3rem;
  font-weight: 500;
  border-radius: 0.4rem;
  border: 1px solid var(--color-border-default);
  background-color: transparent;
  color: var(--color-text-primary);
  cursor: pointer;
  transition: all 0.2s ease;
  min-width: 4rem;
}

.zoom-toggle-btn:hover {
  background-color: var(--color-background-hover, rgb(0 0 0 / 5%));
  border-color: var(--rose-pine-foam, #56949f);
  color: var(--rose-pine-foam, #56949f);
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
  position: relative;
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
  background-color: var(--rose-pine-foam, #56949f);
  color: var(--rose-pine-base, #faf4ed);
}

.toolbar-button:active {
  transform: scale(0.95);
}

.toolbar-button.ai-button {
  background-color: var(--rose-pine-iris, #907aa9);
  color: var(--rose-pine-base, #faf4ed);
  position: absolute;
  bottom: 1rem;
}

.toolbar-button.ai-button:hover {
  background-color: var(--rose-pine-love, #b4637a);
  transform: scale(1.05);
}

/* ==================== 看板标题栏 ==================== */
.kanban-header {
  display: flex;
  align-items: center;
  width: 100%;
  padding: 0 1rem;
}

.filter-button {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.6rem 1.2rem;
  font-size: 1.4rem;
  border-radius: 0.6rem;
  border: 1px solid var(--color-border-default);
  background-color: transparent;
  color: var(--color-text-primary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.filter-button:hover {
  background-color: var(--color-background-hover, rgb(0 0 0 / 5%));
  border-color: var(--rose-pine-foam, #56949f);
  color: var(--rose-pine-foam, #56949f);
}

.date-navigation {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-left: 1rem;
  position: relative;
}

.today-group {
  display: flex;
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  overflow: hidden;
  transition: all 0.2s ease;
}

.today-group:hover {
  border-color: var(--rose-pine-foam, #56949f);
}

.today-button {
  padding: 0.6rem 1.2rem;
  font-size: 1.4rem;
  border: none;
  border-right: 1px solid var(--color-border-default);
  background-color: transparent;
  color: var(--color-text-primary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.today-button:hover {
  background-color: var(--color-background-hover, rgb(0 0 0 / 5%));
  color: var(--rose-pine-foam, #56949f);
}

.expand-button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.8rem;
  height: 100%;
  padding: 0;
  border: none;
  background-color: transparent;
  color: var(--color-text-primary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.expand-button:hover {
  background-color: var(--color-background-hover, rgb(0 0 0 / 5%));
  color: var(--rose-pine-foam, #56949f);
}

.expand-button.active {
  background-color: var(--rose-pine-foam, #56949f);
  color: var(--rose-pine-base, #faf4ed);
}

.date-picker-dropdown {
  position: absolute;
  top: calc(100% + 0.5rem);
  right: 0;
  background-color: var(--color-background-primary);
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
  padding: 1rem;
  box-shadow: 0 4px 12px rgb(0 0 0 / 10%);
  z-index: 100;
}

.date-input {
  padding: 0.6rem 1rem;
  font-size: 1.4rem;
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  background-color: var(--color-background-primary);
  color: var(--color-text-primary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.date-input:hover {
  border-color: var(--rose-pine-foam, #56949f);
}

.date-input:focus {
  outline: none;
  border-color: var(--rose-pine-foam, #56949f);
  box-shadow: 0 0 0 3px rgb(86 148 159 / 10%);
}
</style>
