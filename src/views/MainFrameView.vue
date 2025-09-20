<template>
  <div class="main-frame">
    <div class="title-bar" data-tauri-drag-region>
      <div class="window-controls">
        <button class="control-btn" @click="appWindow.minimize()">—</button>
        <button class="control-btn" @click="appWindow.toggleMaximize()">口</button>
        <button class="control-btn" @click="appWindow.close()">X</button>
      </div>
    </div>
    <div class="main-content">
      <h1>Cutie App</h1>
    </div>
  </div>
</template>

<script setup lang="ts">
// 1. 从 '@tauri-apps/api/window' 这个子模块导入 getCurrentWindow 函数
import { getCurrentWindow } from '@tauri-apps/api/window'
// 2. 调用函数来获取当前窗口的实例
const appWindow = getCurrentWindow()
// 3. 在模板中就可以直接使用 appWindow.close() 等方法了
</script>

<style scoped>
.main-frame {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  background-color: var(--color-background);
}

.title-bar {
  height: 3rem;
  background-color: var(--color-background-secondary);
  display: flex;
  justify-content: flex-end;
  align-items: center;
  padding: 0 1rem;
  flex-shrink: 0;
}

.window-controls {
  display: flex;
  gap: 0.5rem;
  /* This is crucial to allow clicks to pass through the drag region */
  -webkit-app-region: no-drag;
}

.control-btn {
  background: none;
  border: none;
  color: var(--color-text);
  font-size: 1.4rem;
  padding: 0.5rem 1rem;
  cursor: pointer;
  border-radius: 0.4rem;
}

.control-btn:hover {
  background-color: var(--color-border-default);
}

.main-content {
  flex-grow: 1;
  display: flex;
  justify-content: center;
  align-items: center;
}

.main-content h1 {
  font-size: 4.8rem;
  color: var(--color-text-secondary);
}
</style>
