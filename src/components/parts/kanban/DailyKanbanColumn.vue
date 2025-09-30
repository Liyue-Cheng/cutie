<script setup lang="ts">
import { ref, computed } from 'vue'
import draggable from 'vuedraggable'
import type { Task } from '@/types/models'
import { useTaskStore } from '@/stores/task'
import { useScheduleStore } from '@/stores/schedule'
import { useOrderingStore } from '@/stores/ordering'
import CutePane from '@/components/alias/CutePane.vue'
import KanbanTaskCard from './KanbanTaskCard.vue'

const props = defineProps<{
  date: Date
  tasks: Task[]
}>()

const emit = defineEmits(['openEditor', 'taskCreated', 'taskDeleted'])

const taskStore = useTaskStore()
const scheduleStore = useScheduleStore()
const orderingStore = useOrderingStore()
const newTaskTitle = ref('')
const isCreatingTask = ref(false)

// 本地任务列表（响应式副本，用于拖拽）
const localTasks = computed({
  get: () => props.tasks,
  set: () => {
    // 拖动过程中的临时变更由 vue-draggable-next 自动处理
    // 实际更新在 onDragEnd 中进行
  },
})

// 格式化日期显示
const dateTitle = computed(() => {
  const today = new Date()
  today.setHours(0, 0, 0, 0)
  const targetDate = new Date(props.date)
  targetDate.setHours(0, 0, 0, 0)

  const diffDays = Math.floor((targetDate.getTime() - today.getTime()) / (1000 * 60 * 60 * 24))

  if (diffDays === 0) return '今天'
  if (diffDays === 1) return '明天'
  if (diffDays === 2) return '后天'

  return `${targetDate.getMonth() + 1}月${targetDate.getDate()}日`
})

const dateSubtitle = computed(() => {
  const targetDate = new Date(props.date)
  const year = targetDate.getFullYear()
  const month = String(targetDate.getMonth() + 1).padStart(2, '0')
  const day = String(targetDate.getDate()).padStart(2, '0')
  const weekdays = ['周日', '周一', '周二', '周三', '周四', '周五', '周六']
  const weekday = weekdays[targetDate.getDay()]

  return `${year}-${month}-${day} ${weekday}`
})

// 创建任务并自动排程到当天
async function handleAddTask() {
  const title = newTaskTitle.value.trim()
  if (!title || isCreatingTask.value) return

  isCreatingTask.value = true
  const originalTitle = newTaskTitle.value
  newTaskTitle.value = ''

  try {
    // 使用毫秒时间戳作为 context_id，与排程时保持一致
    const dateStr = props.date.toISOString().split('T')[0] as string
    const contextId = new Date(dateStr).getTime().toString()
    
    await taskStore.createTask({
      title,
      context: {
        context_type: 'DAILY_KANBAN',
        context_id: contextId,
      },
    })

    const tasks = Array.from(taskStore.tasks.values())
    const newTask = tasks.find((t) => t.title === title)

    if (newTask) {
      await scheduleStore.scheduleTask({
        task_id: newTask.id,
        scheduled_day: dateStr,
      })

      emit('taskCreated', dateStr)
    }
  } catch (error) {
    console.error(`[DailyKanban] Task creation failed:`, error)
    newTaskTitle.value = originalTitle
  } finally {
    isCreatingTask.value = false
  }
}

// 任务数量统计
const taskCount = computed(() => props.tasks.length)
const completedCount = computed(() => props.tasks.filter((t) => t.completed_at).length)

