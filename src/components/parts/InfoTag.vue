<template>
  <span class="info-tag" :class="{ danger: danger }">
    <span class="info-tag-icon">
      <!-- 图标模式 -->
      <CuteIcon v-if="icon" :name="icon" size="1.65rem" :color="iconColor" />
      <!-- 彩色圆点模式 -->
      <span v-else-if="dotColor" class="info-tag-dot" :style="{ backgroundColor: dotColor }"></span>
      <!-- 自定义内容（如波浪号） -->
      <slot v-else name="icon"></slot>
    </span>
    <span class="info-tag-text">{{ text }}</span>
  </span>
</template>

<script setup lang="ts">
import CuteIcon from '@/components/parts/CuteIcon.vue'

interface Props {
  /** Lucide 图标名称 */
  icon?: string
  /** 图标颜色 */
  iconColor?: string
  /** 彩色圆点颜色（与 icon 互斥） */
  dotColor?: string
  /** 显示的文字 */
  text: string
  /** 是否为危险/过期状态 */
  danger?: boolean
}

withDefaults(defineProps<Props>(), {
  danger: false,
})
</script>

<style scoped>
.info-tag {
  display: inline-flex;
  align-items: center;
  gap: 0.6rem;
  font-size: 1.5rem;
  font-weight: 500;
  color: var(--color-text-tertiary, #f0f);
  line-height: 1.4;
}

.info-tag-icon {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.info-tag-dot {
  width: 1rem;
  height: 1rem;
  border-radius: 50%;
}

.info-tag-text {
  white-space: nowrap;
  /* 使用等宽数字，确保时间/日期标签宽度一致 */
  font-variant-numeric: tabular-nums;
}

/* 危险状态（过期） */
.info-tag.danger {
  color: var(--color-danger, #f0f);
}

.info-tag.danger .info-tag-icon {
  color: var(--color-danger, #f0f);
}
</style>
