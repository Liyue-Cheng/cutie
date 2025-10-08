<script setup lang="ts">
import { computed, ref, watch, onMounted, nextTick } from 'vue'
import { useTaskStore } from '@/stores/task'
import { useAreaStore } from '@/stores/area'
import { useTaskOperations } from '@/composables/useTaskOperations'
import type { TaskDetail } from '@/types/dtos'
import CuteCard from '@/components/templates/CuteCard.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import AreaTag from '@/components/parts/AreaTag.vue'
import { getTodayDateString, parseDateString, toUtcIsoString } from '@/utils/dateUtils'

interface Subtask {
  id: string
  title: string
  is_completed: boolean
  sort_order: string
}

const props = defineProps<{
  taskId: string | null
}>()

const emit = defineEmits(['close'])

const taskStore = useTaskStore()
const areaStore = useAreaStore()
const taskOps = useTaskOperations()

// 本地编辑状态
const titleInput = ref('')
const glanceNote = ref('')
const detailNote = ref('')
const selectedAreaId = ref<string | null>(null)
const newSubtaskTitle = ref('')
const isTitleEditing = ref(false)
const showAreaSelector = ref(false)
const showDueDatePicker = ref(false)
const dueDateInput = ref('') // YYYY-MM-DD format
const dueDateType = ref<'SOFT' | 'HARD'>('SOFT')
const draggingSubtaskId = ref<string | null>(null)
const glanceNoteTextarea = ref<HTMLTextAreaElement | null>(null)
const detailNoteTextarea = ref<HTMLTextAreaElement | null>(null)
const mouseDownOnOverlay = ref(false)

const task = computed(() => {
  return props.taskId ? taskStore.getTaskById(props.taskId) : null
})

const subtasks = computed(() => {
  return task.value?.subtasks || []
})

const selectedArea = computed(() => {
  return selectedAreaId.value ? areaStore.getAreaById(selectedAreaId.value) : null
})

// 获取今天的日期（用于在场状态判断）
const todayDate = computed(() => getTodayDateString())

// 获取今天的 schedule outcome
const currentScheduleOutcome = computed(() => {
  if (!task.value?.schedules || !todayDate.value) return null

  const todaySchedule = task.value.schedules.find((s) => s.scheduled_day === todayDate.value)
  return todaySchedule?.outcome || null
})

// 今天是否已记录在场
const isPresenceLogged = computed(() => {
  return currentScheduleOutcome.value === 'presence_logged'
})

// 自动调整 textarea 高度
function autoResizeTextarea(textarea: HTMLTextAreaElement) {
  textarea.style.height = 'auto'
  textarea.style.height = textarea.scrollHeight + 'px'
}

// 初始化所有 textarea 的高度
function initTextareaHeights() {
  if (glanceNoteTextarea.value) {
    autoResizeTextarea(glanceNoteTextarea.value)
  }
  if (detailNoteTextarea.value) {
    autoResizeTextarea(detailNoteTextarea.value)
  }
}

// 当弹窗打开时，获取任务详情
onMounted(async () => {
  if (props.taskId) {
    const detail = (await taskStore.fetchTaskDetail(props.taskId)) as TaskDetail | null
    if (detail) {
      titleInput.value = detail.title
      glanceNote.value = detail.glance_note || ''
      detailNote.value = detail.detail_note || ''
      selectedAreaId.value = detail.area_id || null

      // 初始化截止日期
      if (detail.due_date) {
        const datePart = detail.due_date.date.split('T')[0]
        dueDateInput.value = datePart || ''
        dueDateType.value = detail.due_date.type
      } else {
        dueDateInput.value = ''
        dueDateType.value = 'SOFT'
      }

      // 等待 DOM 更新后调整 textarea 高度
      await nextTick()
      initTextareaHeights()
    }
  }
})

watch(
  () => props.taskId,
  async (newTaskId) => {
    if (newTaskId) {
      const detail = (await taskStore.fetchTaskDetail(newTaskId)) as TaskDetail | null
      if (detail) {
        titleInput.value = detail.title
        glanceNote.value = detail.glance_note || ''
        detailNote.value = detail.detail_note || ''
        selectedAreaId.value = detail.area_id || null

        // 初始化截止日期
        if (detail.due_date) {
          const datePart = detail.due_date.date.split('T')[0]
          dueDateInput.value = datePart || ''
          dueDateType.value = detail.due_date.type
        } else {
          dueDateInput.value = ''
          dueDateType.value = 'SOFT'
        }

        // 等待 DOM 更新后调整 textarea 高度
        await nextTick()
        initTextareaHeights()
      }
    }
  }
)

