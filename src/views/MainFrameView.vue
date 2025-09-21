<template>
  <CutePane class="main-frame">
    <CutePane class="title-bar" data-tauri-drag-region>
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
    </CutePane>
    <CutePane class="main-content">
      <h1>Cutie App</h1>
    </CutePane>
  </CutePane>
</template>

<script setup lang="ts">
// 1. 从 '@tauri-apps/api/window' 这个子模块导入 getCurrentWindow 函数
import { getCurrentWindow } from '@tauri-apps/api/window'
import CuteButton from '../components/ui/CuteButton.vue'
import CuteIcon from '../components/ui/CuteIcon.vue'
import CutePane from '../components/ui/CutePane.vue'
import CuteSurface from '../components/ui/CuteSurface.vue'
// 2. 调用函数来获取当前窗口的实例
const appWindow = getCurrentWindow()
// 3. 在模板中就可以直接使用 appWindow.close() 等方法了
</script>

<style scoped>
.main-frame {
  height: 100vh;
  width: 100vw;
  display: flex;
  flex-direction: column;
}

.title-bar {
  height: 3.2rem;
  padding: 0 0.8rem;
  flex-shrink: 0;
  display: flex;
  justify-content: flex-end;
  align-items: center;
  border-radius: 0; /* Override default pane radius */
  border-top: none;
  border-left: none;
  border-right: none;
  background-color: var(--color-background-secondary);
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

.main-content {
  flex-grow: 1;
  display: flex;
  justify-content: center;
  align-items: center;
  border-radius: 0; /* Override default pane radius */
  border: none;
}

.main-content h1 {
  font-size: 4.8rem;
  color: var(--color-text-secondary);
}
</style>
