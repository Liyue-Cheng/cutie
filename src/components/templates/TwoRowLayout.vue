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
  padding: 0.5rem;
  border-bottom: 1px solid var(--color-border-default);
}

.bottom-row {
  flex-grow: 1;
  padding: 1rem;
  overflow-y: auto;

  /* scrollbar-gutter: stable; */
}
</style>