async function handleCompleteChange(isChecked: boolean) {
  if (!props.taskId) return

  if (isChecked) {
    await taskOps.completeTask(props.taskId)
  } else {
    await taskOps.reopenTask(props.taskId)
  }
}

async function handlePresenceToggle(isChecked: boolean) {
  if (!props.taskId || !todayDate.value) return

  // 使用新的勾选状态来决定 outcome
  const newOutcome = isChecked ? 'presence_logged' : undefined

  await taskStore.updateSchedule(props.taskId, todayDate.value, { outcome: newOutcome })
}

async function updateTitle() {
  if (!props.taskId || !task.value || titleInput.value === task.value.title) return
  await taskStore.updateTask(props.taskId, {
    title: titleInput.value,
  })
  isTitleEditing.value = false
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

async function updateArea(areaId: string | null) {
  if (!props.taskId || !task.value) return
  selectedAreaId.value = areaId
  await taskStore.updateTask(props.taskId, {
    area_id: areaId,
  })
  showAreaSelector.value = false
}

// 保存截止日期
async function saveDueDate() {
  if (!props.taskId || !task.value || !dueDateInput.value) return

  // 将日期字符串转为 UTC 当天零点（ISO 格式）
  const dateObj = parseDateString(dueDateInput.value) // 本地时区的当天零点
  const utcDate = new Date(
    Date.UTC(dateObj.getFullYear(), dateObj.getMonth(), dateObj.getDate(), 0, 0, 0, 0)
  )

  await taskStore.updateTask(props.taskId, {
    due_date: toUtcIsoString(utcDate),
    due_date_type: dueDateType.value,
  })

  showDueDatePicker.value = false
}

// 清除截止日期
async function clearDueDate() {
  if (!props.taskId || !task.value) return

  await taskStore.updateTask(props.taskId, {
    due_date: null,
    due_date_type: null,
  })

  dueDateInput.value = ''
  dueDateType.value = 'SOFT'
  showDueDatePicker.value = false
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

async function handleDeleteSubtask(subtaskId: string) {
  if (!props.taskId) return

  const updatedSubtasks = subtasks.value.filter((subtask) => subtask.id !== subtaskId)

  await taskStore.updateTask(props.taskId, {
    subtasks: updatedSubtasks,
  })
}

function handleDragStart(subtaskId: string) {
  draggingSubtaskId.value = subtaskId
}

function handleDragEnd() {
  draggingSubtaskId.value = null
}

function handleDragOver(event: DragEvent) {
  event.preventDefault()
}

async function handleDrop(event: DragEvent, targetSubtaskId: string) {
  event.preventDefault()

  if (!props.taskId || !draggingSubtaskId.value) return
  if (draggingSubtaskId.value === targetSubtaskId) return

  const currentSubtasks = [...subtasks.value]
  const dragIndex = currentSubtasks.findIndex((s) => s.id === draggingSubtaskId.value)
  const dropIndex = currentSubtasks.findIndex((s) => s.id === targetSubtaskId)

  if (dragIndex === -1 || dropIndex === -1) return

  // 移除被拖动的项
  const draggedItems = currentSubtasks.splice(dragIndex, 1)
  const draggedItem = draggedItems[0]

  if (!draggedItem) return

  // 插入到新位置
  currentSubtasks.splice(dropIndex, 0, draggedItem)

  // 更新sort_order
  const updatedSubtasks = currentSubtasks.map((subtask, index) => ({
    ...subtask,
    sort_order: `subtask_${Date.now()}_${index}`,
  }))

  await taskStore.updateTask(props.taskId, {
    subtasks: updatedSubtasks,
  })

  draggingSubtaskId.value = null
}

function handleOverlayMouseDown() {
  mouseDownOnOverlay.value = true
}

function handleOverlayClick() {
  // 只有在 overlay 上按下鼠标时才关闭
  if (mouseDownOnOverlay.value) {
    emit('close')
  }
  mouseDownOnOverlay.value = false
}

function handleCardMouseDown() {
  mouseDownOnOverlay.value = false
}

function handleClose() {
  emit('close')
}
</script>

<template>
  <div
    class="modal-overlay"
    @mousedown.self="handleOverlayMouseDown"
    @click.self="handleOverlayClick"
  >
    <CuteCard class="editor-card" @mousedown="handleCardMouseDown" @click.stop>
      <div v-if="task" class="content-wrapper">
        <!-- 第一栏：卡片标题栏 -->
        <div class="card-header-row">
          <div class="left-section">
            <!-- 区域标签 -->
            <div class="area-tag-wrapper" @click="showAreaSelector = !showAreaSelector">
              <AreaTag
                v-if="selectedArea"
                :name="selectedArea.name"
                :color="selectedArea.color"
                size="normal"
              />
              <div v-else class="no-area-placeholder">
                <span class="hash-symbol">#</span>
                <span>无区域</span>
              </div>
            </div>

            <!-- 简易区域选择器 -->
            <div v-if="showAreaSelector" class="area-selector-dropdown">
              <div
                v-for="area in Array.from(areaStore.areas.values())"
                :key="area.id"
                class="area-option"
                @click="updateArea(area.id)"
              >
                <AreaTag :name="area.name" :color="area.color" size="small" />
              </div>
              <div class="area-option" @click="updateArea(null)">
                <span class="no-area-text">清除区域</span>
              </div>
            </div>
          </div>

          <div class="right-section">
            <!-- 截止日期选择器 -->
            <div class="due-date-wrapper">
              <button class="due-date-button" @click="showDueDatePicker = !showDueDatePicker">
                <span v-if="task.due_date">{{
                  new Date(task.due_date.date).toLocaleDateString()
                }}</span>
                <span v-else class="placeholder">设置截止日期</span>
              </button>

              <!-- 截止日期选择器弹窗 -->
              <div v-if="showDueDatePicker" class="due-date-picker-popup" @click.stop>
                <div class="picker-section">
                  <label class="picker-label">日期</label>
                  <input type="date" v-model="dueDateInput" class="date-input" />
                </div>

                <div class="picker-section">
                  <label class="picker-label">类型</label>
                  <div class="deadline-type-buttons">
                    <button
                      class="type-button"
                      :class="{ active: dueDateType === 'SOFT' }"
                      @click="dueDateType = 'SOFT'"
                    >
                      软截止
                    </button>
                    <button
                      class="type-button"
                      :class="{ active: dueDateType === 'HARD' }"
                      @click="dueDateType = 'HARD'"
                    >
                      硬截止
                    </button>
                  </div>
                </div>

                <div class="picker-actions">
                  <button class="action-button save-button" @click="saveDueDate">保存</button>
                  <button
                    v-if="task.due_date"
                    class="action-button clear-button"
                    @click="clearDueDate"
                  >
                    清除
                  </button>
                  <button class="action-button cancel-button" @click="showDueDatePicker = false">
                    取消
                  </button>
                </div>
              </div>
            </div>

            <!-- × 按钮 -->
            <button class="close-button" @click="handleClose">×</button>
          </div>
        </div>

        <!-- 第二栏：任务标题栏 -->
        <div class="title-row">
          <CuteCheckbox
            :checked="task.is_completed"
            size="large"
            variant="check"
            @update:checked="handleCompleteChange"
          />
          <CuteCheckbox
            :checked="isPresenceLogged"
            size="large"
            variant="star"
            @update:checked="handlePresenceToggle"
          />
          <input
            v-model="titleInput"
            class="title-input"
            :class="{ completed: task.is_completed }"
            @blur="updateTitle"
            @keydown.enter="updateTitle"
          />
        </div>

        <!-- 第三栏：Glance Note 区域 -->
        <div class="note-area glance-note-area">
          <div
            v-if="!glanceNote && !isTitleEditing"
            class="note-placeholder"
            @click="isTitleEditing = true"
          >
            快速概览笔记...
          </div>
          <textarea
            ref="glanceNoteTextarea"
            v-model="glanceNote"
            class="note-textarea"
            placeholder="快速概览笔记..."
            rows="1"
            @input="autoResizeTextarea($event.target as HTMLTextAreaElement)"
            @blur="updateGlanceNote"
          ></textarea>
        </div>

        <!-- 分割线 -->
        <div class="separator"></div>

        <!-- 第四栏：子任务编辑区 -->
        <div class="subtasks-section">
          <div class="subtasks-header">子任务</div>
          <div class="subtasks-list">
            <div
              v-for="subtask in subtasks"
              :key="subtask.id"
              class="subtask-item"
              :class="{ dragging: draggingSubtaskId === subtask.id }"
              draggable="true"
              @dragstart="handleDragStart(subtask.id)"
              @dragend="handleDragEnd"
              @dragover="handleDragOver"
              @drop="handleDrop($event, subtask.id)"
            >
              <div class="drag-handle">⋮⋮</div>
              <CuteCheckbox
                :checked="subtask.is_completed"
                size="small"
                @update:checked="
                  (isChecked: boolean) => handleSubtaskStatusChange(subtask.id, isChecked)
                "
              />
              <span class="subtask-title" :class="{ completed: subtask.is_completed }">
                {{ subtask.title }}
              </span>
              <button class="delete-button" @click="handleDeleteSubtask(subtask.id)">×</button>
            </div>
          </div>
          <div class="add-subtask-form">
            <input
              v-model="newSubtaskTitle"
              class="add-subtask-input"
              placeholder="添加子任务..."
              @keydown.enter="handleAddSubtask"
            />
          </div>
        </div>

        <!-- 分割线 -->
        <div class="separator"></div>

        <!-- 第五栏：细节笔记区 -->
        <div class="note-area detail-note-area">
          <div v-if="!detailNote" class="note-placeholder">详细笔记...</div>
          <textarea
            ref="detailNoteTextarea"
            v-model="detailNote"
            class="note-textarea"
            placeholder="详细笔记..."
            rows="1"
            @input="autoResizeTextarea($event.target as HTMLTextAreaElement)"
            @blur="updateDetailNote"
          ></textarea>
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
  width: 70rem;
  max-width: 90vw;
  max-height: 90vh;
  padding: 2.5rem;
  border: 1px solid var(--color-border-default);
  background-color: var(--color-card-available);
  border-radius: 0.8rem;
  overflow-y: auto;
}

.content-wrapper {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

/* 第一栏：卡片标题栏 */
.card-header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 1.5rem;
  border-bottom: 2px solid var(--color-separator);
}

.left-section {
  display: flex;
  align-items: center;
  position: relative;
}

.area-tag-wrapper {
  cursor: pointer;
  transition: opacity 0.2s;
}

.area-tag-wrapper:hover {
  opacity: 0.7;
}

.area-selector-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  margin-top: 0.5rem;
  background: var(--color-card-available);
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  box-shadow: 0 4px 12px rgb(0 0 0 / 15%);
  z-index: 100;
  min-width: 20rem;
  max-height: 30rem;
  overflow-y: auto;
}

