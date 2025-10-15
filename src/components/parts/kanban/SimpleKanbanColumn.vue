<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'
import type { ViewMetadata } from '@/types/drag'
import { useViewStore } from '@/stores/view'
import { useViewTasks } from '@/composables/useViewTasks'
import { deriveViewMetadata } from '@/services/viewAdapter'
import CutePane from '@/components/alias/CutePane.vue'
import KanbanTaskCard from './KanbanTaskCard.vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { commandBus } from '@/commandBus'
import { useInteractDrag } from '@/composables/drag/useInteractDrag'
import { useDragStrategy } from '@/composables/drag/useDragStrategy'
import { dragPreviewState } from '@/infra/drag-interact/preview-state'

const props = defineProps<{
  title: string
  subtitle?: string
  showAddInput?: boolean
  viewKey: string // 🔥 必需：所有看板都必须提供 viewKey
  viewMetadata?: ViewMetadata // 可选：可自动推导
}>()

const viewStore = useViewStore()

// ==================== 数据源管理 ====================

// 🔥 统一数据模式：所有看板都通过 viewKey 获取数据
const { tasks: effectiveTasks } = useViewTasks(props.viewKey)

// ✅ 统一的 ViewMetadata：优先使用父传的，否则自动推导
const effectiveViewMetadata = computed<ViewMetadata>(() => {
  if (props.viewMetadata) {
    return props.viewMetadata
  }

  const derived = deriveViewMetadata(props.viewKey)
  if (derived) {
    return derived
  }

  // 兜底：提供最小可用元数据
  return {
    id: props.viewKey,
    type: 'custom', // 使用 ViewType 中的有效值
    label: props.title,
    config: {}, // 提供空配置对象
  } as ViewMetadata
})

// ==================== 拖放系统 V2 (interact.js + 策略) ====================

const kanbanContainerRef = ref<HTMLElement | null>(null)
const dragStrategy = useDragStrategy()

const { displayTasks } = useInteractDrag({
  viewMetadata: effectiveViewMetadata,
  tasks: effectiveTasks,
  containerRef: kanbanContainerRef,
  draggableSelector: `.task-card-wrapper-${props.viewKey.replace(/::/g, '--')}`,
  onDrop: async (session) => {
    // 🎯 执行拖放策略（V2：灵活的 JSON 上下文）
    const result = await dragStrategy.executeDrop(session, props.viewKey, {
      // 起始组件的上下文数据（从 session.metadata 获取）
      sourceContext: (session.metadata?.sourceContext as Record<string, any>) || {},
      // 结束组件的上下文数据（当前组件提供）
      targetContext: {
        taskIds: displayTasks.value.map((t) => t.id),
        displayTasks: displayTasks.value,
        dropIndex: dragPreviewState.value?.computed.dropIndex,
        viewKey: props.viewKey,
      },
    })

    if (!result.success) {
      logger.error(
        LogTags.COMPONENT_KANBAN_COLUMN,
        'Drag strategy execution failed',
        new Error(result.message || 'Unknown error'),
        { viewKey: props.viewKey }
      )
    }
  },
})

// ==================== 任务创建 ====================

const newTaskTitle = ref('')
const isCreatingTask = ref(false)
const addTaskInputRef = ref<HTMLInputElement | null>(null)

async function handleAddTask() {
  const title = newTaskTitle.value.trim()
  if (!title || isCreatingTask.value) return

  isCreatingTask.value = true
  const originalTitle = newTaskTitle.value
  newTaskTitle.value = ''

  try {
    // 检查是否是日期视图（daily::YYYY-MM-DD）
    const viewMetadata = effectiveViewMetadata.value
    const isDateView = viewMetadata.type === 'date'

    if (isDateView) {
      // 日期视图：使用合并端点一次性创建任务并添加日程
      const dateConfig = viewMetadata.config as import('@/types/drag').DateViewConfig
      const date = dateConfig.date // YYYY-MM-DD

      await commandBus.emit('task.create_with_schedule', {
        title,
        scheduled_day: date,
      })

      logger.info(LogTags.COMPONENT_KANBAN_COLUMN, 'Task created with schedule', {
        title,
        date,
        viewKey: props.viewKey,
      })
    } else {
      // 非日期视图：只创建任务
      await commandBus.emit('task.create', {
        title,
      })
      logger.info(LogTags.COMPONENT_KANBAN_COLUMN, 'Task created', {
        title,
        viewKey: props.viewKey,
      })
    }
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_KANBAN_COLUMN,
      'Task creation failed',
      error instanceof Error ? error : new Error(String(error)),
      { title, viewKey: props.viewKey }
    )
    newTaskTitle.value = originalTitle
  } finally {
    isCreatingTask.value = false
    // 重新聚焦到输入框，方便连续添加任务
    nextTick(() => {
      if (addTaskInputRef.value) {
        addTaskInputRef.value.focus()
      }
    })
  }
}

