<script setup lang="ts">
import { computed, ref, watch, onMounted } from 'vue'
import { useTaskStore } from '@/stores/task'
import { useTaskOperations } from '@/composables/useTaskOperations'
import type { TaskDetail } from '@/types/dtos'
import CuteCard from '@/components/templates/CuteCard.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import CuteButton from '@/components/parts/CuteButton.vue'
import AreaSelector from '@/components/parts/AreaSelector.vue'

interface Subtask {
  id: string
  title: string
  is_completed: boolean
  sort_order: string
}

const props = defineProps<{
  taskId: string | null
}>()

defineEmits(['close'])

const taskStore = useTaskStore()
const taskOps = useTaskOperations()

const glanceNote = ref('')
const detailNote = ref('')
const selectedAreaId = ref<string | null>(null)
const newSubtaskTitle = ref('')

const task = computed(() => {
  return props.taskId ? taskStore.getTaskById(props.taskId) : null
})

const subtasks = computed(() => {
  return task.value?.subtasks || []
})

// 当弹窗打开时，获取任务详情
onMounted(async () => {
  if (props.taskId) {
    const detail = (await taskStore.fetchTaskDetail(props.taskId)) as TaskDetail | null
    if (detail) {
      // TaskDetail 包含完整的 note 信息
      glanceNote.value = detail.glance_note || ''
      detailNote.value = detail.detail_note || ''
      selectedAreaId.value = detail.area?.id || null
    }
  }
})

watch(
  () => props.taskId,
  async (newTaskId) => {
    if (newTaskId) {
      const detail = (await taskStore.fetchTaskDetail(newTaskId)) as TaskDetail | null
      if (detail) {
        glanceNote.value = detail.glance_note || ''
        detailNote.value = detail.detail_note || ''
        selectedAreaId.value = detail.area?.id || null
      }
    }
  }
)

async function handleStatusChange(isChecked: boolean) {
  if (!props.taskId) return

  if (isChecked) {
    // ✅ 使用 TaskOperations 完成任务
    await taskOps.completeTask(props.taskId)
  } else {
    // ✅ 使用 TaskOperations 重新打开任务
    await taskOps.reopenTask(props.taskId)
  }
}

async function updateGlanceNote() {
  if (!props.taskId || !task.value) return
  await taskStore.updateTask(props.taskId, {
    glance_note: glanceNote.value || null,
  })
}

async function updateDetailNote() {
  if (!props.taskId || !task.value) return
  await taskStore.updateTask(props.taskId, {
    detail_note: detailNote.value || null,
  })
}

async function updateArea() {
  if (!props.taskId || !task.value) return
  await taskStore.updateTask(props.taskId, {
    area_id: selectedAreaId.value,
  })
}

async function handleAddSubtask() {
  if (!props.taskId || !newSubtaskTitle.value.trim()) return

  const newSubtask: Subtask = {
    id: crypto.randomUUID(),
    title: newSubtaskTitle.value.trim(),
    is_completed: false,
    sort_order: `subtask_${Date.now()}`,
  }

  const updatedSubtasks = [...subtasks.value, newSubtask]

  await taskStore.updateTask(props.taskId, {
    subtasks: updatedSubtasks,
  })

  newSubtaskTitle.value = ''
}

async function handleSubtaskStatusChange(subtaskId: string, isCompleted: boolean) {
  if (!props.taskId) return

  const updatedSubtasks = subtasks.value.map((subtask) =>
    subtask.id === subtaskId ? { ...subtask, is_completed: isCompleted } : subtask
  )

  await taskStore.updateTask(props.taskId, {
    subtasks: updatedSubtasks,
  })
}
</script>

