<script setup lang="ts">
import { computed, ref, watch, onMounted, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTemplateStore } from '@/stores/template'
import { useAreaStore } from '@/stores/area'
import CuteCard from '@/components/templates/CuteCard.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import AreaTag from '@/components/parts/AreaTag.vue'
import { pipeline } from '@/cpu'
import draggable from 'vuedraggable'
import { logger, LogTags } from '@/infra/logging/logger'

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

const { t } = useI18n()
const templateStore = useTemplateStore()
const areaStore = useAreaStore()

// 本地编辑状态
const titleInput = ref('')
const glanceNoteTemplate = ref('')
const selectedAreaId = ref<string | null>(null)
const newSubtaskTitle = ref('')
const showAreaSelector = ref(false)
const glanceNoteTextarea = ref<HTMLTextAreaElement | null>(null)
const mouseDownOnOverlay = ref(false)

const template = computed(() => {
  return props.templateId ? templateStore.getTemplateById(props.templateId) : null
})

// 🔥 监听模板是否存在，如果模板被删除则自动关闭编辑框
watch(template, (newTemplate) => {
  if (props.templateId && !newTemplate) {
    logger.info(LogTags.COMPONENT_KANBAN, 'Template no longer exists, closing editor', {
      templateId: props.templateId,
    })
    emit('close')
  }
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
}

// 当弹窗打开时，加载模板数据
onMounted(async () => {
  if (props.templateId && template.value) {
    titleInput.value = template.value.title
    glanceNoteTemplate.value = template.value.glance_note_template || ''
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

  // 新子任务添加到最前面
  const updatedSubtasks = [newSubtask, ...subtasks.value]

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
      <div v-if="template">
        <!-- 卡片头部 -->
        <div class="card-header">
          <div class="header-left">
            <!-- 区域标签 -->
            <div class="area-tag-wrapper" @click="showAreaSelector = !showAreaSelector">
              <AreaTag
                v-if="selectedArea"
                :name="selectedArea.name"
                :color="selectedArea.color"
                size="normal"
              />
              <div v-else class="no-area-placeholder">
                <CuteIcon name="Hash" :size="16" />
                <span>{{ $t('task.label.noArea') }}</span>
              </div>
            </div>

            <!-- 区域选择器下拉 -->
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
                <span class="no-area-text">{{ $t('area.action.clearArea') }}</span>
              </div>
            </div>
          </div>

          <div class="header-right">
            <!-- 关闭按钮 -->
            <button class="close-button" @click="handleClose">×</button>
          </div>
        </div>

        <!-- 主内容区 -->
        <div class="card-body">
          <!-- 模板标题区域（无图标，标题左对齐） -->
          <div class="section section-title">
            <input
              v-model="titleInput"
              class="title-input"
              :placeholder="$t('template.placeholder.title')"
              autocomplete="off"
              @blur="updateTitle"
              @keydown.enter="updateTitle"
            />
          </div>

          <!-- 任务描述模板区域 -->
          <div class="section section-note">
            <div class="section-icon">
              <CuteIcon name="FileText" :size="20" />
            </div>
            <div class="section-body">
              <div v-if="!glanceNoteTemplate" class="note-placeholder">
                {{ $t('task.placeholder.glanceNoteTemplate') }}
              </div>
              <textarea
                ref="glanceNoteTextarea"
                v-model="glanceNoteTemplate"
                class="note-textarea"
                :placeholder="$t('task.placeholder.glanceNoteTemplate')"
                rows="1"
                @input="autoResizeTextarea($event.target as HTMLTextAreaElement)"
                @blur="updateGlanceNoteTemplate"
              ></textarea>
            </div>
          </div>

          <!-- 子任务模板区域 -->
          <div class="section section-subtasks">
            <div class="section-header">
              <div class="section-icon">
                <CuteIcon name="List" :size="20" />
              </div>
              <span class="section-title-text">{{ $t('task.label.subtaskTemplate') }}</span>
            </div>
            <div class="section-body">
              <div class="subtasks-input">
                <input
                  v-model="newSubtaskTitle"
                  class="add-subtask-input"
                  :placeholder="$t('task.placeholder.addSubtaskTemplate')"
                  autocomplete="off"
                  @keydown.enter="handleAddSubtask"
                />
              </div>
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
            </div>
          </div>
        </div>

        <!-- 底栏 -->
        <div class="card-footer">
          <div class="footer-actions">
            <button class="footer-button confirm-footer-button" @click="handleClose">
              {{ $t('task.button.done') }}
            </button>
          </div>
        </div>
      </div>
    </CuteCard>
  </div>
</template>

<style scoped>
/* ==================== 模态框基础 ==================== */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background-color: var(--color-overlay-heavy);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.editor-card {
  width: 63rem;
  max-width: 90vw;
  max-height: 90vh;
  border: 1px solid var(--color-border-subtle, #f0f);
  background-color: var(--color-background-content, #f0f);
  border-radius: 0.8rem;
  overflow-y: auto;
  padding: 0;
}

/* ==================== 卡片头部 ==================== */
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 2rem 4.1rem;
  border-bottom: 1px solid var(--color-border-default);
}

.header-left {
  display: flex;
  align-items: center;
  position: relative;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 1rem;
}

/* 区域标签 */
.area-tag-wrapper {
  cursor: pointer;
  transition: opacity 0.2s;
}

.area-tag-wrapper:hover {
  opacity: 0.7;
}

.no-area-placeholder {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 1.2rem;
  line-height: 1.4;
  color: var(--color-text-tertiary);
  padding: 0.4rem 0.8rem;
  border: 1px dashed var(--color-border-default);
  border-radius: 0.4rem;
}

.area-selector-dropdown {
  position: absolute;
  top: calc(100% + 0.5rem);
  left: 0;
  background: var(--color-card-available);
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  box-shadow: var(--shadow-md, #f0f);
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
  background-color: var(--color-background-hover);
}

.no-area-text {
  font-size: 1.3rem;
  line-height: 1.4;
  color: var(--color-text-tertiary);
}

/* 关闭按钮 */
.close-button {
  font-size: 3rem;
  line-height: 1;
  color: var(--color-text-tertiary);
  background: none;
  border: none;
  cursor: pointer;
  padding: 0.4rem;
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

/* ==================== 主内容区 ==================== */
.card-body {
  padding: 0 4.1rem;
}

/* ==================== 统一Section样式 ==================== */
.section {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1.7rem 0 0;
}

.section:first-child {
  padding-top: 2.5rem;
}

.section-icon {
  flex-shrink: 0;
  width: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-tertiary);
}

.section-body {
  flex: 1;
  min-width: 0;
}

/* ==================== 模板标题区域 ==================== */
.title-input {
  width: 100%;
  font-size: 2rem;
  font-weight: 600;
  line-height: 1.4;
  color: var(--color-text-primary);
  background: transparent;
  border: none;
  outline: none;
  padding: 0;
  border-bottom: 2px solid transparent;
  transition: border-color 0.2s;
}

.title-input:focus {
  border-bottom-color: var(--color-border-default);
}

.title-input::placeholder {
  color: var(--color-text-tertiary);
}

/* ==================== 笔记区域 ==================== */
.section-note {
  border-bottom: 1px solid var(--color-border-default);
  align-items: flex-start;
  padding-top: 0.7rem;
}

.section-note .section-icon {
  padding-top: 1rem;
}

.note-placeholder {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  padding: 1rem 0;
  font-size: 1.5rem;
  line-height: 1.4;
  color: var(--color-text-tertiary);
  cursor: text;
  pointer-events: none;
}

.note-textarea {
  width: 100%;
  font-family: inherit;
  font-size: 1.5rem;
  line-height: 1.4;
  color: var(--color-text-primary);
  background: transparent;
  border: none;
  outline: none;
  resize: none;
  padding: 1rem 0;
  border-radius: 0.4rem;
  overflow: hidden;
  min-height: 2rem;
}

.note-textarea:hover,
.note-textarea:focus {
  background: transparent;
}

.note-textarea::placeholder {
  color: transparent;
}

.section-note .section-body {
  position: relative;
  min-height: 10rem;
}

/* ==================== 子任务区域 ==================== */
.section-subtasks {
  flex-direction: column;
  align-items: stretch;
  gap: 0;
  border-bottom: 1px solid var(--color-border-default);
}

.section-header {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.section-subtasks .section-header {
  padding-top: 0;
}

.section-subtasks .section-body {
  padding-top: 1rem;
  min-height: 12rem;
}

.section-title-text {
  font-size: 1.6rem;
  font-weight: 600;
  line-height: 1.4;
  color: var(--color-text-secondary);
}

.subtasks-input {
  padding: 0.5rem 0;
}

.add-subtask-input {
  width: 100%;
  padding: 0.2rem 0;
  font-size: 1.5rem;
  line-height: 1.4;
  border: none;
  background-color: transparent;
  color: var(--color-text-primary);
  outline: none;
  transition: all 0.2s;
}

.add-subtask-input::placeholder {
  color: var(--color-text-tertiary);
}

.subtasks-list {
  display: flex;
  flex-direction: column;
}

.subtask-item {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  padding: 0.5rem 0;
  border-radius: 0.4rem;
  transition: background-color 0.2s;
  cursor: move;
  position: relative;
}

.subtask-item:hover {
  background-color: var(--color-background-hover, #f0f);
}

.drag-handle {
  position: absolute;
  left: -2.8rem;
  top: 50%;
  transform: translateY(-50%);
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.4rem;
  height: 2.8rem;
  cursor: grab;
  color: var(--color-text-tertiary);
  font-size: 1.6rem;
  line-height: 1;
  user-select: none;
  opacity: 0;
  transition:
    opacity 0.2s ease,
    color 0.2s ease;
  border-radius: 0.4rem;
}

.drag-handle:hover {
  color: var(--color-text-secondary);
  background-color: var(--color-background-hover, #f0f);
}

.drag-handle:active {
  cursor: grabbing;
  color: var(--color-text-primary);
}

.subtask-item:hover .drag-handle {
  opacity: 1;
}

.subtask-title {
  flex: 1;
  font-size: 1.6rem;
  line-height: 1.4;
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
  color: var(--color-danger);
}

.subtask-item:hover .delete-button {
  opacity: 1;
}

/* ==================== 底栏 ==================== */
.card-footer {
  padding: 2rem 4.1rem;
  display: flex;
  justify-content: flex-end;
}

.footer-actions {
  display: flex;
  gap: 1rem;
}

.footer-button {
  padding: 0.8rem 1.6rem;
  border-radius: 0.6rem;
  font-size: 1.4rem;
  font-weight: 500;
  line-height: 1.4;
  cursor: pointer;
  transition: all 0.15s ease;
  min-width: 8rem;
}

.confirm-footer-button {
  background-color: var(--color-button-primary-bg);
  color: var(--color-button-primary-text, #f0f);
  border: 1px solid var(--color-button-primary-bg);
}

.confirm-footer-button:hover {
  background-color: var(--color-button-primary-hover, #f0f);
}
</style>
