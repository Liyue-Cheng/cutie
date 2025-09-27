<template>
  <Teleport to="body">
    <div v-if="coordinator.state.value.isDragging" class="drag-ghost" :style="ghostPositionStyle">
      <!-- 动态组件渲染 -->
      <component
        v-if="coordinator.state.value.ghostComponent"
        :is="coordinator.state.value.ghostComponent"
        v-bind="coordinator.state.value.ghostProps"
        class="ghost-wrapper"
      />

      <!-- 基于源元素快照的虚化预览 -->
      <div
        v-else-if="coordinator.state.value.sourceElementSnapshot"
        class="snapshot-ghost"
        :style="snapshotGhostStyle"
        v-html="coordinator.state.value.sourceElementSnapshot.innerHTML"
      />

      <!-- 默认拖拽预览 -->
      <div v-else class="default-ghost">
        <div class="ghost-content">
          {{ coordinator.state.value.dataType }}:
          {{ JSON.stringify(coordinator.state.value.dragData) }}
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { manager as coordinator } from '@/composables/drag/drag-coordinator'

/**
 * 幽灵元素的定位样式
 */
const ghostPositionStyle = computed(() => {
  const snapshot = coordinator.state.value.sourceElementSnapshot
  const currentPos = coordinator.state.value.currentPosition
  const mouseOffset = coordinator.state.value.mouseOffset

  if (snapshot) {
    // 有源元素快照时，从源元素位置开始，然后根据鼠标移动调整位置
    return {
      position: 'fixed' as const,
      left: `${currentPos.x - mouseOffset.x}px`,
      top: `${currentPos.y - mouseOffset.y}px`,
      zIndex: 9999,
      pointerEvents: 'none' as const,
    }
  } else {
    // 程序化拖拽时，直接跟随鼠标中心
    return {
      position: 'fixed' as const,
      left: `${currentPos.x}px`,
      top: `${currentPos.y}px`,
      zIndex: 9999,
      pointerEvents: 'none' as const,
      transform: 'translate(-50%, -50%)',
    }
  }
})

/**
 * 基于源元素快照的幽灵样式
 */
const snapshotGhostStyle = computed(() => {
  const snapshot = coordinator.state.value.sourceElementSnapshot
  if (!snapshot) return {}

  const { computedStyle } = snapshot

  return {
    width: `${snapshot.width}px`,
    height: `${snapshot.height}px`,
    backgroundColor: computedStyle.backgroundColor,
    color: computedStyle.color,
    fontSize: computedStyle.fontSize,
    fontFamily: computedStyle.fontFamily,
    borderRadius: computedStyle.borderRadius,
    padding: computedStyle.padding,
    border: computedStyle.border,
    display: computedStyle.display,
    alignItems: computedStyle.alignItems,
    justifyContent: computedStyle.justifyContent,
    gap: computedStyle.gap,

    // 虚化效果
    opacity: '0.7',
    filter: 'blur(0.5px)',
    boxShadow: '0 4px 12px rgb(0 0 0 / 25%)',
    transform: 'scale(0.95)',

    // 确保文本不被选中
    userSelect: 'none' as const,
    pointerEvents: 'none' as const,

    // 防止内容溢出
    overflow: 'hidden',
  }
})
</script>

<style scoped>
.drag-ghost {
  /* 基础拖拽幽灵样式 */
  user-select: none;
  pointer-events: none;
}

.ghost-wrapper {
  /* 动态组件包装器的虚化效果 */
  opacity: 0.7;
  filter: blur(0.5px);
  box-shadow: 0 4px 12px rgb(0 0 0 / 25%);
  transform: scale(0.95);
  user-select: none;
  pointer-events: none;
}

.snapshot-ghost {
  /* 快照幽灵的基础样式 */
  position: relative;
  transition: none; /* 禁用过渡动画，避免拖拽时的延迟 */
}

/* 确保幽灵元素内的所有子元素都不可交互 */
.snapshot-ghost * {
  pointer-events: none !important;
  user-select: none !important;
}

.default-ghost {
  background: rgb(0 0 0 / 80%);
  color: white;
  padding: 8px 12px;
  border-radius: 4px;
  font-size: 12px;
  max-width: 200px;
  word-break: break-all;

  /* 虚化效果 */
  opacity: 0.8;
  box-shadow: 0 4px 8px rgb(0 0 0 / 20%);
}

.ghost-content {
  /* 默认内容样式 */
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 防止幽灵元素中的图标或其他元素闪烁 */
.drag-ghost :deep(*) {
  backface-visibility: hidden;
}
</style>
