<script setup lang="ts">
// 过去和现在/未来的分割线组件
</script>

<template>
  <div class="timeline-divider">
    <div class="divider-line"></div>
    <div class="divider-label">
      <span>现在</span>
    </div>
    <div class="divider-line"></div>
  </div>
</template>

<style scoped>
.timeline-divider {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1.5rem 0;
  margin: 0.5rem 0;
}

.divider-line {
  flex: 1;
  height: 3px;
  border-top: 3px dashed var(--color-primary, #4a90e2);
  opacity: 0.8;
}

.divider-label {
  background-color: var(--color-background-content);
  padding: 0.5rem 1rem;
  border-radius: 1rem;
  border: 2px solid var(--color-primary, #4a90e2);
  color: var(--color-primary, #4a90e2);
  font-weight: 600;
  font-size: 1.2rem;
  white-space: nowrap;
}
</style>