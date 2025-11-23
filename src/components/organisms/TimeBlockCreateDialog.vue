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
            <CuteIcon name="ListTodo" :size="20" />
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

<!--
  TimeBlockCreateDialog - 时间块创建对话框

  🎯 功能：
  在日历上框选时间段后，弹出此对话框让用户选择创建 Task 或 Event

  🎨 设计特点：
  - 贴在时间块左侧显示（通过 position prop 定位）
  - 不使用遮罩层（点击外部会关闭但不会阻挡视线）
  - 支持类型切换（Task / Event），默认选中 Task
  - 确认按钮在标题为空时禁用

  🔑 交互规则：
  - 点击对话框外部 → 关闭且不创建
  - 点击取消 → 关闭且不创建
  - 点击确认 → 触发 @confirm 事件并传递 { type, title }
  - Enter 键 → 等同于点击确认
  - Esc 键 → 等同于点击取消

  📌 注意：
  - 使用捕获阶段的全局 mousedown 监听器，优先拦截外部点击
  - 通过 event.stopPropagation() + preventDefault() 防止点击穿透
-->
<script setup lang="ts">
import { ref, watch, nextTick, computed, onBeforeUnmount } from 'vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'

const props = defineProps<{
  show: boolean
  position?: {
    top: number // 锚点的视口 Y 坐标（像素）
    left: number // 锚点的视口 X 坐标（像素）
  }
}>()

const emit = defineEmits<{
  confirm: [data: { type: 'task' | 'event'; title: string }]
  cancel: []
}>()

const selectedType = ref<'task' | 'event'>('task') // 选中的类型，默认 Task
const title = ref('') // 用户输入的标题
const inputRef = ref<HTMLInputElement | null>(null) // 输入框 ref（用于自动聚焦）
const popoverRef = ref<HTMLElement | null>(null) // 弹窗 ref（用于检测外部点击）

// 🎨 弹窗样式：根据锚点位置计算
// transform: translate(-100%, -50%) 会让弹窗出现在锚点左侧并垂直居中
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
