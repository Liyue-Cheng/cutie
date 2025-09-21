<template>
  <CutePane class="main-frame">
    <div class="title-bar" data-tauri-drag-region>
      <div class="title-bar-bg">
        <div class="window-controls">
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
        <ul>
          <li>Dashboard</li>
          <li>Analytics</li>
          <li>Settings</li>
          <li>About</li>
        </ul>
      </CutePane>
      <CutePane class="main-content-pane">
        <p>Main content goes here...</p>
      </CutePane>
    </CutePane>
  </CutePane>
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount } from 'vue' // 1. 导入生命周期钩子
import { getCurrentWindow } from '@tauri-apps/api/window'
import CuteButton from '../components/ui/CuteButton.vue'
import CuteIcon from '../components/ui/CuteIcon.vue'
import CutePane from '../components/ui/CutePane.vue'
// CuteSurface 在这个模板里没用到，可以暂时去掉
// import CuteSurface from '../components/ui/CuteSurface.vue'

const appWindow = getCurrentWindow()

const themeClassName = 'theme-temp-susamacopy'

// 2. 使用 onMounted 钩子
// onMounted 会在组件被挂载到 DOM 上之后执行
onMounted(() => {
  // 当组件加载时，给 body 标签加上我们的主题 class
  document.body.classList.add(themeClassName)
})

// 3. 使用 onBeforeUnmount 钩子（好习惯）
// onBeforeUnmount 会在组件被卸载之前执行
onBeforeUnmount(() => {
  // 当组件离开时，把我们加上的 class 清理掉，避免影响其他页面
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
  -webkit-app-region: no-drag;
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
  width: 20rem;
  flex-shrink: 0;
  background-color: var(--color-background-secondary);
  border: none; /* Melts into the background */
}

.sidebar-pane ul {
  list-style: none;
  padding: 1rem;
  margin: 0;
  font-size: 1.4rem;
  color: var(--color-text-secondary);
}

.sidebar-pane li {
  padding: 0.8rem 1.2rem;
  border-radius: 0.6rem;
  cursor: pointer;
  transition: background-color 0.2s;
}

.sidebar-pane li:hover {
  background-color: rgb(0 0 0 / 5%);
}

.main-content-pane {
  flex-grow: 1;
  background-color: var(--color-background-content);
  border: none;
}
</style>
