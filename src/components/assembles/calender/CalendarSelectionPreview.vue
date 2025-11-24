<script setup lang="ts">
/**
 * CalendarSelectionPreview - 日历选区预览组件
 *
 * 用于 selectMirror 模式下显示用户拖选时间段的预览
 * 仅显示时间范围，不显示标题
 */

import { getDefaultAreaColor } from '@/infra/utils/themeUtils'

interface Props {
  startTime: string // ISO 时间字符串
  endTime: string // ISO 时间字符串
  areaColor?: string // 可选的区域颜色
}

const props = defineProps<Props>()

// 格式化时间为 "9:30 AM" 格式（12小时制）
function formatTime(isoString: string): string {
  const date = new Date(isoString)
  if (Number.isNaN(date.getTime())) {
    return '--:--'
  }
  let hours = date.getHours()
  const minutes = date.getMinutes()
  const period = hours >= 12 ? 'PM' : 'AM'
  hours = hours % 12 || 12 // 转换为 12 小时制，0 点显示为 12
  const paddedMinutes = minutes.toString().padStart(2, '0')
  return `${hours}:${paddedMinutes} ${period}`
}

// 将 Area 颜色调浅作为背景色
function getLightenedColor(color: string): string {
  // 如果是 hex 格式
  if (color.startsWith('#')) {
    const hex = color.replace('#', '')
    const r = parseInt(hex.substring(0, 2), 16)
    const g = parseInt(hex.substring(2, 4), 16)
    const b = parseInt(hex.substring(4, 6), 16)

    // 调浅颜色：向白色(255)混合，保持85%的白色
    const lighten = (value: number) => Math.round(value + (255 - value) * 0.85)

    const lightR = lighten(r)
    const lightG = lighten(g)
    const lightB = lighten(b)

    return `rgb(${lightR}, ${lightG}, ${lightB})`
  }

  // 如果是 rgb/rgba 格式
  if (color.startsWith('rgb')) {
    const match = color.match(/\d+/g)
    if (match && match.length >= 3 && match[0] && match[1] && match[2]) {
      const r = parseInt(match[0])
      const g = parseInt(match[1])
      const b = parseInt(match[2])

      const lighten = (value: number) => Math.round(value + (255 - value) * 0.85)

      const lightR = lighten(r)
      const lightG = lighten(g)
      const lightB = lighten(b)

      return `rgb(${lightR}, ${lightG}, ${lightB})`
    }
  }

  // 默认返回浅灰色
  return '#f5f5f5'
}

const effectiveAreaColor = props.areaColor || getDefaultAreaColor()
const timeRange = `${formatTime(props.startTime)} > ${formatTime(props.endTime)}`
const backgroundColor = getLightenedColor(effectiveAreaColor)
</script>

<template>
  <div class="selection-preview" :style="{ backgroundColor }">
    <!-- 左侧强调条 -->
    <div class="accent-bar" :style="{ backgroundColor: effectiveAreaColor }"></div>

    <!-- 内容区域 -->
    <div class="preview-body">
      <div class="time-range">{{ timeRange }}</div>
    </div>
  </div>
</template>

<style scoped>
.selection-preview {
  display: flex;
  width: 100%;
  height: 100%;
  border-radius: 0.4rem;
  overflow: hidden;
  position: relative;
  padding-left: 0.5rem;
  opacity: 0.9;
}

/* 左侧强调条 */
.accent-bar {
  width: 0.4rem;
  flex-shrink: 0;
  border-radius: 0.2rem;
  align-self: stretch;
  margin: 0.5rem 0;
}

/* 内容区域 */
.preview-body {
  flex: 1;
  padding: 0.4rem 0.6rem;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  min-width: 0;
}

/* 时间范围（顶格显示） */
.time-range {
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--color-text-secondary, #6e6a86);
  line-height: 1.3;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
