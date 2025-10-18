<script setup lang="ts">
import { computed, ref } from 'vue'
import type { TaskCard } from '@/types/dtos'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import { useTimelineCardDrag } from '@/composables/drag/useTimelineCardDrag'

const props = defineProps<{
  date: string // YYYY-MM-DD
  tasks: TaskCard[]
  isToday?: boolean
  isPast?: boolean
}>()

// 容器元素引用
const containerRef = ref<HTMLElement | null>(null)

// 使用拖放 composable
const { displayTasks, isReceiving } = useTimelineCardDrag({
  date: props.date,
  originalTasks: computed(() => props.tasks), // 修复：使用 computed 确保响应性
  containerRef,
  onDrop: async (session: any) => {
    console.log('Task dropped on timeline card:', props.date, session)
    // TODO: 实现真正的拖放逻辑（当需要时）
  },
})

// 格式化日期显示
const formattedDate = computed(() => {
  const date = new Date(props.date)
  return {
    month: date.getMonth() + 1,
    day: date.getDate(),
    weekday: ['周日', '周一', '周二', '周三', '周四', '周五', '周六'][date.getDay()],
  }
})

// 根据任务状态获取图标
function getTaskStatusIcon(task: TaskCard) {
  if (task.is_completed) {
    return 'CircleCheck' // 已完成
  } else {
    return 'Circle' // 未完成
  }
}

// 根据任务状态获取颜色类
function getTaskStatusClass(task: TaskCard): string {
  if (task.is_completed) {
    return 'task-completed'
  } else {
    return 'task-incomplete'
  }
}

// 检查任务是否是预览状态
function isPreviewTask(task: TaskCard): boolean {
  return (task as TaskCard & { _isPreview?: boolean })?._isPreview === true
}
</script>

<template>
  <div
    ref="containerRef"
    class="timeline-card"
    :class="{
      'is-today': isToday,
      'is-past': isPast,
      'is-future': !isPast && !isToday,
      'is-receiving': isReceiving,
    }"
  >
    <!-- 日期头部 -->
    <div class="timeline-date-header">
      <div class="date-number">{{ formattedDate.day }}</div>
      <div class="date-info">
        <div class="month">{{ formattedDate.month }}月</div>
        <div class="weekday">{{ formattedDate.weekday }}</div>
      </div>
      <div v-if="isToday" class="today-indicator">今天</div>
    </div>

    <!-- 任务列表 -->
    <div class="timeline-tasks">
      <div
        v-for="task in displayTasks"
        :key="task.id"
        class="timeline-task"
        :class="[getTaskStatusClass(task), { 'task-preview': isPreviewTask(task) }]"
      >
        <div class="task-status-icon">
          <CuteIcon :name="getTaskStatusIcon(task)" :size="14" />
        </div>
        <div class="task-content">
          <div class="task-title">{{ task.title }}</div>
          <div v-if="task.estimated_duration" class="task-duration">
            {{ Math.round(task.estimated_duration / 60) }}h
          </div>
        </div>
      </div>

      <!-- 空状态 -->
      <div v-if="displayTasks.length === 0" class="timeline-empty">
        <CuteIcon name="Calendar" :size="16" />
        <span>无任务</span>
      </div>
    </div>

    <!-- 拖放提示（在拖拽悬停时显示） -->
    <div v-if="isReceiving" class="drop-indicator">
      <CuteIcon name="Plus" :size="16" />
      <span>拖放任务到此日期</span>
    </div>
  </div>
</template>

<style scoped>
.timeline-card {
  background-color: var(--color-background-content);
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
  padding: 1.2rem;
  margin-bottom: 0.8rem;
  display: flex;
  flex-direction: column;
  transition: all 0.2s ease;
  position: relative;
}

.timeline-card.is-today {
  border-color: var(--color-primary);
  background-color: var(--color-primary-bg);
}

.timeline-card.is-past {
  opacity: 0.7;
}

.timeline-card:hover {
  box-shadow: 0 2px 8px rgb(0 0 0 / 8%);
}

/* 日期头部 */
.timeline-date-header {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  margin-bottom: 1rem;
  padding-bottom: 0.8rem;
  border-bottom: 1px solid var(--color-border-soft);
}

.date-number {
  font-size: 2.4rem;
  font-weight: 700;
  color: var(--color-text-primary);
  line-height: 1;
  min-width: 3rem;
}

.timeline-card.is-today .date-number {
  color: var(--color-primary);
}

.date-info {
  flex: 1;
}

.month {
  font-size: 1.3rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  line-height: 1.2;
}

.weekday {
  font-size: 1.1rem;
  color: var(--color-text-tertiary);
  line-height: 1.2;
}

.today-indicator {
  background-color: var(--color-primary);
  color: white;
  padding: 0.2rem 0.6rem;
  border-radius: 1rem;
  font-size: 1rem;
  font-weight: 500;
}

/* 任务列表 */
.timeline-tasks {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.timeline-task {
  display: flex;
  align-items: flex-start;
  gap: 0.8rem;
  padding: 0.6rem;
  border-radius: 0.4rem;
  transition: background-color 0.2s ease;
  cursor: pointer;
}

.timeline-task:hover {
  background-color: var(--color-background-hover);
}

.task-status-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-top: 0.1rem;
  flex-shrink: 0;
}

.task-completed .task-status-icon {
  color: var(--color-success, #10b981);
}

.task-presence .task-status-icon {
  color: var(--color-warning, #f59e0b);
}

.task-incomplete .task-status-icon {
  color: var(--color-text-tertiary);
}

.task-content {
  flex: 1;
  min-width: 0;
}

.task-title {
  font-size: 1.3rem;
  font-weight: 500;
  color: var(--color-text-primary);
  line-height: 1.3;
  overflow-wrap: break-word;
}

.task-completed .task-title {
  text-decoration: line-through;
  color: var(--color-text-tertiary);
}

.task-duration {
  font-size: 1.1rem;
  color: var(--color-text-tertiary);
  margin-top: 0.2rem;
}

/* 空状态 */
.timeline-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.6rem;
  padding: 1.5rem;
  color: var(--color-text-tertiary);
  font-size: 1.2rem;
}

/* 拖放接收状态 */
.timeline-card.is-receiving {
  border-color: var(--color-primary);
  background-color: var(--color-primary-bg);
  transform: scale(1.02);
  box-shadow: 0 4px 12px rgb(0 0 0 / 12%);
}

.timeline-card.is-receiving .timeline-date-header {
  color: var(--color-primary);
}

/* 预览任务样式 */
.timeline-task.task-preview {
  background-color: var(--color-primary-bg);
  border: 1px dashed var(--color-primary);
  opacity: 0.8;
  animation: fade-in 0.2s ease-in-out;
}

.timeline-task.task-preview .task-title {
  color: var(--color-primary);
  font-style: italic;
}

.timeline-task.task-preview .task-status-icon {
  color: var(--color-primary);
}

@keyframes fade-in {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }

  to {
    opacity: 0.8;
    transform: translateY(0);
  }
}

/* 拖放提示更新 */
.drop-indicator {
  position: absolute;
  inset: 0;
  background-color: var(--color-primary-bg);
  border: 2px dashed var(--color-primary);
  border-radius: 0.8rem;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.6rem;
  color: var(--color-primary);
  font-weight: 500;
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 0.8;
  }

  50% {
    opacity: 1;
  }
}
</style>
