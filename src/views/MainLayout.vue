<template>
  <AreaManager v-if="isAreaManagerOpen" @close="isAreaManagerOpen = false" />
  <RecurrenceManagerModal
    :show="isRecurrenceManagerOpen"
    @close="isRecurrenceManagerOpen = false"
  />
  <SettingsModal :show="isSettingsOpen" @close="isSettingsOpen = false" />
  <TimeBlockCreateDialogHost />
  <CutePane class="main-frame">
    <div class="title-bar" @mousedown="handleTitleBarMouseDown">
      <div class="window-controls" @mousedown.stop>
        <CuteButton class="control-btn" @click="appWindow.minimize()">
          <CuteIcon name="Minus" :size="16" />
        </CuteButton>
        <CuteButton class="control-btn" @click="appWindow.toggleMaximize()">
          <CuteIcon name="Square" :size="14" />
        </CuteButton>
        <CuteButton class="control-btn" @click="appWindow.close()">
          <CuteIcon name="X" :size="16" />
        </CuteButton>
      </div>
    </div>

    <CutePane class="content-wrapper">
      <CutePane class="sidebar-pane">
        <div class="sidebar-header">
          <span>{{ $t('sidebar.header') }}</span>
        </div>
        <div class="sidebar-content">
          <ul class="nav-group">
            <li @click="$router.push({ path: '/', query: { view: 'recent' } })">
              <CuteIcon name="Clock" :size="16" /><span>{{ $t('nav.recent') }}</span>
            </li>
            <li class="nav-item-with-action">
              <div class="nav-item-main" @click="$router.push('/staging')">
                <CuteIcon name="Layers" :size="16" /><span>{{ $t('nav.staging') }}</span>
              </div>
              <button
                class="quick-add-button"
                @click.stop="showQuickAddDialog = true"
                :title="$t('nav.quickAddTask')"
              >
                <CuteIcon name="Plus" :size="14" />
              </button>
            </li>
            <li @click="$router.push('/calendar')">
              <CuteIcon name="Calendar" :size="16" /><span>{{ $t('nav.calendar') }}</span>
            </li>
            <li @click="$router.push('/projects')">
              <CuteIcon name="Folder" :size="16" /><span>{{ $t('nav.projects') }}</span>
            </li>
          </ul>

          <div class="section-divider">
            <span class="divider-label">{{ $t('nav.section.dailyRoutines') }}</span>
          </div>
          <ul class="nav-group">
            <li @click="$router.push('/daily-planning')">
              <CuteIcon name="Calendar" :size="16" /><span>{{ $t('nav.dailyOverview') }}</span>
            </li>
            <li @click="$router.push('/daily-shutdown')">
              <CuteIcon name="BookOpen" :size="16" /><span>{{ $t('nav.dailyShutdown') }}</span>
            </li>
          </ul>

          <div class="section-divider">
            <span class="divider-label">{{ $t('nav.section.kanban') }}</span>
          </div>
          <ul class="nav-group">
            <li @click="$router.push('/staging-kanban')">
              <CuteIcon name="Layers" :size="16" /><span>{{ $t('nav.stagingKanban') }}</span>
            </li>
            <li @click="$router.push('/timeline-kanban')">
              <CuteIcon name="LayoutGrid" :size="16" /><span>{{ $t('nav.timelineKanban') }}</span>
            </li>
            <li @click="$router.push('/calendar-kanban')">
              <CuteIcon name="CalendarDays" :size="16" /><span>{{ $t('nav.calendarKanban') }}</span>
            </li>
          </ul>
        </div>
        <div class="sidebar-footer">
          <ul class="nav-group">
            <li @click="isAreaManagerOpen = !isAreaManagerOpen">
              <CuteIcon name="Tag" :size="16" />
              <span>{{ $t('nav.areas') }}</span>
            </li>
            <li @click="isRecurrenceManagerOpen = true">
              <CuteIcon name="RefreshCw" :size="16" />
              <span>{{ $t('nav.recurrence') }}</span>
            </li>
            <li @click="isSettingsOpen = true">
              <CuteIcon name="Settings" :size="16" />
              <span>{{ $t('nav.settings') }}</span>
            </li>
          </ul>
        </div>
      </CutePane>
      <main class="main-content-pane">
        <router-view />
      </main>
    </CutePane>
  </CutePane>

  <!-- 快速添加任务对话框 -->
  <QuickAddTaskModal :show="showQuickAddDialog" @close="showQuickAddDialog = false" />

  <!-- 全局 AI 聊天对话框 -->
  <AiChatDialog v-if="isAiDialogOpen" @close="isAiDialogOpen = false" />

  <!-- 右下角 AI 浮动按钮 -->
  <button class="ai-fab-button" type="button" title="AI 助手" @click="isAiDialogOpen = true">
    <CuteIcon name="MessageCircle" :size="20" />
  </button>
