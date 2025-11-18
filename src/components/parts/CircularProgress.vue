<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 100 100"
    class="circular-progress"
    :class="{ clickable: clickable }"
    @click="handleClick"
  >
    <!-- 背景圆环 -->
    <circle cx="50" cy="50" :r="radius" fill="none" stroke="#f3f4f6" :stroke-width="strokeWidth" />
    <!-- 进度圆环 -->
    <circle
      cx="50"
      cy="50"
      :r="radius"
      fill="none"
      :stroke="progressColor"
      :stroke-width="strokeWidth"
      :stroke-dasharray="circumference"
      :stroke-dashoffset="offset"
      stroke-linecap="round"
      class="progress-ring"
      transform="rotate(-90 50 50)"
    />
    <!-- 中心文本（可选显示） -->
    <text
      v-if="!hideText"
      x="50"
      y="50"
      text-anchor="middle"
      dominant-baseline="middle"
      :class="textClass"
    >
      {{ Math.round(progress) }}%
    </text>
  </svg>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  /** 当前已完成数量 */
  completed: number
  /** 总数量 */
  total: number
  /** 尺寸：small=2.1rem, normal=4.8rem, large=6.4rem */
  size?: 'small' | 'normal' | 'large'
  /** 是否可点击 */
  clickable?: boolean
  /** 是否隐藏中央文本 */
  hideText?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  size: 'normal',
  clickable: false,
  hideText: false,
})

const emit = defineEmits<{
  click: []
}>()

// 计算尺寸
const sizeMap = {
  small: '2.4rem',
  normal: '4.8rem',
  large: '6.4rem',
}

const size = computed(() => sizeMap[props.size])

// SVG 参数
const radius = 40
const strokeWidth = 8
const circumference = 2 * Math.PI * radius

// 计算进度百分比
const progress = computed(() => {
  if (props.total === 0) return 0
  return (props.completed / props.total) * 100
})

// 计算圆环偏移量
const offset = computed(() => {
  const progressValue = progress.value / 100
  return circumference * (1 - progressValue)
})

// 根据进度选择颜色
const progressColor = computed(() => {
  const p = progress.value
  if (p === 0) return '#d1d5db' // 灰色 - 未开始
  if (p < 50) return '#f59e0b' // 橙色 - 进行中（1-49%）
  if (p < 100) return '#4a90e2' // 蓝色 - 进行中（50-99%）
  return '#10b981' // 绿色 - 已完成
})

// 文本样式
const textClass = computed(() => {
  const baseClass = 'progress-text'
  const sizeClass = props.size === 'small' ? 'text-xs' : 'text-base'
  return `${baseClass} ${sizeClass}`
})

// 点击处理
const handleClick = () => {
  if (props.clickable) {
    emit('click')
  }
}
</script>

<style scoped>
.circular-progress {
  display: inline-block;
  vertical-align: middle;
}

.circular-progress.clickable {
  cursor: pointer;
  transition: opacity 0.2s;
}

.circular-progress.clickable:hover {
  opacity: 0.8;
}

.progress-ring {
  transition: stroke-dashoffset 0.6s cubic-bezier(0.4, 0, 0.2, 1);
}

.progress-text {
  font-weight: 600;
  fill: currentcolor;
}
</style>
