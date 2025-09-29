<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Task, Subtask } from '@/types/models'
import { useTaskStore } from '@/stores/task'
import CuteCard from '@/components/templates/CuteCard.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import CuteButton from '../CuteButton.vue'

const props = defineProps<{
  task: Task
}>()

const taskStore = useTaskStore()

const isExpanded = ref(false)
const notes = ref(props.task.detail_note || '')
const newSubtaskTitle = ref('')

const subtasks = computed(() => props.task.subtasks || [])

async function handleStatusChange(isChecked: boolean) {
  if (isChecked) {
    await taskStore.completeTask(props.task.id)
  } else {
    await taskStore.reopenTask(props.task.id)
  }
}

async function toggleExpand() {
  isExpanded.value = !isExpanded.value
}

async function updateNotes() {
  if (notes.value !== (props.task.detail_note || '')) {
    await taskStore.updateTask(props.task.id, {
      detail_note: notes.value,
    })
  }
}

async function handleAddSubtask() {
  if (!newSubtaskTitle.value.trim()) return

  // 生成新的subtask
  const newSubtask: Subtask = {
    id: crypto.randomUUID(),
    title: newSubtaskTitle.value.trim(),
    is_completed: false,
    sort_order: `subtask_${Date.now()}`,
  }

  const updatedSubtasks = [...subtasks.value, newSubtask]

  await taskStore.updateTask(props.task.id, {
    subtasks: updatedSubtasks,
  })

  newSubtaskTitle.value = ''
}

async function handleSubtaskStatusChange(subtaskId: string, isCompleted: boolean) {
  const updatedSubtasks = subtasks.value.map((subtask) =>
    subtask.id === subtaskId ? { ...subtask, is_completed: isCompleted } : subtask
  )

  await taskStore.updateTask(props.task.id, {
    subtasks: updatedSubtasks,
  })
}
</script>

<template>
  <CuteCard class="task-card" :class="{ 'is-expanded': isExpanded }">
    <div class="main-content" @click="toggleExpand">
      <div class="left-section">
        <CuteCheckbox
          :checked="!!task.completed_at"
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

      <div class="subtasks-section">
        <div v-for="subtask in subtasks" :key="subtask.id" class="subtask-item">
          <CuteCheckbox
            :checked="subtask.is_completed"
            @update:checked="
              (isChecked: boolean) => handleSubtaskStatusChange(subtask.id, isChecked)
            "
          />
          <span class="subtask-title">{{ subtask.title }}</span>
        </div>
        <div class="add-subtask-form">
          <input
            v-model="newSubtaskTitle"
            class="add-subtask-input"
            placeholder="Add a subtask..."
            @keyup.enter="handleAddSubtask"
          />
          <CuteButton @click="handleAddSubtask"> Add </CuteButton>
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

.subtask-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.subtask-title {
  font-size: 1rem;
  color: var(--color-text-secondary);
}

/* stylelint-disable-next-line selector-class-pattern */
.subtask-item:has(.n-checkbox--checked) .subtask-title {
  text-decoration: line-through;
  color: var(--color-text-tertiary);
}

.add-subtask-form {
  display: flex;
  gap: 0.5rem;
  margin-top: 1rem;
}

.add-subtask-input {
  flex-grow: 1;
  padding: 0.5rem;
  font-size: 1rem;
  border: 1px solid var(--color-border-default);
  border-radius: 4px;
  background-color: var(--color-background-primary);
}
</style>
