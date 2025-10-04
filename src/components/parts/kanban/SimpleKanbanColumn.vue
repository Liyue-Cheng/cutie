<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import type { TaskCard } from '@/types/dtos'
import type { ViewMetadata } from '@/types/drag'
import { useViewStore } from '@/stores/view'
import { useCrossViewDrag, useDragTransfer } from '@/composables/drag'
import CutePane from '@/components/alias/CutePane.vue'
import KanbanTaskCard from './KanbanTaskCard.vue'

const props = defineProps<{
  title: string
  subtitle?: string
  tasks: TaskCard[]
  showAddInput?: boolean
  viewKey?: string // 视图标识，用于保存排序
  viewMetadata: ViewMetadata // 🆕 看板元数据（用于跨看板拖放）
}>()

const emit = defineEmits<{
  openEditor: [task: TaskCard]
  addTask: [title: string]
  reorderTasks: [newOrder: string[]] // 新顺序的任务ID数组
  crossViewDrop: [taskId: string, targetViewId: string] // 🆕 跨看板放置
}>()

const viewStore = useViewStore()
const crossViewDrag = useCrossViewDrag() // 🆕 跨看板拖放
const dragTransfer = useDragTransfer() // 统一数据传输

const newTaskTitle = ref('')
const isCreatingTask = ref(false)

// 拖拽状态
const draggedTaskId = ref<string | null>(null)
const draggedOverIndex = ref<number | null>(null)

// 🆕 跨看板拖放状态
const crossViewDraggedTask = ref<TaskCard | null>(null) // 从其他看板拖入的任务
const isReceivingCrossViewDrag = ref(false) // 是否正在接收跨看板拖放

// 🆕 容器引用与进入深度计数（稳定 dragenter/dragleave）
const taskListRef = ref<HTMLElement | null>(null)
const dragEnterDepth = ref(0)

// 上一次的任务ID列表（用于检测变化）
const previousTaskIds = ref<Set<string>>(new Set())

// 排序配置是否已加载
const sortingConfigLoaded = ref(false)

// ✅ 组件挂载时，加载该视图的排序配置（如果尚未加载）
onMounted(async () => {
  if (props.viewKey) {
    // ✅ 检查是否已经有排序数据（批量加载可能已完成）
    const alreadyLoaded = viewStore.sortWeights.has(props.viewKey)

    if (!alreadyLoaded) {
      // console.log(`[SimpleKanbanColumn] 🔄 Loading sorting config for "${props.viewKey}"`)
      await viewStore.fetchViewPreference(props.viewKey)
      // console.log(`[SimpleKanbanColumn] ✅ Sorting config loaded for "${props.viewKey}"`)
    } else {
      // console.log(`[SimpleKanbanColumn] ⚡ Sorting config already loaded for "${props.viewKey}"`)
    }

    sortingConfigLoaded.value = true
  } else {
    // 没有 viewKey，标记为已加载（不需要加载）
    sortingConfigLoaded.value = true
  }
})

// ✅ 视觉预览：动态计算显示的任务顺序
const displayTasks = computed(() => {
  let taskList = [...props.tasks]

  // 🔥 关键修复1：如果是源看板，且任务正在被跨看板拖动到其他看板，则移除幽灵元素
  const context = crossViewDrag.currentContext.value
  const targetView = crossViewDrag.targetViewId.value

  if (context && context.sourceView.id === props.viewMetadata.id) {
    // 这是源看板
    // 如果有目标看板且不是当前看板，则隐藏幽灵元素
    if (targetView && targetView !== props.viewMetadata.id) {
      taskList = taskList.filter((t) => t.id !== context.task.id)
    }
  }

  // 🆕 如果正在接收跨看板拖放，且全局目标确认为本列，添加外来任务到列表
  if (
    isReceivingCrossViewDrag.value &&
    crossViewDraggedTask.value &&
    crossViewDrag.targetViewId.value === props.viewMetadata.id
  ) {
    const existingIndex = taskList.findIndex((t) => t.id === crossViewDraggedTask.value!.id)
    if (existingIndex === -1) {
      // 外来任务不在列表中，添加它
      if (draggedOverIndex.value !== null) {
        taskList.splice(draggedOverIndex.value, 0, crossViewDraggedTask.value)
      } else {
        taskList.push(crossViewDraggedTask.value)
      }
    }
  }

  // 原有的同看板内排序逻辑
  if (draggedTaskId.value && draggedOverIndex.value !== null) {
    const draggedIndex = taskList.findIndex((t) => t.id === draggedTaskId.value)
    if (draggedIndex !== -1 && draggedIndex !== draggedOverIndex.value) {
      // 实时重排（仅视觉）
      const [draggedTask] = taskList.splice(draggedIndex, 1)
      if (draggedTask) {
        taskList.splice(draggedOverIndex.value, 0, draggedTask)
      }
    }
  }

  return taskList
})

