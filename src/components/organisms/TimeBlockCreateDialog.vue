<template>
  <Teleport to="body">
    <div v-if="show" ref="popoverRef" class="create-dialog-popover" :style="popoverStyle">
      <div ref="dialogRef" class="create-dialog" @mousedown.stop @click.stop>
        <!-- 书签标签切换器 -->
        <div class="tab-bar">
          <button
            :class="['tab-item', { active: selectedType === 'task' }]"
            @click="selectedType = 'task'"
          >
            <CuteIcon name="SquareCheck" :size="16" />
            <span>{{ $t('timeBlock.type.task') }}</span>
          </button>
          <button
            :class="['tab-item', { active: selectedType === 'event' }]"
            @click="selectedType = 'event'"
          >
            <CuteIcon name="Calendar" :size="16" />
            <span>{{ $t('timeBlock.type.event') }}</span>
          </button>
        </div>

        <!-- 内容区域 -->
        <div class="content-area">
          <!-- 标题行 -->
          <div class="title-row">
            <div class="title-icon">
              <CuteCheckbox v-if="selectedType === 'task'" :checked="false" disabled size="large" />
              <CuteIcon v-else name="Calendar" :size="20" />
            </div>
            <input
              ref="inputRef"
              v-model="title"
              type="text"
              class="title-input"
              :placeholder="getPlaceholder()"
              autocomplete="off"
              @keydown.enter="handleConfirm"
              @keydown.esc="handleCancel"
            />
          </div>

          <!-- 描述行 -->
          <div class="description-row">
            <div class="description-icon">
              <CuteIcon name="FileText" :size="18" />
            </div>
            <textarea
              v-model="description"
              class="description-input"
              :placeholder="$t('timeBlock.placeholder.description')"
              rows="2"
            ></textarea>
          </div>
        </div>

        <!-- 底部按钮 -->
        <div class="button-section">
          <button class="cancel-button" @click="handleCancel">
            {{ $t('common.action.cancel') }}
          </button>
          <button class="confirm-button" :disabled="!title.trim()" @click="handleConfirm">
            {{ $t('common.action.create') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, computed, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'

const VIEWPORT_MARGIN = 12
const ANCHOR_GAP = 8

const props = defineProps<{
  show: boolean
  position?: {
    top: number
    left: number
  }
}>()

const emit = defineEmits<{
  confirm: [data: { type: 'task' | 'event'; title: string; description?: string }]
  cancel: []
}>()

const { t } = useI18n()

const selectedType = ref<'task' | 'event'>('task')
const title = ref('')
const description = ref('')
const inputRef = ref<HTMLInputElement | null>(null)
const popoverRef = ref<HTMLElement | null>(null)
const dialogRef = ref<HTMLElement | null>(null)

const popoverCoords = ref({ top: 0, left: 0 })

const popoverStyle = computed(() => ({
  top: `${popoverCoords.value.top}px`,
  left: `${popoverCoords.value.left}px`,
}))

function getPlaceholder() {
  switch (selectedType.value) {
    case 'task':
      return t('timeBlock.placeholder.taskTitle')
    case 'event':
      return t('timeBlock.placeholder.eventTitle')
    default:
      return t('timeBlock.placeholder.title')
  }
}

// 禁用滚轮事件
function handleWheel(event: WheelEvent) {
  event.preventDefault()
  event.stopPropagation()
}

// 当对话框显示时，重置状态并聚焦输入框
watch(
  () => props.show,
  async (newShow) => {
    if (newShow) {
      selectedType.value = 'task'
      title.value = ''
      description.value = ''
      await nextTick()
      inputRef.value?.focus()

      if (typeof document !== 'undefined') {
        document.addEventListener('mousedown', handleOutsideClick, true)
        document.addEventListener('wheel', handleWheel, { passive: false, capture: true })
      }
      if (typeof window !== 'undefined') {
        window.addEventListener('resize', handleWindowChange, { passive: true })
        window.addEventListener('scroll', handleWindowChange, { passive: true })
      }
      await updatePopoverPosition()
    } else {
      if (typeof document !== 'undefined') {
        document.removeEventListener('mousedown', handleOutsideClick, true)
        document.removeEventListener('wheel', handleWheel, { capture: true } as EventListenerOptions)
      }
      if (typeof window !== 'undefined') {
        window.removeEventListener('resize', handleWindowChange)
        window.removeEventListener('scroll', handleWindowChange)
      }
    }
  }
)

watch(
  () => props.position,
  () => {
    if (props.show) {
      updatePopoverPosition()
    }
  }
)

onBeforeUnmount(() => {
  if (typeof document !== 'undefined') {
    document.removeEventListener('mousedown', handleOutsideClick, true)
    document.removeEventListener('wheel', handleWheel, { capture: true } as EventListenerOptions)
  }
  if (typeof window !== 'undefined') {
    window.removeEventListener('resize', handleWindowChange)
    window.removeEventListener('scroll', handleWindowChange)
  }
})

function handleOutsideClick(event: MouseEvent) {
  const root = popoverRef.value
  if (!root) return

  if (!root.contains(event.target as Node)) {
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
    description: description.value.trim() || undefined,
  })
}

function handleCancel() {
  emit('cancel')
}

function handleWindowChange() {
  updatePopoverPosition()
}

async function updatePopoverPosition() {
  if (!props.show || typeof window === 'undefined') return

  await nextTick()

  const dialogEl = dialogRef.value
  const viewportWidth = window.innerWidth
  const viewportHeight = window.innerHeight

  const anchorTop = props.position?.top ?? viewportHeight / 2
  const anchorLeft = props.position?.left ?? viewportWidth / 2

  let popupWidth = 0
  let popupHeight = 0

  if (dialogEl) {
    const rect = dialogEl.getBoundingClientRect()
    popupWidth = rect.width
    popupHeight = rect.height
  }

  const margin = VIEWPORT_MARGIN
  const gap = ANCHOR_GAP

  // 水平位置：优先使用 anchorLeft，超出则收敛到视口内
  let left = anchorLeft
  const maxLeft = viewportWidth - margin - popupWidth
  left = Math.min(Math.max(left, margin), Math.max(margin, maxLeft))

  // 垂直位置：优先放在下方，空间不足则翻转到上方
  let top = anchorTop + gap
  const overflowBelow = top + popupHeight - (viewportHeight - margin)
  if (overflowBelow > 0) {
    top = Math.max(margin, top - overflowBelow) // 仅向上收敛，不翻转
  }

  popoverCoords.value = {
    top: Math.round(top),
    left: Math.round(left),
  }
}
</script>

<style scoped>
.create-dialog-popover {
  position: fixed;
  z-index: 10000;
  transform: translate(calc(-100% - 1.2rem), 0);
}

.create-dialog {
  position: relative;
  background-color: var(--color-background-content, #f0f);
  border: 1px solid var(--color-border-default, #f0f);
  border-radius: 0.8rem;
  box-shadow: var(--shadow-lg);
  width: 36rem;
  overflow: visible; /* 允许书签上移但锚点仍在内容区顶 */
  max-height: calc(100vh - 24px);
  display: flex;
  flex-direction: column;
}

/* ==================== 书签标签栏 ==================== */
.tab-bar {
  display: flex;
  position: absolute;
  bottom: 100%; /* 锚点与内容区顶部对齐，不依赖固定 rem */
  left: 0;
  right: 0;
  padding: 0 0.8rem;
  gap: 0;
  pointer-events: auto;
}

.tab-item {
  all: unset;
  box-sizing: border-box;
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.6rem;
  padding: 1.1rem 2rem;
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-tertiary, #f0f);
  cursor: pointer;
  transition: color 0.15s ease, background-color 0.15s ease;
  position: relative;
  background-color: var(--color-background-secondary, #f0f);
  border: 1px solid var(--color-border-default, #f0f);
  border-bottom: none;
  border-radius: 0.8rem 0.8rem 0 0;
  margin-bottom: -1px; /* 盖住主体顶部边框 */
  z-index: 2;
}

.tab-item + .tab-item {
  margin-left: -1px; /* 邻接书签共用边框，无缝连接 */
}

.tab-item:hover:not(.active) {
  color: var(--color-text-secondary, #f0f);
  background-color: var(--color-background-hover, #f0f);
}

.tab-item.active {
  color: var(--color-text-primary, #f0f);
  background-color: var(--color-background-elevated, #f0f);
  z-index: 3;
}

.tab-item span {
  line-height: 1.4;
}

/* ==================== 内容区域 ==================== */
.content-area {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 2rem;
  display: flex;
  flex-direction: column;
  gap: 1.6rem;
  background-color: var(--color-background-elevated, #f0f);
  border-radius: 0.8rem 0.8rem 0 0; /* 继承父元素圆角，防止尖角叠在圆角上 */
}

/* 标题行 */
.title-row {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.title-icon {
  flex-shrink: 0;
  width: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-tertiary, #f0f);
}

.title-input {
  flex: 1;
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
  background: transparent;
  border: none;
  outline: none;
  padding: 0.4rem 0;
  border-bottom: 2px solid transparent;
  transition: border-color 0.2s;
  line-height: 1.4;
}

.title-input:focus {
  border-bottom-color: var(--color-border-focus, #f0f);
}

.title-input::placeholder {
  color: var(--color-text-tertiary, #f0f);
  font-weight: 400;
}

/* 描述行 */
.description-row {
  display: flex;
  align-items: flex-start;
  gap: 1rem;
}

.description-icon {
  flex-shrink: 0;
  width: 2rem;
  padding-top: 0.4rem;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-tertiary, #f0f);
}

.description-input {
  flex: 1;
  font-family: inherit;
  font-size: 1.4rem;
  color: var(--color-text-primary, #f0f);
  background: transparent;
  border: none;
  outline: none;
  resize: none;
  padding: 0.4rem 0;
  line-height: 1.5;
}

.description-input::placeholder {
  color: var(--color-text-tertiary, #f0f);
}

/* ==================== 按钮区 ==================== */
.button-section {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 1rem;
  padding: 1.6rem 2rem;
  background-color: var(--color-background-elevated, #f0f);
  border-top: 1px solid var(--color-border-default, #f0f);
  border-radius: 0 0 0.8rem 0.8rem; /* 继承父元素圆角 */
}

.cancel-button,
.confirm-button {
  padding: 0.8rem 1.6rem;
  font-size: 1.4rem;
  font-weight: 500;
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

.confirm-button {
  background-color: var(--color-button-primary-bg, #f0f);
  color: var(--color-button-primary-text, #f0f);
}

.confirm-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.confirm-button:hover:not(:disabled) {
  background-color: var(--color-button-primary-hover, #f0f);
}
</style>
