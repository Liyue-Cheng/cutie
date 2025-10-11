<script setup lang="ts">
// Custom checkbox component with circular style
// This component forwards all native checkbox attributes and events
// Supports both preset sizes ('small', 'large') and custom rem values (e.g., '1.6rem')

import { computed } from 'vue'

interface Props {
  checked?: boolean
  size?: 'small' | 'large' | string // ✅ 支持预设或自定义尺寸
  variant?: 'check' | 'star' // check: 对钩(绿色), star: 星星(蓝色)
}

const props = withDefaults(defineProps<Props>(), {
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

// ✅ 判断是否使用预设尺寸
const isPresetSize = computed(() => props.size === 'small' || props.size === 'large')

// ✅ 自定义尺寸的 CSS 变量
const customSizeStyle = computed(() => {
  if (isPresetSize.value) return {}
  return {
    '--checkbox-size': props.size,
  }
})
</script>

<template>
  <label
    class="cute-checkbox"
    :class="[isPresetSize ? `size-${size}` : 'size-custom', `variant-${variant}`]"
    :style="customSizeStyle"
  >
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
  width: 1.6rem;
  height: 1.6rem;
  border: 0.15rem solid #d9d9d9;
  border-radius: 50%;
  background-color: transparent;
  transition: all 0.2s ease-in-out;
}

/* 对钩始终显示 - 未选中时为灰色 */
.checkmark::after {
  content: '';
  position: absolute;
  left: 0.5rem;
  top: 0.22rem;
  width: 0.35rem;
  height: 0.7rem;
  border: solid #d9d9d9;
  border-width: 0 0.15rem 0.15rem 0;
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
  width: 2.1rem;
  height: 2.1rem;
}

.cute-checkbox.size-large .checkmark::after {
  left: 0.7rem;
  top: 0.35rem;
  width: 0.44rem;
  height: 0.88rem;
}

/* ✅ Custom size variant - 使用 CSS 变量 */
.cute-checkbox.size-custom .checkmark {
  width: var(--checkbox-size);
  height: var(--checkbox-size);
}

.cute-checkbox.size-custom .checkmark::after {
  /* 根据自定义尺寸动态计算对钩位置和大小 */
  left: calc(var(--checkbox-size) * 0.3125);
  top: calc(var(--checkbox-size) * 0.1375);
  width: calc(var(--checkbox-size) * 0.2188);
  height: calc(var(--checkbox-size) * 0.4375);
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
  font-size: 1.3rem;
}

/* 星星变体 Custom size */
.cute-checkbox.variant-star.size-custom .checkmark::after {
  font-size: calc(var(--checkbox-size) * 0.75);
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
