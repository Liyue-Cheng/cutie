<script setup lang="ts">
/**
 * AreaTag - Area 标签组件
 *
 * 用于显示任务关联的 Area（项目/上下文）
 * 格式：# AreaName
 * - # 符号使用 Area 的颜色，字号较大
 * - Area 名称使用正常文字颜色
 */

interface Props {
  /** Area 名称 */
  name: string
  /** Area 颜色（用于 # 符号） */
  color: string
  /** 字号大小 */
  size?: 'small' | 'normal' | 'large'
}

withDefaults(defineProps<Props>(), {
  size: 'normal',
})
</script>

<template>
  <div class="area-tag" :class="`size-${size}`">
    <span class="hash-symbol" :style="{ color: color }">#</span>
    <span class="area-name">{{ name }}</span>
  </div>
</template>

<style scoped>
/* 基础布局 */
.area-tag {
  display: inline-flex;
  align-items: baseline;
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

/* # 符号样式（基础） */
.hash-symbol {
  font-weight: 500;
  line-height: 1;
  letter-spacing: 0.08em;
  font-family:
    'Segoe UI',
    'SF Pro Display',
    -apple-system,
    BlinkMacSystemFont,
    system-ui,
    sans-serif;
  transform: scale(1.1);
  display: inline-block;
}

/* # 符号字号变体 */
.area-tag.size-small .hash-symbol {
  font-size: 1.2rem;
}

.area-tag.size-normal .hash-symbol {
  font-size: 1.4rem;
}

.area-tag.size-large .hash-symbol {
  font-size: 1.6rem;
}

/* Area 名称样式 */
.area-name {
  color: var(--color-text-primary);
  line-height: 1;
}
</style>
