<template>
  <button class="cute-button">
    <slot />
  </button>
</template>
/** * @description The base button component for all interactions. */

<script lang="ts" setup>
// No script logic needed for this basic component yet.
</script>

<style scoped>
.cute-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0.8rem 1.6rem;
  border-radius: 0.8rem;
  font-weight: 500;
  font-size: 1.4rem;
  line-height: 1;
  cursor: pointer;
  transition: all 0.2s ease-in-out;
  border: 1px solid var(--color-border);
  background-color: var(--color-background-soft);
  color: var(--color-text);
}

.cute-button:hover {
  background-color: var(--color-background-mute);
  border-color: var(--color-primary);
}

.cute-button:active {
  transform: scale(0.98);
}
</style>
