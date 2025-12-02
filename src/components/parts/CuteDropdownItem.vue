<template>
  <button
    class="cute-dropdown-item"
    :class="{
      active,
      disabled,
      danger: variant === 'danger',
    }"
    :disabled="disabled"
    @click="handleClick"
  >
    <!-- 左侧图标 -->
    <CuteIcon v-if="icon" :name="icon" :size="iconSize" class="item-icon" />

    <!-- 主内容 -->
    <span class="item-content">
      <slot>{{ label }}</slot>
    </span>

    <!-- 右侧内容 -->
    <span v-if="$slots.suffix || suffix" class="item-suffix">
      <slot name="suffix">
        <span class="suffix-text">{{ suffix }}</span>
      </slot>
    </span>

    <!-- 选中标记 -->
    <CuteIcon v-if="active && showCheck" name="Check" :size="16" class="item-check" />
  </button>
</template>

<script setup lang="ts">
import CuteIcon from './CuteIcon.vue'
import type { IconName } from '@/types/icons'

interface Props {
  /** 选项标签 */
  label?: string
  /** 左侧图标名称 */
  icon?: IconName
  /** 图标大小 */
  iconSize?: number
  /** 右侧后缀文本 */
  suffix?: string
  /** 是否激活（选中） */
  active?: boolean
  /** 是否禁用 */
  disabled?: boolean
  /** 是否显示选中标记 */
  showCheck?: boolean
  /** 变体样式 */
  variant?: 'default' | 'danger'
}

withDefaults(defineProps<Props>(), {
  iconSize: 16,
  active: false,
  disabled: false,
  showCheck: true,
  variant: 'default',
})

const emit = defineEmits<{
  click: []
}>()

function handleClick() {
  emit('click')
}
</script>

<style scoped>
.cute-dropdown-item {
  display: flex;
  align-items: center;
  gap: 1rem;
  width: 100%;
  min-height: 3.6rem;
  padding: 0.8rem 1.2rem;
  font-size: 1.4rem;
  color: var(--color-text-primary);
  background-color: transparent;
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  text-align: left;
  transition: all 0.15s ease;
}

.cute-dropdown-item:hover:not(.disabled) {
  background-color: var(--color-background-hover);
}

.cute-dropdown-item.active {
  background-color: var(--color-background-selected);
  color: var(--color-text-accent);
  font-weight: 500;
}

.cute-dropdown-item.disabled {
  opacity: var(--opacity-disabled);
  cursor: not-allowed;
}

/* 图标 */
.item-icon {
  flex-shrink: 0;
  color: var(--color-text-secondary);
}

.cute-dropdown-item.active .item-icon {
  color: var(--color-text-accent);
}

/* 危险样式变体 */
.cute-dropdown-item.danger {
  color: var(--color-danger-text);
}

.cute-dropdown-item.danger:hover:not(.disabled) {
  background-color: var(--color-danger-light);
}

.cute-dropdown-item.danger .item-icon {
  color: var(--color-danger-text);
}

/* 主内容 */
.item-content {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 后缀 */
.item-suffix {
  flex-shrink: 0;
  margin-left: auto;
}

.suffix-text {
  font-size: 1.2rem;
  color: var(--color-text-tertiary);
}

/* 选中标记 */
.item-check {
  flex-shrink: 0;
  color: var(--color-text-accent);
}
</style>
