<template>
  <div class="calendar-view">
    <!-- 左栏：日历 -->
    <div class="left-column" :style="{ width: leftPaneWidth + '%' }">
      <CalendarPanel
        ref="calendarPanelRef"
        :current-calendar-date="currentCalendarDate"
        :calendar-days="5"
        left-view-type="staging"
        current-right-pane-view="calendar"
        :is-calendar-mode="true"
        @calendar-size-update="updateCalendarSize"
        @date-click="onCalendarDateClick"
      />
    </div>

    <!-- 可拖动的分割线 -->
    <div
      class="divider"
      :class="{ 'auto-adjusting': isAutoAdjusting }"
      @mousedown="startDragging"
      @dblclick="resetPaneWidth"
    ></div>

    <!-- 中栏：根据选中的视图显示不同内容 -->
    <div class="middle-column">
      <!-- 当天任务 -->
      <DailyTaskPanel
        v-if="currentRightView === 'daily'"
        v-model="selectedDate"
      />
      <!-- 暂存区 -->
      <HomeStagingPanel v-else-if="currentRightView === 'staging'" />
    </div>

    <!-- 右侧垂直图标栏 -->
    <VerticalToolbar
      :view-config="toolbarConfig"
      :current-view="currentRightView"
      @view-change="onRightViewChange"
    />

    <!-- 任务编辑器模态框挂载点 -->
    <TaskEditorModal
      v-if="uiStore.isEditorOpen"
      :task-id="uiStore.editorTaskId"
      :view-key="uiStore.editorViewKey ?? undefined"
      @close="uiStore.closeEditor"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import CalendarPanel from '@/components/organisms/CalendarPanel.vue'
import VerticalToolbar from '@/components/functional/VerticalToolbar.vue'
import HomeStagingPanel from '@/components/organisms/HomeStagingPanel.vue'
import DailyTaskPanel from '@/components/organisms/DailyTaskPanel.vue'
import { useRegisterStore } from '@/stores/register'
import { useUIStore } from '@/stores/ui'
import TaskEditorModal from '@/components/assembles/tasks/TaskEditorModal.vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { getTodayDateString } from '@/infra/utils/dateUtils'

const { t } = useI18n()
const registerStore = useRegisterStore()
const uiStore = useUIStore()

// ==================== 右栏状态 ====================
type RightView = 'daily' | 'staging'
const currentRightView = ref<RightView>('daily')
const selectedDate = ref<string>(getTodayDateString())

// 工具栏配置
const toolbarConfig = computed(() => ({
  daily: { icon: 'CalendarDays' as const, label: t('toolbar.dailyTasks') },
  staging: { icon: 'Layers' as const, label: t('toolbar.staging') },
}))

// 右栏视图切换
function onRightViewChange(viewKey: string | null) {
  if (!viewKey) return
  currentRightView.value = viewKey as RightView
  logger.info(LogTags.VIEW_HOME, 'Calendar view right panel changed', { viewKey })
}

// 日历日期点击处理
function onCalendarDateClick(date: string) {
  selectedDate.value = date
  currentRightView.value = 'daily'
  logger.info(LogTags.VIEW_HOME, 'Calendar date clicked', { date })
}

// ==================== 日历状态 ====================
const calendarPanelRef = ref<InstanceType<typeof CalendarPanel> | null>(null)

const currentCalendarDate = computed(() => {
  return (
    registerStore.readRegister<string>(registerStore.RegisterKeys.CURRENT_CALENDAR_DATE_HOME) ||
    getTodayDateString()
  )
})

// ==================== 可拖动分割线逻辑 ====================
const leftPaneWidth = ref(73) // 日历模式固定 2.7:1 比例
const isDragging = ref(false)
const isAutoAdjusting = ref(false)
let rafId: number | null = null

const TOOLBAR_WIDTH = 96
const DIVIDER_WIDTH = 1

function startDragging(e: MouseEvent) {
  if (isAutoAdjusting.value) return

  isDragging.value = true
  document.addEventListener('mousemove', onDragging)
  document.addEventListener('mouseup', stopDragging)
  e.preventDefault()
}

function onDragging(e: MouseEvent) {
  if (!isDragging.value) return

  const container = document.querySelector('.calendar-view') as HTMLElement
  if (!container) return

  const containerRect = container.getBoundingClientRect()
  const containerWidth = containerRect.width
  const mouseX = e.clientX - containerRect.left

  let newWidth = (mouseX / containerWidth) * 100
  newWidth = Math.max(20, Math.min(80, newWidth))

  leftPaneWidth.value = newWidth

  if (rafId !== null) {
    cancelAnimationFrame(rafId)
  }
  rafId = requestAnimationFrame(() => {
    updateCalendarSize()
    rafId = null
  })
}

function updateCalendarSize() {
  if (calendarPanelRef.value?.calendarRef?.calendarRef) {
    const calendarApi = calendarPanelRef.value.calendarRef.calendarRef.getApi()
    if (calendarApi) {
      calendarApi.updateSize()
      nextTick(() => {
        calendarPanelRef.value?.calendarRef?.syncColumnWidths()
      })
    }
  }
}

async function stopDragging() {
  isDragging.value = false
  document.removeEventListener('mousemove', onDragging)
  document.removeEventListener('mouseup', stopDragging)

  if (rafId !== null) {
    cancelAnimationFrame(rafId)
    rafId = null
  }

  await nextTick()
  updateCalendarSize()
}

async function resetPaneWidth() {
  leftPaneWidth.value = 73

  await nextTick()
  updateCalendarSize()
}

// 初始化
onMounted(async () => {
  logger.info(LogTags.VIEW_HOME, 'Initializing Calendar view...')
  registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_VIEW, 'calendar')

  await nextTick()
  updateCalendarSize()
})

onBeforeUnmount(() => {
  document.removeEventListener('mousemove', onDragging)
  document.removeEventListener('mouseup', stopDragging)
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
  }
})
</script>

<style scoped>
.calendar-view {
  width: 100%;
  height: 100%;
  display: flex;
  overflow: hidden;
  background-color: var(--color-background-content);
}

.left-column,
.middle-column {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background-color: transparent;
}

.left-column {
  flex-shrink: 0;
  position: relative;
}

.middle-column {
  flex: 1;
  min-width: 0;
  position: relative;
}

/* 分割线 */
.divider {
  width: 1px;
  height: 100%;
  background-color: var(--color-border-light);
  cursor: col-resize;
  flex-shrink: 0;
  transition: background-color 0.2s;
  position: relative;
  z-index: 10;
}

.divider.auto-adjusting {
  cursor: default;
}

.divider::before {
  content: '';
  position: absolute;
  inset: 0 -4px;
  cursor: col-resize;
}

.divider::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 0.4rem;
  height: 3.2rem;
  background-color: var(--color-border-default);
  border-radius: 0.2rem;
  opacity: 0;
  transition:
    opacity 0.2s,
    background-color 0.2s;
  pointer-events: none;
}

.divider:hover {
  background-color: var(--color-border-default);
}

.divider:hover::after {
  opacity: 1;
  background-color: var(--color-text-secondary);
}
</style>
