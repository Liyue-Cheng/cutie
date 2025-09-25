<script setup lang="ts">
import { NCheckbox } from 'naive-ui'

// This component will forward all props and events to the Naive UI checkbox.
// This is a simple wrapper for style consistency and potential future customization.
</script>

<template>
  <NCheckbox v-bind="$attrs" :focusable="false" />
</template>

<style>
/* Add any custom styles for the checkbox wrapper here if needed. */
.n-checkbox .n-checkbox-box {
  border-radius: 50%;
}

/* stylelint-disable-next-line selector-class-pattern */
.n-checkbox.n-checkbox--checked .n-checkbox-box {
  background-color: var(--color-status-done);
  border-color: var(--color-status-done);
}

/* stylelint-disable-next-line selector-class-pattern */
.n-checkbox.n-checkbox--checked .n-checkbox-box .n-checkbox-box__icon {
  background-color: var(--color-status-done);
}

/* For example, to make the click area larger or adjust margins. */
</style>
