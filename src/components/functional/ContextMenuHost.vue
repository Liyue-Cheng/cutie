<template>
  <div
    v-if="contextMenu.state.value.show"
    ref="menuHostRef"
    class="context-menu-host"
    :style="{ top: `${adjustedY}px`, left: `${adjustedX}px` }"
  >
    <component
      :is="contextMenu.state.value.component"
      v-if="contextMenu.state.value.component"
      v-bind="contextMenu.state.value.props"
      @close="contextMenu.hide()"
    />
  </div>
</template>

<script setup lang="ts">
import { useContextMenu } from '@/composables/useContextMenu'
import { ref, computed, watch, nextTick } from 'vue'

const contextMenu = useContextMenu()
const menuHostRef = ref<HTMLElement | null>(null)
const adjustedX = ref(0)
const adjustedY = ref(0)

// 监听菜单状态变化，根据实际尺寸调整位置
watch(
  () => contextMenu.state.value.show,
  async (show) => {
    if (show) {
      // 先使用原始位置
      adjustedX.value = contextMenu.state.value.x
      adjustedY.value = contextMenu.state.value.y

      // 等待 DOM 更新后获取实际尺寸
      await nextTick()
      
      if (menuHostRef.value) {
        const rect = menuHostRef.value.getBoundingClientRect()
        const PADDING = 8 // 距离边缘的安全距离

        let x = contextMenu.state.value.x
        let y = contextMenu.state.value.y

        // 检查右边缘
        if (x + rect.width + PADDING > window.innerWidth) {
          x = window.innerWidth - rect.width - PADDING
        }

        // 检查底部边缘
        if (y + rect.height + PADDING > window.innerHeight) {
          y = window.innerHeight - rect.height - PADDING
        }

        // 检查左边缘
        if (x < PADDING) {
          x = PADDING
        }

        // 检查顶部边缘
        if (y < PADDING) {
          y = PADDING
        }

        adjustedX.value = x
        adjustedY.value = y
      }
    }
  },
  { immediate: true }
)
</script>

<style scoped>
.context-menu-host {
  position: fixed;
  z-index: 9999; /* Ensure it's on top of other content */
}
</style>
