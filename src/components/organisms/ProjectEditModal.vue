<template>
  <Teleport to="body">
    <div v-if="show" class="modal-overlay" @click="handleOverlayClick">
      <div class="modal-dialog" @click.stop>
        <div class="dialog-header">
          <h3>{{ $t('project.title.edit') }}</h3>
          <button class="close-button" @click="close">
            <CuteIcon name="X" :size="18" />
          </button>
        </div>
        <div class="dialog-body">
          <div class="form-group">
            <label for="edit-project-name">{{ $t('project.field.name') }} <span class="required">{{ $t('common.label.required') }}</span></label>
            <input
              id="edit-project-name"
              ref="nameInputRef"
              v-model="formData.name"
              type="text"
              class="form-input"
              :placeholder="$t('project.placeholder.name')"
              autocomplete="off"
              @keydown.enter="handleSubmit"
            />
          </div>

          <div class="form-group">
            <label for="edit-project-description">{{ $t('project.field.description') }}</label>
            <textarea
              id="edit-project-description"
              v-model="formData.description"
              class="form-textarea"
              :placeholder="$t('project.placeholder.description')"
              rows="3"
            />
          </div>

          <div class="form-row">
            <div class="form-group">
              <label for="edit-project-area">{{ $t('project.field.area') }}</label>
              <select id="edit-project-area" v-model="formData.area_id" class="form-select">
                <option :value="null">{{ $t('project.label.noArea') }}</option>
                <option v-for="area in areas" :key="area.id" :value="area.id">
                  {{ area.name }}
                </option>
              </select>
            </div>

            <div class="form-group">
              <label for="edit-project-due-date">{{ $t('project.field.dueDate') }}</label>
              <input
                id="edit-project-due-date"
                v-model="formData.due_date"
                type="date"
                class="form-input"
              />
            </div>
          </div>

          <div class="form-group">
            <label for="edit-project-status">{{ $t('project.field.status') }}</label>
            <select id="edit-project-status" v-model="formData.status" class="form-select">
              <option value="ACTIVE">{{ $t('project.status.active') }}</option>
              <option value="COMPLETED">{{ $t('project.status.completed') }}</option>
            </select>
          </div>

          <!-- 节段管理区域 -->
          <div class="section-manager">
            <div class="section-header">
              <CuteIcon name="List" :size="18" />
              <span class="section-title">{{ $t('project.field.sections') }}</span>
            </div>
            <div class="section-body">
              <!-- 添加新节段输入框 -->
              <div class="add-section-input">
                <input
                  v-model="newSectionTitle"
                  class="section-input"
                  :placeholder="$t('project.placeholder.addSection')"
                  autocomplete="off"
                  @keydown.enter="handleAddSection"
                />
              </div>
              <!-- 节段列表 -->
              <draggable
                v-model="sections"
                item-key="id"
                class="sections-list"
                handle=".drag-handle"
                @end="handleSectionReorder"
              >
                <template #item="{ element: section }">
                  <div class="section-item">
                    <div class="drag-handle">⋮⋮</div>
                    <div v-if="editingSectionId !== section.id" class="section-content">
                      <span class="section-name">{{ section.title }}</span>
                      <div class="section-actions">
                        <button class="action-btn edit" @click="startEditSection(section)">
                          <CuteIcon name="Pencil" :size="14" />
                        </button>
                        <button class="action-btn delete" @click="handleDeleteSection(section.id)">
                          <CuteIcon name="Trash2" :size="14" />
                        </button>
                      </div>
                    </div>
                    <div v-else class="section-edit">
                      <input
                        ref="editInputRef"
                        v-model="editingSectionTitle"
                        class="section-edit-input"
                        @keydown.enter="saveEditSection"
                        @keydown.escape="cancelEditSection"
                        @blur="saveEditSection"
                      />
                    </div>
                  </div>
                </template>
              </draggable>
              <div v-if="sections.length === 0" class="empty-sections">
                {{ $t('project.label.noSections') }}
              </div>
            </div>
          </div>
        </div>
        <div class="dialog-footer">
          <button class="cancel-button" @click="close">{{ $t('common.action.cancel') }}</button>
          <button
            class="submit-button"
            :disabled="!formData.name.trim() || isSubmitting"
            @click="handleSubmit"
          >
            {{ isSubmitting ? $t('project.button.saving') : $t('project.button.save') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import { pipeline } from '@/cpu'
import { useAreaStore } from '@/stores/area'
import { useProjectStore } from '@/stores/project'
import { logger, LogTags } from '@/infra/logging/logger'
import { dialog } from '@/composables/useDialog'
import draggable from 'vuedraggable'
import type { ProjectSection } from '@/types/dtos'

const props = defineProps<{
  show: boolean
  projectId: string | null
}>()

const emit = defineEmits<{
  close: []
  success: []
}>()

const { t } = useI18n()
const areaStore = useAreaStore()
const projectStore = useProjectStore()

const nameInputRef = ref<HTMLInputElement | null>(null)
const editInputRef = ref<HTMLInputElement | null>(null)
const isSubmitting = ref(false)

const formData = ref({
  name: '',
  description: '',
  area_id: null as string | null,
  due_date: '',
  status: 'ACTIVE' as 'ACTIVE' | 'COMPLETED',
})

// 节段相关状态
const newSectionTitle = ref('')
const sections = ref<ProjectSection[]>([])
const editingSectionId = ref<string | null>(null)
const editingSectionTitle = ref('')

const areas = computed(() => areaStore.allAreas)

// 当对话框显示时，加载项目数据
watch(
  () => props.show,
  async (newShow) => {
    if (newShow && props.projectId) {
      const project = projectStore.getProjectById(props.projectId)
      if (project) {
        formData.value = {
          name: project.name,
          description: project.description || '',
          area_id: project.area_id,
          due_date: project.due_date || '',
          status: project.status,
        }
      }
      // 加载节段数据
      await loadSections()
      isSubmitting.value = false
      await nextTick()
      nameInputRef.value?.focus()
    } else {
      // 关闭时重置状态
      newSectionTitle.value = ''
      sections.value = []
      editingSectionId.value = null
      editingSectionTitle.value = ''
    }
  }
)

// 监听 store 中 sections 的变化
watch(
  () => props.projectId ? projectStore.getSectionsByProject(props.projectId) : [],
  (newSections) => {
    if (props.show && props.projectId) {
      sections.value = [...newSections]
    }
  },
  { deep: true }
)

async function loadSections() {
  if (!props.projectId) return

  try {
    await pipeline.dispatch('project_section.fetch_all', {
      project_id: props.projectId,
    })
    sections.value = [...projectStore.getSectionsByProject(props.projectId)]
  } catch (error) {
    logger.error(LogTags.UI, '加载节段失败', error)
  }
}

function close() {
  emit('close')
}

function handleOverlayClick() {
  close()
}

async function handleSubmit() {
  const name = formData.value.name.trim()
  if (!name || isSubmitting.value || !props.projectId) return

  isSubmitting.value = true

  try {
    logger.info(LogTags.UI, '更新项目', formData.value)

    await pipeline.dispatch('project.update', {
      id: props.projectId,
      name,
      description: formData.value.description.trim() || null,
      area_id: formData.value.area_id,
      due_date: formData.value.due_date || null,
      status: formData.value.status,
    })

    logger.info(LogTags.UI, '项目更新成功')
    emit('success')
    close()
  } catch (error) {
    logger.error(LogTags.UI, '项目更新失败', error)
    await dialog.alert(t('message.error.updateProjectFailed'))
  } finally {
    isSubmitting.value = false
  }
}

// 添加新节段
async function handleAddSection() {
  if (!props.projectId || !newSectionTitle.value.trim()) return

  try {
    await pipeline.dispatch('project_section.create', {
      project_id: props.projectId,
      title: newSectionTitle.value.trim(),
      sort_order: `section_${Date.now()}`,
    })
    newSectionTitle.value = ''
    logger.info(LogTags.UI, '节段创建成功')
  } catch (error) {
    logger.error(LogTags.UI, '节段创建失败', error)
    await dialog.alert(t('message.error.createSectionFailed'))
  }
}

// 开始编辑节段
function startEditSection(section: ProjectSection) {
  editingSectionId.value = section.id
  editingSectionTitle.value = section.title
  nextTick(() => {
    editInputRef.value?.focus()
    editInputRef.value?.select()
  })
}

// 保存编辑
async function saveEditSection() {
  if (!editingSectionId.value || !editingSectionTitle.value.trim() || !props.projectId) {
    cancelEditSection()
    return
  }

  const section = sections.value.find(s => s.id === editingSectionId.value)
  if (!section || section.title === editingSectionTitle.value.trim()) {
    cancelEditSection()
    return
  }

  try {
    await pipeline.dispatch('project_section.update', {
      project_id: props.projectId,
      id: editingSectionId.value,
      title: editingSectionTitle.value.trim(),
    })
    logger.info(LogTags.UI, '节段更新成功')
  } catch (error) {
    logger.error(LogTags.UI, '节段更新失败', error)
    await dialog.alert(t('message.error.updateSectionFailed'))
  } finally {
    cancelEditSection()
  }
}

// 取消编辑
function cancelEditSection() {
  editingSectionId.value = null
  editingSectionTitle.value = ''
}

// 删除节段
async function handleDeleteSection(sectionId: string) {
  if (!props.projectId) return

  const confirmed = await dialog.confirm(t('confirm.deleteSection'))
  if (!confirmed) return

  try {
    await pipeline.dispatch('project_section.delete', {
      project_id: props.projectId,
      id: sectionId,
    })
    logger.info(LogTags.UI, '节段删除成功')
  } catch (error) {
    logger.error(LogTags.UI, '节段删除失败', error)
    await dialog.alert(t('message.error.deleteSectionFailed'))
  }
}

// 节段重新排序
async function handleSectionReorder(event: { oldIndex?: number; newIndex?: number }) {
  if (!props.projectId) return

  const { oldIndex, newIndex } = event
  if (oldIndex === undefined || newIndex === undefined || oldIndex === newIndex) return

  // 获取被移动的节段（此时 sections.value 已被 vuedraggable 更新为新顺序）
  const movedSection = sections.value[newIndex]
  if (!movedSection) return

  // 计算新位置的前后邻居
  const prevSection = newIndex > 0 ? sections.value[newIndex - 1] : null
  const nextSection = newIndex < sections.value.length - 1 ? sections.value[newIndex + 1] : null

  try {
    await pipeline.dispatch('project_section.reorder', {
      project_id: props.projectId,
      section_id: movedSection.id,
      prev_section_id: prevSection?.id ?? null,
      next_section_id: nextSection?.id ?? null,
    })
    logger.info(LogTags.UI, '节段排序成功', { sectionId: movedSection.id, newIndex })
  } catch (error) {
    logger.error(LogTags.UI, '节段排序失败', error)
    // 排序失败时，从 store 重新加载以恢复正确顺序
    await loadSections()
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background-color: var(--color-overlay-heavy, #f0f);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  backdrop-filter: blur(2px);
}

.modal-dialog {
  background-color: var(--color-background-content, #f0f);
  border: 1px solid var(--color-border-subtle, #f0f);
  border-radius: 1.2rem;
  box-shadow: var(--shadow-lg);
  width: 90%;
  max-width: 560px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.6rem 2rem;
  border-bottom: 1px solid var(--color-border-light);
}

.dialog-header h3 {
  margin: 0;
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
}

.close-button {
  all: unset;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3.2rem;
  height: 3.2rem;
  border-radius: 0.6rem;
  cursor: pointer;
  color: var(--color-text-secondary, #f0f);
  transition: all 0.15s ease;
}

.close-button:hover {
  background-color: var(--color-background-hover, #f0f);
  color: var(--color-text-primary, #f0f);
}

.dialog-body {
  padding: 2.4rem;
  overflow-y: auto;
  flex: 1;
}

.form-group {
  margin-bottom: 2rem;
}

.form-group:last-child {
  margin-bottom: 0;
}

.form-group label {
  display: block;
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary);
  margin-bottom: 0.8rem;
}

.required {
  color: var(--color-danger);
}

.form-input,
.form-textarea,
.form-select {
  width: 100%;
  padding: 1rem 1.2rem;
  font-size: 1.4rem;
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-primary, #f0f);
  border: 2px solid var(--color-border-default, #f0f);
  border-radius: 0.8rem;
  transition: all 0.15s ease;
  box-sizing: border-box;
}

.form-textarea {
  resize: vertical;
  min-height: 80px;
}

.form-input:focus,
.form-textarea:focus,
.form-select:focus {
  outline: none;
  border-color: var(--color-border-focus);
  box-shadow: var(--shadow-focus);
}

.form-input::placeholder,
.form-textarea::placeholder {
  color: var(--color-text-tertiary, #f0f);
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1.6rem;
}

/* ==================== 节段管理区域 ==================== */
.section-manager {
  margin-top: 2rem;
  padding-top: 2rem;
  border-top: 1px solid var(--color-border-light, #f0f);
}

.section-header {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  margin-bottom: 1.2rem;
  color: var(--color-text-primary, #f0f);
}

.section-title {
  font-size: 1.4rem;
  font-weight: 500;
}

.section-body {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

.add-section-input {
  margin-bottom: 0.4rem;
}

.section-input {
  width: 100%;
  padding: 0.8rem 1rem;
  font-size: 1.4rem;
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-primary, #f0f);
  border: 1px solid var(--color-border-default, #f0f);
  border-radius: 0.6rem;
  transition: all 0.15s ease;
}

.section-input:focus {
  outline: none;
  border-color: var(--color-border-focus, #f0f);
}

.section-input::placeholder {
  color: var(--color-text-tertiary, #f0f);
}

.sections-list {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.section-item {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  padding: 0.8rem 1rem;
  background-color: var(--color-background-hover, #f0f);
  border-radius: 0.6rem;
  transition: background-color 0.15s ease;
}

.section-item:hover {
  background-color: var(--color-background-active, #f0f);
}

.drag-handle {
  cursor: grab;
  color: var(--color-text-tertiary, #f0f);
  font-size: 1.2rem;
  user-select: none;
  padding: 0.2rem;
  border-radius: 0.4rem;
  transition: all 0.15s ease;
}

.drag-handle:hover {
  color: var(--color-text-secondary, #f0f);
  background-color: var(--color-background-hover, #f0f);
}

.drag-handle:active {
  cursor: grabbing;
}

.section-content {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.8rem;
}

.section-name {
  font-size: 1.4rem;
  color: var(--color-text-primary, #f0f);
  line-height: 1.4;
}

.section-actions {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  opacity: 0;
  transition: opacity 0.15s ease;
}

.section-item:hover .section-actions {
  opacity: 1;
}

.action-btn {
  all: unset;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.4rem;
  height: 2.4rem;
  border-radius: 0.4rem;
  cursor: pointer;
  color: var(--color-text-tertiary, #f0f);
  transition: all 0.15s ease;
}

.action-btn.edit:hover {
  background-color: var(--color-background-hover, #f0f);
  color: var(--color-text-primary, #f0f);
}

.action-btn.delete:hover {
  background-color: var(--color-danger-light, #f0f);
  color: var(--color-danger, #f0f);
}

.section-edit {
  flex: 1;
}

.section-edit-input {
  width: 100%;
  padding: 0.4rem 0.6rem;
  font-size: 1.4rem;
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-primary, #f0f);
  border: 1px solid var(--color-border-focus, #f0f);
  border-radius: 0.4rem;
  outline: none;
}

.empty-sections {
  padding: 1.6rem;
  text-align: center;
  font-size: 1.3rem;
  color: var(--color-text-tertiary, #f0f);
}

/* ==================== 底部按钮 ==================== */
.dialog-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 1rem;
  padding: 1.6rem 2rem;
  border-top: 1px solid var(--color-border-light);
}

.cancel-button,
.submit-button {
  padding: 0.8rem 1.6rem;
  font-size: 1.4rem;
  font-weight: 600;
  border-radius: 0.6rem;
  border: none;
  cursor: pointer;
  transition: all 0.15s ease;
}

.cancel-button {
  background-color: transparent;
  color: var(--color-text-secondary, #f0f);
}

.cancel-button:hover {
  background-color: var(--color-background-hover, #f0f);
  color: var(--color-text-primary, #f0f);
}

.submit-button {
  background-color: var(--color-button-primary-bg);
  color: var(--color-button-primary-text);
}

.submit-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.submit-button:hover:not(:disabled) {
  background-color: var(--color-button-primary-hover);
}
</style>
