/** * @description A wrapper for lucide-vue-next icons, providing a consistent API. * Supports both
pixel numbers and rem string values for size. */
<script setup lang="ts">
import { computed } from 'vue'
import { icons } from 'lucide-vue-next'
import type { IconName } from '@/types/icons'

interface Props {
  name: IconName
  size?: number | string
  strokeWidth?: number
  color?: string
}

const props = withDefaults(defineProps<Props>(), {
  size: 16,
  strokeWidth: 2,
  color: 'currentColor',
})

const icon = computed(() => icons[props.name])

// ✅ 计算实际尺寸：支持数字（像素）或字符串（rem等单位）
const actualSize = computed(() => {
  if (typeof props.size === 'string') {
    // 如果传入的是字符串（如 "1.4rem"），转换为像素值供 lucide 使用
    // 假设 1rem = 10px（基于 html { font-size: 62.5% }）
    const remMatch = props.size.match(/^([\d.]+)rem$/i)
    if (remMatch && remMatch[1]) {
      return parseFloat(remMatch[1]) * 10
    }
    // 如果是其他字符串格式，尝试解析为数字
    return parseFloat(props.size) || 16
  }
  return props.size
})
</script>

<template>
  <component :is="icon" :size="actualSize" :stroke-width="strokeWidth" :color="color" />
</template>