<template>
  <div class="modal-overlay" @click="$emit('close')">
    <CuteCard class="editor-card" @click.stop>
      <div v-if="task" class="content-wrapper">
        <div class="header-section">
          <CuteCheckbox
            :checked="task.is_completed"
            size="large"
            @update:checked="handleStatusChange"
          ></CuteCheckbox>
          <span class="title">{{ task.title }}</span>
        </div>

        <div class="separator"></div>

        <div class="notes-section">
          <label class="notes-label">快览笔记 (Glance Note)</label>
          <input
            v-model="glanceNote"
            type="text"
            class="glance-note-input"
            placeholder="简短笔记（卡片上显示）..."
            @blur="updateGlanceNote"
          />
        </div>

        <div class="separator"></div>

        <div class="notes-section">
          <label class="notes-label">详细笔记 (Detail Note)</label>
          <textarea
            v-model="detailNote"
            class="detail-note-input"
            placeholder="详细笔记（仅在编辑器中）..."
            @blur="updateDetailNote"
          ></textarea>
        </div>

        <div class="separator"></div>

        <div class="area-section">
          <label class="notes-label">区域 (Area)</label>
          <AreaSelector v-model="selectedAreaId" @blur="updateArea" />
        </div>

        <div class="separator"></div>

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
              placeholder="Add subtask..."
              @keyup.enter="handleAddSubtask"
            />
            <CuteButton @click="handleAddSubtask">Add Subtask</CuteButton>
          </div>
        </div>

        <div class="separator"></div>

        <!-- 调试信息 -->
        <details class="debug-info">
          <summary class="debug-summary">🔍 调试数据（完整 TaskCard 结构）</summary>
          <pre class="debug-content">{{ JSON.stringify(task, null, 2) }}</pre>
        </details>
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

.notes-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.notes-label {
  font-size: 1.3rem;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.glance-note-input {
  width: 100%;
  padding: 0.8rem;
  font-family: inherit;
  font-size: 1.5rem;
  border: 1px solid var(--color-border-default);
  border-radius: 6px;
  background-color: var(--color-background-secondary);
  color: var(--color-text-primary);
}

.glance-note-input:focus {
  outline: none;
  border-color: var(--color-primary, #4a90e2);
}

.detail-note-input {
  width: 100%;
  min-height: 120px;
  padding: 1rem;
  font-family: inherit;
  font-size: 1.5rem;
  border: 1px solid var(--color-border-default);
  border-radius: 6px;
  background-color: var(--color-background-secondary);
  color: var(--color-text-primary);
  resize: vertical;
}

.detail-note-input:focus {
  outline: none;
  border-color: var(--color-primary, #4a90e2);
}

.subtasks-section {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.subtask-item {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.subtask-title {
  font-size: 1.6rem;
  color: var(--color-text-secondary);
}

/* stylelint-disable-next-line selector-class-pattern */
.subtask-item:has(.n-checkbox--checked) .subtask-title {
  text-decoration: line-through;
  color: var(--color-text-tertiary);
}

.add-subtask-form {
  display: flex;
  gap: 1rem;
  margin-top: 1rem;
}

.add-subtask-input {
  flex-grow: 1;
  padding: 1rem;
  font-size: 1.6rem;
  border: 1px solid var(--color-border-default);
  border-radius: 6px;
  background-color: var(--color-background-primary);
}

/* 调试信息样式 */
.debug-info {
  margin-top: 1rem;
}

.debug-summary {
  font-size: 1.3rem;
  color: var(--color-text-tertiary);
  cursor: pointer;
  user-select: none;
  padding: 0.8rem;
  background-color: var(--color-background-soft, #f9f9f9);
  border-radius: 6px;
  transition: all 0.2s;
}

.debug-summary:hover {
  color: var(--color-text-secondary);
  background-color: var(--color-background-hover, #e8e8e8);
}

.debug-content {
  font-family: Consolas, Monaco, 'Courier New', monospace;
  font-size: 1.2rem;
  line-height: 1.6;
  color: var(--color-text-secondary);
  background-color: var(--color-background-soft, #f9f9f9);
  padding: 1.5rem;
  border-radius: 6px;
  margin-top: 1rem;
  overflow: auto;
  max-height: 400px;
  border: 1px solid var(--color-border-default);
}

.debug-content::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.debug-content::-webkit-scrollbar-thumb {
  background: var(--color-border-hover);
  border-radius: 4px;
}

.debug-content::-webkit-scrollbar-track {
  background: transparent;
}
</style>
