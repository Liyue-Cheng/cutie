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
      <!-- AI 聊天按钮 (可选，置底) -->
      <button
        v-if="showAiButton"
        class="toolbar-button ai-button"
        title="AI 助手"
        @click="$emit('ai-click')"
      >
        <CuteIcon name="Sparkles" :size="24" />
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
  showAiButton?: boolean
}

defineProps<Props>()

// Emits
defineEmits<{
  'view-change': [viewKey: string]
  'ai-click': []
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
  border-left: 1px solid var(--color-border-default);
  border-radius: 0 0.8rem 0.8rem 0;
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
  color: var(--color-text-tertiary);
  position: relative;
  flex-shrink: 0;
}

.toolbar-button:hover {
  background-color: var(--color-background-hover, rgb(0 0 0 / 5%));
  color: var(--color-text-secondary);
}

/* 激活状态 */
.toolbar-button.active {
  background-color: var(--rose-pine-foam, #56949f);
  color: var(--rose-pine-base, #faf4ed);
}

.toolbar-button:active {
  transform: scale(0.95);
}

/* AI 按钮特殊样式 */
.toolbar-button.ai-button {
  background-color: var(--rose-pine-iris, #907aa9);
  color: var(--rose-pine-base, #faf4ed);
  position: absolute;
  bottom: 1rem;
}

.toolbar-button.ai-button:hover {
  background-color: var(--rose-pine-love, #b4637a);
  transform: scale(1.05);
}
</style>
