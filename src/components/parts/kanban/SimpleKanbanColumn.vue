<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'
import type { ViewMetadata } from '@/types/drag'
import { useViewStore } from '@/stores/view'
import { useViewTasks } from '@/composables/useViewTasks'
import { deriveViewMetadata } from '@/services/viewAdapter'
import CutePane from '@/components/alias/CutePane.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import KanbanTaskCard from './KanbanTaskCard.vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'
import { useInteractDrag } from '@/composables/drag/useInteractDrag'
import { useDragStrategy } from '@/composables/drag/useDragStrategy'
import { dragPreviewState } from '@/infra/drag-interact/preview-state'

const props = defineProps<{
  title: string
  subtitle?: string
  showAddInput?: boolean
  viewKey: string // 🔥 必需：所有看板都必须提供 viewKey
  viewMetadata?: ViewMetadata // 可选：可自动推导
  isExpired?: boolean // 🆕 是否过期（用于灰度显示）
  isCalendarDate?: boolean // 🆕 是否是当前日历日期（用于日历图标长显）
  disableTitleClick?: boolean // 🆕 禁用标题点击
  hideCalendarIcon?: boolean // 🆕 隐藏日历图标
}>()

const emit = defineEmits<{
  'title-click': [date: string] // 标题点击事件，传递日期
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

const { displayItems } = useInteractDrag({
  viewMetadata: effectiveViewMetadata,
  items: effectiveTasks,
  containerRef: kanbanContainerRef,
  draggableSelector: `.task-card-wrapper-${props.viewKey.replace(/::/g, '--')}`,
  objectType: 'task',
  getObjectId: (task) => task.id,
  onDrop: async (session) => {
    // 🔍 打印完整的拖放会话信息（调试用）
    console.group('🎯 Drop Event - Full Session Info')
    console.log('📦 Session:', {
      id: session.id,
      source: session.source,
      object: session.object,
      dragMode: session.dragMode,
      target: session.target,
      metadata: session.metadata,
    })
    console.log('🎨 Source Context:', session.metadata?.sourceContext)
    console.log('🎯 Target View Key:', props.viewKey)
    console.log(
      '📋 Display Items:',
      displayItems.value.map((t) => ({
        id: t.id,
        title: t.title,
        schedule_status: t.schedule_status,
      }))
    )
    console.log('📍 Drop Index:', dragPreviewState.value?.computed.dropIndex)
    console.groupEnd()

    // 🎯 执行拖放策略（V2：灵活的 JSON 上下文）
    const result = await dragStrategy.executeDrop(session, props.viewKey, {
      // 起始组件的上下文数据（从 session.metadata 获取）
      sourceContext: (session.metadata?.sourceContext as Record<string, any>) || {},
      // 结束组件的上下文数据（当前组件提供）
      targetContext: {
        taskIds: displayItems.value.map((t) => t.id),
        displayTasks: displayItems.value,
        dropIndex: dragPreviewState.value?.computed.dropIndex,
        viewKey: props.viewKey,
      },
    })

    if (!result.success) {
      const errorMessage = result.message || result.error || 'Unknown error'

      // 🔍 打印策略匹配失败的详细信息
      console.group('❌ Strategy Execution Failed')
      console.log('Error Message:', errorMessage)
      console.log('Result:', result)
      console.log('Session Source ViewKey:', session.source.viewKey)
      console.log('Session Source ViewType:', session.source.viewType)
      console.log('Target Zone:', props.viewKey)
      console.log('Task Schedule Status:', session.object.data.schedule_status)
      console.log('Task Schedules:', session.object.data.schedules)
      console.groupEnd()

      logger.error(
        LogTags.COMPONENT_KANBAN_COLUMN,
        'Drag strategy execution failed',
        new Error(errorMessage),
        { viewKey: props.viewKey, result, session }
      )
    }
  },
})

// ==================== 任务创建 ====================

const newTaskTitle = ref('')
const isCreatingTask = ref(false)
const addTaskInputRef = ref<HTMLInputElement | null>(null)

function handleAddTask() {
  const title = newTaskTitle.value.trim()
  if (!title || isCreatingTask.value) return

  isCreatingTask.value = true
  newTaskTitle.value = ''

  // 检查是否是日期视图（daily::YYYY-MM-DD）
  const viewMetadata = effectiveViewMetadata.value
  const isDateView = viewMetadata.type === 'date'

  if (isDateView) {
    // 日期视图：使用合并端点一次性创建任务并添加日程
    const dateConfig = viewMetadata.config as import('@/types/drag').DateViewConfig
    const date = dateConfig.date // YYYY-MM-DD

    // 🚀 使用 CPU Pipeline 发射指令
    pipeline.dispatch('task.create_with_schedule', {
      title,
      estimated_duration: 60, // 🔥 默认 60 分钟
      scheduled_day: date,
    })

    logger.info(LogTags.COMPONENT_KANBAN_COLUMN, 'Task creation dispatched (with schedule)', {
      title,
      date,
      viewKey: props.viewKey,
    })
  } else {
    // 非日期视图：只创建任务，需要根据 viewKey 提取上下文信息
    const taskData: any = {
      title,
      estimated_duration: 60, // 🔥 默认 60 分钟
    }

    // 🔥 根据 viewKey 提取上下文信息
    const parts = props.viewKey.split('::')
    const [type, subtype, identifier] = parts

    if (type === 'misc' && subtype === 'staging' && identifier) {
      // misc::staging::${areaId} - 指定 area 的 staging 任务
      taskData.area_id = identifier
      logger.debug(LogTags.COMPONENT_KANBAN_COLUMN, 'Creating task with area context', {
        areaId: identifier,
        viewKey: props.viewKey,
      })
    } else if (type === 'area' && subtype) {
      // area::${areaId} - 指定 area 的所有任务
      taskData.area_id = subtype
      logger.debug(LogTags.COMPONENT_KANBAN_COLUMN, 'Creating task with area context', {
        areaId: subtype,
        viewKey: props.viewKey,
      })
    } else if (type === 'project' && subtype) {
      // project::${projectId} - 指定项目的任务
      taskData.project_id = subtype
      logger.debug(LogTags.COMPONENT_KANBAN_COLUMN, 'Creating task with project context', {
        projectId: subtype,
        viewKey: props.viewKey,
      })
    }

    // 🚀 使用 CPU Pipeline 发射指令
    pipeline.dispatch('task.create', taskData)

    logger.info(LogTags.COMPONENT_KANBAN_COLUMN, 'Task creation dispatched', {
      title,
      viewKey: props.viewKey,
      taskData,
    })
  }

  isCreatingTask.value = false
  // 重新聚焦到输入框，方便连续添加任务
  nextTick(() => {
    if (addTaskInputRef.value) {
      addTaskInputRef.value.focus()
    }
  })
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

  // 🔥 使用 CPU Pipeline 更新排序（乐观更新）
  const originalOrder = viewStore.getSortedTaskIds(props.viewKey, effectiveTasks.value)
  pipeline
    .dispatch('viewpreference.update_sorting', {
      view_key: props.viewKey,
      sorted_task_ids: newOrder,
      original_sorted_task_ids: originalOrder, // 用于失败回滚
    })
    .catch((error: unknown) => {
      logger.error(
        LogTags.COMPONENT_KANBAN_COLUMN,
        'Failed to persist completed task reorder',
        error instanceof Error ? error : new Error(String(error)),
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
    // 拖拽过程中不进行自动持久化，避免与策略重复发指令
    if (dragPreviewState.value) {
      return
    }

    const currentTaskIds = new Set(newTasks.map((t) => t.id))
    const hasChanges =
      currentTaskIds.size !== previousTaskIds.value.size ||
      !Array.from(currentTaskIds).every((id) => previousTaskIds.value.has(id))

    if (hasChanges) {
      previousTaskIds.value = currentTaskIds
      const currentOrder = newTasks.map((t) => t.id)

      // 🔥 使用 CPU Pipeline 自动持久化排序（乐观更新）
      const originalOrder = viewStore.getSortedTaskIds(props.viewKey, effectiveTasks.value)
      pipeline
        .dispatch('viewpreference.update_sorting', {
          view_key: props.viewKey,
          sorted_task_ids: currentOrder,
          original_sorted_task_ids: originalOrder,
        })
        .catch((error: unknown) => {
          logger.error(
            LogTags.COMPONENT_KANBAN_COLUMN,
            'Failed to auto-persist view tasks',
            error instanceof Error ? error : new Error(String(error)),
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

// ==================== 标题点击处理 ====================
function handleTitleClick() {
  // 如果禁用了标题点击，则不处理
  if (props.disableTitleClick) return

  // 从 viewMetadata 中提取日期
  if (effectiveViewMetadata.value.type === 'date') {
    const config = effectiveViewMetadata.value.config as import('@/types/drag').DateViewConfig
    const date = config.date // YYYY-MM-DD
    emit('title-click', date)
  }
}

// ==================== 注意 ====================
// displayItems 已由 useInteractDrag 自动提供
// 所有拖放事件处理已由 interact.js 控制器自动管理
// 不需要手动处理 dragstart/dragover/drop 等事件
</script>

<template>
  <CutePane class="simple-kanban-column" :class="{ 'is-expired': isExpired }">
    <!-- 🔥 整个看板作为 dropzone（包含 header、input、task list） -->
    <div ref="kanbanContainerRef" class="kanban-dropzone-wrapper">
      <div class="header">
        <div class="title-row" :class="{ clickable: !disableTitleClick }" @click="handleTitleClick">
          <h2 class="title">{{ title }}</h2>
          <CuteIcon
            v-if="!hideCalendarIcon"
            name="Calendar"
            :size="16"
            class="calendar-icon"
            :class="{ 'is-active': isCalendarDate }"
          />
        </div>
        <div v-if="subtitle" class="subtitle-row">
          <span class="subtitle">{{ subtitle }}</span>
          <button class="sort-button" title="排序">
            <CuteIcon name="ArrowUpDown" :size="14" />
          </button>
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
          v-for="task in displayItems"
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

        <div v-if="displayItems.length === 0" class="empty-state">暂无任务</div>
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
}

/* 🔥 整个看板作为 dropzone wrapper */
.kanban-dropzone-wrapper {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;

  /* 将列的左右内边距转移到真正的 dropzone 包裹层，
     确保可放置区域覆盖视觉上的整列，避免列与列之间出现不可放置的空隙 */
  padding-left: 0.5rem;
  padding-right: 0.5rem;
}

.header {
  padding: 1rem 1rem 0.5rem;
  border-bottom: 1px solid var(--color-border-default);
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  cursor: default;
  padding: 0.2rem 0;
  margin: -0.2rem 0;
  border-radius: 0.4rem;
  transition: all 0.2s ease;
}

.title-row.clickable {
  cursor: pointer;
}

.title {
  font-size: 2.2rem;
  font-weight: 600;
  margin: 0;
  color: var(--color-text-primary);
  transition: color 0.2s ease;
}

/* 过期看板中的标题 */
.simple-kanban-column.is-expired .title {
  opacity: 0.6;
}

.title-row.clickable:hover .title {
  color: var(--rose-pine-foam, #56949f);
}

.calendar-icon {
  opacity: 0;
  color: var(--color-text-secondary); /* 默认使用次要文本颜色 */
  transition:
    opacity 0.2s ease,
    color 0.2s ease;
  flex-shrink: 0;
}

.calendar-icon.is-active {
  opacity: 1;

  /* 不改变颜色，保持默认的次要文本颜色 */
}

.title-row.clickable:hover .calendar-icon {
  opacity: 1;
  color: var(--rose-pine-foam, #56949f); /* hover 时才变绿色 */
}

.subtitle-row {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.subtitle {
  font-size: 1.2rem;
  color: var(--color-text-secondary);
  margin: 0;
  flex: 1;
}

.sort-button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  padding: 0;
  background: transparent;
  border: none;
  border-radius: 0.4rem;
  color: var(--color-text-tertiary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.sort-button:hover {
  background-color: var(--color-background-hover, rgb(0 0 0 / 5%));
  color: var(--color-text-secondary);
}

.count {
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  margin-left: auto;
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
  border-color: var(--rose-pine-foam, #56949f);
  box-shadow: 0 0 0 3px rgb(86 148 159 / 10%);
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

/* 🆕 过期看板灰度效果（Rose Pine Dawn 主题适配） */
.simple-kanban-column.is-expired {
  /* 覆盖文本颜色为更灰的 muted 色（Rose Pine Dawn: #9893a5） */
  --color-text-primary: var(--rose-pine-muted);
  --color-text-secondary: var(--rose-pine-muted);
}

/* 过期看板中的subtitle和数量 */
.simple-kanban-column.is-expired .subtitle,
.simple-kanban-column.is-expired .count {
  opacity: 0.6;
}

/* 过期看板中的任务卡片整体透明度降低 */
.simple-kanban-column.is-expired .task-card-wrapper {
  opacity: 0.7;
}

/* 过期看板中的输入框也变灰 */
.simple-kanban-column.is-expired .add-task-input {
  opacity: 0.6;
}

.simple-kanban-column.is-expired .add-task-input::placeholder {
  color: var(--rose-pine-muted);
}
</style>
