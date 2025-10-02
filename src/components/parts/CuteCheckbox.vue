<script setup lang="ts">
// Custom checkbox component with circular style
// This component forwards all native checkbox attributes and events

interface Props {
  checked?: boolean
  size?: 'small' | 'large'
}

withDefaults(defineProps<Props>(), {
  checked: false,
  size: 'small',
})

const emit = defineEmits<{
  'update:checked': [value: boolean]
}>()

const handleChange = (event: Event) => {
  const target = event.target as HTMLInputElement
  emit('update:checked', target.checked)
}
</script>

<template>
  <label class="cute-checkbox" :class="[`size-${size}`]">
    <input type="checkbox" :checked="checked" @change="handleChange" />
    <span class="checkmark"></span>
  </label>
</template>

<style scoped>
.cute-checkbox {
  position: relative;
  display: inline-flex;
  align-items: center;
  cursor: pointer;
  user-select: none;
}

.cute-checkbox input[type='checkbox'] {
  position: absolute;
  opacity: 0;
  cursor: pointer;
  height: 0;
  width: 0;
}

.checkmark {
  position: relative;
  display: inline-block;
  width: 18px;
  height: 18px;
  border: 2px solid #d9d9d9;
  border-radius: 50%;
  background-color: #fff;
  transition: all 0.2s ease-in-out;
}

.cute-checkbox:hover .checkmark {
  border-color: var(--color-status-done);
}

/* Large size variant - must come before checked state */
.cute-checkbox.size-large .checkmark {
  width: 24px;
  height: 24px;
}

/* Checked state */
.cute-checkbox input[type='checkbox']:checked ~ .checkmark {
  background-color: var(--color-status-done);
  border-color: var(--color-status-done);
}

.cute-checkbox input[type='checkbox']:checked ~ .checkmark::after {
  content: '';
  position: absolute;
  left: 5px;
  top: 2px;
  width: 4px;
  height: 8px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

/* Disabled state */
.cute-checkbox input[type='checkbox']:disabled ~ .checkmark {
  background-color: #f5f5f5;
  border-color: #d9d9d9;
  cursor: not-allowed;
}

/* Large size with checked state - must come last for specificity */
.cute-checkbox.size-large input[type='checkbox']:checked ~ .checkmark::after {
  left: 7px;
  top: 3px;
  width: 5px;
  height: 10px;
}
</style>
