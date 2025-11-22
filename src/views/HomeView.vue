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
    <div
      class="divider"
      :class="{ 'auto-adjusting': isAutoAdjusting }"
      @mousedown="startDragging"
      @dblclick="resetPaneWidth"
    ></div>

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
    <VerticalToolbar
      :view-config="rightPaneViewConfig"
      :current-view="currentRightPaneView"
      @view-change="switchRightPaneView"
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
import { ref, onMounted, onBeforeUnmount, computed, nextTick, watch } from 'vue'
import { useRoute } from 'vue-router'
import RecentTaskPanel from '@/components/organisms/RecentTaskPanel.vue'
import StagingTaskPanel from '@/components/organisms/StagingTaskPanel.vue'
import ProjectsPanel from '@/components/organisms/ProjectsPanel.vue'
import HomeCalendarPanel from '@/components/organisms/HomeCalendarPanel.vue'
import VerticalToolbar from '@/components/functional/VerticalToolbar.vue'
import { useRegisterStore } from '@/stores/register'
import { useUIStore } from '@/stores/ui'
import TaskEditorModal from '@/components/assembles/tasks/TaskEditorModal.vue'
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
function switchRightPaneView(viewKey: string) {
  currentRightPaneView.value = viewKey as RightPaneView
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

  // 初始化时执行一次自动调节
  await nextTick()
  if (shouldAutoAdjust()) {
    animateToOptimalRatio()
  }
})

// ==================== 自动调节监听器 ====================

// 监听左栏视图变化
watch(currentView, async (newView, oldView) => {
  logger.debug(LogTags.VIEW_HOME, 'Left view changed', { from: oldView, to: newView })

  // 切换到 Recent 视图时，如果右栏是需要调节的视图，执行自动调节
  if (newView === 'recent' && shouldAutoAdjust()) {
    await nextTick()
    animateToOptimalRatio()
  }
})

// 监听右栏视图变化
watch(currentRightPaneView, async (newView, oldView) => {
  logger.debug(LogTags.VIEW_HOME, 'Right view changed', { from: oldView, to: newView })

  // 右栏切换到需要调节的视图时，如果左栏是 Recent，执行自动调节
  if (currentView.value === 'recent' && shouldAutoAdjust()) {
    await nextTick()
    animateToOptimalRatio()
  }
})

