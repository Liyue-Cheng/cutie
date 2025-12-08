<template>
  <button
    class="cute-menu-item"
    :class="{
      'is-danger': variant === 'danger',
      'is-disabled': disabled,
      'has-divider': divider,
    }"
    :disabled="disabled"
    @click="handleClick"
  >
    <CuteIcon v-if="icon" :name="icon" :size="iconSize" />
    <span class="menu-item-text"><slot /></span>
  </button>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import type { IconName } from '@/types/icons'

const props = withDefaults(
  defineProps<{
    icon?: IconName
    iconSize?: number
    variant?: 'default' | 'danger'
    disabled?: boolean
    divider?: boolean
  }>(),
  {
    iconSize: 14,
    variant: 'default',
    disabled: false,
    divider: false,
  }
)

const emit = defineEmits(['click'])

function handleClick(event: MouseEvent) {
  if (!props.disabled) {
    emit('click', event)
  }
}
</script>

<style scoped>
.cute-menu-item {
  /* 重置 */
  all: unset;
  box-sizing: border-box;

  /* 布局 */
  display: flex;
  align-items: center;
  gap: 0.8rem;
  padding: 0.8rem 1.2rem;
  width: 100%;

  /* 外观 */
  background-color: transparent;
  border-radius: 0.6rem;
  color: var(--color-text-primary, #575279);
  font-size: 1.4rem;
  font-weight: 500;
  text-align: left;
  cursor: pointer;

  /* 动画 */
  transition:
    background-color 0.15s ease,
    color 0.15s ease;
}

.cute-menu-item:hover:not(.is-disabled) {
  background-color: var(--color-background-hover, rgb(87 82 121 / 5%));
}

.cute-menu-item:active:not(.is-disabled) {
  background-color: var(--color-background-active, rgb(87 82 121 / 8%));
}

/* 危险操作样式 */
.cute-menu-item.is-danger {
  color: var(--color-danger, #b4637a);
}

.cute-menu-item.is-danger:hover:not(.is-disabled) {
  background-color: var(--color-danger-bg, rgb(180 99 122 / 8%));
}

.cute-menu-item.is-danger:active:not(.is-disabled) {
  background-color: var(--color-danger-bg-active, rgb(180 99 122 / 12%));
}

/* 禁用状态 */
.cute-menu-item.is-disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 分隔线 - 使用 border-top 替代独立的 divider 组件 */
.cute-menu-item.has-divider {
  margin-top: 0.4rem;
  padding-top: calc(0.8rem + 0.4rem); /* 原 padding + 分隔线间距 */
  border-top: 1px solid var(--color-border-default, #f0f);
}

.menu-item-text {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
