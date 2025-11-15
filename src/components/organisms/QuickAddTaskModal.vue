<template>
  <Teleport to="body">
    <div v-if="show" class="quick-add-overlay" @click="handleOverlayClick">
      <div class="quick-add-dialog" @click.stop>
        <div class="dialog-header">
          <h3>快速添加任务</h3>
          <button class="close-button" @click="close">
            <CuteIcon name="X" :size="18" />
          </button>
        </div>
        <div class="dialog-body">
          <input
            ref="inputRef"
            v-model="taskTitle"
            type="text"
            class="task-input"
            placeholder="输入任务标题..."
            @keydown.enter="handleAdd"
            @keydown.esc="close"
          />
        </div>
        <div class="dialog-footer">
          <button class="cancel-button" @click="close">取消</button>
          <button class="add-button" :disabled="!taskTitle.trim()" @click="handleAdd">
            添加到 Staging
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import { pipeline } from '@/cpu'
import { logger, LogTags } from '@/infra/logging/logger'

const props = defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

const taskTitle = ref('')
const inputRef = ref<HTMLInputElement | null>(null)

// 当对话框显示时，自动聚焦到输入框
watch(
  () => props.show,
  async (newShow) => {
    if (newShow) {
      taskTitle.value = ''
      await nextTick()
      inputRef.value?.focus()
    }
  }
)

function close() {
  emit('close')
}

function handleOverlayClick() {
  close()
}

async function handleAdd() {
  const title = taskTitle.value.trim()
  if (!title) return

  try {
    // 创建 staging 任务
    await pipeline.dispatch('task.create', {
      title,
      estimated_duration: 60, // 默认 60 分钟
    })

    logger.info(LogTags.COMPONENT_KANBAN, 'Quick add task to staging', { title })

    // 清空输入框并关闭对话框
    taskTitle.value = ''
    close()
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_KANBAN,
      'Failed to quick add task',
      error instanceof Error ? error : new Error(String(error))
    )
  }
}
</script>

<style scoped>
.quick-add-overlay {
  position: fixed;
  inset: 0;
  background-color: var(--color-overlay-medium);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  backdrop-filter: blur(2px);
}

.quick-add-dialog {
  background-color: var(--color-background-content, #faf4ed);
  border: 1px solid var(--color-border-default, #dfdad9);
  border-radius: 1.2rem;
  box-shadow: var(--shadow-lg);
  width: 90%;
  max-width: 50rem;
  padding: 0;
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
  padding: 2rem;
}

.task-input {
  width: 100%;
  padding: 1.2rem 1.6rem;
  font-size: 1.6rem;
  border: 2px solid var(--color-border-default, #dfdad9);
  border-radius: 0.8rem;
  background-color: var(--color-background-primary, #fffaf3);
  color: var(--color-text-primary, #575279);
  transition: all 0.15s ease;
  box-sizing: border-box;
}

.task-input:focus {
  outline: none;
  border-color: var(--color-border-focus);
  box-shadow: var(--shadow-focus);
}

.task-input::placeholder {
  color: var(--color-text-tertiary, #9893a5);
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
.add-button {
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

.add-button {
  background-color: var(--color-button-primary-bg);
  color: #fff;
}

.add-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.add-button:hover:not(:disabled) {
  background-color: var(--color-button-primary-hover);
}
</style>

