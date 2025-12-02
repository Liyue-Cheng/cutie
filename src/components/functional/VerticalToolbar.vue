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
        @click="handleClick(viewKey)"
      >
        <CuteIcon :name="config.icon" :size="24" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import type { IconName } from '@/types/icons'

// Props
interface ViewConfig {
  icon: IconName
  label: string
}

interface Props {
  viewConfig: Record<string, ViewConfig>
  currentView: string | null // null 表示收起状态
  allowCollapse?: boolean // 是否允许再次点击收起，默认 false
  defaultView?: string | null // 默认视图，null 表示默认收起（仅 allowCollapse 为 true 时有效）
}

const props = withDefaults(defineProps<Props>(), {
  allowCollapse: false,
  defaultView: undefined, // undefined 表示使用第一个视图
})

// Emits
const emit = defineEmits<{
  'view-change': [viewKey: string | null]
}>()

// 计算第一个视图 key
const firstViewKey = computed(() => {
  const keys = Object.keys(props.viewConfig)
  return keys.length > 0 ? keys[0] : null
})

// 计算实际的默认视图
const resolvedDefaultView = computed(() => {
  if (props.defaultView !== undefined) {
    // 如果不允许收起，defaultView 不能为 null
    if (!props.allowCollapse && props.defaultView === null) {
      return firstViewKey.value
    }
    return props.defaultView
  }
  return firstViewKey.value
})

// 点击处理
function handleClick(viewKey: string) {
  if (props.currentView === viewKey) {
    // 再次点击已激活图标
    if (props.allowCollapse) {
      emit('view-change', null) // 收起
    }
    // 不允许收起时不触发事件
  } else {
    emit('view-change', viewKey) // 切换视图
  }
}

// 初始化时触发默认视图
onMounted(() => {
  // 如果当前没有选中视图，触发默认视图
  const defaultView = resolvedDefaultView.value
  if (props.currentView === null && defaultView !== null && defaultView !== undefined) {
    emit('view-change', defaultView)
  }
})
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