// 🆕 监听全局目标看板变化，若目标离开本列则立即清理本地接收状态（防残留）
watch(
  () => crossViewDrag.targetViewId.value,
  (newId) => {
    if (newId !== props.viewMetadata.id && isReceivingCrossViewDrag.value) {
      console.log('[SimpleKanbanColumn] 🧹 Target moved away, clearing receiving state:', {
        column: props.viewMetadata.id,
        newTarget: newId,
      })
      isReceivingCrossViewDrag.value = false
      crossViewDraggedTask.value = null
      draggedTaskId.value = null
      draggedOverIndex.value = null
      dragEnterDepth.value = 0
    }
  }
)

async function handleAddTask() {
  const title = newTaskTitle.value.trim()
  if (!title || isCreatingTask.value) return

  isCreatingTask.value = true
  const originalTitle = newTaskTitle.value
  newTaskTitle.value = ''

  try {
    emit('addTask', title)
  } catch (error) {
    console.error(`[SimpleKanbanColumn] Task creation failed:`, error)
    newTaskTitle.value = originalTitle
  } finally {
    isCreatingTask.value = false
  }
}

// ==================== 拖拽排序逻辑 ====================

// 节流控制：防止过度频繁的DOM更新
let lastDragOverTime = 0
const DRAG_THROTTLE_MS = 50 // 50ms节流

/**
 * 拖动开始
 */
function handleDragStart(event: DragEvent, task: TaskCard) {
  if (!event.dataTransfer) return

  // 记录被拖动的任务
  draggedTaskId.value = task.id

  // 🆕 启动跨看板拖放
  crossViewDrag.startNormalDrag(task, props.viewMetadata)

  // 设置拖拽数据（统一使用 dragTransfer）
  dragTransfer.setDragData(event, {
    type: 'task',
    task: task,
    sourceView: props.viewMetadata,
    dragMode: { mode: 'normal' },
  })

  // 设置拖拽效果
  if (event.target instanceof HTMLElement) {
    event.target.style.opacity = '0.5'
  }
}

/**
 * 拖动结束
 */
function handleDragEnd(event: DragEvent) {
  // 恢复样式
  if (event.target instanceof HTMLElement) {
    event.target.style.opacity = '1'
  }

  // 清理状态
  draggedTaskId.value = null
  draggedOverIndex.value = null
  crossViewDraggedTask.value = null
  isReceivingCrossViewDrag.value = false
  lastDragOverTime = 0 // 重置节流时间戳

  // ✅ 修复：若未在进行 drop，或 dropEffect = 'none'（浏览器拒绝）才取消
  // 若 drop 正在进行，由 handleDrop 负责清理，避免竞态
  if (crossViewDrag.currentContext.value) {
    const dropInProgress = (crossViewDrag as any).isDropInProgress?.value
    if (!dropInProgress && event.dataTransfer?.dropEffect === 'none') {
      console.log('[SimpleKanbanColumn] 🚫 Drop rejected or cancelled, cleaning up context')
      crossViewDrag.cancelDrag()
    } else {
      console.log('[SimpleKanbanColumn] ✅ Drop succeeded, let handleDrop clean up context')
    }
  }
  // 确保目标看板标记也被清理
  crossViewDrag.setTargetViewId(null)
}

/**
 * 拖动经过其他卡片时（实时重排 + 优化）
 */
function handleDragOver(event: DragEvent, targetIndex: number) {
  event.preventDefault() // 必须调用，否则无法 drop

  // 🔥 关键修复2：支持跨看板拖放的实时排序
  if (!draggedTaskId.value) return

  // ✅ 节流：限制执行频率，减少闪烁
  const now = Date.now()
  if (now - lastDragOverTime < DRAG_THROTTLE_MS) {
    return
  }
  lastDragOverTime = now

  // 检查是否是跨看板拖放
  const context = crossViewDrag.currentContext.value
  if (context && context.sourceView.id !== props.viewMetadata.id) {
    // 跨看板拖放：直接更新目标索引
    draggedOverIndex.value = targetIndex
    return
  }

  // 同看板内拖放
  const draggedIndex = props.tasks.findIndex((t) => t.id === draggedTaskId.value)
  if (draggedIndex === -1) return // 被拖动的任务不在本列表

  // 避免不必要的重排：只阻止拖到自己
  if (draggedIndex === targetIndex) {
    return
  }

  draggedOverIndex.value = targetIndex

  // ⚠️ 注意：这里不调用后端！
  // dragover 只是视觉预览，真正的持久化在 drop 时进行
}

