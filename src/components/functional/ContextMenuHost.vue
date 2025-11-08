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
import { ref, watch, nextTick } from 'vue'

const contextMenu = useContextMenu()
const menuHostRef = ref<HTMLElement | null>(null)
const adjustedX = ref(0)
const adjustedY = ref(0)

watch(
  () => contextMenu.state.value,
  async (menuState) => {
    if (!menuState.show) {
      return
    }

    // 使用原始位置初始化
    adjustedX.value = menuState.x
    adjustedY.value = menuState.y

    await nextTick()

    const host = menuHostRef.value
    if (!host) {
      return
    }

    const rect = host.getBoundingClientRect()
    const PADDING = 8

    let x = menuState.x
    let y = menuState.y

    const maxX = window.innerWidth - rect.width - PADDING
    const maxY = window.innerHeight - rect.height - PADDING

    if (x > maxX) {
      x = Math.max(PADDING, maxX)
    }

    if (y > maxY) {
      y = Math.max(PADDING, maxY)
    }

    if (x < PADDING) {
      x = PADDING
    }

    if (y < PADDING) {
      y = PADDING
    }

    adjustedX.value = x
    adjustedY.value = y
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
