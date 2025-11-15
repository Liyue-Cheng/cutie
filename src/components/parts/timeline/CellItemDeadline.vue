<script setup lang="ts">
/**
 * CellItemDeadline - 时间线单元格截止日期项
 *
 * 用于在 TimelineDayCell 中显示截止日期
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
    <div class="due-icon">
      <CuteIcon name="Flag" :size="16" />
    </div>
    <div class="due-content">
      <div class="due-title">{{ task.title }}</div>
      <div class="due-label">截止</div>
    </div>
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
  border-left: 3px solid var(--color-warning, #f59e0b);
  background: rgb(245 158 11 / 5%);
}

.cell-item-deadline:hover {
  background: rgb(245 158 11 / 10%);
}

.cell-item-deadline.is-overdue {
  border-left-color: var(--color-error, #ef4444);
  background: rgb(239 68 68 / 5%);
}

.cell-item-deadline.is-overdue:hover {
  background: rgb(239 68 68 / 10%);
}

.due-icon {
  flex-shrink: 0;
  color: var(--color-warning);
  display: flex;
  align-items: center;
}

.cell-item-deadline.is-overdue .due-icon {
  color: var(--color-error);
}

.due-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
}

.due-title {
  font-size: 1.5rem;
  font-weight: 500;
  color: var(--color-text-primary);
  line-height: 1.4;
  overflow-wrap: break-word;
}

.due-label {
  font-size: 1.2rem;
  color: var(--color-text-secondary);
}
</style>
