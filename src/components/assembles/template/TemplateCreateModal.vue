<script setup lang="ts">
import { ref, nextTick, onMounted } from 'vue'
import { useAreaStore } from '@/stores/area'
import CuteCard from '@/components/templates/CuteCard.vue'
import AreaTag from '@/components/parts/AreaTag.vue'
import { pipeline } from '@/cpu'
import draggable from 'vuedraggable'
import { logger, LogTags } from '@/infra/logging/logger'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'

interface Subtask {
  id: string
  title: string
  is_completed: boolean
  sort_order: string
}

const emit = defineEmits(['close'])

const areaStore = useAreaStore()

// 本地编辑状态
const titleInput = ref('')
const glanceNoteTemplate = ref('')
const detailNoteTemplate = ref('')
const selectedAreaId = ref<string | null>(null)
const newSubtaskTitle = ref('')
const showAreaSelector = ref(false)
const glanceNoteTextarea = ref<HTMLTextAreaElement | null>(null)
const detailNoteTextarea = ref<HTMLTextAreaElement | null>(null)
const mouseDownOnOverlay = ref(false)
const titleInputRef = ref<HTMLInputElement | null>(null)

// 使用 ref 而不是 computed，以便 vuedraggable 可以修改
const subtasks = ref<Subtask[]>([])

const selectedArea = ref(selectedAreaId.value ? areaStore.getAreaById(selectedAreaId.value) : null)

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

// 当弹窗打开时，聚焦到标题输入框
onMounted(async () => {
  await nextTick()
  titleInputRef.value?.focus()
})

function updateArea(areaId: string | null) {
  selectedAreaId.value = areaId
  selectedArea.value = areaId ? areaStore.getAreaById(areaId) : null
  showAreaSelector.value = false
}

function handleAddSubtask() {
  if (!newSubtaskTitle.value.trim()) return

  const newSubtask: Subtask = {
    id: crypto.randomUUID(),
    title: newSubtaskTitle.value.trim(),
    is_completed: false,
    sort_order: `subtask_${Date.now()}`,
  }

  subtasks.value = [...subtasks.value, newSubtask]
  newSubtaskTitle.value = ''
}

function handleSubtaskStatusChange(subtaskId: string, isCompleted: boolean) {
  subtasks.value = subtasks.value.map((subtask) =>
    subtask.id === subtaskId ? { ...subtask, is_completed: isCompleted } : subtask
  )
}

function handleDeleteSubtask(subtaskId: string) {
  subtasks.value = subtasks.value.filter((subtask) => subtask.id !== subtaskId)
}

function handleSubtaskReorder() {
  // 更新 sort_order
  subtasks.value = subtasks.value.map((subtask, index) => ({
    ...subtask,
    sort_order: `subtask_${Date.now()}_${index}`,
  }))
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

async function handleCreate() {
  const title = titleInput.value.trim()
  if (!title) {
    alert('请输入模板标题')
    return
  }

  try {
    await pipeline.dispatch('template.create', {
      title,
      glance_note_template: glanceNoteTemplate.value || undefined,
      detail_note_template: detailNoteTemplate.value || undefined,
      area_id: selectedAreaId.value || undefined,
      subtasks_template: subtasks.value.length > 0 ? subtasks.value : undefined,
    })

    logger.info(LogTags.COMPONENT_KANBAN_COLUMN, 'Template created successfully', { title })
    emit('close')
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_KANBAN_COLUMN,
      'Failed to create template',
      error instanceof Error ? error : new Error(String(error))
    )
    alert('创建模板失败')
  }
}
</script>

