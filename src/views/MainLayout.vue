<template>
  <AreaManager v-if="isAreaManagerOpen" @close="isAreaManagerOpen = false" />
  <RecurrenceManagerModal
    :show="isRecurrenceManagerOpen"
    @close="isRecurrenceManagerOpen = false"
  />
  <SettingsModal :show="isSettingsOpen" @close="isSettingsOpen = false" />
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
              <CuteIcon name="Clock" :size="16" /><span>Recent</span>
            </li>
            <li class="nav-item-with-action">
              <div
                class="nav-item-main"
                @click="$router.push('/staging-kanban')"
              >
                <CuteIcon name="Layers" :size="16" /><span>Staging</span>
              </div>
              <button
                class="quick-add-button"
                @click.stop="showQuickAddDialog = true"
                title="快速添加任务"
              >
                <CuteIcon name="Plus" :size="14" />
              </button>
            </li>
            <li @click="$router.push('/upcoming')">
              <CuteIcon name="CalendarClock" :size="16" /><span>Upcoming</span>
            </li>
            <li @click="$router.push('/projects')">
              <CuteIcon name="Folder" :size="16" /><span>Projects</span>
            </li>
          </ul>

          <div class="section-divider">
            <span class="divider-label">DAILY ROUTINES</span>
          </div>
          <ul class="nav-group">
            <li @click="$router.push('/daily-overview')">
              <CuteIcon name="Calendar" :size="16" /><span>Daily overview</span>
            </li>
            <li @click="$router.push('/daily-shutdown')">
              <CuteIcon name="BookOpen" :size="16" /><span>Daily shutdown</span>
            </li>
          </ul>

          <div class="section-divider">
            <span class="divider-label">KANBAN</span>
          </div>
          <ul class="nav-group">
            <li @click="$router.push('/timeline-kanban')">
              <CuteIcon name="LayoutGrid" :size="16" /><span>Timeline Kanban</span>
            </li>
            <li @click="$router.push('/calendar-kanban')">
              <CuteIcon name="CalendarDays" :size="16" /><span>Calendar Kanban</span>
            </li>
          </ul>
        </div>
        <div class="sidebar-footer">
          <ul class="nav-group">
            <li @click="isAreaManagerOpen = !isAreaManagerOpen">
              <CuteIcon name="Tag" :size="16" />
              <span>Areas</span>
            </li>
            <li @click="isRecurrenceManagerOpen = true">
              <CuteIcon name="RefreshCw" :size="16" />
              <span>循环任务</span>
            </li>
            <li @click="isSettingsOpen = true">
              <CuteIcon name="Settings" :size="16" />
              <span>{{ $t('sidebar.settings') }}</span>
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
import { onMounted, onBeforeUnmount, ref } from 'vue'
import { getCurrentWindow, PhysicalPosition } from '@tauri-apps/api/window'
import CuteButton from '@/components/parts/CuteButton.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CutePane from '@/components/alias/CutePane.vue'
import AreaManager from '@/components/parts/AreaManager.vue'
import QuickAddTaskModal from '@/components/organisms/QuickAddTaskModal.vue'
import RecurrenceManagerModal from '@/components/organisms/RecurrenceManagerModal.vue'
import SettingsModal from '@/components/organisms/SettingsModal.vue'
import AiChatDialog from '@/components/parts/ai/AiChatDialog.vue'
import { useMidnightRefresh } from '@/composables/useMidnightRefresh'

const appWindow = getCurrentWindow()

// 启动全局午夜刷新监测
useMidnightRefresh()

// ==================== 窗口拖动处理 (完全手动实现 - Workaround for Tauri bug #10767) ====================
// 不使用 startDragging()，完全手动计算和设置窗口位置
// 关键：e.screenX/screenY 是逻辑坐标，需要乘以 devicePixelRatio 转换为物理坐标
// 使用 requestAnimationFrame 优化性能，避免卡顿
let isDragging = false
let windowStartX = 0
let windowStartY = 0
let mouseStartX = 0
let mouseStartY = 0
let scaleFactor = 1
let pendingFrame = false
let currentMouseX = 0
let currentMouseY = 0

