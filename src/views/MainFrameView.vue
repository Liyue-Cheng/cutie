<template>
  <div class="main-frame" data-tauri-drag-region>
    <div class="title-bar">
      <div class="window-controls">
        <button class="control-btn">—</button>
        <button class="control-btn">口</button>
        <button class="control-btn">X</button>
      </div>
    </div>
    <div class="main-content">
      <h1>Cutie App</h1>
    </div>
  </div>
</template>

<script setup lang="ts">
// No script logic needed for this basic component yet.
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
  background-color: var(--color-background-mute);
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