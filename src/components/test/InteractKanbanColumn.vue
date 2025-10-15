<!--
  基于 SimpleKanbanColumn 的新拖放系统测试组件
  
  使用 useInteractDrag 替代原有的多个拖放 composables
-->

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'
import type { ViewMetadata } from '@/types/drag'
import { useViewStore } from '@/stores/view'
import { useInteractDrag } from '@/composables/drag/useInteractDrag'
import { useViewTasks } from '@/composables/useViewTasks'
import { deriveViewMetadata } from '@/services/viewAdapter'
import CutePane from '@/components/alias/CutePane.vue'
import KanbanTaskCard from '@/components/parts/kanban/KanbanTaskCard.vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { commandBus } from '@/commandBus'

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

// ==================== 新拖放系统 ====================

const taskListRef = ref<HTMLElement | null>(null)

// 🔥 使用新的 interact.js 拖放系统
const { displayTasks, isDragging, isReceiving, getDebugInfo } = useInteractDrag({
  viewMetadata: effectiveViewMetadata,
  tasks: computed(() => effectiveTasks.value),
  containerRef: taskListRef,
  draggableSelector: `.task-card-wrapper-${props.viewKey.replace(/:/g, '-')}`,
  onDrop: async (session) => {
    // 🎯 拖放策略执行
    console.log('🎯 拖放策略执行:', {
      from: session.source.viewId,
      to: props.viewKey,
      task: session.object.data.title,
      strategy: getStrategyName(session.source.viewId, props.viewKey),
    })

    // 模拟异步操作
    await new Promise((resolve) => setTimeout(resolve, 300))

    console.log('✅ 拖放完成')
  },
})

/**
 * 获取策略名称（模拟）
 */
function getStrategyName(sourceId: string, targetId: string): string {
  if (sourceId === 'misc::staging' && targetId.startsWith('daily::')) {
    return 'StagingToDaily'
  } else if (sourceId.startsWith('daily::') && targetId === 'misc::staging') {
    return 'DailyToStaging'
  } else if (sourceId.startsWith('daily::') && targetId.startsWith('daily::')) {
    return 'DailyToDaily'
  } else if (sourceId === 'misc::staging' && targetId === 'misc::staging') {
    return 'StagingSameView'
  }
  return 'UnknownStrategy'
}

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
watch(
  () => effectiveTasks.value,
  (newTasks) => {
    // ✅ 移除 sortingConfigLoaded 检查，避免闪烁
    if (isDragging.value) {
      previousTaskIds.value = new Set(newTasks.map((t) => t.id))
      return
    }

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
  { deep: false, immediate: true }
)
</script>

<template>
  <CutePane class="interact-kanban-column">
    <div class="header">
      <div class="title-section">
        <h2 class="title">{{ title }}</h2>
        <p v-if="subtitle" class="subtitle">{{ subtitle }}</p>
      </div>
      <div class="task-count">
        <span class="count">{{ effectiveTasks.length }}</span>
      </div>
      <div class="status-indicators">
        <div v-if="isDragging" class="status-indicator dragging">拖动中</div>
        <div v-if="isReceiving" class="status-indicator receiving">接收中</div>
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

    <div
      ref="taskListRef"
      class="task-list-scroll-area"
      :class="{
        'is-dragging': isDragging,
        'is-receiving': isReceiving,
      }"
    >
      <div
        v-for="task in displayTasks"
        :key="task.id"
        class="task-card-wrapper"
        :class="[
          { 'is-preview': (task as any)._isPreview },
          `task-card-wrapper-${viewKey.replace(/:/g, '-')}`,
        ]"
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

    <!-- 调试信息 -->
    <div class="debug-info">
      <details>
        <summary>调试信息</summary>
        <pre>{{ JSON.stringify(getDebugInfo(), null, 2) }}</pre>
      </details>
    </div>
  </CutePane>
</template>

<style scoped>
.interact-kanban-column {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--color-background-content);
  width: 23rem;
  flex-shrink: 0;
  padding-left: 0.5rem;
  padding-right: 0.5rem;
}

.header {
  padding: 1rem 1rem 0.5rem;
  border-bottom: 1px solid var(--color-border-default);
  display: flex;
  align-items: center;
  gap: 1rem;
}

.title-section {
  flex: 1;
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

.status-indicators {
  display: flex;
  gap: 0.25rem;
}

.status-indicator {
  padding: 0.125rem 0.375rem;
  border-radius: 6px;
  font-size: 0.625rem;
  font-weight: 500;
  text-transform: uppercase;
}

.status-indicator.dragging {
  background: rgb(239 68 68 / 20%);
  color: #dc2626;
}

.status-indicator.receiving {
  background: rgb(34 197 94 / 20%);
  color: #16a34a;
}

.add-task-wrapper {
  padding: 1rem 1rem 0.5rem;
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

.task-list-scroll-area {
  flex-grow: 1;
  overflow-y: auto;
  padding: 0.5rem 1rem 1rem;
  min-height: 100px;
  transition: all 0.2s ease;
}

.task-list-scroll-area.is-dragging {
  background-color: rgb(59 130 246 / 5%);
}

.task-list-scroll-area.is-receiving {
  background-color: rgb(16 185 129 / 5%);
  border: 2px dashed var(--color-primary);
  border-radius: 8px;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 10rem;
  color: var(--color-text-tertiary);
  font-size: 1.4rem;
}

.task-card-wrapper {
  position: relative;
  cursor: grab;
  transition: all 0.2s ease;
  margin-bottom: 0.5rem;
}

.task-card-wrapper:active {
  cursor: grabbing;
}

.task-card-wrapper.is-preview {
  transform: translateY(-2px) scale(1.02);
  box-shadow: 0 8px 24px rgb(0 0 0 / 15%);
  border: 2px solid var(--color-primary);
  border-radius: 8px;
  background: rgb(255 255 255 / 95%);
}

.kanban-task-card {
  cursor: grab;
  pointer-events: auto;
}

.kanban-task-card:active {
  cursor: grabbing;
}

.debug-info {
  padding: 0.5rem;
  border-top: 1px solid var(--color-border-default);
  background: var(--color-background-muted);
}

.debug-info details {
  font-size: 0.75rem;
}

.debug-info summary {
  cursor: pointer;
  color: var(--color-text-secondary);
  font-weight: 500;
}

.debug-info pre {
  margin: 0.5rem 0 0;
  padding: 0.5rem;
  background: var(--color-card-available);
  border: 1px solid var(--color-border-default);
  border-radius: 4px;
  overflow-x: auto;
  font-size: 0.625rem;
  line-height: 1.4;
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
</style>
