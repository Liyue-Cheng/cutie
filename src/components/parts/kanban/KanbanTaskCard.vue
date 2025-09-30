<script setup lang="ts">
import { computed } from 'vue'
import type { TaskCard } from '@/types/dtos'
import { useTaskStore } from '@/stores/task'
import { useContextMenu } from '@/composables/useContextMenu'
import KanbanTaskCardMenu from './KanbanTaskCardMenu.vue'
import CuteCard from '@/components/templates/CuteCard.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'

const props = defineProps<{
  task: TaskCard
}>()

const taskStore = useTaskStore()
const emit = defineEmits(['openEditor'])

const contextMenu = useContextMenu()

// 使用任务的subtasks字段替代checkpoints
const subtasks = computed(() => props.task.subtasks || [])

function showContextMenu(event: MouseEvent) {
  contextMenu.show(KanbanTaskCardMenu, { task: props.task }, event)
}

async function handleStatusChange(isChecked: boolean) {
  if (isChecked) {
    // 完成任务
    await taskStore.completeTask(props.task.id)
  } else {
    // 重新打开任务
    await taskStore.reopenTask(props.task.id)
  }
}

async function handleSubtaskStatusChange(subtaskId: string, isCompleted: boolean) {
  // 更新subtask状态
  const updatedSubtasks = subtasks.value.map((subtask) =>
    subtask.id === subtaskId ? { ...subtask, is_completed: isCompleted } : subtask
  )

  // 更新任务的subtasks
  await taskStore.updateTask(props.task.id, {
    subtasks: updatedSubtasks,
  })
}
</script>

<template>
  <CuteCard class="task-card" @click="emit('openEditor')" @contextmenu="showContextMenu">
    <div class="main-content">
      <span class="title">{{ task.title }}</span>

      <div v-if="task.glance_note" class="notes-section">
        <CuteIcon name="CornerDownRight" :size="14" />
        <span class="note-text">{{ task.glance_note }}</span>
      </div>

      <div v-if="subtasks.length > 0" class="subtasks-section">
        <div v-for="subtask in subtasks" :key="subtask.id" class="subtask-item">
          <CuteCheckbox
            :checked="subtask.is_completed"
            size="small"
            @update:checked="
              (isChecked: boolean) => handleSubtaskStatusChange(subtask.id, isChecked)
            "
            @click.stop
          />
          <span class="subtask-title">{{ subtask.title }}</span>
        </div>
      </div>

      <CuteCheckbox
        class="main-checkbox"
        :checked="task.is_completed"
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

.subtasks-section {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}

.subtask-item {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.subtask-title {
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
.subtask-item:has(.n-checkbox--checked) .subtask-title {
  text-decoration: line-through;
  color: var(--color-text-secondary);
}
</style>
