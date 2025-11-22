<script setup lang="ts">
import { ref, onMounted } from 'vue'
import CuteCalendar from '@/components/assembles/calender/CuteCalendar.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import StagingColumn from '@/components/assembles/tasks/kanban/StagingColumn.vue'
import ArchiveColumn from '@/components/assembles/tasks/kanban/ArchiveColumn.vue'
import TaskEditorModal from '@/components/assembles/tasks/TaskEditorModal.vue'
import GlobalRecurrenceEditDialog from '@/components/parts/recurrence/GlobalRecurrenceEditDialog.vue'
import { useTaskStore } from '@/stores/task'
import { useUIStore } from '@/stores/ui'
import { logger, LogTags } from '@/infra/logging/logger'

// ==================== 视图类型 ====================
type CenterPaneView = 'staging' | 'archive'

// ==================== Stores ====================
const taskStore = useTaskStore()
const uiStore = useUIStore()

// ==================== 初始化 ====================
onMounted(async () => {
  logger.info(LogTags.VIEW_CALENDAR, 'Initializing, loading incomplete tasks...')
  await taskStore.fetchAllIncompleteTasks_DMA()
  logger.info(LogTags.VIEW_CALENDAR, 'Loaded incomplete tasks', {
    count: taskStore.incompleteTasks.length,
  })
})

// ==================== 状态 ====================
const currentCenterView = ref<CenterPaneView>('staging') // 中间面板当前视图
const calendarZoom = ref<1 | 2 | 3>(1) // 日历缩放倍率
const calendarViewType = ref<'week' | 'month'>('week') // 日历视图类型（周/月）
const calendarRef = ref<InstanceType<typeof CuteCalendar> | null>(null) // 日历组件引用

// 视图配置
const viewConfig = {
  staging: { icon: 'Layers', label: 'Staging' },
  archive: { icon: 'Archive', label: '归档' },
} as const

// ==================== 事件处理 ====================
function switchCenterView(view: CenterPaneView) {
  logger.debug(LogTags.VIEW_CALENDAR, 'Switching center pane view', { view })
  currentCenterView.value = view
}

// 日历导航：上一周/月
function goToPrevious() {
  if (!calendarRef.value) return
  const api = (calendarRef.value as any).calendarRef?.getApi()
  if (api) {
    api.prev()
    logger.debug(LogTags.VIEW_CALENDAR, 'Navigate to previous', {
      viewType: calendarViewType.value,
    })
  }
}

// 日历导航：下一周/月
function goToNext() {
  if (!calendarRef.value) return
  const api = (calendarRef.value as any).calendarRef?.getApi()
  if (api) {
    api.next()
    logger.debug(LogTags.VIEW_CALENDAR, 'Navigate to next', { viewType: calendarViewType.value })
  }
}

// 日历导航：回到今天
function goToToday() {
  if (!calendarRef.value) return
  const api = (calendarRef.value as any).calendarRef?.getApi()
  if (api) {
    api.today()
    logger.debug(LogTags.VIEW_CALENDAR, 'Navigate to today')
  }
}
</script>

<template>
  <div class="calendar-view-container">
    <!-- 左侧：7天日历视图 -->
    <div class="calendar-main-pane">
      <TwoRowLayout>
        <template #top>
          <div class="calendar-header">
            <!-- 左侧：日期导航 -->
            <div class="calendar-nav">
              <button class="nav-btn" @click="goToPrevious" title="上一周/月">
                <CuteIcon name="ChevronLeft" :size="20" />
              </button>
              <button class="nav-today" @click="goToToday">
                {{ calendarViewType === 'week' ? 'This Week' : 'This Month' }}
              </button>
              <button class="nav-btn" @click="goToNext" title="下一周/月">
                <CuteIcon name="ChevronRight" :size="20" />
              </button>
            </div>

            <!-- 中间：占位 -->
            <div class="spacer"></div>

            <!-- 右侧：视图类型切换 + 缩放 -->
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
              <!-- 日历缩放按钮（仅在周视图显示） -->
              <div v-if="calendarViewType === 'week'" class="calendar-zoom-controls">
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
          </div>
        </template>
        <template #bottom>
          <CuteCalendar ref="calendarRef" :view-type="calendarViewType" :zoom="calendarZoom" />
        </template>
      </TwoRowLayout>
    </div>

    <!-- 中间：展示面板（Staging / Archive） -->
    <div class="center-pane">
      <TwoRowLayout>
        <template #top>
          <div class="center-pane-header">
            <h3>{{ viewConfig[currentCenterView].label }}</h3>
          </div>
        </template>
        <template #bottom>
          <!-- Staging 视图 -->
          <StagingColumn v-if="currentCenterView === 'staging'" />
          <!-- Archive 视图 -->
          <ArchiveColumn v-else-if="currentCenterView === 'archive'" />
        </template>
      </TwoRowLayout>
    </div>

    <!-- 右侧：控制栏 -->
    <div class="toolbar-pane">
      <div class="toolbar-content">
        <!-- 视图切换按钮 -->
        <button
          v-for="(config, viewKey) in viewConfig"
          :key="viewKey"
          class="toolbar-button"
          :class="{ active: currentCenterView === viewKey }"
          :title="config.label"
          @click="switchCenterView(viewKey as CenterPaneView)"
        >
          <CuteIcon :name="config.icon" :size="24" />
        </button>
      </div>
    </div>

    <!-- 任务编辑器模态框 -->
    <TaskEditorModal
      v-if="uiStore.isEditorOpen"
      :task-id="uiStore.editorTaskId"
      :view-key="uiStore.editorViewKey ?? undefined"
      @close="uiStore.closeEditor"
    />

    <!-- 全局循环任务编辑对话框 -->
    <GlobalRecurrenceEditDialog />
  </div>
</template>

<style scoped>
.calendar-view-container {
  display: flex;
  height: 100%;
  width: 100%;
  background-color: var(--color-background-content);
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
}

/* ==================== 左侧日历面板 ==================== */
.calendar-main-pane {
  flex: 1;
  min-width: 0;
  border-right: 1px solid var(--color-border-default);
  box-shadow: inset -4px 0 12px -2px rgb(0 0 0 / 5%);
  position: relative;
}

.calendar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 0 1rem;
  gap: 1rem;
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

/* ==================== 中间展示面板 ==================== */
.center-pane {
  width: 28rem;
  min-width: 0;
  border-right: 1px solid var(--color-border-default);
}

.center-pane-header {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
}

.center-pane-header h3 {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

/* ==================== 右侧控制栏 ==================== */
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