<template>
  <div
    class="modal-overlay"
    @mousedown.self="handleOverlayMouseDown"
    @click.self="handleOverlayClick"
  >
    <CuteCard class="editor-card" @mousedown="handleCardMouseDown" @click.stop>
      <div class="content-wrapper">
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
            <!-- × 按钮 -->
            <button class="close-button" @click="handleClose">×</button>
          </div>
        </div>

        <!-- 第二栏：标题输入栏 -->
        <div class="title-row">
          <input
            ref="titleInputRef"
            v-model="titleInput"
            class="title-input"
            placeholder="模板标题"
            @keydown.enter="handleCreate"
          />
        </div>

        <!-- 第三栏：Glance Note Template 区域 -->
        <div class="note-area glance-note-area">
          <div v-if="!glanceNoteTemplate" class="note-placeholder">快速概览笔记模板...</div>
          <textarea
            ref="glanceNoteTextarea"
            v-model="glanceNoteTemplate"
            class="note-textarea"
            placeholder="快速概览笔记模板..."
            rows="1"
            @input="autoResizeTextarea($event.target as HTMLTextAreaElement)"
          ></textarea>
        </div>

        <!-- 分割线 -->
        <div class="separator"></div>

        <!-- 第四栏：子任务模板编辑区 -->
        <div class="subtasks-section">
          <div class="subtasks-header">子任务模板</div>
          <draggable
            v-model="subtasks"
            item-key="id"
            class="subtasks-list"
            handle=".drag-handle"
            @end="handleSubtaskReorder"
          >
            <template #item="{ element: subtask }">
              <div class="subtask-item">
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
            </template>
          </draggable>
          <div class="add-subtask-form">
            <input
              v-model="newSubtaskTitle"
              class="add-subtask-input"
              placeholder="添加子任务模板..."
              @keydown.enter="handleAddSubtask"
            />
          </div>
        </div>

        <!-- 分割线 -->
        <div class="separator"></div>

        <!-- 第五栏：细节笔记模板区 -->
        <div class="note-area detail-note-area">
          <div v-if="!detailNoteTemplate" class="note-placeholder">详细笔记模板...</div>
          <textarea
            ref="detailNoteTextarea"
            v-model="detailNoteTemplate"
            class="note-textarea"
            placeholder="详细笔记模板..."
            rows="1"
            @input="autoResizeTextarea($event.target as HTMLTextAreaElement)"
          ></textarea>
        </div>

        <!-- 第六栏：操作按钮 -->
        <div class="action-buttons">
          <button class="cancel-button" @click="handleClose">取消</button>
          <button class="create-button" @click="handleCreate">创建</button>
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
  width: 100%;
  height: 100%;
  background-color: rgb(0 0 0 / 50%);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
}

.editor-card {
  width: min(90%, 80rem);
  max-height: 90vh;
  overflow-y: auto;
  background-color: var(--color-background-content);
  border-radius: 1.2rem;
  box-shadow: 0 8px 32px rgb(0 0 0 / 16%);
}

.content-wrapper {
  padding: 2rem;
  display: flex;
  flex-direction: column;
  gap: 1.6rem;
}

/* 第一栏：卡片标题栏 */
.card-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.left-section {
  position: relative;
  display: flex;
  align-items: center;
  gap: 1rem;
}

.area-tag-wrapper {
  cursor: pointer;
  transition: opacity 0.2s;
}

.area-tag-wrapper:hover {
  opacity: 0.7;
}

.no-area-placeholder {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.4rem 0.8rem;
  border-radius: 0.4rem;
  background-color: var(--color-background-secondary);
  color: var(--color-text-tertiary);
  font-size: 1.3rem;
  font-weight: 500;
}

.hash-symbol {
  font-size: 1.4rem;
  font-weight: 600;
}

.area-selector-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  margin-top: 0.4rem;
  background-color: var(--color-background-content);
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
  box-shadow: 0 4px 12px rgb(0 0 0 / 10%);
  overflow: hidden;
  z-index: 1000;
  min-width: 16rem;
}

.area-option {
  padding: 0.8rem 1.2rem;
  cursor: pointer;
  transition: background-color 0.2s;
}

.area-option:hover {
  background-color: var(--color-background-hover);
}

.no-area-text {
  font-size: 1.3rem;
  color: var(--color-text-tertiary);
}

