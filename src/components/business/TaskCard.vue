<script setup lang="ts">
import type { Task } from '@/types/models'
import { useTaskStore } from '@/stores/task'
import CuteCard from '@/components/ui/CuteCard.vue'
import CuteCheckbox from '@/components/ui/CuteCheckbox.vue'

const props = defineProps<{
  task: Task
}>()

const taskStore = useTaskStore()

async function handleStatusChange(isChecked: boolean) {
  const newStatus = isChecked ? 'done' : 'todo'
  const completedAt = isChecked ? new Date().toISOString() : null

  await taskStore.updateTask(props.task.id, {
    status: newStatus,
    completed_at: completedAt,
  })
}
</script>

<template>
  <CuteCard class="task-card">
    <div class="left-section">
      <CuteCheckbox
        :checked="task.status === 'done'"
        size="large"
        @update:checked="handleStatusChange"
      />
      <span class="title">{{ task.title }}</span>
    </div>
  </CuteCard>
</template>

<style scoped>
.task-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem;
  margin-bottom: 0.5rem;
  border: 1px solid transparent;
  transition:
    background-color 0.2s,
    border-color 0.2s;
}

.task-card:hover {
  background-color: var(--color-card-available);
  border-color: var(--color-border-default);
}

.left-section {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.title {
  font-size: 1.5rem;
  font-weight: 500;
  color: var(--color-text-primary);
}

.right-section {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 0.25rem;
}

.time-tracking,
.tag {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
}

.tags {
  display: flex;
  gap: 0.25rem;
}

/* stylelint-disable-next-line selector-class-pattern */
.task-card:has(.n-checkbox--checked) .title {
  text-decoration: line-through;
  color: var(--color-text-secondary);
}
</style>