const handleTitleBarMouseDown = async (e: MouseEvent) => {
  // 只响应左键
  if (e.button !== 0) return

  try {
    // 获取当前窗口位置（Physical 坐标）
    const position = await appWindow.outerPosition()
    windowStartX = position.x
    windowStartY = position.y

    // 获取屏幕缩放因子
    scaleFactor = window.devicePixelRatio

    // 记录鼠标起始位置（逻辑坐标）
    mouseStartX = e.screenX
    mouseStartY = e.screenY
    currentMouseX = e.screenX
    currentMouseY = e.screenY

    console.log(
      'Drag start - Window:',
      position,
      'Mouse:',
      { x: mouseStartX, y: mouseStartY },
      'Scale:',
      scaleFactor
    )

    isDragging = true

    // 添加全局监听器
    document.addEventListener('mousemove', handleMouseMove, { passive: false })
    document.addEventListener('mouseup', handleMouseUp)

    // 防止默认行为
    e.preventDefault()
  } catch (err) {
    console.error('Failed to start drag:', err)
  }
}

const updateWindowPosition = () => {
  if (!isDragging) return

  // 计算鼠标移动的距离（逻辑坐标），然后转换为物理坐标
  const deltaX = (currentMouseX - mouseStartX) * scaleFactor
  const deltaY = (currentMouseY - mouseStartY) * scaleFactor

  // 计算新的窗口位置（物理坐标）
  const newX = Math.round(windowStartX + deltaX)
  const newY = Math.round(windowStartY + deltaY)

  // 不等待 setPosition 完成，直接发送命令
  appWindow.setPosition(new PhysicalPosition(newX, newY)).catch((err) => {
    console.error('Failed to set window position:', err)
  })

  pendingFrame = false
}

const handleMouseMove = (e: MouseEvent) => {
  if (!isDragging) return

  // 检查鼠标按钮状态，如果没有按下任何按钮，停止拖动
  if (e.buttons === 0) {
    console.log('Mouse button released, stopping drag')
    handleMouseUp()
    return
  }

  // 更新当前鼠标位置
  currentMouseX = e.screenX
  currentMouseY = e.screenY

  // 使用 requestAnimationFrame 节流，避免过多的更新
  if (!pendingFrame) {
    pendingFrame = true
    requestAnimationFrame(updateWindowPosition)
  }

  // 防止默认行为
  e.preventDefault()
}

const handleMouseUp = () => {
  if (!isDragging) return

  isDragging = false
  pendingFrame = false

  // 清理监听器
  document.removeEventListener('mousemove', handleMouseMove)
  document.removeEventListener('mouseup', handleMouseUp)

  console.log('Drag ended')
}

// 组件卸载时清理
onBeforeUnmount(() => {
  if (isDragging) {
    handleMouseUp()
  }
  document.body.classList.remove(themeClassName)
})

const isAreaManagerOpen = ref(false)
const isRecurrenceManagerOpen = ref(false)
const isSettingsOpen = ref(false)
const showQuickAddDialog = ref(false)
const isAiDialogOpen = ref(false)

const themeClassName = 'theme-temp-susamacopy'

// 立即应用主题类名，避免初始渲染时的样式闪烁
document.body.classList.add(themeClassName)

// 2. Use onMounted hook
// onMounted is executed after the component is mounted to the DOM
onMounted(() => {
  // 确保主题类名已应用（防御性编程）
  if (!document.body.classList.contains(themeClassName)) {
    document.body.classList.add(themeClassName)
  }
})
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

  /* 🔧 关键：防止子视图撑破主内容区域 */
  min-height: 0;
  min-width: 0;
  overflow: hidden;
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
