<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useTaskStore } from '@/stores/task'
import { useCheckpointStore } from '@/stores/checkpoint'
import type { Checkpoint } from '@/types/models'
import CuteCard from '@/components/templates/CuteCard.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import CuteButton from '@/components/parts/CuteButton.vue'

const props = defineProps<{
  taskId: string | null
}>()

defineEmits(['close'])

const taskStore = useTaskStore()
const checkpointStore = useCheckpointStore()

const notes = ref('')
const newCheckpointTitle = ref('')

const task = computed(() => {
  return props.taskId ? taskStore.getTaskById(props.taskId) : null
})

const checkpoints = computed(() => {
  return props.taskId ? checkpointStore.getCheckpointsForTask(props.taskId) : []
})

watch(
  task,
  (newTask) => {
    notes.value = newTask?.metadata?.notes || ''
    if (newTask && props.taskId) {
      checkpointStore.fetchCheckpointsForTask(props.taskId)
    }
  },
  { immediate: true }
)

async function handleStatusChange(isChecked: boolean) {
  if (!props.taskId) return
  const newStatus = isChecked ? 'done' : 'todo'
  const completedAt = isChecked ? new Date().toISOString() : null

  await taskStore.updateTask(props.taskId, {
    status: newStatus,
    completed_at: completedAt,
  })
}

async function updateNotes() {
  if (!props.taskId || !task.value) return
  if (notes.value !== (task.value.metadata?.notes || '')) {
    await taskStore.updateTask(props.taskId, {
      metadata: { ...task.value.metadata, notes: notes.value },
    })
  }
}

async function handleAddCheckpoint() {
  if (!props.taskId || !newCheckpointTitle.value.trim()) return
  await checkpointStore.createCheckpoint({
    task_id: props.taskId,
    title: newCheckpointTitle.value.trim(),
    sort_key: `checkpoint_${Date.now()}`,
  })
  newCheckpointTitle.value = ''
}

async function handleCheckpointStatusChange(checkpoint: Checkpoint, isCompleted: boolean) {
  if (!props.taskId) return
  await checkpointStore.updateCheckpoint(checkpoint.id, { is_completed: isCompleted }, props.taskId)
}
</script>

<template>
  <div class="modal-overlay" @click="$emit('close')">
    <CuteCard class="editor-card" @click.stop>
      <div v-if="task" class="content-wrapper">
        <div class="header-section">
          <CuteCheckbox
            :checked="task.status === 'done'"
            size="large"
            @update:checked="handleStatusChange"
          ></CuteCheckbox>
          <span class="title">{{ task.title }}</span>
        </div>

        <div class="separator"></div>

        <textarea
          v-model="notes"
          class="notes-input"
          placeholder="Add some notes..."
          @blur="updateNotes"
        ></textarea>

        <div class="separator"></div>

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
              placeholder="Add subtask..."
              @keyup.enter="handleAddCheckpoint"
            />
            <CuteButton @click="handleAddCheckpoint">Add Subtask</CuteButton>
          </div>
        </div>
      </div>
    </CuteCard>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background-color: rgb(0 0 0 / 50%);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.editor-card {
  width: 60rem;
  min-height: 40rem;
  padding: 2.5rem;
  border: 1px solid var(--color-border-default);
  background-color: var(--color-card-available);
  border-radius: 0.8rem;
}

.content-wrapper {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.header-section {
  display: flex;
  align-items: center;
  gap: 1.5rem;
}

.title {
  font-size: 2.4rem;
  font-weight: 600;
  color: var(--color-text-primary);
  flex-grow: 1;
}

/* stylelint-disable-next-line selector-class-pattern */
.editor-card:has(.n-checkbox--checked) .title {
  text-decoration: line-through;
  color: var(--color-text-secondary);
}

.separator {
  height: 1px;
  background-color: var(--color-separator);
  margin: 2rem 0;
}

.notes-input {
  width: 100%;
  min-height: 120px;
  padding: 1rem;
  font-family: inherit;
  font-size: 1.6rem;
  border: 1px solid var(--color-border-default);
  border-radius: 6px;
  background-color: var(--color-background-secondary);
  color: var(--color-text-primary);
  resize: vertical;
}

.checkpoints-section {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.checkpoint-item {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.checkpoint-title {
  font-size: 1.6rem;
  color: var(--color-text-secondary);
}

/* stylelint-disable-next-line selector-class-pattern */
.checkpoint-item:has(.n-checkbox--checked) .checkpoint-title {
  text-decoration: line-through;
  color: var(--color-text-tertiary);
}

.add-checkpoint-form {
  display: flex;
  gap: 1rem;
  margin-top: 1rem;
}

.add-checkpoint-input {
  flex-grow: 1;
  padding: 1rem;
  font-size: 1.6rem;
  border: 1px solid var(--color-border-default);
  border-radius: 6px;
  background-color: var(--color-background-primary);
}
</style>
