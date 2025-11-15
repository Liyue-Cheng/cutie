<script setup lang="ts">
/**
 * CellItemDeadline - 时间线单元格截止日期项
 *
 * 简约设计：只显示旗子图标和任务标题，通过图标颜色区分状态
 */
import type { TaskCard } from '@/types/dtos'
import CuteIcon from '@/components/parts/CuteIcon.vue'

interface Props {
  task: TaskCard
}

const props = defineProps<Props>()

const emit = defineEmits<{
  click: []
}>()

function handleClick() {
  emit('click')
}
</script>

<template>
  <div
    class="cell-item-deadline"
    :class="{ 'is-overdue': task.due_date?.is_overdue }"
    @click="handleClick"
  >
    <div class="deadline-icon">
      <CuteIcon name="Flag" :size="21" />
    </div>
    <div class="deadline-title">{{ task.title }}</div>
  </div>
</template>

<style scoped>
.cell-item-deadline {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  padding: 0.8rem;
  border-radius: 0.6rem;
  transition: background-color 0.15s ease;
  cursor: pointer;
}

.cell-item-deadline:hover {
  background: var(--color-background-hover, rgb(0 0 0 / 3%));
}

.deadline-icon {
  flex-shrink: 0;
  color: var(--color-warning, #f59e0b);
  display: flex;
  align-items: center;
  margin-top: 0.1rem;
}

.cell-item-deadline.is-overdue .deadline-icon {
  color: var(--color-error, #ef4444);
}

.deadline-title {
  flex: 1;
  min-width: 0;
  font-size: 1.5rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  line-height: 1.4;
  overflow-wrap: break-word;
}

.cell-item-deadline.is-overdue .deadline-title {
  color: var(--color-text-primary);
}
</style>
