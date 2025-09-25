<template>
  <div
    v-if="contextMenu.state.value.show"
    class="context-menu-host"
    :style="{ top: `${contextMenu.state.value.y}px`, left: `${contextMenu.state.value.x}px` }"
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

const contextMenu = useContextMenu()
</script>

<style scoped>
.context-menu-host {
  position: fixed;
  z-index: 9999; /* Ensure it's on top of other content */
}
</style>
