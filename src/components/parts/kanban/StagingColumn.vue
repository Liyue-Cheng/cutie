<script setup lang="ts">
import type { TaskCard } from '@/types/dtos'
import type { ViewMetadata, StatusViewConfig } from '@/types/drag'
import SimpleKanbanColumn from './SimpleKanbanColumn.vue'

const emit = defineEmits<{
  openEditor: [task: TaskCard]
}>()

// 遵循 VIEW_CONTEXT_KEY_SPEC.md 规范
const VIEW_KEY = 'misc::staging'

// ViewMetadata 配置
const viewMetadata: ViewMetadata = {
  type: 'status',
  id: VIEW_KEY,
  config: { status: 'staging' } as StatusViewConfig,
  label: 'Staging',
}

// 🗑️ 移除：任务操作现在由 SimpleKanbanColumn 内部处理
// async function handleAddTask() { ... }
// async function handleReorderTasks() { ... }
// async function handleCrossViewDrop() { ... }
</script>

<template>
  <div class="staging-column-wrapper">
    <SimpleKanbanColumn
      title="Staging"
      subtitle="未安排的任务"
      :show-add-input="true"
      :view-key="VIEW_KEY"
      :view-metadata="viewMetadata"
      @open-editor="emit('openEditor', $event)"
    />
  </div>
</template>

<style scoped>
.staging-column-wrapper {
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
}

/* 覆盖 SimpleKanbanColumn 的内部滚动，让外层容器处理滚动 */
.staging-column-wrapper :deep(.simple-kanban-column) {
  height: auto; /* 不限制高度，让内容自然扩展 */
}

.staging-column-wrapper :deep(.task-list-scroll-area) {
  overflow-y: visible; /* 移除内部滚动 */
  flex-grow: 0; /* 不占据剩余空间 */
  flex-shrink: 0; /* 不收缩 */
  min-height: 0;
}
</style>
