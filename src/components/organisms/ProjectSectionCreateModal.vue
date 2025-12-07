<template>
  <Teleport to="body">
    <div v-if="show" class="modal-overlay" @click="handleOverlayClick">
      <div class="modal-dialog" @click.stop>
        <div class="dialog-header">
          <h3>{{ $t('project.action.createSection') }}</h3>
          <button class="close-button" @click="close">
            <CuteIcon name="X" :size="18" />
          </button>
        </div>
        <div class="dialog-body">
          <div class="form-group">
            <label for="section-title">{{ $t('project.field.sectionTitle') }} <span class="required">{{ $t('common.label.required') }}</span></label>
            <input
              id="section-title"
              ref="titleInputRef"
              v-model="formData.title"
              type="text"
              class="form-input"
              :placeholder="$t('project.placeholder.sectionTitle')"
              @keydown.enter="handleSubmit"
            />
          </div>

          <div class="form-group">
            <label for="section-description">{{ $t('project.field.sectionDescription') }}</label>
            <textarea
              id="section-description"
              v-model="formData.description"
              class="form-textarea"
              :placeholder="$t('project.placeholder.sectionDescription')"
              rows="3"
            />
          </div>
        </div>
        <div class="dialog-footer">
          <button class="cancel-button" @click="close">{{ $t('common.action.cancel') }}</button>
          <button
            class="submit-button"
            :disabled="!formData.title.trim() || isSubmitting"
            @click="handleSubmit"
          >
            {{ isSubmitting ? $t('project.button.creating') : $t('project.button.createSection') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import { pipeline } from '@/cpu'
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
const titleInputRef = ref<HTMLInputElement | null>(null)
const isSubmitting = ref(false)

const formData = ref({
  title: '',
  description: '',
})

// 当对话框显示时，自动聚焦到标题输入框
watch(
  () => props.show,
  async (newShow) => {
    if (newShow) {
      // 重置表单
      formData.value = {
        title: '',
        description: '',
      }
      isSubmitting.value = false
      await nextTick()
      titleInputRef.value?.focus()
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
  const title = formData.value.title.trim()
  if (!title || isSubmitting.value || !props.projectId) return

  isSubmitting.value = true

  try {
    logger.info(LogTags.UI, '创建章节', formData.value)

    await pipeline.dispatch('project_section.create', {
      project_id: props.projectId,
      title,
      description: formData.value.description.trim() || null,
    })

    logger.info(LogTags.UI, '章节创建成功')
    emit('success')
    close()
  } catch (error) {
    logger.error(LogTags.UI, '章节创建失败', error)
    alert(t('message.error.createSectionFailed'))
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
  border: 1px solid var(--color-border-subtle, #f0f);
  border-radius: 1.2rem;
  box-shadow: var(--shadow-lg);
  width: 90%;
  max-width: 480px;
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
.form-textarea {
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
.form-textarea:focus {
  outline: none;
  border-color: var(--color-border-focus);
  box-shadow: var(--shadow-focus);
}

.form-input::placeholder,
.form-textarea::placeholder {
  color: var(--color-text-tertiary, #9893a5);
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
