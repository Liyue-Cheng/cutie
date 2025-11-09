<template>
  <div class="cute-menu-divider" />
</template>

<script setup lang="ts">
/**
 * CuteMenuDivider - 菜单分隔线
 * 
 * 使用语义化颜色，不使用 margin 避免 border 重叠问题
 */
</script>

<style scoped>
.cute-menu-divider {
  height: 1px;
  background-color: var(--color-border-soft, rgb(87 82 121 / 8%));
  
  /* 不使用 margin，依赖父容器的 gap */
  margin: 0;
  
  /* 防止被 flex gap 压缩 */
  flex-shrink: 0;
}
</style>