/**
 * 🆕 容器级 dragover：根据鼠标 Y 定位插入位置，避免在子项之间来回抖动
 */
function handleContainerDragOver(event: DragEvent) {
  // 仅在跨看板接收时启用容器级定位
  if (!isReceivingCrossViewDrag.value || !draggedTaskId.value) return

  event.preventDefault()

  // 节流
  const now = Date.now()
  if (now - lastDragOverTime < DRAG_THROTTLE_MS) {
    return
  }
  lastDragOverTime = now

  const container = taskListRef.value
  if (!container) return

  const mouseY = event.clientY
  const wrappers = Array.from(container.querySelectorAll<HTMLElement>('.task-card-wrapper'))

  // 忽略幽灵元素自身（防止自我影响引起抖动）
  const ghostId = crossViewDraggedTask.value?.id || null
  const candidates = ghostId
    ? wrappers.filter((el) => (el.dataset.taskId || '') !== ghostId)
    : wrappers

  // 计算插入索引：第一个“中心点”在鼠标之下的元素索引
  let index = candidates.length
  for (let i = 0; i < candidates.length; i++) {
    const el = candidates[i]
    if (!el) continue
    const rect = el.getBoundingClientRect()
    const centerY = rect.top + rect.height / 2
    if (mouseY < centerY) {
      index = i
      break
    }
  }

  draggedOverIndex.value = index
}

/**
 * 🆕 看板区域的 dragenter - 检测跨看板拖放
 */
function handleColumnDragEnter(event: DragEvent) {
  event.preventDefault()

  const context = crossViewDrag.currentContext.value
  if (!context) return

  // 检查是否是跨看板拖放
  if (context.sourceView.id !== props.viewMetadata.id) {
    // 进入深度计数，避免在子元素间移动造成抖动
    dragEnterDepth.value += 1

    if (dragEnterDepth.value === 1) {
      console.log('[SimpleKanbanColumn] 🌍 Cross-view drag entered:', {
        from: context.sourceView.id,
        to: props.viewMetadata.id,
        task: context.task.title,
      })

      // 🆕 设置当前目标看板ID（用于源看板隐藏幽灵元素）
      crossViewDrag.setTargetViewId(props.viewMetadata.id)

      // 激活跨看板接收模式（首次真正进入本容器）
      isReceivingCrossViewDrag.value = true
      crossViewDraggedTask.value = context.task
      draggedTaskId.value = context.task.id
      draggedOverIndex.value = null // 初始不设置位置，等待第一次 dragover
    }
  }
}

/**
 * 🆕 看板区域的 dragleave - 清理跨看板状态
 */
function handleColumnDragLeave(event: DragEvent) {
  const context = crossViewDrag.currentContext.value
  if (context && context.sourceView.id !== props.viewMetadata.id) {
    // 优先基于几何判断是否真正离开容器，避免事件丢失导致的残留
    const container = event.currentTarget as HTMLElement
    const rect = container.getBoundingClientRect()
    const x = event.clientX
    const y = event.clientY
    const reallyLeft = x < rect.left || x > rect.right || y < rect.top || y > rect.bottom

    // 维持原有深度计数，兼容子元素切换
    dragEnterDepth.value = Math.max(0, dragEnterDepth.value - 1)

    if (reallyLeft || dragEnterDepth.value === 0) {
      console.log('[SimpleKanbanColumn] 🚪 Cross-view drag left:', props.viewMetadata.id, {
        reallyLeft,
        depth: dragEnterDepth.value,
      })

      // 🧹 清理接收状态
      isReceivingCrossViewDrag.value = false
      crossViewDraggedTask.value = null
      draggedTaskId.value = null
      draggedOverIndex.value = null
      dragEnterDepth.value = 0

      // 🧹 清理全局目标（通知源看板恢复幽灵元素 / 其他列停止显示幽灵）
      if (crossViewDrag.targetViewId.value === props.viewMetadata.id) {
        crossViewDrag.setTargetViewId(null)
      }
    }
  }
}

