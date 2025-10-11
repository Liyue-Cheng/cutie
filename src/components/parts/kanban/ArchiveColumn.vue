<script setup lang="ts">
import type { ViewMetadata, StatusViewConfig } from '@/types/drag'
import SimpleKanbanColumn from './SimpleKanbanColumn.vue'

// 🗑️ 移除 emit - 不再需要转发事件

// 遵循 VIEW_CONTEXT_KEY_SPEC.md 规范
const VIEW_KEY = 'misc::archive'

// ViewMetadata 配置
const viewMetadata: ViewMetadata = {
  type: 'status',
  id: VIEW_KEY,
  config: { status: 'archived' } as StatusViewConfig,
  label: 'Archive',
}

// 🗑️ 移除：任务操作现在由 SimpleKanbanColumn 内部处理
// const archivedTasks = computed(() => { ... })
// async function handleReorderTasks() { ... }
// async function handleCrossViewDrop() { ... }
</script>

<template>
  <div class="archive-column-wrapper">
    <SimpleKanbanColumn
      title="Archive"
      subtitle="已归档的任务"
      :show-add-input="false"
      :view-key="VIEW_KEY"
      :view-metadata="viewMetadata"
    />
  </div>
</template>

<style scoped>
.archive-column-wrapper {
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
}

/* 覆盖 SimpleKanbanColumn 的内部滚动，让外层容器处理滚动 */
.archive-column-wrapper :deep(.simple-kanban-column) {
  height: auto; /* 不限制高度，让内容自然扩展 */
}

.archive-column-wrapper :deep(.task-list-scroll-area) {
  overflow-y: visible; /* 移除内部滚动 */
  flex-grow: 0; /* 不占据剩余空间 */
  flex-shrink: 0; /* 不收缩 */
  min-height: 0;
}

/* 归档任务样式微调 */
.archive-column-wrapper :deep(.kanban-task-card) {
  opacity: 0.8;
}

.archive-column-wrapper :deep(.kanban-task-card:hover) {
  opacity: 1;
}
</style>
