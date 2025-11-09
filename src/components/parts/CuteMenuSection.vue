<template>
  <div class="cute-menu-section">
    <div v-if="title" class="menu-section-title">{{ title }}</div>
    <slot />
  </div>
</template>

<script setup lang="ts">
/**
 * CuteMenuSection - 菜单分组容器
 *
 * 用于在菜单中创建带标题的分组
 */
defineProps<{
  title?: string
}>()
</script>

<style scoped>
.cute-menu-section {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}

.menu-section-title {
  padding: 0.4rem 1.2rem;
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--color-text-tertiary, #9893a5);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  line-height: 1.4;
}
</style>