</template>

<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'
import { getCurrentWindow, PhysicalPosition } from '@tauri-apps/api/window'
import CuteButton from '@/components/parts/CuteButton.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CutePane from '@/components/alias/CutePane.vue'
import AreaManager from '@/components/parts/AreaManager.vue'
import QuickAddTaskModal from '@/components/organisms/QuickAddTaskModal.vue'
import RecurrenceManagerModal from '@/components/organisms/RecurrenceManagerModal.vue'
import SettingsModal from '@/components/organisms/SettingsModal.vue'
import TimeBlockCreateDialogHost from '@/components/organisms/TimeBlockCreateDialogHost.vue'
import AiChatDialog from '@/components/parts/ai/AiChatDialog.vue'
import { useMidnightRefresh } from '@/composables/useMidnightRefresh'

const appWindow = getCurrentWindow()

// 启动全局午夜刷新监测
useMidnightRefresh()

// ==================== 窗口拖动处理 (完全手动实现 - Workaround for Tauri bug #10767) ====================
// 不使用 startDragging()，完全手动计算和设置窗口位置
// 关键：e.screenX/screenY 是逻辑坐标，需要乘以 devicePixelRatio 转换为物理坐标
//
// 优化策略：
// 1. 使用 dragSession 版本号忽略过时的 setPosition 请求，防止拉扯
// 2. 同步启动拖动，异步获取窗口位置后再真正开始移动，避免 mousedown 延迟
// 3. 串行化 setPosition 调用，等待上一次完成才发送下一次，防止请求堆积
// 4. 动态获取 scaleFactor，支持跨显示器拖动

let isDragging = false
let isPositionReady = false // 窗口位置是否已获取
let windowStartX = 0
let windowStartY = 0
let mouseStartX = 0
let mouseStartY = 0
let currentMouseX = 0
let currentMouseY = 0
let dragSession = 0 // 拖动会话版本号，用于忽略过时请求
let isSettingPosition = false // 是否正在执行 setPosition
let hasPendingUpdate = false // 是否有待处理的位置更新

const handleTitleBarMouseDown = (e: MouseEvent) => {
  // 只响应左键
  if (e.button !== 0) return

  // 立即记录鼠标起始位置（同步，无延迟）
  mouseStartX = e.screenX
  mouseStartY = e.screenY
  currentMouseX = e.screenX
  currentMouseY = e.screenY

  // 增加会话版本号，使旧会话的所有请求失效
  dragSession++
  const currentSession = dragSession

  isDragging = true
  isPositionReady = false
  isSettingPosition = false
  hasPendingUpdate = false

  // 添加全局监听器（立即响应鼠标移动）
  document.addEventListener('mousemove', handleMouseMove, { passive: false })
  document.addEventListener('mouseup', handleMouseUp)

  // 防止默认行为
  e.preventDefault()

  // 异步获取窗口位置（不阻塞拖动启动）
  appWindow
    .outerPosition()
    .then((position) => {
      // 检查会话是否仍然有效
      if (currentSession !== dragSession) return

      windowStartX = position.x
      windowStartY = position.y
      isPositionReady = true

      // 如果在获取位置期间鼠标已经移动，立即触发一次更新
      if (currentMouseX !== mouseStartX || currentMouseY !== mouseStartY) {
        schedulePositionUpdate()
      }
    })
    .catch((err) => {
      console.error('Failed to get window position:', err)
      // 获取失败时停止拖动
      if (currentSession === dragSession) {
        handleMouseUp()
      }
    })
}

const schedulePositionUpdate = () => {
  // 如果正在设置位置，标记有待处理的更新
  if (isSettingPosition) {
    hasPendingUpdate = true
    return
  }

  // 使用 rAF 确保在下一帧更新
  requestAnimationFrame(updateWindowPosition)
}

