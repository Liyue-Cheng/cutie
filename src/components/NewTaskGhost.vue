<template>
  <div class="new-task-ghost">
    <span class="item-icon">📋</span>
    <span class="task-title">{{ title }}</span>
  </div>
</template>

<script setup lang="ts">
interface Props {
  title?: string
  id?: number
  createdAt?: string
}

withDefaults(defineProps<Props>(), {
  title: '新任务',
  id: 0,
  createdAt: '',
})
</script>

<style scoped>
.new-task-ghost {
  padding: 12px 16px;
  background: linear-gradient(135deg, #007acc, #005a9e);
  color: white;
  border-radius: 8px;
  display: flex;
  align-items: center;
  gap: 10px;
  box-shadow: 0 2px 8px rgb(0 0 0 / 15%);
  border: 1px solid rgb(255 255 255 / 20%);
  font-weight: 500;
  min-width: 180px;
  user-select: none;

  /* 新任务的特殊效果 */
  animation: new-task-pulse 0.3s ease-out;
}

.item-icon {
  font-size: 16px;
  flex-shrink: 0;
}

.task-title {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

@keyframes new-task-pulse {
  0% {
    transform: scale(0.9);
    opacity: 0.7;
  }

  50% {
    transform: scale(1.05);
  }

  100% {
    transform: scale(1);
    opacity: 1;
  }
}
</style>
