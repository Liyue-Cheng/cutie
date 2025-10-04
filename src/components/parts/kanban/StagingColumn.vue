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
const VIEW_KEY = 'misc::staging'

// ViewMetadata 配置
const viewMetadata: ViewMetadata = {
  type: 'status',
  id: VIEW_KEY,
  config: { status: 'staging' } as StatusViewConfig,
  label: 'Staging',
}

// 获取 staging 任务（应用排序）
const stagingTasks = computed(() => {
  const tasks = taskStore.stagingTasks
  return viewStore.applySorting(tasks, VIEW_KEY)
})

// 初始化
onMounted(async () => {
  console.log('[StagingColumn] Initializing staging column...')
  await taskStore.fetchStagingTasks()
  console.log('[StagingColumn] Loaded', stagingTasks.value.length, 'staging tasks')
})

// 添加任务
async function handleAddTask(title: string) {
  try {
    const newTask = await taskStore.createTask({ title })
    if (newTask) {
      console.log('[StagingColumn] Task created:', newTask.id)
    }
  } catch (error) {
    console.error('[StagingColumn] Failed to create task:', error)
  }
}

// 重新排序任务
async function handleReorderTasks(newOrder: string[]) {
  try {
    await viewStore.updateSorting(VIEW_KEY, newOrder)
    console.log('[StagingColumn] Tasks reordered')
  } catch (error) {
    console.error('[StagingColumn] Failed to reorder tasks:', error)
  }
}

// 跨视图拖放
async function handleCrossViewDrop(taskId: string, targetViewId: string) {
  console.log('[StagingColumn] Cross-view drop:', { taskId, targetViewId })
  // 这里可以处理从其他看板拖入 staging 的逻辑
  // 暂时不需要特殊处理，因为任务的 schedule_status 会自动决定它是否显示在 staging
}
</script>

<template>
  <div class="staging-column-wrapper">
    <SimpleKanbanColumn
      title="Staging"
      subtitle="未安排的任务"
      :tasks="stagingTasks"
      :show-add-input="true"
      :view-key="VIEW_KEY"
      :view-metadata="viewMetadata"
      @open-editor="emit('openEditor', $event)"
      @add-task="handleAddTask"
      @reorder-tasks="handleReorderTasks"
      @cross-view-drop="handleCrossViewDrop"
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
