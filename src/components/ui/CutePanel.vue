<template>
  <div class="cute-panel">
    <slot />
  </div>
</template>
/** * @description A simple rectangular container, often used to group elements. */

<script setup lang="ts">
// No script logic needed for this basic component.
</script>

<style scoped>
.cute-panel {
  background-color: var(--color-background-soft);
  border-radius: 1.2rem;
  border: 1px solid var(--color-border);
}
</style>
