<template>
  <div class="task-bar" :class="{ collapsed: isCollapsed }">
    <!-- 标题栏（可点击折叠） -->
    <div class="task-bar-header" @click="toggleCollapse">
      <div class="header-left">
        <CuteIcon
          name="ChevronDown"
          :size="16"
          class="collapse-icon"
          :class="{ rotated: isCollapsed }"
        />
        <h3 class="task-bar-title">{{ title }}</h3>
        <span class="task-count">{{ tasks.length }}</span>
      </div>
    </div>

    <!-- 内容区（可折叠） -->
    <div v-if="!isCollapsed" class="task-bar-content">
      <!-- 任务输入框 -->
      <div class="task-input-wrapper">
        <input
          v-model="newTaskTitle"
          type="text"
          class="task-input"
          placeholder="添加新任务..."
          @keydown.enter="addTask"
        />
        <button v-if="newTaskTitle" class="add-task-btn" @click="addTask">
          <CuteIcon name="Plus" :size="16" />
        </button>
      </div>

      <!-- 任务纸条列表 -->
      <div class="task-list">
        <TaskStrip
          v-for="task in tasks"
          :key="task.id"
          :title="task.title"
          :note="task.note"
          :subtasks="task.subtasks"
          :completed="task.completed"
          @toggle-complete="toggleTaskComplete(task.id)"
          @toggle-subtask="(subtaskId) => toggleSubtask(task.id, subtaskId)"
        />
        <div v-if="tasks.length === 0" class="empty-state">
          <p>暂无任务</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import CuteIcon from './CuteIcon.vue'
import TaskStrip from './TaskStrip.vue'

// Types
interface Subtask {
  id: string
  title: string
  completed: boolean
}

interface Task {
  id: string
  title: string
  note?: string
  subtasks?: Subtask[]
  completed: boolean
}

interface Props {
  title?: string
  tasks?: Task[]
  defaultCollapsed?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  title: '任务栏',
  tasks: () => [],
  defaultCollapsed: false,
})

// Emits
const emit = defineEmits<{
  'add-task': [title: string]
  'toggle-task': [taskId: string]
  'toggle-subtask': [taskId: string, subtaskId: string]
}>()

// State
const isCollapsed = ref(props.defaultCollapsed)
const newTaskTitle = ref('')

// Methods
function toggleCollapse() {
  isCollapsed.value = !isCollapsed.value
}

function addTask() {
  if (newTaskTitle.value.trim()) {
    emit('add-task', newTaskTitle.value.trim())
    newTaskTitle.value = ''
  }
}

function toggleTaskComplete(taskId: string) {
  emit('toggle-task', taskId)
}

function toggleSubtask(taskId: string, subtaskId: string) {
  emit('toggle-subtask', taskId, subtaskId)
}
</script>

<style scoped>
.task-bar {
  background-color: transparent;
  margin-bottom: 1.6rem;
}

/* 标题栏 */
.task-bar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 0.8rem;
  cursor: pointer;
  user-select: none;
  transition: background-color 0.2s;
  border-radius: 0.6rem;
}

.task-bar-header:hover {
  background-color: rgb(0 0 0 / 3%);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.collapse-icon {
  color: var(--color-text-secondary);
  transition: transform 0.2s ease;
}

.collapse-icon.rotated {
  transform: rotate(-90deg);
}

.task-bar-title {
  font-size: 1.6rem;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.task-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2rem;
  height: 2rem;
  padding: 0 0.6rem;
  font-size: 1.2rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  background-color: var(--color-background-secondary, #f5f5f5);
  border-radius: 1rem;
}

/* 内容区 */
.task-bar-content {
  padding: 0 0.8rem;
}

/* 任务输入框 */
.task-input-wrapper {
  position: relative;
  margin-bottom: 1.2rem;
}

.task-input {
  width: 100%;
  padding: 1rem 1.2rem;
  padding-right: 4rem;
  font-size: 1.4rem;
  color: var(--color-text-primary);
  background-color: var(--color-background-content);
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  outline: none;
  transition: all 0.2s ease;
  box-sizing: border-box;
}

.task-input::placeholder {
  color: var(--color-text-tertiary);
}

.task-input:focus {
  border-color: var(--color-primary, #4a90e2);
  box-shadow: 0 0 0 3px var(--color-primary-bg, #e3f2fd);
}

.add-task-btn {
  position: absolute;
  right: 0.6rem;
  top: 50%;
  transform: translateY(-50%);
  width: 3rem;
  height: 3rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-primary, #4a90e2);
  color: white;
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.add-task-btn:hover {
  background-color: var(--color-primary-hover, #357abd);
}

.add-task-btn:active {
  transform: translateY(-50%) scale(0.95);
}

/* 任务列表 */
.task-list {
  display: flex;
  flex-direction: column;
}

/* 空状态 */
.empty-state {
  padding: 3rem 2rem;
  text-align: center;
}

.empty-state p {
  font-size: 1.4rem;
  color: var(--color-text-tertiary);
  margin: 0;
}

/* 折叠状态 */
.task-bar.collapsed .task-bar-content {
  display: none;
}
</style>
