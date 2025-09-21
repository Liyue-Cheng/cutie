<template>
  <div class="cute-page-layout">
    <aside class="cute-page-layout-sidebar">
      <slot name="sidebar" />
    </aside>
    <main class="cute-page-layout-main">
      <slot />
    </main>
  </div>
</template>

<script lang="ts" setup>
/**
 * @description A structural component for macro page layout.
 * It defines main functional areas like a sidebar and a main content area.
 * It has no visual styling itself (no color, padding, or borders).
 */
</script>

<style scoped>
.cute-page-layout {
  display: flex;
  height: 100%;
  width: 100%;
}

.cute-page-layout-sidebar {
  width: 24rem;
  flex-shrink: 0;
  height: 100%;
}

.cute-page-layout-main {
  flex-grow: 1;
  height: 100%;
  overflow-y: auto;
}
</style>
