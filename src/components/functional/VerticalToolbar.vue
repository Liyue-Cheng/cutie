<template>
  <div class="toolbar-pane">
    <div class="toolbar-content">
      <!-- 视图切换按钮 -->
      <button
        v-for="(config, viewKey) in viewConfig"
        :key="viewKey"
        class="toolbar-button"
        :class="{ active: currentView === viewKey }"
        :title="config.label"
        @click="$emit('view-change', viewKey)"
      >
        <CuteIcon :name="config.icon" :size="24" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import CuteIcon from '@/components/parts/CuteIcon.vue'

// Props
interface ViewConfig {
  icon: string
  label: string
}

interface Props {
  viewConfig: Record<string, ViewConfig>
  currentView: string
}

defineProps<Props>()

// Emits
defineEmits<{
  'view-change': [viewKey: string]
}>()
</script>

<style scoped>
/* 右侧垂直图标栏 */
.toolbar-pane {
  width: 6rem; /* 96px */
  min-width: 6rem;
  display: flex;
  flex-direction: column;
  background-color: transparent;
  border-left: 1px solid var(--color-border-light, #f0f);
}

.toolbar-content {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 1rem 0;
  gap: 0.5rem;
  overflow-y: auto;
  scrollbar-width: none;
  position: relative;
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
  border: none;
  border-radius: 0.8rem;
  cursor: pointer;
  transition: all 0.2s ease;
  color: var(--color-text-tertiary, #f0f);
  position: relative;
  flex-shrink: 0;
}

.toolbar-button:hover {
  background-color: var(--color-background-hover, #f0f);
  color: var(--color-text-secondary, #f0f);
}

/* 激活状态 */
.toolbar-button.active {
  background-color: var(--color-button-primary-bg, #f0f);
  color: var(--color-button-primary-text, #f0f);
}

.toolbar-button:active {
  transform: scale(0.95);
}
</style>
