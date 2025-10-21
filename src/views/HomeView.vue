<script setup lang="ts">
import { ref, onMounted, computed, watch, nextTick } from 'vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import InfiniteDailyKanban from '@/components/templates/InfiniteDailyKanban.vue'
import CuteCalendar from '@/components/parts/CuteCalendar.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import KanbanTaskEditorModal from '@/components/parts/kanban/KanbanTaskEditorModal.vue'
import GlobalRecurrenceEditDialog from '@/components/parts/recurrence/GlobalRecurrenceEditDialog.vue'
import StagingColumn from '@/components/parts/kanban/StagingColumn.vue'
import ArchiveColumn from '@/components/parts/kanban/ArchiveColumn.vue'
import UpcomingColumn from '@/components/parts/kanban/UpcomingColumn.vue'
import { useTaskStore } from '@/stores/task'
import { useUIStore } from '@/stores/ui'
import { useRegisterStore } from '@/stores/register'
import { logger, LogTags } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'

// ==================== 类型定义 ====================
type HomeViewMode = 'default' | 'board' | 'calendar'
type ContentView = 'calendar' | 'staging' | 'upcoming' | 'archive'

// ==================== Stores ====================
const taskStore = useTaskStore()
const uiStore = useUIStore()
const registerStore = useRegisterStore()

// ==================== 初始化 ====================
onMounted(async () => {
  logger.info(LogTags.VIEW_HOME, 'Initializing new home view...')

  // 🔥 设置当前视图寄存器
  registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_VIEW, 'home')

  await taskStore.fetchAllIncompleteTasks_DMA()
  logger.info(LogTags.VIEW_HOME, 'Loaded incomplete tasks', {
    count: taskStore.incompleteTasks.length,
  })
})

// ==================== 状态 ====================
const calendarRef = ref<InstanceType<typeof CuteCalendar> | null>(null)
const kanbanRef = ref<InstanceType<typeof InfiniteDailyKanban> | null>(null)
const currentContentView = ref<ContentView>('calendar') // 中间区域显示的内容
const calendarZoom = ref<1 | 2 | 3>(1) // 日历缩放倍率
const calendarViewType = ref<'day' | 'week' | 'month'>('day') // 日历视图类型

// ✅ 从 register store 读取当前日历日期
const currentCalendarDate = computed(() => {
  return registerStore.readRegister<string>(registerStore.RegisterKeys.CURRENT_CALENDAR_DATE_HOME)
})

// ✅ 从 register store 读取当前视图模式
const viewMode = computed<HomeViewMode>(() => {
  return (
    registerStore.readRegister<HomeViewMode>(
      registerStore.RegisterKeys.HOME_VIEW_MODE,
      'default'
    ) ?? 'default'
  )
})

// 根据模式计算日历天数（仅 Board 和其他模式）
const calendarDays = computed<1 | 3>(() => {
  // Board 模式下日历收缩为1天，其他模式都是3天
  return viewMode.value === 'board' ? 1 : 3
})

// 是否显示工具栏
const showToolbar = computed(() => {
  return viewMode.value === 'board'
})

// 视图配置
const viewConfig = {
  calendar: { icon: 'Calendar', label: '日历' },
  staging: { icon: 'Layers', label: 'Staging' },
  upcoming: { icon: 'Clock', label: '即将到期' },
  archive: { icon: 'Archive', label: '归档' },
} as const

// 监听视图模式变化
watch(viewMode, async (newMode) => {
  await nextTick()

  // 切换模式时，重置为日历视图
  currentContentView.value = 'calendar'

  // Calendar 模式下切换到周视图，其他模式切换到天视图
  if (newMode === 'calendar') {
    calendarViewType.value = 'week'
  } else {
    calendarViewType.value = 'day'
  }

  forceCalendarRefresh()
  logger.info(LogTags.VIEW_HOME, 'View mode changed', {
    mode: newMode,
    days: calendarDays.value,
    viewType: calendarViewType.value,
  })
})

// 监听内容视图变化
watch(currentContentView, async () => {
  await nextTick()
  // 如果切换到日历，需要刷新日历尺寸
  if (currentContentView.value === 'calendar') {
    forceCalendarRefresh()
  }
})

// 强制刷新日历
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

// 切换中间内容视图
function switchContentView(view: ContentView) {
  currentContentView.value = view
  logger.debug(LogTags.VIEW_HOME, 'Switching content view', { view })
}

// 日历导航：上一周/天
function goToPrevious() {
  if (!calendarRef.value) return
  const api = (calendarRef.value as any).calendarRef?.getApi()
  if (api) {
    api.prev()
    logger.debug(LogTags.VIEW_HOME, 'Navigate to previous')
  }
}