/**
 * 放置（持久化排序 + 跨看板拖放）
 */
async function handleDrop(event: DragEvent) {
  event.preventDefault()

  const context = crossViewDrag.currentContext.value

  // 🆕 检查是否是跨看板拖放
  if (context && context.sourceView.id !== props.viewMetadata.id) {
    console.log('[SimpleKanbanColumn] 🎯 Cross-view drop detected')

    // 调用跨看板拖放框架（仅输出日志）
    const result = await crossViewDrag.handleDrop(props.viewMetadata, event)

    if (result.success) {
      console.log('✅ 跨看板拖放成功:', result.message)

      // 触发跨看板事件（让父组件处理实际的数据更新）
      emit('crossViewDrop', context.task.id, props.viewMetadata.id)
    } else {
      console.error('❌ 跨看板拖放失败:', result.error)
    }

    // 清理状态
    draggedTaskId.value = null
    draggedOverIndex.value = null
    crossViewDraggedTask.value = null
    isReceivingCrossViewDrag.value = false
    // 重置进入深度计数与目标ID
    dragEnterDepth.value = 0
    crossViewDrag.setTargetViewId(null)
    return
  }

  // 原有逻辑：同看板内排序
  if (!draggedTaskId.value) return

  // ✅ 使用 displayTasks（包含最新的拖拽结果）
  const finalOrder = displayTasks.value.map((t) => t.id)

  // console.log('[SimpleKanbanColumn] Drop完成，最终顺序:', finalOrder)

  // ✅ 持久化到后端（通过父组件）
  emit('reorderTasks', finalOrder)

  // 清理状态
  draggedTaskId.value = null
  draggedOverIndex.value = null
}

// ==================== 自动检测任务列表变化并持久化 ====================

/**
 * ✅ 核心功能：自动检测任务列表变化
 *
 * 触发条件：
 * - 新任务创建（任务ID集合增加）
 * - 任务删除（任务ID集合减少）
 * - 任务状态变化导致进出视图（如完成/重开任务）
 *
 * 行为：
 * - 自动为当前顺序赋予权重并持久化到后端
 * - 确保刷新页面后顺序不变
 *
 * 注意：
 * - 不在拖拽过程中触发（拖拽有自己的持久化逻辑）
 * - 只在有 viewKey 时执行
 */
watch(
  () => props.tasks,
  (newTasks) => {
    // console.log(`[SimpleKanbanColumn] 🔄 Watch triggered for "${props.viewKey || 'NO_KEY'}":`, {
    //   taskCount: newTasks.length,
    //   taskIds: newTasks.map((t) => t.id),
    //   hasViewKey: !!props.viewKey,
    //   isDragging: draggedTaskId.value !== null,
    //   sortingConfigLoaded: sortingConfigLoaded.value,
    // })

    // 等待排序配置加载完成
    if (!sortingConfigLoaded.value) {
      // console.log(
      //   `[SimpleKanbanColumn] ⏭️ Skip: Waiting for sorting config to load for "${props.viewKey}"`
      // )
      // 更新任务ID记录，但不持久化
      previousTaskIds.value = new Set(newTasks.map((t) => t.id))
      return
    }

    // 没有 viewKey，无法持久化
    if (!props.viewKey) {
      // console.log(`[SimpleKanbanColumn] ⏭️ Skip: No viewKey`)
      return
    }

    // 正在拖拽中，不要干扰（拖拽结束会自己持久化）
    if (draggedTaskId.value !== null) {
      // console.log(
      //   `[SimpleKanbanColumn] ⏭️ Skip: Dragging in progress (draggedTaskId=${draggedTaskId.value})`
      // )
      return
    }

    // 构建当前任务ID集合
    const currentTaskIds = new Set(newTasks.map((t) => t.id))

    // 检查是否真的有变化（新增或删除）
    const hasChanges =
      currentTaskIds.size !== previousTaskIds.value.size ||
      !Array.from(currentTaskIds).every((id) => previousTaskIds.value.has(id))

    // console.log(`[SimpleKanbanColumn] 🔍 Change detection for "${props.viewKey}":`, {
    //   previousSize: previousTaskIds.value.size,
    //   currentSize: currentTaskIds.size,
    //   hasChanges,
    //   newTasks: Array.from(currentTaskIds).filter((id) => !previousTaskIds.value.has(id)),
    //   removedTasks: Array.from(previousTaskIds.value).filter((id) => !currentTaskIds.has(id)),
    // })

    if (hasChanges) {
      // console.log(`[SimpleKanbanColumn] ✅ Detected task list changes in "${props.viewKey}":`, {
      //   before: previousTaskIds.value.size,
      //   after: currentTaskIds.size,
      //   new: Array.from(currentTaskIds).filter((id) => !previousTaskIds.value.has(id)),
      //   removed: Array.from(previousTaskIds.value).filter((id) => !currentTaskIds.has(id)),
      // })

      // 更新记录
      previousTaskIds.value = currentTaskIds

      // ✅ 自动持久化当前顺序
      const currentOrder = newTasks.map((t) => t.id)
      // console.log(
      //   `[SimpleKanbanColumn] 💾 Calling updateSorting for "${props.viewKey}" with order:`,
      //   currentOrder
      // )

      viewStore
        .updateSorting(props.viewKey, currentOrder)
        .then((success) => {
          if (success) {
            // console.log(`[SimpleKanbanColumn] ✅ Auto-persisted sorting for "${props.viewKey}"`)
          } else {
            console.error(
              `[SimpleKanbanColumn] ❌ Failed to auto-persist sorting for "${props.viewKey}"`
            )
          }
        })
        .catch((error) => {
          console.error(
            `[SimpleKanbanColumn] ❌ Error during auto-persist for "${props.viewKey}":`,
            error
          )
        })
    } else {
      // console.log(
      //   `[SimpleKanbanColumn] ⏭️ No changes detected for "${props.viewKey}", skipping persistence`
      // )
      // 没有真正的变化，只是响应式更新，更新记录即可
      previousTaskIds.value = currentTaskIds
    }
  },
  { deep: false, immediate: true } // immediate: 初始化时也执行一次
)
</script>

