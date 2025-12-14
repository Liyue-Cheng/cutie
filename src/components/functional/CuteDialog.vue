<template>
  <Teleport to="body">
    <Transition name="dialog-fade">
      <div v-if="dialogState.visible" class="dialog-overlay" @click.self="handleOverlayClick">
        <div class="dialog-container" :class="{ 'dialog-danger': options.danger }">
          <!-- Header -->
          <div v-if="options.title" class="dialog-header">
            <h3 class="dialog-title">{{ options.title }}</h3>
          </div>

          <!-- Body -->
          <div class="dialog-body">
            <p class="dialog-message">{{ options.message }}</p>

            <!-- Prompt input -->
            <input
              v-if="options.type === 'prompt'"
              ref="inputRef"
              v-model="inputValue"
              class="dialog-input"
              :placeholder="options.inputPlaceholder"
              @keydown.enter="handleConfirm"
            />
          </div>

          <!-- Footer -->
          <div class="dialog-footer">
            <button
              v-if="options.type !== 'alert'"
              class="dialog-btn dialog-btn-secondary"
              @click="handleCancel"
            >
              {{ options.cancelText }}
            </button>
            <button
              class="dialog-btn dialog-btn-primary"
              :class="{ 'dialog-btn-danger': options.danger }"
              @click="handleConfirm"
            >
              {{ options.confirmText }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { useDialog } from '@/composables/useDialog'

const { state: dialogState, closeDialog } = useDialog()

const inputValue = ref('')
const inputRef = ref<HTMLInputElement | null>(null)

const options = computed(() => dialogState.value.options)

// 当对话框打开时，如果是 prompt 类型，聚焦输入框
watch(
  () => dialogState.value.visible,
  async (visible) => {
    if (visible) {
      inputValue.value = options.value.inputValue || ''
      if (options.value.type === 'prompt') {
        await nextTick()
        inputRef.value?.focus()
        inputRef.value?.select()
      }
    }
  }
)

function handleOverlayClick() {
  // alert 类型点击遮罩不关闭
  if (options.value.type !== 'alert') {
    handleCancel()
  }
}

function handleConfirm() {
  if (options.value.type === 'prompt') {
    closeDialog(inputValue.value)
  } else {
    closeDialog(true)
  }
}

function handleCancel() {
  if (options.value.type === 'prompt') {
    closeDialog(null)
  } else {
    closeDialog(false)
  }
}
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  z-index: 99999;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(2px);
}

.dialog-container {
  width: 90%;
  max-width: 400px;
  background-color: var(--color-background-elevated, #f0f);
  border: 1px solid var(--color-border-default, #f0f);
  border-radius: 1.2rem;
  box-shadow: var(--shadow-xl, #f0f);
  overflow: hidden;
}

.dialog-header {
  padding: 1.6rem 2rem 0;
}

.dialog-title {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
  line-height: 1.4;
}

.dialog-body {
  padding: 1.6rem 2rem;
}

.dialog-message {
  margin: 0;
  font-size: 1.4rem;
  color: var(--color-text-secondary, #f0f);
  line-height: 1.6;
  white-space: pre-wrap;
}

.dialog-input {
  width: 100%;
  margin-top: 1.2rem;
  padding: 1rem 1.2rem;
  font-size: 1.4rem;
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-primary, #f0f);
  border: 1px solid var(--color-border-default, #f0f);
  border-radius: 0.6rem;
  outline: none;
  transition: border-color 0.2s ease;
}

.dialog-input:focus {
  border-color: var(--color-border-focus, #f0f);
}

.dialog-input::placeholder {
  color: var(--color-text-tertiary, #f0f);
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.8rem;
  padding: 0 2rem 1.6rem;
}

.dialog-btn {
  padding: 0.8rem 1.6rem;
  font-size: 1.4rem;
  font-weight: 500;
  border: none;
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.dialog-btn-secondary {
  background-color: var(--color-button-secondary-bg, #f0f);
  color: var(--color-text-secondary, #f0f);
  border: 1px solid var(--color-button-secondary-border, #f0f);
}

.dialog-btn-secondary:hover {
  background-color: var(--color-button-secondary-hover, #f0f);
}

.dialog-btn-primary {
  background-color: var(--color-button-primary-bg, #f0f);
  color: var(--color-button-primary-text, #f0f);
}

.dialog-btn-primary:hover {
  background-color: var(--color-button-primary-hover, #f0f);
}

.dialog-btn-danger {
  background-color: var(--color-button-danger-bg, #f0f);
  color: var(--color-button-danger-text, #f0f);
}

.dialog-btn-danger:hover {
  background-color: var(--color-button-danger-hover, #f0f);
}

/* Danger variant for the whole dialog */
.dialog-danger .dialog-title {
  color: var(--color-danger, #f0f);
}

/* Transition animations */
.dialog-fade-enter-active,
.dialog-fade-leave-active {
  transition: opacity 0.2s ease;
}

.dialog-fade-enter-active .dialog-container,
.dialog-fade-leave-active .dialog-container {
  transition: transform 0.2s ease;
}

.dialog-fade-enter-from,
.dialog-fade-leave-to {
  opacity: 0;
}

.dialog-fade-enter-from .dialog-container,
.dialog-fade-leave-to .dialog-container {
  transform: scale(0.95);
}
</style>
