<template>
  <div class="cute-surface">
    <slot />
  </div>
</template>
/** * @description A background area for hosting content and interactive elements. */

<script setup lang="ts">
// No script logic needed for this basic component.
</script>

<style scoped>
.cute-surface {
  background-color: var(--color-background);
  padding: 2.4rem;
  width: 100%;
  height: 100%;
}
</style>
