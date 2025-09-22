<script setup lang="ts">
import { computed, onMounted } from 'vue'
import type { Task, Checkpoint } from '@/types/models'
import { useTaskStore } from '@/stores/task'
import { useCheckpointStore } from '@/stores/checkpoint'
import CuteCard from '@/components/ui/CuteCard.vue'
import CuteCheckbox from '@/components/ui/CuteCheckbox.vue'
import CuteIcon from '@/components/ui/CuteIcon.vue'

const props = defineProps<{
  task: Task
}>()

const taskStore = useTaskStore()
const checkpointStore = useCheckpointStore()
const emit = defineEmits(['openEditor'])

const checkpoints = computed(() => checkpointStore.getCheckpointsForTask(props.task.id))

onMounted(() => {
  checkpointStore.fetchCheckpointsForTask(props.task.id)
})

async function handleStatusChange(isChecked: boolean) {
  const newStatus = isChecked ? 'done' : 'todo'
  const completedAt = isChecked ? new Date().toISOString() : null
  await taskStore.updateTask(props.task.id, {
    status: newStatus,
    completed_at: completedAt,
  })
}

async function handleCheckpointStatusChange(checkpoint: Checkpoint, isCompleted: boolean) {
  await checkpointStore.updateCheckpoint(
    checkpoint.id,
    { is_completed: isCompleted },
    props.task.id
  )
}
</script>

<template>
  <CuteCard class="task-card" @click="emit('openEditor')">
    <div class="main-content">
      <span class="title">{{ task.title }}</span>

      <div v-if="task.metadata?.notes" class="notes-section">
        <CuteIcon name="CornerDownRight" :size="14" />
        <span class="note-text">{{ task.metadata.notes }}</span>
      </div>

      <div v-if="checkpoints.length > 0" class="checkpoints-section">
        <div v-for="checkpoint in checkpoints" :key="checkpoint.id" class="checkpoint-item">
          <CuteCheckbox
            :checked="checkpoint.is_completed"
            size="small"
            @update:checked="
              (isChecked: boolean) => handleCheckpointStatusChange(checkpoint, isChecked)
            "
            @click.stop
          />
          <span class="checkpoint-title">{{ checkpoint.title }}</span>
        </div>
      </div>

      <CuteCheckbox
        class="main-checkbox"
        :checked="task.status === 'done'"
        size="large"
        @update:checked="handleStatusChange"
        @click.stop
      ></CuteCheckbox>
    </div>
  </CuteCard>
</template>

<style scoped>
.task-card {
  display: flex;
  flex-direction: column;
  padding: 1rem;
  margin-bottom: 0.75rem;
  border: 1px solid var(--color-border-default);
  background-color: var(--color-card-available);
  border-radius: 0.4rem;
  transition:
    border-color 0.2s,
    box-shadow 0.2s;
  cursor: pointer;
}

.task-card:hover {
  border-color: var(--color-border-hover);
  box-shadow: 0 4px 12px rgb(0 0 0 / 10%);
}

.main-content {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.title {
  font-size: 1.5rem;
  font-weight: 500;
  color: var(--color-text-primary);
}

.notes-section {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--color-text-primary);
}

.note-text {
  font-size: 1.3rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.checkpoints-section {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}

.checkpoint-item {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.checkpoint-title {
  font-size: 1.4rem;
  color: var(--color-text-primary);
}

.main-checkbox {
  margin-top: 0.5rem;
  align-self: flex-start;
}

/* stylelint-disable-next-line selector-class-pattern */
.task-card:has(.n-checkbox--checked) .title,
/* stylelint-disable-next-line selector-class-pattern */
.checkpoint-item:has(.n-checkbox--checked) .checkpoint-title {
  text-decoration: line-through;
  color: var(--color-text-secondary);
}
</style>
