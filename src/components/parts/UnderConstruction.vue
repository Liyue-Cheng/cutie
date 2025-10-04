<script setup lang="ts">
defineProps<{
  title: string
  description?: string
}>()
</script>

<template>
  <div class="under-construction">
    <div class="content">
      <div class="icon">🚧</div>
      <h2 class="title">{{ title }}</h2>
      <p v-if="description" class="description">{{ description }}</p>
      <p class="status">功能开发中...</p>
    </div>
  </div>
</template>

<style scoped>
.under-construction {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  width: 100%;
  background-color: var(--color-background-content);
  padding: 2rem;
}

.content {
  text-align: center;
  max-width: 40rem;
}

.icon {
  font-size: 6rem;
  margin-bottom: 1.5rem;
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%,
  100% {
    transform: scale(1);
    opacity: 1;
  }

  50% {
    transform: scale(1.05);
    opacity: 0.8;
  }
}

.title {
  font-size: 2.4rem;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0 0 1rem;
}

.description {
  font-size: 1.5rem;
  color: var(--color-text-secondary);
  margin: 0 0 1rem;
  line-height: 1.6;
}

.status {
  font-size: 1.3rem;
  color: var(--color-text-tertiary);
  margin: 0;
  font-style: italic;
}
</style>