// ==================== 任务完成后重新排序 ====================

function handleTaskCompleted(completedTaskId: string) {
  // ✅ 使用 effectiveTasks 替代 props.tasks
  const tasks = effectiveTasks.value

  // 找到已完成任务的当前索引
  const currentIndex = tasks.findIndex((t) => t.id === completedTaskId)
  if (currentIndex === -1) return

  // 找到最后一个未完成任务的索引
  let lastIncompleteIndex = -1
  for (let i = tasks.length - 1; i >= 0; i--) {
    const task = tasks[i]
    if (task && !task.is_completed && task.id !== completedTaskId) {
      lastIncompleteIndex = i
      break
    }
  }

  // 如果没有其他未完成的任务，或者已完成任务已经在正确位置，则不需要移动
  if (lastIncompleteIndex === -1 || currentIndex === lastIncompleteIndex + 1) {
    return
  }

  // 创建新的任务顺序
  const newOrder = [...tasks.map((t) => t.id)]
  // 移除已完成的任务
  newOrder.splice(currentIndex, 1)

  // 计算插入位置（移除元素后索引会变化）
  // 如果被完成的任务原本在最后一个未完成任务之前，移除后 lastIncompleteIndex 需要减 1
  const insertPosition =
    currentIndex < lastIncompleteIndex ? lastIncompleteIndex : lastIncompleteIndex + 1

  // 插入到最后一个未完成任务的后面
  newOrder.splice(insertPosition, 0, completedTaskId)

  // 🔥 使用 Command Bus 更新排序（乐观更新）
  const originalOrder = viewStore.getSortedTaskIds(props.viewKey, effectiveTasks.value)
  commandBus
    .emit('view.update_sorting', {
      view_key: props.viewKey,
      sorted_task_ids: newOrder,
      original_sorted_task_ids: originalOrder, // 用于失败回滚
    })
    .catch((error) => {
      logger.error(
        LogTags.COMPONENT_KANBAN_COLUMN,
        'Failed to persist completed task reorder',
        error,
        { viewKey: props.viewKey }
      )
    })
}

// ==================== 排序配置管理 ====================

const previousTaskIds = ref<Set<string>>(new Set())

onMounted(async () => {
  // 🔥 简化：所有看板都有 viewKey，直接加载排序配置
  const alreadyLoaded = viewStore.sortWeights.has(props.viewKey)
  if (!alreadyLoaded) {
    await viewStore.fetchViewPreference(props.viewKey)
  }
  // ✅ 移除 sortingConfigLoaded 状态，避免闪烁
  // 🆕 注册 daily 视图
  const parts = props.viewKey.split('::')
  if (parts.length >= 2 && parts[0] === 'daily' && parts[1]) {
    viewStore.registerDailyView(parts[1])
  }
})

onBeforeUnmount(() => {
  const parts = props.viewKey.split('::')
  if (parts.length >= 2 && parts[0] === 'daily' && parts[1]) {
    viewStore.unregisterDailyView(parts[1])
  }
})

// ✅ 自动检测任务列表变化并持久化（使用 effectiveTasks）
// 注意：拖放过程中的排序更新已由策略系统处理，这里只处理其他来源的任务列表变化
watch(
  () => effectiveTasks.value,
  (newTasks) => {
    const currentTaskIds = new Set(newTasks.map((t) => t.id))
    const hasChanges =
      currentTaskIds.size !== previousTaskIds.value.size ||
      !Array.from(currentTaskIds).every((id) => previousTaskIds.value.has(id))

    if (hasChanges) {
      previousTaskIds.value = currentTaskIds
      const currentOrder = newTasks.map((t) => t.id)

      // 🔥 使用 Command Bus 自动持久化排序（乐观更新）
      const originalOrder = viewStore.getSortedTaskIds(props.viewKey, effectiveTasks.value)
      commandBus
        .emit('view.update_sorting', {
          view_key: props.viewKey,
          sorted_task_ids: currentOrder,
          original_sorted_task_ids: originalOrder,
        })
        .catch((error) => {
          logger.error(
            LogTags.COMPONENT_KANBAN_COLUMN,
            'Failed to auto-persist view tasks',
            error,
            {
              viewKey: props.viewKey,
            }
          )
        })
    } else {
      previousTaskIds.value = currentTaskIds
    }
  },
  {
    deep: false,
    immediate: true,
  }
)