const updateWindowPosition = async () => {
  if (!isDragging || !isPositionReady) return

  const currentSession = dragSession

  // 动态获取 scaleFactor（支持跨显示器）
  const scaleFactor = window.devicePixelRatio

  // 计算鼠标移动的距离（逻辑坐标），然后转换为物理坐标
  const deltaX = (currentMouseX - mouseStartX) * scaleFactor
  const deltaY = (currentMouseY - mouseStartY) * scaleFactor

  // 计算新的窗口位置（物理坐标）
  const newX = Math.round(windowStartX + deltaX)
  const newY = Math.round(windowStartY + deltaY)

  isSettingPosition = true
  hasPendingUpdate = false

  try {
    await appWindow.setPosition(new PhysicalPosition(newX, newY))
  } catch (err) {
    // 只有当前会话仍然有效时才报错
    if (currentSession === dragSession) {
      console.error('Failed to set window position:', err)
    }
  }

  // 检查会话是否仍然有效
  if (currentSession !== dragSession) return

  isSettingPosition = false

  // 如果有待处理的更新，继续执行
  if (hasPendingUpdate && isDragging) {
    schedulePositionUpdate()
  }
}

const handleMouseMove = (e: MouseEvent) => {
  if (!isDragging) return

  // 检查鼠标按钮状态，如果没有按下任何按钮，停止拖动
  if (e.buttons === 0) {
    handleMouseUp()
    return
  }

  // 更新当前鼠标位置
  currentMouseX = e.screenX
  currentMouseY = e.screenY

  // 如果窗口位置已就绪，调度位置更新
  if (isPositionReady) {
    schedulePositionUpdate()
  }

  // 防止默认行为
  e.preventDefault()
}

const handleMouseUp = () => {
  if (!isDragging) return

  // 增加会话版本号，使所有进行中的请求失效
  dragSession++
  isDragging = false
  isPositionReady = false
  isSettingPosition = false
  hasPendingUpdate = false

  // 清理监听器
  document.removeEventListener('mousemove', handleMouseMove)
  document.removeEventListener('mouseup', handleMouseUp)
}

// 组件卸载时清理
onBeforeUnmount(() => {
  if (isDragging) {
    handleMouseUp()
  }
})

const isAreaManagerOpen = ref(false)
const isRecurrenceManagerOpen = ref(false)
const isSettingsOpen = ref(false)
const showQuickAddDialog = ref(false)
const isAiDialogOpen = ref(false)
</script>

<style scoped>
.main-frame {
  height: 100vh;
  width: 100vw;
  display: flex;
  flex-direction: column;
  position: relative;
  border: none;
  background-color: var(--color-background-primary);
  padding: 0.2rem 1rem 1rem;

  /* 🔧 防止内容溢出 */
  overflow: hidden;
  box-sizing: border-box;
}

.title-bar {
  height: 3.2rem;
  padding: 0 0.8rem;
  display: flex;
  justify-content: flex-end;
  align-items: center;
  background-color: var(--color-background-primary);
  z-index: 10;
  flex-shrink: 0;

  /* 防止拖动时选中文本 */
  user-select: none;
  cursor: default;
}

.window-controls {
  display: flex;
  gap: 0.5rem;
}

.control-btn {
  padding: 0.2rem 1rem;
  background-color: transparent;
  border: none;
}

.content-wrapper {
  flex-grow: 1;
  display: flex;
  gap: 1rem;

  /* 🔧 关键：防止 flex 子元素撑破容器 */
  min-height: 0;
  overflow: hidden;
}

.sidebar-pane {
  width: 19.2rem;
  flex-shrink: 0;
  background-color: var(--color-background-primary);
  border: none; /* Melts into the background */
  display: flex;
  flex-direction: column;
  padding: 1rem;

  /* 🔧 防止侧边栏溢出 */
  min-height: 0;
  overflow: hidden;
}

.sidebar-header {
  font-size: 1.8rem;
  font-weight: bold;
  padding: 1rem 1.2rem;
  margin-bottom: 1rem;
  color: var(--color-text-secondary);
}

.sidebar-content {
  flex-grow: 1;

  /* 🔧 允许侧边栏内容滚动，但不影响外层布局 */
  min-height: 0;
  overflow-y: auto;
}

.sidebar-footer {
  flex-shrink: 0;
}

