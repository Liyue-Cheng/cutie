<template>
  <AreaManager v-if="isAreaManagerOpen" @close="isAreaManagerOpen = false" />
  <RecurrenceManagerModal
    :show="isRecurrenceManagerOpen"
    @close="isRecurrenceManagerOpen = false"
  />
  <SettingsModal :show="isSettingsOpen" @close="isSettingsOpen = false" />
  <CutePane class="main-frame">
    <div class="title-bar" data-tauri-drag-region @mousedown="appWindow.startDragging()">
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
                @click="$router.push({ path: '/', query: { view: 'staging' } })"
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
            <li @click="$router.push({ path: '/', query: { view: 'upcoming' } })">
              <CuteIcon name="CalendarClock" :size="16" /><span>Upcoming</span>
            </li>
            <li @click="$router.push({ path: '/', query: { view: 'projects' } })">
              <CuteIcon name="Folder" :size="16" /><span>Projects</span>
            </li>
          </ul>

          <div class="collapsible-section">
            <div class="section-header" @click="isLegacyOpen = !isLegacyOpen">
              <div class="section-title">
                <CuteIcon name="Archive" :size="16" />
                <span>Legacy</span>
              </div>
              <CuteIcon name="ChevronDown" :size="16" :class="{ 'is-rotated': isLegacyOpen }" />
            </div>
            <ul v-if="isLegacyOpen" class="sub-list">
              <li @click="navigateToLegacyHome('default')">
                <CuteIcon name="House" :size="16" /><span>Home</span>
              </li>
              <li @click="navigateToLegacyHome('board')">
                <CuteIcon name="LayoutDashboard" :size="16" /><span>Board</span>
              </li>
              <li @click="navigateToLegacyHome('calendar')">
                <CuteIcon name="Calendar" :size="16" /><span>Calendar</span>
              </li>
              <li @click="$router.push('/sunsama-legacy')">
                <CuteIcon name="LayoutGrid" :size="16" /><span>Sunsama Legacy</span>
              </li>
              <li @click="$router.push('/calendar-legacy')">
                <CuteIcon name="CalendarDays" :size="16" /><span>Calendar Legacy</span>
              </li>
              <li @click="$router.push('/area-test')">
                <CuteIcon name="Tag" :size="16" /><span>Area Test</span>
              </li>
              <li @click="$router.push('/debug')">
                <CuteIcon name="Bug" :size="16" /><span>Debug</span>
              </li>
              <li @click="$router.push('/interact-test')">
                <CuteIcon name="MousePointer2" :size="16" /><span>Interact Test</span>
              </li>
              <li @click="$router.push('/cpu-debug')">
                <CuteIcon name="Cpu" :size="16" /><span>CPU Pipeline</span>
              </li>
              <li @click="$router.push('/checkbox-test')">
                <CuteIcon name="SquareCheck" :size="16" /><span>Checkbox Test</span>
              </li>
            </ul>
          </div>
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
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref } from 'vue' // 1. Import lifecycle hooks and ref
import { useRouter } from 'vue-router'
import { getCurrentWindow } from '@tauri-apps/api/window'
import CuteButton from '@/components/parts/CuteButton.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CutePane from '@/components/alias/CutePane.vue'
import AreaManager from '@/components/parts/AreaManager.vue'
import QuickAddTaskModal from '@/components/organisms/QuickAddTaskModal.vue'
import RecurrenceManagerModal from '@/components/organisms/RecurrenceManagerModal.vue'
import SettingsModal from '@/components/organisms/SettingsModal.vue'
import { useRegisterStore } from '@/stores/register'
import { useMidnightRefresh } from '@/composables/useMidnightRefresh'

const appWindow = getCurrentWindow()
const router = useRouter()
const registerStore = useRegisterStore()

// 启动全局午夜刷新监测
useMidnightRefresh()

const isLegacyOpen = ref(false)
const isAreaManagerOpen = ref(false)
const isRecurrenceManagerOpen = ref(false)
const isSettingsOpen = ref(false)
const showQuickAddDialog = ref(false)

// 导航到 Legacy Home 并设置模式
function navigateToLegacyHome(mode: 'default' | 'board' | 'calendar') {
  registerStore.writeRegister(registerStore.RegisterKeys.HOME_VIEW_MODE, mode)
  router.push('/home-legacy')
}

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

// 3. Use onBeforeUnmount hook (good practice)
// onBeforeUnmount is executed before the component is unmounted
onBeforeUnmount(() => {
  // When the component unmounts, remove the class to avoid affecting other pages
  document.body.classList.remove(themeClassName)
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
  border-top: 1px solid var(--color-border-soft, rgb(0 0 0 / 8%));
}

.divider-label {
  font-size: 1.1rem;
  font-weight: 600;
  letter-spacing: 0.1em;
  color: var(--color-text-tertiary, #999);
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
  background-color: rgb(0 0 0 / 5%);
}

.nav-group li:hover {
  background-color: rgb(0 0 0 / 5%);
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
  color: var(--color-text-tertiary, #9893a5);
  opacity: 0;
  transition: all 0.15s ease;
}

.quick-add-button:hover {
  background-color: var(--color-primary-bg, rgb(40 105 131 / 10%));
  color: var(--color-primary, #286983);
}

.quick-add-button:active {
  background-color: var(--color-primary-bg, rgb(40 105 131 / 15%));
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
  background-color: rgb(0 0 0 / 5%);
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
</style>
