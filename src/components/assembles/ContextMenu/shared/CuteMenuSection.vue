<template>
  <div class="cute-menu-section" :class="{ 'has-divider': divider }">
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
withDefaults(
  defineProps<{
    title?: string
    divider?: boolean
  }>(),
  {
    divider: false,
  }
)
</script>

<style scoped>
.cute-menu-section {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}

/* 分隔线 - 使用 border-top */
.cute-menu-section.has-divider {
  margin-top: 0.4rem;
  padding-top: 0.4rem;
  border-top: 1px solid var(--color-border-default, #f0f);
}

.menu-section-title {
  padding: 0.4rem 1.2rem;
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--color-text-tertiary, #f0f);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  line-height: 1.4;
}
</style>
