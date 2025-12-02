<!--
  UpcomingSixColumnLayout.vue - 即将到来任务的6栏布局组件
  
  功能：
  - 可复用的6栏时间范围布局（逾期、今日、本周、下周、本月、更远）
  - 每栏包含三类任务：截止日期、循环任务、排期任务
  - 支持横向拖拽滚动
-->
<script setup lang="ts">
import { ref } from 'vue'
import TaskStrip from '@/components/assembles/tasks/list/TaskStrip.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import type { TaskCard } from '@/types/dtos'

interface ColumnData {
  key: string
  title: string
  dueDate: TaskCard[]
  recurrence: TaskCard[]
  scheduled: TaskCard[]
}

defineProps<{
  columns: ColumnData[]
}>()

const emit = defineEmits<{
  completing: [taskId: string]
}>()

// 横向拖拽滚动
const scrollContainer = ref<HTMLElement | null>(null)
const isDragging = ref(false)
const startX = ref(0)
const scrollLeft = ref(0)

function handleMouseDown(e: MouseEvent) {
  if (!scrollContainer.value) return
  isDragging.value = true
  startX.value = e.pageX - scrollContainer.value.offsetLeft
  scrollLeft.value = scrollContainer.value.scrollLeft
  scrollContainer.value.style.cursor = 'grabbing'
}

function handleMouseMove(e: MouseEvent) {
  if (!isDragging.value || !scrollContainer.value) return
  e.preventDefault()
  const x = e.pageX - scrollContainer.value.offsetLeft
  const walk = (x - startX.value) * 1.5
  scrollContainer.value.scrollLeft = scrollLeft.value - walk
}

function handleMouseUp() {
  if (!scrollContainer.value) return
  isDragging.value = false
  scrollContainer.value.style.cursor = 'grab'
}

function handleMouseLeave() {
  if (!scrollContainer.value) return
  isDragging.value = false
  scrollContainer.value.style.cursor = 'grab'
}

function handleTaskCompleting(taskId: string) {
  emit('completing', taskId)
}

function getColumnTotalCount(column: ColumnData): number {
  return column.dueDate.length + column.recurrence.length + column.scheduled.length
}
</script>

<template>
  <div
    ref="scrollContainer"
    class="columns-container"
    @mousedown="handleMouseDown"
    @mousemove="handleMouseMove"
    @mouseup="handleMouseUp"
    @mouseleave="handleMouseLeave"
  >
    <!-- 时间范围栏 -->
    <div v-for="column in columns" :key="column.key" class="time-column-wrapper">
      <div class="time-column">
        <!-- 内容容器（限制宽度） -->
        <div class="content-container">
          <!-- 头部 -->
          <div class="column-header">
            <div class="header-title-row">
              <h3 class="column-title">{{ column.title }}</h3>
              <span class="column-count">{{ getColumnTotalCount(column) }}</span>
            </div>
          </div>

          <!-- 任务列表区域 -->
          <div class="tasks-area">
            <!-- 截止日期任务组 -->
            <div v-if="column.dueDate.length > 0" class="task-section">
              <div class="section-header">
                <span>{{ $t('upcoming.taskType.dueDate') }}</span>
                <span class="section-count">{{ column.dueDate.length }}</span>
              </div>
              <TransitionGroup name="task-list" tag="div" class="section-tasks">
                <TaskStrip
                  v-for="task in column.dueDate"
                  :key="task.id"
                  :task="task"
                  :view-key="`upcoming::${column.key}::dueDate`"
                  display-mode="simple"
                  @completing="handleTaskCompleting"
                />
              </TransitionGroup>
            </div>

            <!-- 循环任务组 -->
            <div v-if="column.recurrence.length > 0" class="task-section">
              <div class="section-header">
                <span>{{ $t('upcoming.taskType.recurrence') }}</span>
                <span class="section-count">{{ column.recurrence.length }}</span>
              </div>
              <TransitionGroup name="task-list" tag="div" class="section-tasks">
                <TaskStrip
                  v-for="task in column.recurrence"
                  :key="task.id"
                  :task="task"
                  :view-key="`upcoming::${column.key}::recurrence`"
                  display-mode="simple"
                  @completing="handleTaskCompleting"
                />
              </TransitionGroup>
            </div>

            <!-- 排期任务组 -->
            <div v-if="column.scheduled.length > 0" class="task-section">
              <div class="section-header">
                <span>{{ $t('upcoming.taskType.scheduled') }}</span>
                <span class="section-count">{{ column.scheduled.length }}</span>
              </div>
              <TransitionGroup name="task-list" tag="div" class="section-tasks">
                <TaskStrip
                  v-for="task in column.scheduled"
                  :key="task.id"
                  :task="task"
                  :view-key="`upcoming::${column.key}::scheduled`"
                  display-mode="simple"
                  @completing="handleTaskCompleting"
                />
              </TransitionGroup>
            </div>

            <!-- 空状态 -->
            <div v-if="getColumnTotalCount(column) === 0" class="empty-state">
              <CuteIcon name="Check" :size="40" />
              <p>{{ $t('task.label.noTasks') }}</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ==================== 6栏容器 ==================== */