.area-option {
  padding: 0.8rem 1.2rem;
  cursor: pointer;
  transition: background-color 0.2s;
}

.area-option:hover {
  background-color: var(--color-background-soft, #f9f9f9);
}

.no-area-text {
  font-size: 1.3rem;
  color: var(--color-text-tertiary);
}

.no-area-placeholder {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 1.2rem;
  color: var(--color-text-tertiary);
  padding: 0.4rem 0.8rem;
  border: 1px dashed var(--color-border-default);
  border-radius: 0.4rem;
}

.no-area-placeholder .hash-symbol {
  font-size: 1.4rem;
  font-weight: 500;
}

.right-section {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.due-date-wrapper {
  position: relative;
}

.due-date-button {
  padding: 0.6rem 1.2rem;
  font-size: 1.3rem;
  color: var(--color-text-secondary);
  background: transparent;
  border: 1px solid var(--color-border-default);
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.2s;
}

.due-date-button:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.due-date-button .placeholder {
  color: var(--color-text-tertiary);
}

/* 截止日期选择器弹窗 */
.due-date-picker-popup {
  position: absolute;
  top: calc(100% + 0.4rem);
  right: 0;
  width: 26rem;
  background: white;
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  box-shadow: 0 4px 16px rgb(0 0 0 / 15%);
  padding: 1.5rem;
  z-index: 1000;
  display: flex;
  flex-direction: column;
  gap: 1.2rem;
}

.picker-section {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.picker-label {
  font-size: 1.2rem;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.date-input {
  padding: 0.6rem 1rem;
  font-size: 1.3rem;
  border: 1px solid var(--color-border-default);
  border-radius: 0.4rem;
  color: var(--color-text-primary);
}

.deadline-type-buttons {
  display: flex;
  gap: 0.8rem;
}

.type-button {
  flex: 1;
  padding: 0.6rem 1rem;
  font-size: 1.3rem;
  border: 1px solid var(--color-border-default);
  border-radius: 0.4rem;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.type-button:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.type-button.active {
  background-color: var(--color-primary, #1976d2);
  color: white;
  border-color: var(--color-primary, #1976d2);
}

.picker-actions {
  display: flex;
  gap: 0.8rem;
  margin-top: 0.5rem;
}

.action-button {
  flex: 1;
  padding: 0.6rem 1rem;
  font-size: 1.3rem;
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.15s;
}

.save-button {
  background-color: var(--color-primary, #1976d2);
  color: white;
}

.save-button:hover {
  background-color: var(--color-primary-dark, #1565c0);
}

.clear-button {
  background-color: #f44336;
  color: white;
}

.clear-button:hover {
  background-color: #d32f2f;
}

.cancel-button {
  background-color: #e0e0e0;
  color: var(--color-text-primary);
}

.cancel-button:hover {
  background-color: #bdbdbd;
}

.close-button {
  font-size: 3rem;
  line-height: 1;
  color: var(--color-text-tertiary);
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
  width: 3rem;
  height: 3rem;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color 0.2s;
}

.close-button:hover {
  color: var(--color-text-primary);
}

/* 第二栏：任务标题栏 */
.title-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.title-input {
  flex: 1;
  font-size: 2.4rem;
  font-weight: 600;
  color: var(--color-text-primary);
  background: transparent;
  border: none;
  outline: none;
  padding: 0.5rem 0;
  border-bottom: 2px solid transparent;
  transition: border-color 0.2s;
}

.title-input:focus {
  border-bottom-color: var(--color-primary);
}

.title-input.completed {
  text-decoration: line-through;
  color: var(--color-text-secondary);
}

/* 笔记区域 */
.note-area {
  position: relative;
  min-height: 4rem;
}

.note-placeholder {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  padding: 1rem;
  font-size: 1.5rem;
  color: var(--color-text-tertiary);
  cursor: text;
  pointer-events: none;
}

.note-textarea {
  width: 100%;
  font-family: inherit;
  font-size: 1.5rem;
  color: var(--color-text-primary);
  background: transparent;
  border: none;
  outline: none;
  resize: none;
  padding: 1rem;
  border-radius: 0.4rem;
  overflow: hidden;
  min-height: 2rem;
}

.note-textarea:hover {
  background: transparent;
}

.note-textarea:focus {
  background: transparent;
}

.note-textarea::placeholder {
  color: transparent;
}

/* 分割线 */
.separator {
  height: 1px;
  background-color: var(--color-separator);
}

/* 第四栏：子任务区 */
.subtasks-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.subtasks-header {
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.subtasks-list {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.subtask-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.4rem 0.6rem;
  border-radius: 0.4rem;
  transition: background-color 0.2s;
  cursor: move;
}

.subtask-item:hover {
  background-color: var(--color-background-soft, #f9f9f9);
}

.subtask-item.dragging {
  opacity: 0.5;
  background-color: var(--color-background-hover, #e8e8e8);
}

.drag-handle {
  cursor: grab;
  color: var(--color-text-tertiary);
  font-size: 1.4rem;
  line-height: 1;
  user-select: none;
}

.drag-handle:active {
  cursor: grabbing;
}

.subtask-title {
  flex: 1;
  font-size: 1.6rem;
  color: var(--color-text-primary);
}

.subtask-title.completed {
  text-decoration: line-through;
  color: var(--color-text-tertiary);
}

.delete-button {
  font-size: 2rem;
  line-height: 1;
  color: var(--color-text-tertiary);
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition:
    opacity 0.2s,
    color 0.2s;
}

.delete-button:hover {
  color: var(--color-danger, #ff4d4f);
}

.subtask-item:hover .delete-button {
  opacity: 1;
}

.add-subtask-form {
  margin-top: 0.5rem;
}

.add-subtask-input {
  width: 100%;
  padding: 1rem;
  font-size: 1.5rem;
  border: 1px dashed var(--color-border-default);
  border-radius: 0.4rem;
  background-color: transparent;
  color: var(--color-text-primary);
  transition: all 0.2s;
}

.add-subtask-input:focus {
  outline: none;
  border-style: solid;
  border-color: var(--color-primary);
  background-color: var(--color-background-soft, #f9f9f9);
}

.add-subtask-input::placeholder {
  color: var(--color-text-tertiary);
}
</style>
