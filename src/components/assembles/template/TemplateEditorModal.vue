<script setup lang="ts">
import { computed, ref, watch, onMounted, nextTick } from 'vue'
import { useTemplateStore } from '@/stores/template'
import { useAreaStore } from '@/stores/area'
import type { Template } from '@/types/dtos'
import CuteCard from '@/components/templates/CuteCard.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import AreaTag from '@/components/parts/AreaTag.vue'
import { pipeline } from '@/cpu'
import draggable from 'vuedraggable'

interface Subtask {
  id: string
  title: string
  is_completed: boolean
  sort_order: string
}

const props = defineProps<{
  templateId: string | null
}>()

const emit = defineEmits(['close'])

const templateStore = useTemplateStore()
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

const template = computed(() => {
  return props.templateId ? templateStore.getTemplateById(props.templateId) : null
})

// 使用 ref 而不是 computed，以便 vuedraggable 可以修改
const subtasks = ref<Subtask[]>([])

// 监听 template 变化，同步 subtasks
watch(
  () => template.value?.subtasks_template,
  (newSubtasks) => {
    if (newSubtasks) {
      subtasks.value = [...newSubtasks]
    } else {
      subtasks.value = []
    }
  },
  { immediate: true }
)

const selectedArea = computed(() => {
  return selectedAreaId.value ? areaStore.getAreaById(selectedAreaId.value) : null
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

// 当弹窗打开时，加载模板数据
onMounted(async () => {
  if (props.templateId && template.value) {
    titleInput.value = template.value.title
    glanceNoteTemplate.value = template.value.glance_note_template || ''
    detailNoteTemplate.value = template.value.detail_note_template || ''
    selectedAreaId.value = template.value.area_id || null

    // 等待 DOM 更新后调整 textarea 高度
    await nextTick()
    initTextareaHeights()
  }
})

watch(
  () => props.templateId,
  async (newTemplateId) => {
    if (newTemplateId && template.value) {
      titleInput.value = template.value.title
      glanceNoteTemplate.value = template.value.glance_note_template || ''
      detailNoteTemplate.value = template.value.detail_note_template || ''
      selectedAreaId.value = template.value.area_id || null

      // 等待 DOM 更新后调整 textarea 高度
      await nextTick()
      initTextareaHeights()
    }
  }
)

async function updateTitle() {
  if (!props.templateId || !template.value || titleInput.value === template.value.title) return
  await pipeline.dispatch('template.update', {
    id: props.templateId,
    title: titleInput.value,
  })
}

async function updateGlanceNoteTemplate() {
  if (!props.templateId || !template.value) return
  await pipeline.dispatch('template.update', {
    id: props.templateId,
    glance_note_template: glanceNoteTemplate.value || undefined,
  })
}

async function updateDetailNoteTemplate() {
  if (!props.templateId || !template.value) return
  await pipeline.dispatch('template.update', {
    id: props.templateId,
    detail_note_template: detailNoteTemplate.value || undefined,
  })
}

async function updateArea(areaId: string | null) {
  if (!props.templateId || !template.value) return
  selectedAreaId.value = areaId
  await pipeline.dispatch('template.update', {
    id: props.templateId,
    area_id: areaId || undefined,
  })
  showAreaSelector.value = false
}

async function handleAddSubtask() {
  if (!props.templateId || !newSubtaskTitle.value.trim()) return

  const newSubtask: Subtask = {
    id: crypto.randomUUID(),
    title: newSubtaskTitle.value.trim(),
    is_completed: false,
    sort_order: `subtask_${Date.now()}`,
  }

  const updatedSubtasks = [...subtasks.value, newSubtask]

  await pipeline.dispatch('template.update', {
    id: props.templateId,
    subtasks_template: updatedSubtasks,
  })

  newSubtaskTitle.value = ''
}

async function handleSubtaskStatusChange(subtaskId: string, isCompleted: boolean) {
  if (!props.templateId) return

  const updatedSubtasks = subtasks.value.map((subtask) =>
    subtask.id === subtaskId ? { ...subtask, is_completed: isCompleted } : subtask
  )

  await pipeline.dispatch('template.update', {
    id: props.templateId,
    subtasks_template: updatedSubtasks,
  })
}

async function handleDeleteSubtask(subtaskId: string) {
  if (!props.templateId) return

  const updatedSubtasks = subtasks.value.filter((subtask) => subtask.id !== subtaskId)

  await pipeline.dispatch('template.update', {
    id: props.templateId,
    subtasks_template: updatedSubtasks,
  })
}

async function handleSubtaskReorder() {
  if (!props.templateId) return

  // 更新 sort_order
  const updatedSubtasks = subtasks.value.map((subtask, index) => ({
    ...subtask,
    sort_order: `subtask_${Date.now()}_${index}`,
  }))

  await pipeline.dispatch('template.update', {
    id: props.templateId,
    subtasks_template: updatedSubtasks,
  })
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
      <div v-if="template" class="content-wrapper">
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
          <CuteCheckbox :checked="false" size="large" variant="check" :disabled="true" />
          <input
            v-model="titleInput"
            class="title-input"
            placeholder="模板标题"
            @blur="updateTitle"
            @keydown.enter="updateTitle"
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
            @blur="updateGlanceNoteTemplate"
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
            @blur="updateDetailNoteTemplate"
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

/* 第二栏：标题输入栏 */
.title-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.title-input {
  flex: 1;
  font-size: 2rem;
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