// 日历导航：下一周/天
function goToNext() {
  if (!calendarRef.value) return
  const api = (calendarRef.value as any).calendarRef?.getApi()
  if (api) {
    api.next()
    logger.debug(LogTags.VIEW_HOME, 'Navigate to next')
  }
}

// 日历导航：回到今天
function goToToday() {
  if (!calendarRef.value) return
  const api = (calendarRef.value as any).calendarRef?.getApi()
  if (api) {
    api.today()
    logger.debug(LogTags.VIEW_HOME, 'Navigate to today')
  }
}

// ==================== 日历和看板联动 ====================

// 处理看板日期点击 - 跳转日历
function handleKanbanDateClick(dateStr: string) {
  logger.info(LogTags.VIEW_HOME, 'Kanban date clicked, jumping calendar', {
    date: dateStr,
  })

  // 跳转日历
  if (calendarRef.value?.calendarRef) {
    const calendarApi = calendarRef.value.calendarRef.getApi()
    if (calendarApi) {
      calendarApi.gotoDate(dateStr)
      logger.debug(LogTags.VIEW_HOME, 'Calendar jumped to date', { dateStr })
    }
  }
}

// 处理任务添加
async function handleAddTask(title: string, date: string) {
  logger.info(LogTags.VIEW_HOME, 'Add task with schedule', { title, date })

  try {
    await pipeline.dispatch('task.create_with_schedule', {
      title,
      scheduled_day: date,
      estimated_duration: 60,
    })

    logger.info(LogTags.VIEW_HOME, 'Task created with schedule', {
      title,
      date,
    })
  } catch (error) {
    logger.error(
      LogTags.VIEW_HOME,
      'Error adding task with schedule',
      error instanceof Error ? error : new Error(String(error))
    )
  }
}
</script>

<template>
  <div class="home-container" :class="`mode-${viewMode}`">
    <!-- 左栏：看板区域 -->
    <div class="left-pane">
      <TwoRowLayout>
        <template #top>
          <div class="kanban-header">
            <!-- Home 和 Board 模式：预留空间，未来设计 -->
          </div>
        </template>
        <template #bottom>
          <InfiniteDailyKanban
            ref="kanbanRef"
            :disable-title-click="true"
            :hide-calendar-icon="true"
            :disable-horizontal-drag="true"
            @add-task="handleAddTask"
            @date-click="handleKanbanDateClick"
          />
        </template>
      </TwoRowLayout>
    </div>

    <!-- 中间：内容区域（日历或其他工具） -->
    <div class="content-pane">
      <TwoRowLayout>
        <template #top>
          <div class="content-header">
            <!-- Calendar 模式下显示完整控件栏 -->
            <template v-if="viewMode === 'calendar' && currentContentView === 'calendar'">
              <!-- 左侧：日期导航 -->
              <div class="calendar-nav">
                <button class="nav-btn" @click="goToPrevious" title="上一周/天">
                  <CuteIcon name="ChevronLeft" :size="20" />
                </button>
                <button class="nav-today" @click="goToToday">This Week</button>
                <button class="nav-btn" @click="goToNext" title="下一周/天">
                  <CuteIcon name="ChevronRight" :size="20" />
                </button>
              </div>

              <!-- 中间：占位 -->
              <div class="spacer"></div>

              <!-- 右侧：日历控制 -->
              <div class="calendar-controls">
                <!-- 视图类型切换按钮 -->
                <div class="view-type-controls">
                  <button
                    :class="['view-type-btn', { active: calendarViewType === 'week' }]"
                    @click="calendarViewType = 'week'"
                  >
                    周视图
                  </button>
                  <button
                    :class="['view-type-btn', { active: calendarViewType === 'month' }]"
                    @click="calendarViewType = 'month'"
                  >
                    月视图
                  </button>
                </div>
                <!-- 日历缩放按钮 -->
                <div class="calendar-zoom-controls">
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
            <!-- Board 模式的其他视图标题 -->
            <template v-else-if="currentContentView !== 'calendar'">
              <h3>{{ viewConfig[currentContentView].label }}</h3>
            </template>
          </div>
        </template>
        <template #bottom>
          <!-- 日历视图 -->
          <CuteCalendar
            v-if="currentContentView === 'calendar'"
            ref="calendarRef"
            :zoom="calendarZoom"
            :days="calendarDays"
            :view-type="calendarViewType"
            :current-date="currentCalendarDate"
          />
          <!-- Staging 视图 -->
          <StagingColumn v-else-if="currentContentView === 'staging'" />
          <!-- Upcoming 视图 -->
          <UpcomingColumn v-else-if="currentContentView === 'upcoming'" />
          <!-- Archive 视图 -->
          <ArchiveColumn v-else-if="currentContentView === 'archive'" />
        </template>
      </TwoRowLayout>
    </div>

    <!-- 右栏：工具栏 (Board 模式下显示) -->
    <div v-if="showToolbar" class="toolbar-pane">
      <div class="toolbar-content">
        <!-- 视图切换按钮 -->
        <button
          v-for="(config, viewKey) in viewConfig"
          :key="viewKey"
          class="toolbar-button"
          :class="{ active: currentContentView === viewKey }"
          :title="config.label"
          @click="switchContentView(viewKey as ContentView)"
        >
          <CuteIcon :name="config.icon" :size="24" />
        </button>
      </div>
    </div>

    <!-- 对话框 -->
    <KanbanTaskEditorModal
      v-if="uiStore.isEditorOpen"
      :task-id="uiStore.editorTaskId"
      :view-key="uiStore.editorViewKey ?? undefined"
      @close="uiStore.closeEditor"
    />
    <GlobalRecurrenceEditDialog />
  </div>