.collapsible-section {
  font-size: 1.4rem;
}

.section-divider {
  padding: 1rem 1.2rem 0.5rem;
  margin: 0.5rem 0 1rem;
}

.divider-label {
  font-size: 1.1rem;
  font-weight: 600;
  letter-spacing: 0.1em;
  color: var(--color-text-tertiary);
  text-transform: uppercase;
}

.nav-group {
  list-style: none;
  padding: 0;
  margin: 0;
  font-size: 1.5rem; /* Increased font size */
  color: var(--color-text-secondary);
  margin-bottom: 1.5rem;
}

.nav-group li {
  display: flex;
  align-items: center;
  gap: 1.2rem; /* Space between icon and text */
  padding: 0.6rem 1rem; /* Reduced padding */
  border-radius: 0.6rem;
  cursor: pointer;
  transition: background-color 0.2s;
  line-height: 1.4; /* stabilize line height to avoid spacing jump */
}

.sub-list {
  list-style: none;
  padding: 0;
  margin: 0.5rem 0;
  font-size: 1.5rem;
  color: var(--color-text-secondary);
}

.sub-list li {
  display: flex;
  align-items: center;
  gap: 1.2rem;
  padding: 0.6rem 1rem;
  border-radius: 0.6rem;
  cursor: pointer;
}

.sub-list li:hover {
  color: var(--color-text-primary);
  background-color: var(--color-overlay-light);
}

.nav-group li:hover {
  background-color: var(--color-overlay-light);
}

/* 带操作按钮的导航项 */
.nav-group li.nav-item-with-action {
  display: flex;
  align-items: center;
  gap: 0;
  padding: 0;
}

.nav-item-main {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 1.2rem;
  padding: 0.6rem 1rem;
  cursor: pointer;
}

.quick-add-button {
  all: unset;
  box-sizing: border-box;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.4rem;
  height: 2.4rem;
  margin-right: 0.4rem;
  border-radius: 0.4rem;
  cursor: pointer;
  color: var(--color-text-tertiary);
  opacity: 0;
  transition: all 0.15s ease;
}

.quick-add-button:hover {
  background-color: var(--color-background-accent-light);
  color: var(--color-text-accent);
}

.quick-add-button:active {
  background-color: var(--color-background-selected);
  transform: scale(0.95);
}

.nav-item-with-action:hover .quick-add-button {
  opacity: 1;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.8rem 1.2rem;
  cursor: pointer;
  border-radius: 0.6rem;
  color: var(--color-text-secondary);
}

.section-header:hover {
  background-color: var(--color-overlay-light);
}

.section-header .icon {
  transition: transform 0.2s ease-in-out;
}

.section-header .icon.is-rotated {
  transform: rotate(180deg);
}

.section-title {
  display: flex;
  align-items: center;
  gap: 1.2rem;
}

.main-content-pane {
  flex-grow: 1;
  display: flex;
  flex-direction: column;

  /* 内容区边框和圆角（从各 View 移至此处统一管理） */
  border: 1px solid var(--color-border-subtle, #f0f);
  border-radius: 0.8rem;
  background-color: var(--color-background-content, #f0f);
  overflow: hidden;

  /* 🔧 关键：防止子视图撑破主内容区域 */
  min-height: 0;
  min-width: 0;
}

.main-content-pane > :deep(*) {
  flex-grow: 1;

  /* 🔧 确保子视图也遵守尺寸约束 */
  min-height: 0;
  min-width: 0;
}

.ai-fab-button {
  position: fixed;
  right: 2rem;
  bottom: 2rem;
  width: 4rem;
  height: 4rem;
  border-radius: 0.8rem;
  border: 1px solid var(--color-border-light);
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-background-secondary);
  color: var(--color-text-accent);
  box-shadow: var(--shadow-sm);
  cursor: pointer;
  z-index: 1100;
  transition:
    transform 0.15s ease,
    box-shadow 0.15s ease,
    background 0.15s ease,
    border-color 0.15s ease;
}

.ai-fab-button:hover {
  transform: scale(1.05);
  box-shadow: var(--shadow-md);
  background: var(--color-background-hover);
  border-color: var(--color-border-hover);
}

.ai-fab-button:active {
  transform: scale(0.95);
  box-shadow: var(--shadow-sm);
  background: var(--color-background-active);
}
</style>