// ==================== 注意 ====================
// displayTasks 已由 useInteractDrag 自动提供
// 所有拖放事件处理已由 interact.js 控制器自动管理
// 不需要手动处理 dragstart/dragover/drop 等事件
</script>

<template>
  <CutePane class="simple-kanban-column">
    <!-- 🔥 整个看板作为 dropzone（包含 header、input、task list） -->
    <div ref="kanbanContainerRef" class="kanban-dropzone-wrapper">
      <div class="header">
        <div class="title-section">
          <h2 class="title">{{ title }}</h2>
          <p v-if="subtitle" class="subtitle">{{ subtitle }}</p>
        </div>
        <div class="task-count">
          <span class="count">{{ effectiveTasks.length }}</span>
        </div>
      </div>

      <div v-if="showAddInput" class="add-task-wrapper">
        <input
          ref="addTaskInputRef"
          v-model="newTaskTitle"
          type="text"
          placeholder="+ 添加任务"
          class="add-task-input"
          :disabled="isCreatingTask"
          @keydown.enter="handleAddTask"
        />
      </div>

      <div class="task-list-scroll-area">
        <div
          v-for="task in displayTasks"
          :key="task.id"
          :class="`task-card-wrapper task-card-wrapper-${viewKey.replace(/::/g, '--')}`"
          :data-task-id="task.id"
        >
          <KanbanTaskCard
            :task="task"
            :view-metadata="effectiveViewMetadata"
            class="kanban-task-card"
            @task-completed="handleTaskCompleted"
          />
        </div>

        <div v-if="displayTasks.length === 0" class="empty-state">暂无任务</div>
      </div>
    </div>
  </CutePane>
</template>

<style scoped>
.simple-kanban-column {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--color-background-content);
  width: 23rem;
  flex-shrink: 0;
  padding-left: 0.5rem;
  padding-right: 0.5rem;
}

/* 🔥 整个看板作为 dropzone wrapper */
.kanban-dropzone-wrapper {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
}

.header {
  padding: 1rem 1rem 0.5rem;
  border-bottom: 1px solid var(--color-border-default);
  flex-shrink: 0;
}

.title-section {
  margin-bottom: 0.5rem;
}

.title {
  font-size: 2.2rem;
  font-weight: 600;
  margin: 0;
  color: var(--color-text-primary);
}

.subtitle {
  font-size: 1.2rem;
  color: var(--color-text-secondary);
  margin: 0.25rem 0 0;
}

.task-count {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 1.4rem;
  font-weight: 500;
}

.task-count .count {
  color: var(--color-text-secondary);
}

.add-task-wrapper {
  padding: 1rem 1rem 0.5rem;
  flex-shrink: 0;
}

.add-task-input {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid var(--color-border-default);
  border-radius: 8px;
  background-color: var(--color-card-available);
  color: var(--color-text-primary);
  font-size: 1.5rem;
  transition: all 0.2s ease;
}

.add-task-input:focus {
  outline: none;
  border-color: var(--color-primary, #4a90e2);
  box-shadow: 0 0 0 3px rgb(74 144 226 / 10%);
}

.add-task-input::placeholder {
  color: var(--color-text-secondary);
}

.add-task-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* ✅ 移除 .creating-indicator 样式 */

.task-list-scroll-area {
  flex-grow: 1;
  overflow-y: auto;
  padding: 0.5rem 1rem 1rem;
  min-height: 100px;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 10rem;
  color: var(--color-text-tertiary);
  font-size: 1.4rem;
}

/* 滚动条样式 */
.task-list-scroll-area::-webkit-scrollbar {
  width: 6px;
}

.task-list-scroll-area::-webkit-scrollbar-track {
  background: transparent;
}

.task-list-scroll-area::-webkit-scrollbar-thumb {
  background: var(--color-border-default);
  border-radius: 3px;
}

.task-list-scroll-area::-webkit-scrollbar-thumb:hover {
  background: var(--color-text-tertiary);
}

/* 🔥 拖拽样式由 interact.js 控制器自动管理 */
.task-card-wrapper {
  position: relative;
  transition: transform 0.2s ease;
}

.kanban-task-card {
  pointer-events: auto;
}
</style>
