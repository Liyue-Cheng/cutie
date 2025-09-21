<script setup lang="ts">
import { NCheckbox } from 'naive-ui'

// This component will forward all props and events to the Naive UI checkbox.
// This is a simple wrapper for style consistency and potential future customization.
</script>

<template>
  <NCheckbox v-bind="$attrs" />
</template>

<style scoped>
/* Add any custom styles for the checkbox wrapper here if needed. */

/* For example, to make the click area larger or adjust margins. */
</style>
