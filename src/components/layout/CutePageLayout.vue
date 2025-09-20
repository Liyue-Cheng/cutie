<template>
  <div class="cute-page-layout">
    <aside class="cute-page-layout__sidebar">
      <slot name="sidebar" />
    </aside>
    <main class="cute-page-layout__main">
      <slot />
    </main>
  </div>
</template>

<script lang="ts" setup>
// No script logic needed for this basic component yet.
</script>

<style scoped>
.cute-page-layout {
  display: flex;
  height: 100vh;
  width: 100vw;
  background-color: var(--color-background);
}

.cute-page-layout__sidebar {
  width: 24rem;
  padding: 1.6rem;
  border-right: 1px solid var(--color-border);
  flex-shrink: 0;
}

.cute-page-layout__main {
  flex-grow: 1;
  padding: 1.6rem;
  overflow-y: auto;
}
</style>