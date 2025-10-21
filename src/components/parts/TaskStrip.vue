<template>
  <div class="task-strip">
    <!-- 顶部：完成按钮 + 标题 -->
    <div class="task-header">
      <button class="complete-btn" :class="{ completed: isCompleted }" @click="toggleComplete">
        <CuteIcon v-if="isCompleted" name="Check" :size="16" />
      </button>
      <div class="task-title" :class="{ completed: isCompleted }">
        {{ title || '新任务' }}
      </div>
    </div>

    <!-- 概览笔记 -->
    <div v-if="note" class="task-note">
      {{ note }}
    </div>

    <!-- 子任务显示区 -->
    <div v-if="subtasks && subtasks.length > 0" class="subtasks-section">
      <div v-for="subtask in subtasks" :key="subtask.id" class="subtask-item">
        <button
          class="subtask-complete-btn"
          :class="{ completed: subtask.completed }"
          @click="toggleSubtask(subtask.id)"
        >
          <CuteIcon v-if="subtask.completed" name="Check" :size="12" />
        </button>
        <span class="subtask-title" :class="{ completed: subtask.completed }">
          {{ subtask.title }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import CuteIcon from './CuteIcon.vue'

// Props
interface Subtask {
  id: string
  title: string
  completed: boolean
}

interface Props {
  title?: string
  note?: string
  subtasks?: Subtask[]
  completed?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  title: '',
  note: '',
  subtasks: () => [],
  completed: false,
})

// Emits
const emit = defineEmits<{
  'toggle-complete': []
  'toggle-subtask': [subtaskId: string]
}>()

// State
const isCompleted = ref(props.completed)

// Methods
function toggleComplete() {
  isCompleted.value = !isCompleted.value
  emit('toggle-complete')
}

function toggleSubtask(subtaskId: string) {
  emit('toggle-subtask', subtaskId)
}
</script>

<style scoped>
.task-strip {
  background-color: var(--color-background-content);
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
  padding: 1.2rem;
  margin-bottom: 0.8rem;
  transition: all 0.2s ease;
}

.task-strip:hover {
  border-color: var(--color-border-hover);
  box-shadow: 0 2px 8px rgb(0 0 0 / 8%);
}

/* 顶部：完成按钮 + 标题 */
.task-header {
  display: flex;
  align-items: flex-start;
  gap: 1rem;
  margin-bottom: 0.8rem;
}

.complete-btn {
  flex-shrink: 0;
  width: 2rem;
  height: 2rem;
  border: 2px solid var(--color-border-default);
  border-radius: 0.4rem;
  background-color: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  color: transparent;
}

.complete-btn:hover {
  border-color: var(--color-primary, #4a90e2);
  background-color: var(--color-primary-bg, #e3f2fd);
}

.complete-btn.completed {
  border-color: var(--color-primary, #4a90e2);
  background-color: var(--color-primary, #4a90e2);
  color: white;
}

.task-title {
  flex: 1;
  font-size: 1.5rem;
  font-weight: 500;
  color: var(--color-text-primary);
  line-height: 1.4;
  overflow-wrap: break-word;
  margin-top: 0.1rem;
}

.task-title.completed {
  color: var(--color-text-secondary);
  text-decoration: line-through;
}

/* 概览笔记 */
.task-note {
  font-size: 1.4rem;
  color: var(--color-text-secondary);
  line-height: 1.6;
  margin-bottom: 0.8rem;
  padding-left: 3rem;
  white-space: pre-wrap;
  overflow-wrap: break-word;
}

/* 子任务显示区 */
.subtasks-section {
  padding-left: 3rem;
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.subtask-item {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.subtask-complete-btn {
  flex-shrink: 0;
  width: 1.6rem;
  height: 1.6rem;
  border: 2px solid var(--color-border-default);
  border-radius: 0.3rem;
  background-color: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  color: transparent;
}

.subtask-complete-btn:hover {
  border-color: var(--color-primary, #4a90e2);
  background-color: var(--color-primary-bg, #e3f2fd);
}

.subtask-complete-btn.completed {
  border-color: var(--color-primary, #4a90e2);
  background-color: var(--color-primary, #4a90e2);
  color: white;
}

.subtask-title {
  font-size: 1.4rem;
  color: var(--color-text-secondary);
  line-height: 1.4;
}

.subtask-title.completed {
  color: var(--color-text-tertiary);
  text-decoration: line-through;
}
</style>
