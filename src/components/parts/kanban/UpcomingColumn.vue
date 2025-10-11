<script setup lang="ts">
import { computed } from 'vue'
import { useTaskStore } from '@/stores/task'
import { useUIStore } from '@/stores/ui'
import { logger, LogTags } from '@/services/logger'
import KanbanTaskCard from './KanbanTaskCard.vue'
import type { TaskCard } from '@/types/dtos'

// 🗑️ 移除 emit - 使用 UI Store

const taskStore = useTaskStore()
const uiStore = useUIStore()

// 获取所有有截止日期的任务，按截止日期排序
const upcomingTasks = computed(() => {
  const tasksWithDueDate = taskStore.allTasks.filter(
    (task) => task.due_date && !task.is_archived && !task.is_completed
  )

  // 按截止日期排序（最近的在前）
  return tasksWithDueDate.sort((a, b) => {
    const dateA = new Date(a.due_date!.date).getTime()
    const dateB = new Date(b.due_date!.date).getTime()
    return dateA - dateB
  })
})

// 任务总数
const taskCount = computed(() => upcomingTasks.value.length)

// 过期任务数量
const overdueCount = computed(() => {
  return upcomingTasks.value.filter((task) => task.due_date?.is_overdue).length
})

function handleOpenEditor(task: TaskCard) {
  uiStore.openEditor(task.id, 'misc::upcoming')
}

// 拖动相关：允许拖动但不保存状态
function handleDragStart(event: DragEvent, task: TaskCard) {
  if (!event.dataTransfer) return
  event.dataTransfer.effectAllowed = 'move'
  event.dataTransfer.setData('application/json', JSON.stringify({ taskId: task.id }))
}

function handleDragOver(event: DragEvent) {
  event.preventDefault()
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move'
  }
}

function handleDrop(event: DragEvent) {
  event.preventDefault()
  // 不做任何处理，让任务卡片自动弹回原位
  logger.debug(LogTags.COMPONENT_KANBAN_COLUMN, 'Drop ignored - tasks are sorted by due date')
}
</script>

<template>
  <div class="upcoming-column">
    <div class="column-header">
      <div class="header-title">
        <h3>即将到期</h3>
        <span class="task-count">{{ taskCount }}</span>
      </div>
      <div v-if="overdueCount > 0" class="overdue-badge">{{ overdueCount }} 个已逾期</div>
    </div>

    <div class="column-content" @dragover="handleDragOver" @drop="handleDrop">
      <div v-if="upcomingTasks.length === 0" class="empty-state">
        <p>没有设置截止日期的任务</p>
      </div>

      <div v-else class="tasks-list">
        <div
          v-for="task in upcomingTasks"
          :key="task.id"
          class="task-wrapper"
          draggable="true"
          @dragstart="handleDragStart($event, task)"
        >
          <KanbanTaskCard
            :task="task"
            :view-metadata="{ type: 'misc', id: 'upcoming' }"
            @open-editor="handleOpenEditor(task)"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.upcoming-column {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--color-background-content);
}

.column-header {
  padding: 1.5rem 1.5rem 1rem;
  border-bottom: 1px solid var(--color-border-default);
  background-color: var(--color-background-content);
}

.header-title {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  margin-bottom: 0.8rem;
}

.header-title h3 {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.task-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2.4rem;
  height: 2.4rem;
  padding: 0 0.6rem;
  font-size: 1.2rem;
  font-weight: 600;
  color: var(--color-text-tertiary);
  background-color: var(--color-background-hover);
  border-radius: 1.2rem;
}

.overdue-badge {
  display: inline-flex;
  align-items: center;
  padding: 0.4rem 0.8rem;
  font-size: 1.2rem;
  font-weight: 500;
  color: #f44336;
  background-color: rgb(244 67 54 / 10%);
  border-radius: 0.4rem;
}

.column-content {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 2rem;
}

.empty-state p {
  font-size: 1.4rem;
  color: var(--color-text-tertiary);
  text-align: center;
}

.tasks-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.task-wrapper {
  cursor: move;
  transition: opacity 0.2s ease;
}

.task-wrapper:active {
  opacity: 0.6;
}
</style>