// 拖拽结束时的处理
async function onDragEnd(event: any) {
  const { oldIndex, newIndex, from, to } = event

  console.log('[DailyKanban] Drag end:', {
    oldIndex,
    newIndex,
    from,
    to,
    tasksLength: props.tasks.length,
  })

  // 如果没有实际移动，直接返回
  if (oldIndex === newIndex && from === to) return

  // 注意：拖动过程中 vuedraggable 已经临时更新了数组
  // 所以我们需要根据 oldIndex 来找到被移动的任务
  // 但由于数组已变化，我们需要用 movedTask 的 ID 来追踪
  let movedTask: Task | undefined

  // 在同一列表内拖动时，从当前位置找到被移动的任务
  if (from === to) {
    movedTask = props.tasks[newIndex]
  } else {
    // 跨列表拖动时，从新列表的 newIndex 位置找
    movedTask = props.tasks[newIndex]
  }

  if (!movedTask) {
    console.error('[DailyKanban] Cannot find moved task')
    return
  }

  console.log('[DailyKanban] Moved task:', movedTask.title, 'to index:', newIndex)

  const dateStr = props.date.toISOString().split('T')[0] as string
  const contextId = new Date(dateStr).getTime().toString()

  try {
    // 如果是跨列表拖动（from !== to），需要先排程到新日期
    if (from !== to) {
      await scheduleStore.scheduleTask({
        task_id: movedTask.id,
        scheduled_day: dateStr,
      })
    }

    // 计算新的 sort_order
    // 获取目标位置前后的任务（排除被移动的任务本身）
    let prevTask: Task | undefined
    let nextTask: Task | undefined

    // 向前查找 prevTask（不是被移动的任务）
    for (let i = newIndex - 1; i >= 0; i--) {
      if (props.tasks[i]?.id !== movedTask.id) {
        prevTask = props.tasks[i]
        break
      }
    }

    // 向后查找 nextTask（不是被移动的任务）
    for (let i = newIndex + 1; i < props.tasks.length; i++) {
      if (props.tasks[i]?.id !== movedTask.id) {
        nextTask = props.tasks[i]
        break
      }
    }

    console.log('[DailyKanban] Prev task:', prevTask?.title, 'Next task:', nextTask?.title)

    // 获取前后任务的 sort_order，如果不存在则等待后端生成
    let prevSortOrder = prevTask ? getSortOrderForTask(prevTask.id) : undefined
    let nextSortOrder = nextTask ? getSortOrderForTask(nextTask.id) : undefined

    console.log('[DailyKanban] Prev sort_order:', prevSortOrder, 'Next sort_order:', nextSortOrder)

    // 如果前后任务存在但没有 sort_order，说明数据不一致，需要重新加载
    if ((prevTask && !prevSortOrder) || (nextTask && !nextSortOrder)) {
      console.warn('[DailyKanban] Missing sort_order for adjacent tasks, reloading ordering data')
      await orderingStore.fetchOrderingsForContext('DAILY_KANBAN', contextId)

      // 重新获取 sort_order
      prevSortOrder = prevTask ? getSortOrderForTask(prevTask.id) : undefined
      nextSortOrder = nextTask ? getSortOrderForTask(nextTask.id) : undefined

      console.log(
        '[DailyKanban] After reload - Prev sort_order:',
        prevSortOrder,
        'Next sort_order:',
        nextSortOrder
      )
    }

    const newSortOrder = await orderingStore.calculateSortOrder({
      context_type: 'DAILY_KANBAN',
      context_id: contextId,
      prev_sort_order: prevSortOrder,
      next_sort_order: nextSortOrder,
    })

    if (!newSortOrder) {
      console.error('[DailyKanban] Failed to calculate sort order')
      return
    }

    console.log('[DailyKanban] Calculated new sort_order:', newSortOrder)

    // 更新排序
    await orderingStore.updateOrder({
      task_id: movedTask.id,
      context_type: 'DAILY_KANBAN',
      context_id: contextId,
      sort_order: newSortOrder,
    })

    // 刷新任务列表
    emit('taskCreated', dateStr)
  } catch (error) {
    console.error('[DailyKanban] Failed to handle drag end:', error)
    // 刷新以恢复正确状态
    emit('taskCreated', dateStr)
  }
}

function getSortOrderForTask(taskId: string): string | undefined {
  const dateStr = props.date.toISOString().split('T')[0] as string
  const contextId = new Date(dateStr).getTime().toString()
  return orderingStore.getOrdering('DAILY_KANBAN', contextId, taskId)?.sort_order
}

// Draggable 配置
const draggableOptions = {
  animation: 200,
  group: 'tasks',
  ghostClass: 'ghost-dragging',
  chosenClass: 'chosen-dragging',
  dragClass: 'dragging',
  forceFallback: false,
  fallbackClass: 'fallback-dragging',
  handle: '.kanban-task-card',
}
</script>

<template>
  <CutePane class="daily-kanban-column">
    <div class="header">
      <div class="date-info">
        <h2 class="date-title">{{ dateTitle }}</h2>
        <p class="date-subtitle">{{ dateSubtitle }}</p>
      </div>
      <div class="task-count">
        <span class="completed">{{ completedCount }}</span>
        <span class="separator">/</span>
        <span class="total">{{ taskCount }}</span>
      </div>
    </div>

    <div class="add-task-wrapper">
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

    <draggable
      v-model="localTasks"
      class="task-list-scroll-area"
      v-bind="draggableOptions"
      @end="onDragEnd"
      item-key="id"
    >
      <template #item="{ element: task }">
        <div class="task-card-wrapper">
          <KanbanTaskCard
            :task="task"
            class="kanban-task-card"
            @open-editor="emit('openEditor', task)"
            @task-deleted="(taskId: string) => emit('taskDeleted', taskId)"
          />
        </div>
      </template>
    </draggable>

    <div v-if="tasks.length === 0" class="empty-state">暂无任务</div>
  </CutePane>
</template>

<style scoped>
.daily-kanban-column {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--color-background-content);
  width: 21rem;
  flex-shrink: 0;
}

.header {
  padding: 1rem 1rem 0.5rem;
  border-bottom: 1px solid var(--color-border-default);
}

.date-info {
  margin-bottom: 0.5rem;
}

.date-title {
  font-size: 2.2rem;
  font-weight: 600;
  margin: 0;
  color: var(--color-text-primary);
}

.date-subtitle {
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

.task-count .completed {
  color: var(--color-success, #22c55e);
}

.task-count .separator {
  color: var(--color-text-tertiary);
}

.task-count .total {
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
  cursor: move;
}

.kanban-task-card {
  cursor: move;
}

/* 幽灵元素样式 - 跟随鼠标的半透明副本 */
:deep(.ghost-dragging) {
  opacity: 0.5;
  background: var(--color-card-available);
}

/* 被选中开始拖动的元素 - 保持原位置可见 */
:deep(.chosen-dragging) {
  opacity: 1;
}

/* 正在拖动时的样式 */
:deep(.dragging) {
  opacity: 0;
}

/* Fallback 模式的拖动样式 */
:deep(.fallback-dragging) {
  opacity: 0.5;
}
</style>
