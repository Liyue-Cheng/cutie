<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useTaskStore } from '@/stores/task'
import { useViewStore } from '@/stores/view'
import type { TaskCard } from '@/types/dtos'
import type { ViewMetadata, StatusViewConfig } from '@/types/drag'
import SimpleKanbanColumn from './SimpleKanbanColumn.vue'

const emit = defineEmits<{
  openEditor: [task: TaskCard]
}>()

const taskStore = useTaskStore()
const viewStore = useViewStore()

// 遵循 VIEW_CONTEXT_KEY_SPEC.md 规范
const VIEW_KEY = 'misc::archive'

// ViewMetadata 配置
const viewMetadata: ViewMetadata = {
  type: 'status',
  id: VIEW_KEY,
  config: { status: 'archived' } as StatusViewConfig,
  label: 'Archive',
}

// 获取归档任务（应用排序）
const archivedTasks = computed(() => {
  const tasks = taskStore.archivedTasks
  return viewStore.applySorting(tasks, VIEW_KEY)
})

// 初始化
onMounted(async () => {
  console.log('[ArchiveColumn] Initializing archive column...')
  // 归档任务已经包含在 allTasks 中，无需额外加载
  console.log('[ArchiveColumn] Loaded', archivedTasks.value.length, 'archived tasks')
})

// 重新排序任务
async function handleReorderTasks(newOrder: string[]) {
  try {
    await viewStore.updateSorting(VIEW_KEY, newOrder)
    console.log('[ArchiveColumn] Tasks reordered')
  } catch (error) {
    console.error('[ArchiveColumn] Failed to reorder tasks:', error)
  }
}

// 跨视图拖放
async function handleCrossViewDrop(taskId: string, targetViewId: string) {
  console.log('[ArchiveColumn] Cross-view drop:', { taskId, targetViewId })
  // 归档列不支持拖入新任务，因为任务需要通过 unarchive 操作来恢复
  // 但支持从归档列拖出到其他视图（自动取消归档）
}
</script>

<template>
  <div class="archive-column-wrapper">
    <SimpleKanbanColumn
      title="Archive"
      subtitle="已归档的任务"
      :tasks="archivedTasks"
      :show-add-input="false"
      :view-key="VIEW_KEY"
      :view-metadata="viewMetadata"
      @open-editor="emit('openEditor', $event)"
      @reorder-tasks="handleReorderTasks"
      @cross-view-drop="handleCrossViewDrop"
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
