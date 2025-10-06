<script setup lang="ts">
import { computed } from 'vue'
import type { TaskCard } from '@/types/dtos'
import type { ViewMetadata, DateViewConfig } from '@/types/drag'
import { useTaskStore } from '@/stores/task'
import { useAreaStore } from '@/stores/area'
import { useTaskOperations } from '@/composables/useTaskOperations'
import { useContextMenu } from '@/composables/useContextMenu'
import KanbanTaskCardMenu from './KanbanTaskCardMenu.vue'
import CuteCard from '@/components/templates/CuteCard.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'

const props = defineProps<{
  task: TaskCard
  viewMetadata?: ViewMetadata
}>()

const taskStore = useTaskStore()
const areaStore = useAreaStore()
const taskOps = useTaskOperations()
const emit = defineEmits<{
  openEditor: []
  taskCompleted: [taskId: string]
}>()

const contextMenu = useContextMenu()

// 使用任务的subtasks字段替代checkpoints
const subtasks = computed(() => props.task.subtasks || [])

// ✅ 通过 area_id 从 store 获取完整 area 信息
const area = computed(() => {
  return props.task.area_id ? areaStore.getAreaById(props.task.area_id) : null
})

// ✅ 判断是否为日期看板（daily::*）
const isDateKanban = computed(() => {
  return props.viewMetadata?.type === 'date'
})

// ✅ 判断是否为过去的日期
const isPastDate = computed(() => {
  if (!isDateKanban.value || !props.viewMetadata?.config) return false

  const config = props.viewMetadata.config as DateViewConfig
  const kanbanDate = new Date(config.date + 'T00:00:00')
  const today = new Date()
  today.setHours(0, 0, 0, 0)

  return kanbanDate < today
})

function showContextMenu(event: MouseEvent) {
  contextMenu.show(KanbanTaskCardMenu, { task: props.task }, event)
}

async function handleStatusChange(isChecked: boolean) {
  if (isChecked) {
    // ✅ 完成任务
    await taskOps.completeTask(props.task.id)
    // 通知父组件任务已完成，以便重新排序
    emit('taskCompleted', props.task.id)
  } else {
    // ✅ 重新打开任务
    await taskOps.reopenTask(props.task.id)
  }
}

async function handleSubtaskStatusChange(subtaskId: string, isCompleted: boolean) {
  // 更新subtask状态
  const updatedSubtasks = subtasks.value.map((subtask) =>
    subtask.id === subtaskId ? { ...subtask, is_completed: isCompleted } : subtask
  )

  // ✅ 更新任务的subtasks（仍然使用 taskStore，因为这是简单的更新操作）
  await taskStore.updateTask(props.task.id, {
    subtasks: updatedSubtasks,
  })
}
</script>

<template>
  <CuteCard
    class="task-card"
    :data-completed="task.is_completed"
    @click="emit('openEditor')"
    @contextmenu="showContextMenu"
  >
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

      <div class="card-footer">
        <div class="main-checkbox-wrapper">
          <!-- 完成按钮：过去日期且后续有记录时不显示 -->
          <CuteCheckbox
            v-if="!isPastDate"
            class="main-checkbox"
            :checked="task.is_completed"
            size="large"
            @update:checked="handleStatusChange"
            @click.stop
          ></CuteCheckbox>

          <!-- 星星按钮：只在日期看板显示 -->
          <CuteCheckbox
            v-if="isDateKanban"
            class="star-checkbox"
            variant="star"
            size="large"
            :checked="false"
            @click.stop
          ></CuteCheckbox>
        </div>
        <div v-if="area" class="area-tag" :style="{ color: area.color }">#{{ area.name }}</div>
      </div>
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

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-top: 0.5rem;
}

.main-checkbox-wrapper {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  align-self: flex-start;
}

.star-checkbox {
  opacity: 0;
  transition: opacity 0.2s ease-in-out;
}

.task-card:hover .star-checkbox {
  opacity: 1;
}

.area-tag {
  font-size: 1.2rem;
  font-weight: 500;
  align-self: flex-start;
  margin-top: 0.5rem;
}

/* 只有主复选框被选中时，主标题才划线 */
/* stylelint-disable-next-line selector-class-pattern */
.main-checkbox-wrapper:has(.n-checkbox--checked) ~ .title {
  text-decoration: line-through;
  color: var(--color-text-secondary);
}

/* 或者使用更直接的方式：检查 task.is_completed */
.task-card[data-completed='true'] .title {
  text-decoration: line-through;
  color: var(--color-text-secondary);
}

/* 子任务选中时，只划子任务的线 */
/* stylelint-disable-next-line selector-class-pattern */
.subtask-item:has(.n-checkbox--checked) .subtask-title {
  text-decoration: line-through;
  color: var(--color-text-secondary);
}
</style>
