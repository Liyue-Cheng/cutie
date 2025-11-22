<template>
  <div class="home-view">
    <!-- 左栏 -->
    <div class="left-column" :style="{ width: leftPaneWidth + '%' }">
      <RecentTaskPanel
        v-if="currentView === 'recent'"
        v-model="calendarDays"
        @date-change="onRecentDateChange"
      />
      <StagingTaskPanel v-else-if="currentView === 'staging'" />
      <ProjectsPanel v-else-if="currentView === 'projects'" />
    </div>

    <!-- 可拖动的分割线 -->
    <div class="divider" @mousedown="startDragging" @dblclick="resetPaneWidth"></div>

    <!-- 中栏（原右栏）-->
    <div class="middle-column">
      <HomeCalendarPanel
        ref="calendarPanelRef"
        :current-calendar-date="currentCalendarDate"
        :calendar-days="calendarDays"
        :left-view-type="currentView === 'projects' ? 'recent' : currentView"
        :current-right-pane-view="currentRightPaneView"
        @calendar-size-update="updateCalendarSize"
      />
    </div>

    <!-- 右侧垂直图标栏 -->
    <div class="toolbar-pane">
      <div class="toolbar-content">
        <!-- 视图切换按钮 -->
        <button
          v-for="(config, viewKey) in rightPaneViewConfig"
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
import { ref, onMounted, onBeforeUnmount, computed, nextTick, watch } from 'vue'
import { useRoute } from 'vue-router'
import RecentTaskPanel from '@/components/organisms/RecentTaskPanel.vue'
import StagingTaskPanel from '@/components/organisms/StagingTaskPanel.vue'
import ProjectsPanel from '@/components/organisms/ProjectsPanel.vue'
import HomeCalendarPanel from '@/components/organisms/HomeCalendarPanel.vue'
import { useRegisterStore } from '@/stores/register'
import { useUIStore } from '@/stores/ui'
import TaskEditorModal from '@/components/assembles/tasks/TaskEditorModal.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { getTodayDateString } from '@/infra/utils/dateUtils'

const route = useRoute()
const registerStore = useRegisterStore()
const uiStore = useUIStore()

// ==================== 视图切换状态 ====================
const currentView = ref<'recent' | 'staging' | 'projects'>('recent') // 当前左栏视图

// ==================== 右栏视图管理 ====================
type RightPaneView = 'calendar' | 'staging' | 'upcoming' | 'templates' | 'timeline'
const currentRightPaneView = ref<RightPaneView>('calendar')

// 右栏视图配置
const rightPaneViewConfig = {
  calendar: { icon: 'Calendar', label: '日历' },
  timeline: { icon: 'Clock', label: '时间线' },
  staging: { icon: 'Layers', label: 'Staging' },
  upcoming: { icon: 'CalendarClock', label: 'Upcoming' },
  templates: { icon: 'FileText', label: 'Templates' },
} as const

// 切换右栏视图
function switchRightPaneView(viewKey: RightPaneView) {
  currentRightPaneView.value = viewKey
  logger.info(LogTags.VIEW_HOME, 'Right pane view switched', { viewKey })
}

// ==================== 日历天数联动状态 ====================
const calendarDays = ref<1 | 3 | 5 | 7>(3) // 默认显示3天，与 RecentTaskPanel 联动
const calendarPanelRef = ref<InstanceType<typeof HomeCalendarPanel> | null>(null)

// 监听路由变化，切换视图
watch(
  () => route.query.view,
  (newView) => {
    if (newView === 'staging') {
      currentView.value = 'staging'
      logger.info(LogTags.VIEW_HOME, 'Switched to Staging view')
    } else if (newView === 'projects') {
      currentView.value = 'projects'
      logger.info(LogTags.VIEW_HOME, 'Switched to Projects view')
    } else {
      currentView.value = 'recent'
      // 切换回 Recent 视图时，确保日历跳转到今天
      const today = getTodayDateString()
      registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_CALENDAR_DATE_HOME, today)
      logger.info(LogTags.VIEW_HOME, 'Switched to Recent view', { date: today })
    }
  },
  { immediate: true }
)

// 处理 RecentTaskPanel 的日期变化
function onRecentDateChange(date: string) {
  // 更新寄存器中的日历日期
  registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_CALENDAR_DATE_HOME, date)
  logger.debug(LogTags.VIEW_HOME, 'Calendar date synced from RecentTaskPanel', { date })
}

// 初始化
onMounted(async () => {
  logger.info(LogTags.VIEW_HOME, 'Initializing Home view with Recent + Calendar...')
  registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_VIEW, 'home')
})

// ==================== 日历状态 ====================
const currentCalendarDate = computed(() => {
  return (
    registerStore.readRegister<string>(registerStore.RegisterKeys.CURRENT_CALENDAR_DATE_HOME) ||
    getTodayDateString()
  )
})