// 监听日历天数变化
watch(calendarDays, async (newDays, oldDays) => {
  logger.debug(LogTags.VIEW_HOME, 'Calendar days changed', { from: oldDays, to: newDays })

  // 在 Recent + Calendar 组合时，日历天数变化触发自动调节
  if (shouldAutoAdjust()) {
    await nextTick()
    animateToOptimalRatio()
  }
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
const isAutoAdjusting = ref(false) // 自动调节状态标记
let rafId: number | null = null

// ==================== 自动宽度调节系统 ====================

const TOOLBAR_WIDTH = 96 // 工具栏固定宽度 (6rem = 96px)
const DIVIDER_WIDTH = 3   // 分割线宽度

// 根据视图模式计算最佳比例
function calculateOptimalRatio(): number {
  // 只有在 Recent 左栏时才自动调节
  if (currentView.value !== 'recent') {
    return leftPaneWidth.value
  }

  // 获取容器实际宽度
  const container = document.querySelector('.home-view') as HTMLElement
  if (!container) return leftPaneWidth.value

  const containerWidth = container.getBoundingClientRect().width
  // 可分配宽度 = 总宽度 - 工具栏宽度 - 分割线宽度
  const availableWidth = containerWidth - TOOLBAR_WIDTH - DIVIDER_WIDTH

  let leftRatio: number

  // 根据右栏视图类型确定比例
  if (currentRightPaneView.value === 'calendar') {
    // Calendar 视图：根据天数调整
    switch (calendarDays.value) {
      case 1:
        leftRatio = 0.5 // 1:1 比例
        break
      case 3:
      case 5:
        leftRatio = 0.4 // 4:6 比例
        break
      case 7:
        leftRatio = 0.333 // 1:2 比例
        break
      default:
        leftRatio = 0.4
    }
  } else if (currentRightPaneView.value === 'staging' || currentRightPaneView.value === 'templates') {
    // Staging 和 Templates 视图：固定 1:1 比例
    leftRatio = 0.5
  } else {
    // 其他视图保持当前比例
    return leftPaneWidth.value
  }

  // 计算左栏在整个容器中的百分比
  const leftWidthPx = availableWidth * leftRatio
  const leftWidthPercent = (leftWidthPx / containerWidth) * 100

  return Math.max(20, Math.min(80, leftWidthPercent)) // 限制在 20%-80% 范围内
}

// 平滑动画调节到目标比例
async function animateToOptimalRatio() {
  const targetWidth = calculateOptimalRatio()

  // 如果目标比例与当前比例相同，无需动画
  if (Math.abs(leftPaneWidth.value - targetWidth) < 0.1) {
    return
  }

  // 防止重复动画和拖拽冲突
  if (isAutoAdjusting.value || isDragging.value) return

  isAutoAdjusting.value = true
  const startWidth = leftPaneWidth.value
  const duration = 350 // 350ms 动画时长
  const startTime = performance.now()

  logger.info(LogTags.VIEW_HOME, 'Auto-adjusting pane width', {
    from: startWidth,
    to: targetWidth,
    days: calendarDays.value,
    leftView: currentView.value,
    rightView: currentRightPaneView.value,
    actualRatio: getActualRatio(targetWidth), // 显示实际的左栏:中栏比例
  })

  function animateFrame(currentTime: number) {
    const elapsed = currentTime - startTime
    const progress = Math.min(elapsed / duration, 1)

    // 使用 ease-out-cubic 缓动函数，更自然的动画效果
    const easeOutCubic = 1 - Math.pow(1 - progress, 3)

    // 计算当前宽度
    const currentWidth = startWidth + (targetWidth - startWidth) * easeOutCubic
    leftPaneWidth.value = currentWidth

    // 实时更新日历尺寸
    updateCalendarSize()

    if (progress < 1) {
      rafId = requestAnimationFrame(animateFrame)
    } else {
      // 动画完成
      leftPaneWidth.value = targetWidth
      isAutoAdjusting.value = false
      updateCalendarSize()
      logger.debug(LogTags.VIEW_HOME, 'Auto-adjustment animation completed', { finalWidth: targetWidth })
    }
  }

  rafId = requestAnimationFrame(animateFrame)
}

// 检查是否需要自动调节
function shouldAutoAdjust(): boolean {
  // 左栏必须是 Recent
  if (currentView.value !== 'recent') return false

  // 右栏是 Calendar、Staging 或 Templates 时需要自动调节
  return (
    currentRightPaneView.value === 'calendar' ||
    currentRightPaneView.value === 'staging' ||
    currentRightPaneView.value === 'templates'
  )
}

// 计算实际的左栏:中栏比例（用于日志显示）
function getActualRatio(leftWidthPercent: number): string {
  const container = document.querySelector('.home-view') as HTMLElement
  if (!container) return 'unknown'

  const containerWidth = container.getBoundingClientRect().width
  const leftWidthPx = (containerWidth * leftWidthPercent) / 100
  const middleWidthPx = containerWidth - leftWidthPx - DIVIDER_WIDTH - TOOLBAR_WIDTH

  const leftRatio = leftWidthPx / (leftWidthPx + middleWidthPx)
  const middleRatio = middleWidthPx / (leftWidthPx + middleWidthPx)

  // 转换为简单的比例形式
  const scale = Math.min(leftRatio, middleRatio)
  const left = Math.round(leftRatio / scale)
  const middle = Math.round(middleRatio / scale)

  return `${left}:${middle}`
}

function startDragging(e: MouseEvent) {
  // 自动调节动画期间禁用拖拽
  if (isAutoAdjusting.value) return

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
  isAutoAdjusting.value = false // 清理自动调节状态
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

/* 自动调节时的样式 - 只改变光标，保持视觉一致 */
.divider.auto-adjusting {
  cursor: default; /* 自动调节期间禁用拖拽光标 */
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
</style>
