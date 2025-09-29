<template>
  <SettingsView v-if="isSettingsOpen" @close="isSettingsOpen = false" />
  <CutePane class="main-frame">
    <div class="title-bar" data-tauri-drag-region @mousedown="appWindow.startDragging()">
      <div class="title-bar-bg">
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
    </div>

    <CutePane class="content-wrapper">
      <CutePane class="sidebar-pane">
        <div class="sidebar-header">
          <span>{{ $t('sidebar.header') }}</span>
        </div>
        <div class="sidebar-content">
          <ul class="nav-group">
            <li @click="$router.push('/')">
              <CuteIcon name="House" :size="16" /><span>Home</span>
            </li>
            <li @click="$router.push('/staging')">
              <CuteIcon name="Layers" :size="16" /><span>Staging</span>
            </li>
            <li @click="$router.push('/calendar')">
              <CuteIcon name="Calendar" :size="16" /><span>Calendar</span>
            </li>
            <li @click="$router.push('/daily-shutdown')">
              <CuteIcon name="PowerOff" :size="16" /><span>Daily Shutdown</span>
            </li>
          </ul>

          <div class="collapsible-section">
            <div class="section-header" @click="isProjectsOpen = !isProjectsOpen">
              <div class="section-title">
                <CuteIcon name="Folder" :size="16" />
                <span>{{ $t('sidebar.projects') }}</span>
              </div>
              <CuteIcon name="ChevronDown" :size="16" :class="{ 'is-rotated': isProjectsOpen }" />
            </div>
            <ul v-if="isProjectsOpen" class="sub-list">
              <li>{{ $t('projects.alpha') }}</li>
              <li>{{ $t('projects.beta') }}</li>
            </ul>
          </div>

          <div class="collapsible-section">
            <div class="section-header" @click="isExperienceOpen = !isExperienceOpen">
              <div class="section-title">
                <CuteIcon name="Briefcase" :size="16" />
                <span>{{ $t('sidebar.experience') }}</span>
              </div>
              <CuteIcon name="ChevronDown" :size="16" :class="{ 'is-rotated': isExperienceOpen }" />
            </div>
            <ul v-if="isExperienceOpen" class="sub-list">
              <li>{{ $t('experience.acme') }}</li>
              <li>{{ $t('experience.opensource') }}</li>
            </ul>
          </div>
        </div>
        <div class="sidebar-footer">
          <ul class="nav-group">
            <li @click="isSettingsOpen = !isSettingsOpen">
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
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref } from 'vue' // 1. Import lifecycle hooks and ref
import { getCurrentWindow } from '@tauri-apps/api/window'
import CuteButton from '../components/parts/CuteButton.vue'
import CuteIcon from '../components/parts/CuteIcon.vue'
import CutePane from '../components/alias/CutePane.vue'
import SettingsView from '../components/temp/TempSetting.vue'

const appWindow = getCurrentWindow()

const isProjectsOpen = ref(false)
const isExperienceOpen = ref(false)
const isSettingsOpen = ref(false)

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
  padding: 1rem;
  padding-top: 2.6rem;
}

.title-bar {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 4rem; /* Increased height for better drag area */
  z-index: 10;
}

.title-bar-bg {
  position: absolute;
  top: 0;
  right: 0;
  height: 3.2rem;
  padding: 0 0.8rem;
  border: 1px solid var(--color-border-default);
  border-top: none;
  display: flex;
  justify-content: flex-end;
  align-items: center;
  background-color: var(--color-background-content);
  border-bottom-left-radius: 0.8rem;
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

  /* padding: 1rem;
  padding-top: 3.2rem; Make space for the title bar */
  gap: 1rem;
}

.sidebar-pane {
  width: 19.2rem;
  flex-shrink: 0;
  background-color: var(--color-background-secondary);
  border: none; /* Melts into the background */
  display: flex;
  flex-direction: column;
  padding: 1rem;
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
}

.sidebar-footer {
  flex-shrink: 0;
}

.collapsible-section {
  font-size: 1.4rem;
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
}

.sub-list {
  list-style: none;
  padding: 0;
  margin: 0.5rem 0;
  font-size: 1.5rem;
  color: var(--color-text-secondary);
}

.sub-list li {
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
}

.main-content-pane > :deep(*) {
  flex-grow: 1;
}
</style>
