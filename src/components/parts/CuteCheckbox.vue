<script setup lang="ts">
// Custom checkbox component with circular style
// This component forwards all native checkbox attributes and events

interface Props {
  checked?: boolean
  size?: 'small' | 'large'
  variant?: 'check' | 'star' // check: 对钩(绿色), star: 星星(蓝色)
}

withDefaults(defineProps<Props>(), {
  checked: false,
  size: 'small',
  variant: 'check',
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
  <label class="cute-checkbox" :class="[`size-${size}`, `variant-${variant}`]">
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
  width: 1.913rem;
  height: 1.913rem;
  border: 0.2rem solid #d9d9d9;
  border-radius: 50%;
  background-color: transparent;
  transition: all 0.2s ease-in-out;
}

/* 对钩始终显示 - 未选中时为灰色 */
.checkmark::after {
  content: '';
  position: absolute;
  left: 0.584rem;
  top: 0.266rem;
  width: 0.425rem;
  height: 0.85rem;
  border: solid #d9d9d9;
  border-width: 0 0.2rem 0.2rem 0;
  transform: rotate(45deg);
  transition: all 0.2s ease-in-out;
}

/* 鼠标悬停：灰色图案变绿色 */
.cute-checkbox:hover .checkmark {
  border-color: var(--color-status-done);
}

.cute-checkbox:hover .checkmark::after {
  border-color: var(--color-status-done);
}

/* Large size variant */
.cute-checkbox.size-large .checkmark {
  width: 2.55rem;
  height: 2.55rem;
}

.cute-checkbox.size-large .checkmark::after {
  left: 0.85rem;
  top: 0.425rem;
  width: 0.531rem;
  height: 1.063rem;
}

/* ===============================================
 * 星星变体样式
 * =============================================== */

/* 星星变体：使用星星符号代替对钩 */
.cute-checkbox.variant-star .checkmark::after {
  content: '★';
  border: none;
  color: #d9d9d9;
  font-size: 1.2rem;
  line-height: 1;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  width: auto;
  height: auto;
}

/* 星星变体 Large size */
.cute-checkbox.variant-star.size-large .checkmark::after {
  font-size: 1.6rem;
}

/* 星星变体：鼠标悬停变蓝色 */
.cute-checkbox.variant-star:hover .checkmark {
  border-color: var(--color-primary, #4a90e2);
}

.cute-checkbox.variant-star:hover .checkmark::after {
  color: var(--color-primary, #4a90e2);
}

/* ===============================================
 * 状态样式（放在最后以确保正确的特异性）
 * =============================================== */

/* 选中状态：确认为绿色 */
.cute-checkbox input[type='checkbox']:checked ~ .checkmark {
  border-color: var(--color-status-done);
}

.cute-checkbox input[type='checkbox']:checked ~ .checkmark::after {
  border-color: var(--color-status-done);
}

/* Disabled state */
.cute-checkbox input[type='checkbox']:disabled ~ .checkmark {
  border-color: #d9d9d9;
  cursor: not-allowed;
  opacity: 0.5;
}

.cute-checkbox input[type='checkbox']:disabled ~ .checkmark::after {
  border-color: #d9d9d9;
}

/* 星星变体：选中状态为蓝色 */
.cute-checkbox.variant-star input[type='checkbox']:checked ~ .checkmark {
  border-color: var(--color-primary, #4a90e2);
}

.cute-checkbox.variant-star input[type='checkbox']:checked ~ .checkmark::after {
  color: var(--color-primary, #4a90e2);
}

/* 星星变体：禁用状态 */
.cute-checkbox.variant-star input[type='checkbox']:disabled ~ .checkmark::after {
  color: #d9d9d9;
}
</style>