.columns-container {
  display: flex;
  padding: 2rem;
  overflow: auto hidden;
  height: 100%;
  cursor: grab;
  user-select: none;
}

.columns-container::-webkit-scrollbar {
  height: 0.8rem;
}

.columns-container::-webkit-scrollbar-track {
  background: var(--color-background-secondary);
}

.columns-container::-webkit-scrollbar-thumb {
  background: var(--color-border-default);
  border-radius: 0.4rem;
}

.columns-container::-webkit-scrollbar-thumb:hover {
  background: var(--color-border-strong);
}

/* ==================== 时间栏外包装 ==================== */
.time-column-wrapper {
  flex: 0 0 auto;
  padding-left: 1.6rem;
  padding-right: 1.6rem;
  border-right: 2px dashed var(--color-border-default, #f0f);
}

.time-column-wrapper:first-child {
  padding-left: 0;
}

.time-column-wrapper:last-child {
  padding-right: 0;
  border-right: none;
}

/* ==================== 时间栏 ==================== */
.time-column {
  width: 32rem;
  min-width: 32rem;
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

/* 内容容器 - 限制宽度并居中 */
.content-container {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

/* 头部 */
.column-header {
  flex-shrink: 0;
  padding: 2rem 1.6rem 1.6rem;
}

.header-title-row {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.column-title {
  margin: 0;
  font-size: 1.8rem;
  font-weight: 700;
  color: var(--color-text-primary, #f0f);
  line-height: 1.4;
}

.column-count {
  font-size: 1.3rem;
  color: var(--color-text-tertiary, #f0f);
  background-color: var(--color-background-hover, #f0f);
  padding: 0.4rem 0.8rem;
  border-radius: 0.4rem;
}

/* 任务列表区域 */
.tasks-area {
  flex: 1;
  overflow-y: auto;
  padding: 0;
}

/* 任务分组 */
.task-section {
  margin-bottom: 2.4rem;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.8rem 0;
  margin: 0 1.6rem 1rem;
  font-size: 1.4rem;
  font-weight: 600;
  color: var(--color-text-accent, #f0f);
  line-height: 1.4;
  border-bottom: 2px solid var(--color-border-light, #f0f);
}

.section-count {
  font-size: 1.1rem;
  color: var(--color-text-tertiary, #f0f);
}

.section-tasks {
  display: flex;
  flex-direction: column;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 4rem 2rem;
  color: var(--color-text-tertiary, #f0f);
  gap: 1rem;
}

.empty-state p {
  margin: 0;
  font-size: 1.4rem;
}

/* ==================== 过渡动画 ==================== */
.task-list-move {
  transition: transform 0.3s ease;
}

.task-list-enter-active {
  transition: all 0.3s ease;
}

.task-list-leave-active {
  transition: all 0.3s ease;
}

.task-list-enter-from {
  opacity: 0;
  transform: translateY(-0.5rem);
}

.task-list-leave-to {
  opacity: 0;
  transform: scale(0.95);
  max-height: 0;
  overflow: hidden;
}
</style>
