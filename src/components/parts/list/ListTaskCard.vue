<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { Task, Checkpoint } from '@/types/models'
import { useTaskStore } from '@/stores/task'
import { useCheckpointStore } from '@/stores/checkpoint'
import CuteCard from '@/components/templates/CuteCard.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import CuteButton from '../CuteButton.vue'

const props = defineProps<{
  task: Task
}>()

const taskStore = useTaskStore()
const checkpointStore = useCheckpointStore()

const isExpanded = ref(false)
const notes = ref(props.task.metadata?.notes || '')
const newCheckpointTitle = ref('')

const checkpoints = computed(() => checkpointStore.getCheckpointsForTask(props.task.id))

async function handleStatusChange(isChecked: boolean) {
  const newStatus = isChecked ? 'done' : 'todo'
  const completedAt = isChecked ? new Date().toISOString() : null

  await taskStore.updateTask(props.task.id, {
    status: newStatus,
    completed_at: completedAt,
  })
}

async function toggleExpand() {
  isExpanded.value = !isExpanded.value
  if (isExpanded.value && checkpoints.value.length === 0) {
    await checkpointStore.fetchCheckpointsForTask(props.task.id)
  }
}

async function updateNotes() {
  if (notes.value !== (props.task.metadata?.notes || '')) {
    await taskStore.updateTask(props.task.id, {
      metadata: { ...props.task.metadata, notes: notes.value },
    })
  }
}

async function handleAddCheckpoint() {
  if (!newCheckpointTitle.value.trim()) return

  // Basic sort key generation, just append current time
  const sort_key = `checkpoint_${Date.now()}`

  await checkpointStore.createCheckpoint({
    task_id: props.task.id,
    title: newCheckpointTitle.value.trim(),
    sort_key,
  })
  newCheckpointTitle.value = ''
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
  <CuteCard class="task-card" :class="{ 'is-expanded': isExpanded }">
    <div class="main-content" @click="toggleExpand">
      <div class="left-section">
        <CuteCheckbox
          :checked="task.status === 'done'"
          size="large"
          @update:checked="handleStatusChange"
          @click.stop
        ></CuteCheckbox>
        <span class="title">{{ task.title }}</span>
      </div>
    </div>
    <div v-if="isExpanded" class="details-section">
      <textarea
        v-model="notes"
        class="notes-input"
        placeholder="Add some notes..."
        @blur="updateNotes"
      />
      <div class="separator" />

      <div class="checkpoints-section">
        <div v-for="checkpoint in checkpoints" :key="checkpoint.id" class="checkpoint-item">
          <CuteCheckbox
            :checked="checkpoint.is_completed"
            @update:checked="
              (isChecked: boolean) => handleCheckpointStatusChange(checkpoint, isChecked)
            "
          />
          <span class="checkpoint-title">{{ checkpoint.title }}</span>
        </div>
        <div class="add-checkpoint-form">
          <input
            v-model="newCheckpointTitle"
            class="add-checkpoint-input"
            placeholder="Add a checkpoint..."
            @keyup.enter="handleAddCheckpoint"
          />
          <CuteButton @click="handleAddCheckpoint"> Add </CuteButton>
        </div>
      </div>
    </div>
  </CuteCard>
</template>

<style scoped>
.task-card {
  display: flex;
  flex-direction: column;
  padding: 0.75rem;
  margin-bottom: 0.5rem;
  border: 1px solid transparent;
  transition:
    background-color 0.2s,
    border-color 0.2s;
}

.task-card.is-expanded {
  padding-bottom: 1rem;
}

.task-card:hover,
.task-card.is-expanded {
  background-color: var(--color-card-available);
  border-color: var(--color-border-default);
}

.main-content {
  cursor: pointer;
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

/* stylelint-disable-next-line selector-class-pattern */
.task-card:has(.n-checkbox--checked) .title {
  text-decoration: line-through;
  color: var(--color-text-secondary);
}

.details-section {
  margin-top: 1rem;
  padding-left: 2.5rem; /* Align with title */
}

.notes-input {
  width: 100%;
  padding: 0.5rem;
  font-family: inherit;
  font-size: 1rem;
  border: 1px solid var(--color-border-default);
  border-radius: 4px;
  background-color: var(--color-background-secondary);
  color: var(--color-text-primary);
  resize: vertical;
  min-height: 60px;
}

.separator {
  height: 1px;
  background-color: var(--color-separator);
  margin: 1rem 0;
}

.checkpoint-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.checkpoint-title {
  font-size: 1rem;
  color: var(--color-text-secondary);
}

/* stylelint-disable-next-line selector-class-pattern */
.checkpoint-item:has(.n-checkbox--checked) .checkpoint-title {
  text-decoration: line-through;
  color: var(--color-text-tertiary);
}

.add-checkpoint-form {
  display: flex;
  gap: 0.5rem;
  margin-top: 1rem;
}

.add-checkpoint-input {
  flex-grow: 1;
  padding: 0.5rem;
  font-size: 1rem;
  border: 1px solid var(--color-border-default);
  border-radius: 4px;
  background-color: var(--color-background-primary);
}
</style>
