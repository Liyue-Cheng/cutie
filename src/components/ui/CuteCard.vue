<template>
  <div class="cute-card">
    <slot />
  </div>
</template>
/** * @description A container for content with a distinct visual style, including a shadow. */

<script lang="ts" setup>
// No script logic needed for this basic component yet.
</script>

<style scoped>
.cute-card {
  padding: 1.6rem;
  border-radius: 1.2rem;
  background-color: var(--color-background-soft);
  border: 1px solid var(--color-border);
  box-shadow:
    0 4px 6px -1px var(--color-shadow-a),
    0 2px 4px -2px var(--color-shadow-b);
}
</style>
