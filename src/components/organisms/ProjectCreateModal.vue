<template>
  <Teleport to="body">
    <div v-if="show" class="modal-overlay" @click="handleOverlayClick">
      <div class="modal-dialog" @click.stop>
        <div class="dialog-header">
          <h3>新建项目</h3>
          <button class="close-button" @click="close">
            <CuteIcon name="X" :size="18" />
          </button>
        </div>
        <div class="dialog-body">
          <div class="form-group">
            <label for="project-name">项目名称 <span class="required">*</span></label>
            <input
              id="project-name"
              ref="nameInputRef"
              v-model="formData.name"
              type="text"
              class="form-input"
              placeholder="输入项目名称..."
              @keydown.enter="handleSubmit"
            />
          </div>

          <div class="form-group">
            <label for="project-description">项目描述</label>
            <textarea
              id="project-description"
              v-model="formData.description"
              class="form-textarea"
              placeholder="输入项目描述..."
              rows="3"
            />
          </div>

          <div class="form-row">
            <div class="form-group">
              <label for="project-area">所属区域</label>
              <select id="project-area" v-model="formData.area_id" class="form-select">
                <option :value="null">无区域</option>
                <option v-for="area in areas" :key="area.id" :value="area.id">
                  {{ area.name }}
                </option>
              </select>
            </div>

            <div class="form-group">
              <label for="project-due-date">截止日期</label>
              <input
                id="project-due-date"
                v-model="formData.due_date"
                type="date"
                class="form-input"
              />
            </div>
          </div>
        </div>
        <div class="dialog-footer">
          <button class="cancel-button" @click="close">取消</button>
          <button
            class="submit-button"
            :disabled="!formData.name.trim() || isSubmitting"
            @click="handleSubmit"
          >
            {{ isSubmitting ? '创建中...' : '创建项目' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, computed } from 'vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import { pipeline } from '@/cpu'
import { useAreaStore } from '@/stores/area'
import { logger, LogTags } from '@/infra/logging/logger'

const props = defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  close: []
  success: [projectId: string]
}>()

const areaStore = useAreaStore()

const nameInputRef = ref<HTMLInputElement | null>(null)
const isSubmitting = ref(false)

const formData = ref({
  name: '',
  description: '',
  area_id: null as string | null,
  due_date: '',
})

const areas = computed(() => areaStore.allAreas)

// 当对话框显示时，自动聚焦到名称输入框
watch(
  () => props.show,
  async (newShow) => {
    if (newShow) {
      // 重置表单
      formData.value = {
        name: '',
        description: '',
        area_id: null,
        due_date: '',
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
  if (!name || isSubmitting.value) return

  isSubmitting.value = true

  try {
    logger.info(LogTags.UI, '创建项目', formData.value)

    const result = await pipeline.dispatch('project.create', {
      name,
      description: formData.value.description.trim() || null,
      area_id: formData.value.area_id,
      due_date: formData.value.due_date || null,
    })

    logger.info(LogTags.UI, '项目创建成功', result)
    emit('success', result.id)
    close()
  } catch (error) {
    logger.error(LogTags.UI, '项目创建失败', error)
    alert('创建项目失败，请重试')
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
  border-bottom: 1px solid var(--color-border-soft, rgb(87 82 121 / 8%));
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
  color: #e74c3c;
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
  border-top: 1px solid var(--color-border-soft, rgb(87 82 121 / 8%));
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
  color: #fff;
}

.submit-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.submit-button:hover:not(:disabled) {
  background-color: var(--color-button-primary-hover);
}
</style>