.right-section {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.close-button {
  width: 3.2rem;
  height: 3.2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: transparent;
  border: none;
  border-radius: 0.4rem;
  font-size: 2.4rem;
  font-weight: 300;
  color: var(--color-text-tertiary);
  cursor: pointer;
  transition: all 0.2s;
  line-height: 1;
}

.close-button:hover {
  background-color: var(--color-background-hover);
  color: var(--color-text-primary);
}

/* 第二栏：标题输入栏 */
.title-row {
  display: flex;
  align-items: center;
}

.title-input {
  width: 100%;
  font-size: 2rem;
  font-weight: 600;
  color: var(--color-text-primary);
  background-color: transparent;
  border: none;
  outline: none;
  padding: 0.6rem 0;
}

.title-input::placeholder {
  color: var(--color-text-tertiary);
}

/* 第三栏：Glance Note 区域 */
.note-area {
  position: relative;
  min-height: 4rem;
}

.note-placeholder {
  position: absolute;
  top: 0.8rem;
  left: 0;
  font-size: 1.4rem;
  color: var(--color-text-tertiary);
  pointer-events: none;
}

.note-textarea {
  width: 100%;
  font-size: 1.4rem;
  color: var(--color-text-primary);
  background-color: transparent;
  border: none;
  outline: none;
  resize: none;
  overflow: hidden;
  padding: 0.8rem 0;
  line-height: 1.6;
}

.note-textarea::placeholder {
  color: var(--color-text-tertiary);
}

/* 分割线 */
.separator {
  height: 1px;
  background-color: var(--color-border-default);
  margin: 0.8rem 0;
}

/* 第四栏：子任务模板编辑区 */
.subtasks-section {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.subtasks-header {
  font-size: 1.4rem;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.subtasks-list {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

.subtask-item {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  padding: 0.8rem;
  background-color: var(--color-background-secondary);
  border-radius: 0.6rem;
  transition: background-color 0.2s;
}

.subtask-item:hover {
  background-color: var(--color-background-hover);
}

.drag-handle {
  cursor: grab;
  color: var(--color-text-tertiary);
  font-size: 1.4rem;
  user-select: none;
}

.drag-handle:active {
  cursor: grabbing;
}

.subtask-title {
  flex: 1;
  font-size: 1.4rem;
  color: var(--color-text-primary);
  transition: all 0.2s;
}

.subtask-title.completed {
  color: var(--color-text-tertiary);
  text-decoration: line-through;
  opacity: 0.6;
}

.delete-button {
  width: 2.4rem;
  height: 2.4rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: transparent;
  border: none;
  border-radius: 0.4rem;
  font-size: 2rem;
  font-weight: 300;
  color: var(--color-text-tertiary);
  cursor: pointer;
  transition: all 0.2s;
  line-height: 1;
}

.delete-button:hover {
  background-color: var(--color-background-content);
  color: var(--color-danger, #e74c3c);
}

.add-subtask-form {
  display: flex;
  gap: 0.8rem;
}

.add-subtask-input {
  flex: 1;
  padding: 0.8rem 1.2rem;
  font-size: 1.4rem;
  color: var(--color-text-primary);
  background-color: var(--color-background-secondary);
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  outline: none;
  transition: all 0.2s;
}

.add-subtask-input:focus {
  border-color: var(--color-primary, #4a90e2);
  box-shadow: 0 0 0 3px rgb(74 144 226 / 10%);
}

.add-subtask-input::placeholder {
  color: var(--color-text-tertiary);
}

/* 第六栏：操作按钮 */
.action-buttons {
  display: flex;
  justify-content: flex-end;
  gap: 1rem;
  padding-top: 1rem;
}

.cancel-button,
.create-button {
  padding: 0.8rem 2rem;
  font-size: 1.4rem;
  font-weight: 600;
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
}

.cancel-button {
  background-color: var(--color-background-secondary);
  color: var(--color-text-secondary);
}

.cancel-button:hover {
  background-color: var(--color-background-hover);
  color: var(--color-text-primary);
}

.create-button {
  background-color: var(--color-primary, #4a90e2);
  color: white;
}

.create-button:hover {
  opacity: 0.9;
}
</style>
