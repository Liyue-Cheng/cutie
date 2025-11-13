<template>
  <div class="cute-dropdown-divider" :class="variant">
    <span v-if="label" class="divider-label">{{ label }}</span>
  </div>
</template>

<script setup lang="ts">
interface Props {
  /** 分割线标签（可选） */
  label?: string
  /** 变体样式 */
  variant?: 'default' | 'strong'
}

withDefaults(defineProps<Props>(), {
  variant: 'default',
})
</script>

<style scoped>
.cute-dropdown-divider {
  position: relative;
  height: 1px;
  margin: 0.6rem 0.8rem;
  background-color: var(--color-border-light);
}

.cute-dropdown-divider.strong {
  background-color: var(--color-border-default);
}

/* 带标签的分割线 */
.divider-label {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  padding: 0 0.8rem;
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--color-text-tertiary);
  background-color: var(--color-background-content);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  white-space: nowrap;
}
</style>
