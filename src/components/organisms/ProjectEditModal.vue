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
const isSubmitting = ref(false)

const formData = ref({
  name: '',
  description: '',
  area_id: null as string | null,
  due_date: '',
  status: 'ACTIVE' as 'ACTIVE' | 'COMPLETED',
})

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
      isSubmitting.value = false
      await nextTick()
      nameInputRef.value?.focus()
    }
  }
)

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
    alert(t('message.error.updateProjectFailed'))
  } finally {
    isSubmitting.value = false
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background-color: var(--color-overlay-medium);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  backdrop-filter: blur(2px);
}

.modal-dialog {
  background-color: var(--color-background-content, #faf4ed);
  border: 1px solid var(--color-border-default, #dfdad9);
  border-radius: 1.2rem;
  box-shadow: var(--shadow-lg);
  width: 90%;
  max-width: 520px;
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
  color: var(--color-text-primary, #575279);
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
  color: var(--color-text-secondary, #797593);
  transition: all 0.15s ease;
}

.close-button:hover {
  background-color: var(--color-background-hover, rgb(87 82 121 / 5%));
  color: var(--color-text-primary, #575279);
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
  color: var(--color-text-primary, #575279);
  background-color: var(--color-background-primary, #fffaf3);
  border: 2px solid var(--color-border-default, #dfdad9);
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
  color: var(--color-text-tertiary, #9893a5);
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1.6rem;
}

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
  color: var(--color-text-secondary, #797593);
}

.cancel-button:hover {
  background-color: var(--color-background-hover, rgb(87 82 121 / 5%));
  color: var(--color-text-primary, #575279);
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
