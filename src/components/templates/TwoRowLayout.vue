<template>
  <div class="two-row-layout">
    <div class="top-row">
      <slot name="top"></slot>
    </div>
    <div class="bottom-row">
      <slot name="bottom"></slot>
    </div>
  </div>
</template>

<style scoped>
.two-row-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.top-row {
  height: 5.5rem; /* 固定高度，确保所有分割线对齐 */
  flex-shrink: 0;
  display: flex;
  align-items: center;
  padding: 0 1rem;
}

.bottom-row {
  flex-grow: 1;
  min-height: 0; /* 关键：允许flex子元素收缩 */
  overflow-y: auto;

  /* scrollbar-gutter: stable; */
}
</style>
