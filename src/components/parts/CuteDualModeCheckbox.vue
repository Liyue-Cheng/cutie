<script setup lang="ts">
// 双模式复选框组件（CuteDualModeCheckbox）
// 圆角方形外观，支持三种状态，自动适配主题颜色：
// - null: 未选中（灰色边框）
// - 'completed': 完成任务（玫瑰色边框 + 玫瑰色对钩，Rose Pine Dawn: #d7827e）
// - 'present': 标记在场（青蓝色边框 + 青蓝色圆点，Rose Pine Dawn: #286983）
//
// 交互逻辑：
// - 单击：未选中 → 完成；已选中（任何状态）→ 未选中
// - 长按（0.5秒）：标记在场

import { computed, ref } from 'vue'

type CheckboxState = null | 'completed' | 'present'

interface Props {
  state?: CheckboxState
  size?: 'small' | 'large' | string
  longPressDelay?: number // 长按触发时间（毫秒）
  disableLongPress?: boolean // 禁用长按功能（用于单模式场景）
}

const props = withDefaults(defineProps<Props>(), {
  state: null,
  size: 'small',
  longPressDelay: 500,
  disableLongPress: false,
})

const emit = defineEmits<{
  'update:state': [value: CheckboxState]
}>()

// 长按相关状态
const pressTimer = ref<number | null>(null)
const isLongPress = ref(false)

// 判断是否使用预设尺寸
const isPresetSize = computed(() => props.size === 'small' || props.size === 'large')

// 自定义尺寸的 CSS 变量
const customSizeStyle = computed(() => {
  if (isPresetSize.value) return {}
  return {
    '--checkbox-size': props.size,
  }
})

// 鼠标/触摸按下
const handlePressStart = (event: MouseEvent | TouchEvent) => {
  event.preventDefault()
  isLongPress.value = false

  // 如果禁用长按，直接返回
  if (props.disableLongPress) return

  // 设置长按定时器
  pressTimer.value = window.setTimeout(() => {
    isLongPress.value = true
    // 长按触发：标记在场
    emit('update:state', 'present')
  }, props.longPressDelay)
}

// 鼠标/触摸释放
const handlePressEnd = () => {
  if (pressTimer.value) {
    clearTimeout(pressTimer.value)
    pressTimer.value = null
  }

  // 如果不是长按，执行单击逻辑
  if (!isLongPress.value) {
    if (props.state) {
      // 任何选中状态 -> 未选中
      emit('update:state', null)
    } else {
      // 未选中 -> 完成任务
      emit('update:state', 'completed')
    }
  }
}

// 鼠标/触摸离开（取消长按）
const handlePressCancel = () => {
  if (pressTimer.value) {
    clearTimeout(pressTimer.value)
    pressTimer.value = null
  }
}
</script>

<template>
  <div
    class="cute-dual-mode-checkbox"
    :class="[
      isPresetSize ? `size-${size}` : 'size-custom',
      state ? `state-${state}` : 'state-unchecked',
    ]"
    :style="customSizeStyle"
    @mousedown="handlePressStart"
    @mouseup="handlePressEnd"
    @mouseleave="handlePressCancel"
    @touchstart.passive="handlePressStart"
    @touchend.passive="handlePressEnd"
    @touchcancel.passive="handlePressCancel"
  >
    <div class="checkbox-box">
      <!-- 完成状态：对钩 -->
      <span v-if="state === 'completed'" class="icon-check"></span>
      <!-- 在场状态：圆点 -->
      <span v-else-if="state === 'present'" class="icon-dot"></span>
    </div>
  </div>
</template>

<style scoped>
.cute-dual-mode-checkbox {
  position: relative;
  display: inline-flex;
  align-items: center;
  cursor: pointer;
  user-select: none;
  -webkit-tap-highlight-color: transparent;
}

.checkbox-box {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.6rem;
  height: 1.6rem;
  border: 0.2rem solid var(--color-status-pending-checkbox);
  border-radius: 0.4rem; /* 圆角方形 */
  background-color: transparent;

  /* 只对边框颜色和背景色应用过渡，避免拖拽时的残影 */
  transition:
    border-color 0.2s ease-in-out,
    background-color 0.2s ease-in-out;
}

/* Large size variant */
.cute-dual-mode-checkbox.size-large .checkbox-box {
  width: 2.1rem;
  height: 2.1rem;
  border-radius: 0.5rem;
}

/* Custom size variant */
.cute-dual-mode-checkbox.size-custom .checkbox-box {
  width: var(--checkbox-size);
  height: var(--checkbox-size);
  border-radius: calc(var(--checkbox-size) * 0.25);
}

/* 完成状态 */
.cute-dual-mode-checkbox.state-completed .checkbox-box {
  border-color: var(--color-status-completed);
  background-color: transparent;
}

/* 在场状态 */
.cute-dual-mode-checkbox.state-present .checkbox-box {
  border-color: var(--color-status-present);
  background-color: transparent;
}

/* 未选中状态 - hover效果 */
.cute-dual-mode-checkbox.state-unchecked:hover .checkbox-box {
  border-color: var(--color-status-completed);
  background-color: var(--color-background-hover);
}

/* 已选中状态的hover效果 */
.cute-dual-mode-checkbox.state-completed:hover .checkbox-box,
.cute-dual-mode-checkbox.state-present:hover .checkbox-box {
  opacity: 0.8;
}

/* 完成状态的对钩 */
.icon-check {
  display: block;
  transform: rotate(45deg);
  width: 0.4rem;
  height: 0.75rem;
  border: solid var(--color-status-completed);
  border-width: 0 0.2rem 0.2rem 0;
  margin-top: -0.15rem; /* 微调视觉居中 */
}

/* 在场状态的圆点 */
.icon-dot {
  display: block;
  width: 0.6rem;
  height: 0.6rem;
  border-radius: 50%;
  background-color: var(--color-status-present);
}

.cute-dual-mode-checkbox.size-large .icon-check {
  width: 0.5rem;
  height: 0.95rem;
  border-width: 0 0.25rem 0.25rem 0;
  margin-top: -0.2rem; /* 大尺寸的视觉居中调整 */
}

.cute-dual-mode-checkbox.size-large .icon-dot {
  width: 0.8rem;
  height: 0.8rem;
}

.cute-dual-mode-checkbox.size-custom .icon-check {
  width: calc(var(--checkbox-size) * 0.25);
  height: calc(var(--checkbox-size) * 0.47);
  border-width: 0 calc(var(--checkbox-size) * 0.125) calc(var(--checkbox-size) * 0.125) 0;
  margin-top: calc(var(--checkbox-size) * -0.09); /* 自定义尺寸的视觉居中调整 */
}

.cute-dual-mode-checkbox.size-custom .icon-dot {
  width: calc(var(--checkbox-size) * 0.375);
  height: calc(var(--checkbox-size) * 0.375);
}
</style>
