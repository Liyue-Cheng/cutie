<template>
  <Teleport to="body">
    <div v-if="show" ref="popoverRef" class="create-dialog-popover" :style="popoverStyle">
      <div class="create-dialog" @mousedown.stop @click.stop>
        <!-- 类型选择器 -->
        <div class="type-selector">
          <button
            :class="['type-button', { active: selectedType === 'task' }]"
            @click="selectedType = 'task'"
          >
            <CuteIcon name="CheckSquare" :size="20" />
            <span>Task</span>
          </button>
          <button
            :class="['type-button', { active: selectedType === 'event' }]"
            @click="selectedType = 'event'"
          >
            <CuteIcon name="Calendar" :size="20" />
            <span>Event</span>
          </button>
        </div>

        <!-- 标题输入框 -->
        <div class="input-section">
          <input
            ref="inputRef"
            v-model="title"
            type="text"
            class="title-input"
            placeholder="输入标题..."
            @keydown.enter="handleConfirm"
            @keydown.esc="handleCancel"
          />
        </div>

        <!-- 底部按钮 -->
        <div class="button-section">
          <button class="cancel-button" @click="handleCancel">取消</button>
          <button class="confirm-button" :disabled="!title.trim()" @click="handleConfirm">
            确认
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, computed, onBeforeUnmount } from 'vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'

const props = defineProps<{
  show: boolean
  position?: {
    top: number
    left: number
  }
}>()

const emit = defineEmits<{
  confirm: [data: { type: 'task' | 'event'; title: string }]
  cancel: []
}>()

const selectedType = ref<'task' | 'event'>('task')
const title = ref('')
const inputRef = ref<HTMLInputElement | null>(null)
const popoverRef = ref<HTMLElement | null>(null)

const popoverStyle = computed(() => {
  const top = props.position?.top ?? (typeof window !== 'undefined' ? window.innerHeight / 2 : 0)
  const left = props.position?.left ?? (typeof window !== 'undefined' ? window.innerWidth / 2 : 0)

  return {
    top: `${top}px`,
    left: `${left}px`,
  }
})

// 当对话框显示时，重置状态并聚焦输入框
watch(
  () => props.show,
  async (newShow) => {
    if (newShow) {
      selectedType.value = 'task'
      title.value = ''
      await nextTick()
      inputRef.value?.focus()

      if (typeof document !== 'undefined') {
        // 使用捕获阶段监听，优先于页面其他点击处理逻辑，避免“点透”
        document.addEventListener('mousedown', handleOutsideClick, true)
      }
    } else {
      if (typeof document !== 'undefined') {
        document.removeEventListener('mousedown', handleOutsideClick, true)
      }
    }
  }
)

onBeforeUnmount(() => {
  if (typeof document !== 'undefined') {
    document.removeEventListener('mousedown', handleOutsideClick, true)
  }
})

function handleOutsideClick(event: MouseEvent) {
  const root = popoverRef.value
  if (!root) return

  if (!root.contains(event.target as Node)) {
    // 阻止事件继续冒泡到日历或其他组件，避免“穿透”点击
    event.stopPropagation()
    event.preventDefault()
    handleCancel()
  }
}

function handleConfirm() {
  const trimmedTitle = title.value.trim()
  if (!trimmedTitle) return

  emit('confirm', {
    type: selectedType.value,
    title: trimmedTitle,
  })
}

function handleCancel() {
  emit('cancel')
}
</script>

<style scoped>
.create-dialog-popover {
  position: fixed;
  z-index: 10000;
  transform: translate(-100%, -50%); /* 在锚点左侧垂直居中展示 */
}

.create-dialog {
  background-color: var(--color-background-content, #faf4ed);
  border: 1px solid var(--color-border-default, #dfdad9);
  border-radius: 1.2rem;
  box-shadow: var(--shadow-lg);
  width: 90%;
  max-width: 45rem;
  padding: 2rem;
  display: flex;
  flex-direction: column;
  gap: 2rem;
}

/* 类型选择器 */
.type-selector {
  display: flex;
  gap: 1rem;
  justify-content: center;
}

.type-button {
  all: unset;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.8rem;
  padding: 1.6rem 2.4rem;
  border: 2px solid var(--color-border-default, #dfdad9);
  border-radius: 0.8rem;
  background-color: var(--color-background-primary, #fffaf3);
  color: var(--color-text-secondary, #797593);
  cursor: pointer;
  transition: all 0.2s ease;
  flex: 1;
  min-width: 12rem;
}

.type-button:hover {
  border-color: var(--color-border-focus, #907aa9);
  background-color: var(--color-background-hover, rgb(87 82 121 / 5%));
}

.type-button.active {
  border-color: var(--color-button-primary-bg, #907aa9);
  background-color: var(--color-button-primary-bg, #907aa9);
  color: #fff;
}

.type-button span {
  font-size: 1.4rem;
  font-weight: 600;
  line-height: 1;
}

/* 输入框 */
.input-section {
  display: flex;
  flex-direction: column;
}

.title-input {
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

.title-input:focus {
  outline: none;
  border-color: var(--color-border-focus);
  box-shadow: var(--shadow-focus);
}

.title-input::placeholder {
  color: var(--color-text-tertiary, #9893a5);
}

/* 按钮区 */
.button-section {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 1rem;
}

.cancel-button,
.confirm-button {
  padding: 1rem 2rem;
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

.confirm-button {
  background-color: var(--color-button-primary-bg);
  color: #fff;
}

.confirm-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.confirm-button:hover:not(:disabled) {
  background-color: var(--color-button-primary-hover);
}
</style>