</template>

<style scoped>
.home-container {
  display: flex;
  width: 100%;
  height: 100%;
  background-color: var(--color-background-content);
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
}

/* ==================== 通用头部样式 ==================== */

.kanban-header,
.content-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 0 1rem;
  gap: 1rem;
  min-height: 4rem;
}

.content-header h3 {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

/* 日期导航 */
.calendar-nav {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.nav-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3.2rem;
  height: 3.2rem;
  padding: 0;
  background-color: transparent;
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
  color: var(--color-text-secondary);
}

.nav-btn:hover {
  background-color: var(--color-background-hover);
  border-color: var(--color-border-hover);
  color: var(--color-text-primary);
}

.nav-today {
  padding: 0.6rem 1.4rem;
  font-size: 1.4rem;
  font-weight: 600;
  color: var(--color-primary, #4a90e2);
  background-color: var(--color-primary-bg, #e3f2fd);
  border: 1px solid var(--color-primary-border, #90caf9);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
  min-width: 10rem;
  text-align: center;
}

.nav-today:hover {
  background-color: var(--color-primary-hover, #bbdefb);
}

/* 占位元素 */
.spacer {
  flex: 1;
}

/* 右侧控制组 */
.calendar-controls {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.view-type-controls {
  display: flex;
  gap: 0.4rem;
  background-color: var(--color-background-secondary, #f5f5f5);
  padding: 0.3rem;
  border-radius: 0.6rem;
}

.view-type-btn {
  padding: 0.5rem 1.2rem;
  font-size: 1.3rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  background-color: transparent;
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.view-type-btn:hover {
  color: var(--color-text-primary);
}

.view-type-btn.active {
  color: var(--color-primary);
  background-color: white;
  font-weight: 600;
  box-shadow: 0 1px 3px rgb(0 0 0 / 10%);
}

.calendar-zoom-controls {
  display: flex;
  gap: 0.4rem;
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

/* ==================== 左栏：看板 ==================== */
.left-pane {
  flex: 0 0 70rem;
  min-width: 0;
  border-right: 1px solid var(--color-border-default);
  box-shadow: inset -4px 0 12px -2px rgb(0 0 0 / 5%);
  position: relative;
  overflow: hidden;
  transition: flex 0.3s ease;
}

/* Board 模式下：左边看板自适应 */
.home-container.mode-board .left-pane {
  flex: 1;
}

/* Calendar 模式下：左边看板收缩为1列 */
.home-container.mode-calendar .left-pane {
  flex: 0 0 23rem;
}

/* ==================== 中间：内容区域 ==================== */
.content-pane {
  flex: 1;
  min-width: 0;
  position: relative;
  transition: flex 0.3s ease;
}

/* Board 模式下：右边固定宽度（日历 28rem） */
.home-container.mode-board .content-pane {
  flex: 0 0 28rem;
}

/* ==================== 右栏：工具栏 ==================== */
.toolbar-pane {
  width: 6rem;
  min-width: 6rem;
  display: flex;
  flex-direction: column;
  border-left: 1px solid var(--color-border-default);
  animation: slide-in-from-right 0.3s ease;
}

@keyframes slide-in-from-right {
  from {
    width: 0;
    min-width: 0;
    opacity: 0;
    transform: translateX(6rem);
  }

  to {
    width: 6rem;
    min-width: 6rem;
    opacity: 1;
    transform: translateX(0);
  }
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
</style>