<template>
  <CutePane
    class="simple-kanban-column"
    @dragenter="handleColumnDragEnter"
    @dragleave="handleColumnDragLeave"
    @drop="handleDrop"
    @dragover.prevent
  >
    <div class="header">
      <div class="title-section">
        <h2 class="title">{{ title }}</h2>
        <p v-if="subtitle" class="subtitle">{{ subtitle }}</p>
      </div>
      <div class="task-count">
        <span class="count">{{ tasks.length }}</span>
      </div>
    </div>

    <div v-if="showAddInput" class="add-task-wrapper">
      <input
        v-model="newTaskTitle"
        type="text"
        placeholder="+ 添加任务"
        class="add-task-input"
        :disabled="isCreatingTask"
        @keydown.enter="handleAddTask"
      />
      <div v-if="isCreatingTask" class="creating-indicator">创建中...</div>
    </div>

    <div ref="taskListRef" class="task-list-scroll-area" @dragover="handleContainerDragOver">
      <div
        v-for="(task, index) in displayTasks"
        :key="task.id"
        class="task-card-wrapper"
        :data-task-id="task.id"
        :data-dragging="draggedTaskId === task.id"
        draggable="true"
        @dragstart="handleDragStart($event, task)"
        @dragend="handleDragEnd"
        @dragover="handleDragOver($event, index)"
      >
        <KanbanTaskCard
          :task="task"
          class="kanban-task-card"
          @open-editor="emit('openEditor', task)"
        />
      </div>

      <div v-if="displayTasks.length === 0" class="empty-state">暂无任务</div>
    </div>
  </CutePane>
</template>

<style scoped>
.simple-kanban-column {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--color-background-content);
  width: 21rem;
  flex-shrink: 0;
  padding-left: 0.5rem;
  padding-right: 0.5rem;
}

.header {
  padding: 1rem 1rem 0.5rem;
  border-bottom: 1px solid var(--color-border-default);
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

.creating-indicator {
  font-size: 1.2rem;
  color: var(--color-text-secondary);
  padding: 0.5rem 0.75rem;
  font-style: italic;
}

.task-list-scroll-area {
  flex-grow: 1;
  overflow-y: auto;
  padding: 0.5rem 1rem 1rem;
  min-height: 100px;

  /* scrollbar-gutter: stable; */
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

/* 拖拽相关样式 */
.task-card-wrapper {
  position: relative;
  cursor: grab;
  transition: transform 0.2s ease;
}

.task-card-wrapper:active {
  cursor: grabbing;
}

.task-card-wrapper[data-dragging='true'] {
  opacity: 0.5;
}

.kanban-task-card {
  cursor: grab;
  pointer-events: auto;
}

.kanban-task-card:active {
  cursor: grabbing;
}
</style>
