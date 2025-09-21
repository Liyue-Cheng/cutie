<template>
  <div class="cute-pane">
    <slot />
  </div>
</template>

<script setup lang="ts">
/**
 * @description An embedded "functional canvas" or "workspace".
 * It fills the space of its parent container and provides a consistent background and padding.
 * It has no margins or shadows, designed to be seamlessly embedded.
 */
</script>

<style scoped>
.cute-pane {
  width: 100%;
  height: 100%;
}
</style>
