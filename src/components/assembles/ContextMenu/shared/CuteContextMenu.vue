<template>
  <div class="cute-context-menu">
    <slot />
  </div>
</template>

<script setup lang="ts">
/**
 * CuteContextMenu - 统一的右键菜单容器组件
 * 
 * 特性：
 * - 使用语义化颜色变量
 * - 统一的阴影和圆角
 * - 防止 border 重叠的视觉问题
 * 
 * 使用方式：
 * <CuteContextMenu>
 *   <CuteMenuItem @click="handleAction">操作1</CuteMenuItem>
 *   <CuteMenuDivider />
 *   <CuteMenuItem variant="danger" @click="handleDelete">删除</CuteMenuItem>
 * </CuteContextMenu>
 */
</script>

<style scoped>
.cute-context-menu {
  /* 布局 */
  display: flex;
  flex-direction: column;
  min-width: 16rem;
  padding: 0.4rem;
  gap: 0.2rem; /* 使用 gap 替代 margin，避免 border 重叠 */
  
  /* 外观 */
  background-color: var(--color-background-content, #faf4ed);
  border: 1px solid var(--color-border-default, #dfdad9);
  border-radius: 0.8rem;
  box-shadow: var(--shadow-md);
  
  /* 防止内容溢出 */
  overflow: hidden;
}
</style>

