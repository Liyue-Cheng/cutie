<script setup lang="ts">
import { ref, computed } from 'vue'
import type { TaskCard } from '@/types/dtos'
import { useTaskStore } from '@/stores/task'
// import { useScheduleStore } from '@/stores/schedule' // TODO: Schedule store 需要重建
import CutePane from '@/components/alias/CutePane.vue'
import KanbanTaskCard from './KanbanTaskCard.vue'

const props = defineProps<{
  date: Date
  tasks: TaskCard[]
}>()

const emit = defineEmits(['openEditor', 'taskCreated'])

const taskStore = useTaskStore()
// const scheduleStore = useScheduleStore() // TODO: Schedule store 需要重建
const newTaskTitle = ref('')
const isCreatingTask = ref(false)

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
    const newTask = await taskStore.createTask({
      title,
      // TODO: 新的 payload 结构不需要 context 字段
      // context: {
      //   context_type: 'DAILY_KANBAN',
      //   context_id: props.date.toISOString(),
      // },
    })

    if (newTask) {
      const dateStr = props.date.toISOString().split('T')[0] as string
      // TODO: Schedule store 需要重建
      // await scheduleStore.scheduleTask({
      //   task_id: newTask.id,
      //   scheduled_day: dateStr,
      // })

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
const completedCount = computed(() => props.tasks.filter((t) => t.is_completed).length)

// 处理原生拖拽开始事件
function handleDragStart(event: DragEvent, task: TaskCard) {
  if (!event.dataTransfer) return

  // 设置任务数据，日历可以读取
  event.dataTransfer.setData(
    'application/json',
    JSON.stringify({
      type: 'task',
      task: task,
    })
  )
  event.dataTransfer.effectAllowed = 'copyMove'
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

    <div class="task-list-scroll-area">
      <div
        v-for="task in tasks"
        :key="task.id"
        class="task-card-wrapper"
        :data-task-id="task.id"
        draggable="true"
        @dragstart="handleDragStart($event, task)"
      >
        <KanbanTaskCard
          :task="task"
          class="kanban-task-card"
          @open-editor="emit('openEditor', task)"
        />
      </div>

      <div v-if="tasks.length === 0" class="empty-state">暂无任务</div>
    </div>
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
  cursor: grab;
}

.task-card-wrapper:active {
  cursor: grabbing;
}

.kanban-task-card {
  cursor: grab;
}

.kanban-task-card:active {
  cursor: grabbing;
}
</style>