// ==================== 可拖动分割线逻辑 ====================
const leftPaneWidth = ref(40) // 默认比例 2:3，左栏占 40%
const isDragging = ref(false)
let rafId: number | null = null

function startDragging(e: MouseEvent) {
  isDragging.value = true
  document.addEventListener('mousemove', onDragging)
  document.addEventListener('mouseup', stopDragging)
  e.preventDefault()
}

function onDragging(e: MouseEvent) {
  if (!isDragging.value) return

  const container = document.querySelector('.home-view') as HTMLElement
  if (!container) return

  const containerRect = container.getBoundingClientRect()
  const containerWidth = containerRect.width
  const mouseX = e.clientX - containerRect.left

  // 计算新的左栏宽度百分比
  let newWidth = (mouseX / containerWidth) * 100

  // 限制最小和最大宽度（20% - 80%）
  newWidth = Math.max(20, Math.min(80, newWidth))

  leftPaneWidth.value = newWidth

  // 使用 requestAnimationFrame 实现流畅的实时更新
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
  }
  rafId = requestAnimationFrame(() => {
    updateCalendarSize()
    rafId = null
  })
}

// 更新日历尺寸的辅助函数
function updateCalendarSize() {
  if (calendarPanelRef.value?.calendarRef?.calendarRef) {
    const calendarApi = calendarPanelRef.value.calendarRef.calendarRef.getApi()
    if (calendarApi) {
      calendarApi.updateSize()
      // 同步自定义头部的列宽
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

  // 清除 requestAnimationFrame
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
    rafId = null
  }

  // 最后确保更新一次日历尺寸
  await nextTick()
  updateCalendarSize()
  logger.debug(LogTags.VIEW_HOME, 'Calendar size updated after pane resize')
}

// 双击重置为默认比例
async function resetPaneWidth() {
  leftPaneWidth.value = 40

  // 通知日历更新尺寸
  await nextTick()
  updateCalendarSize()
  logger.debug(LogTags.VIEW_HOME, 'Calendar size updated after pane reset')
}

// 清理事件监听器和动画帧
onBeforeUnmount(() => {
  document.removeEventListener('mousemove', onDragging)
  document.removeEventListener('mouseup', stopDragging)
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
  }
})
</script>

<style scoped>
.home-view {
  width: 100%;
  height: 100%;
  display: flex;
  overflow: hidden;
  background-color: var(--color-background-content);
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
}

/* 左、中、右栏 */
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
  width: 3px;
  height: 100%;
  background-color: var(--color-border-light);
  cursor: col-resize;
  flex-shrink: 0;
  transition: background-color 0.2s;
  position: relative;
  z-index: 10;
}

/* 扩大可点击区域 */
.divider::before {
  content: '';
  position: absolute;
  inset: 0 -4px;
  cursor: col-resize;
}

/* 中间的小手柄 */
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

/* 右侧垂直图标栏 */
.toolbar-pane {
  width: 6rem; /* 96px */
  min-width: 6rem;
  display: flex;
  flex-direction: column;
  background-color: transparent;
  border-left: 1px solid var(--color-border-default);
  border-radius: 0 0.8rem 0.8rem 0;
}

.toolbar-content {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 1.6rem 0;
  gap: 0.8rem;
  overflow-y: auto;
  scrollbar-width: none;
}

.toolbar-content::-webkit-scrollbar {
  display: none;
}

/* 图标按钮样式 */
.toolbar-button {
  width: 4.8rem;
  height: 4.8rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: transparent;
  border: 1px solid transparent;
  border-radius: 0.8rem;
  cursor: pointer;
  transition: all 0.2s ease;
  color: var(--color-text-secondary);
  position: relative;
  flex-shrink: 0;
}

.toolbar-button:hover {
  background-color: var(--color-background-hover, rgba(0, 0, 0, 0.04));
  border-color: var(--color-border-light);
  color: var(--color-text-primary);
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.toolbar-button:active {
  transform: translateY(0);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.06);
}

/* 激活状态 */
.toolbar-button.active {
  background-color: var(--color-primary-light, #e3f2fd);
  border-color: var(--color-primary, #2196f3);
  color: var(--color-primary, #2196f3);
  box-shadow: 0 2px 8px rgba(33, 150, 243, 0.16);
}

.toolbar-button.active:hover {
  background-color: var(--color-primary-light, #e3f2fd);
  border-color: var(--color-primary, #2196f3);
  color: var(--color-primary, #2196f3);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(33, 150, 243, 0.24);
}

/* 工具提示 */
.toolbar-button::before {
  content: attr(title);
  position: absolute;
  right: 110%;
  top: 50%;
  transform: translateY(-50%);
  background-color: var(--color-background-tooltip, rgba(0, 0, 0, 0.8));
  color: var(--color-text-tooltip, white);
  padding: 0.6rem 1rem;
  border-radius: 0.4rem;
  font-size: 1.3rem;
  white-space: nowrap;
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.2s ease;
  z-index: 1000;
}

.toolbar-button:hover::before {
  opacity: 1;
}
</style>
