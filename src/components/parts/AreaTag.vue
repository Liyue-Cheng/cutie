<script setup lang="ts">
/**
 * AreaTag - Area 标签组件
 *
 * 用于显示任务关联的 Area（项目/上下文）
 * 格式：# AreaName
 * - # 图标使用 Area 的颜色
 * - Area 名称使用正常文字颜色
 */

import CuteIcon from '@/components/parts/CuteIcon.vue'

interface Props {
  /** Area 名称 */
  name: string
  /** Area 颜色（用于 # 图标） */
  color: string
  /** 字号大小 */
  size?: 'small' | 'normal' | 'large'
}

const props = withDefaults(defineProps<Props>(), {
  size: 'normal',
})

// ✅ 根据尺寸计算图标大小
const iconSize = {
  small: 12,
  normal: 14,
  large: 16,
}[props.size]
</script>

<template>
  <div class="area-tag" :class="`size-${size}`">
    <CuteIcon name="Hash" :size="iconSize" :color="color" class="hash-icon" />
    <span class="area-name">{{ name }}</span>
  </div>
</template>

<style scoped>
/* 基础布局 */
.area-tag {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  font-weight: 500;
}

/* 字号大小变体 */
.area-tag.size-small {
  font-size: 1rem;
}

.area-tag.size-normal {
  font-size: 1.2rem;
}

.area-tag.size-large {
  font-size: 1.4rem;
}

/* ✅ Hash 图标样式 */
.hash-icon {
  flex-shrink: 0;
  display: flex;
  align-items: center;
}

/* Area 名称样式 */
.area-name {
  color: var(--color-text-primary);
  line-height: 1;
}
</style>
