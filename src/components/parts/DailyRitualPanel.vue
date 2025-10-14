<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import CuteCheckbox from '@/components/parts/CuteCheckbox.vue'
import { getTodayDateString } from '@/infra/utils/dateUtils'

interface RitualItem {
  id: string
  title: string
  order: number
}

interface DailyRitualState {
  date: string
  completedIds: string[]
}

// ==================== 本地存储键 ====================
const RITUALS_STORAGE_KEY = 'daily-rituals-items'
const STATE_STORAGE_KEY = 'daily-rituals-state'

// ==================== 状态 ====================
const today = ref(getTodayDateString())
const rituals = ref<RitualItem[]>([])
const completedIds = ref<Set<string>>(new Set())
const newRitualTitle = ref('')

// ==================== 计算属性 ====================
const sortedRituals = computed(() => {
  return [...rituals.value].sort((a, b) => a.order - b.order)
})

const completedCount = computed(() => completedIds.value.size)
const totalCount = computed(() => rituals.value.length)

// ==================== 初始化 ====================
onMounted(() => {
  loadRituals()
  loadState()
  checkDateReset()
})

// 监听日期变化（如果跨天，重置完成状态）
watch(today, () => {
  checkDateReset()
})

// ==================== 数据持久化 ====================
function loadRituals() {
  const stored = localStorage.getItem(RITUALS_STORAGE_KEY)
  if (stored) {
    rituals.value = JSON.parse(stored)
  }
}

function saveRituals() {
  localStorage.setItem(RITUALS_STORAGE_KEY, JSON.stringify(rituals.value))
}

function loadState() {
  const stored = localStorage.getItem(STATE_STORAGE_KEY)
  if (stored) {
    const state = JSON.parse(stored) as { date: string; completedIds: string[] }
    if (state.date === today.value) {
      completedIds.value = new Set(state.completedIds)
    } else {
      // 日期不同，重置
      completedIds.value = new Set()
      saveState()
    }
  }
}

function saveState() {
  const state: DailyRitualState = {
    date: today.value,
    completedIds: Array.from(completedIds.value),
  }
  localStorage.setItem(STATE_STORAGE_KEY, JSON.stringify(state))
}

function checkDateReset() {
  const stored = localStorage.getItem(STATE_STORAGE_KEY)
  if (stored) {
    const state: DailyRitualState = JSON.parse(stored)
    if (state.date !== today.value) {
      // 新的一天，重置完成状态
      completedIds.value = new Set()
      saveState()
    }
  }
}

// ==================== 操作 ====================
function handleToggle(ritualId: string) {
  if (completedIds.value.has(ritualId)) {
    completedIds.value.delete(ritualId)
  } else {
    completedIds.value.add(ritualId)
  }
  saveState()
}

function handleAddRitual() {
  const title = newRitualTitle.value.trim()
  if (!title) return

  const newRitual: RitualItem = {
    id: crypto.randomUUID(),
    title,
    order: rituals.value.length,
  }

  rituals.value.push(newRitual)
  saveRituals()
  newRitualTitle.value = ''
}

function handleDeleteRitual(ritualId: string) {
  rituals.value = rituals.value.filter((r) => r.id !== ritualId)
  completedIds.value.delete(ritualId)
  saveRituals()
  saveState()
}

// ==================== 拖拽排序 ====================
const draggingId = ref<string | null>(null)

function handleDragStart(ritualId: string) {
  draggingId.value = ritualId
}

function handleDragEnd() {
  draggingId.value = null
}

function handleDragOver(event: DragEvent) {
  event.preventDefault()
}

function handleDrop(event: DragEvent, targetId: string) {
  event.preventDefault()
  if (!draggingId.value || draggingId.value === targetId) return

  const items = [...rituals.value]
  const dragIndex = items.findIndex((r) => r.id === draggingId.value)
  const dropIndex = items.findIndex((r) => r.id === targetId)

  if (dragIndex === -1 || dropIndex === -1) return

  const [draggedItem] = items.splice(dragIndex, 1)
  if (!draggedItem) return

  items.splice(dropIndex, 0, draggedItem)

  // 更新 order
  items.forEach((item, index) => {
    item.order = index
  })

  rituals.value = items
  saveRituals()
  draggingId.value = null
}
</script>

<template>
  <div class="daily-ritual-panel">
    <div class="panel-header">
      <h3 class="panel-title">Daily Rituals</h3>
      <div class="progress-indicator">
        <span class="progress-text">{{ completedCount }}/{{ totalCount }}</span>
      </div>
    </div>

    <div class="rituals-list">
      <div
        v-for="ritual in sortedRituals"
        :key="ritual.id"
        class="ritual-item"
        :class="{ completed: completedIds.has(ritual.id), dragging: draggingId === ritual.id }"
        draggable="true"
        @dragstart="handleDragStart(ritual.id)"
        @dragend="handleDragEnd"
        @dragover="handleDragOver"
        @drop="handleDrop($event, ritual.id)"
      >
        <div class="drag-handle">⋮⋮</div>
        <CuteCheckbox
          :checked="completedIds.has(ritual.id)"
          size="small"
          @update:checked="handleToggle(ritual.id)"
        />
        <span class="ritual-title">{{ ritual.title }}</span>
        <button class="delete-button" @click="handleDeleteRitual(ritual.id)">×</button>
      </div>
    </div>

    <div class="add-ritual-form">
      <input
        v-model="newRitualTitle"
        type="text"
        class="add-ritual-input"
        placeholder="+ Add ritual"
        @keydown.enter="handleAddRitual"
      />
    </div>

    <div class="panel-footer">
      <span class="reset-info">Resets daily at midnight</span>
    </div>
  </div>
</template>

<style scoped>
.daily-ritual-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--color-background-content);
  padding: 1.5rem;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1.5rem;
}

.panel-title {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.progress-indicator {
  padding: 0.4rem 0.8rem;
  background-color: var(--color-primary-bg, #e8f4ff);
  border-radius: 1.2rem;
}

.progress-text {
  font-size: 1.3rem;
  font-weight: 600;
  color: var(--color-primary, #4a90e2);
}

.rituals-list {
  flex: 1;
  overflow-y: auto;
  margin-bottom: 1rem;
}

.ritual-item {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  padding: 0.8rem;
  border-radius: 0.6rem;
  transition: background-color 0.2s;
  cursor: move;
}

.ritual-item:hover {
  background-color: var(--color-background-soft, #f9f9f9);
}

.ritual-item.dragging {
  opacity: 0.5;
}

.drag-handle {
  cursor: grab;
  color: var(--color-text-tertiary);
  font-size: 1.2rem;
  line-height: 1;
  user-select: none;
}

.drag-handle:active {
  cursor: grabbing;
}

.ritual-title {
  flex: 1;
  font-size: 1.4rem;
  color: var(--color-text-primary);
}

.ritual-item.completed .ritual-title {
  text-decoration: line-through;
  color: var(--color-text-tertiary);
}

.delete-button {
  font-size: 2rem;
  line-height: 1;
  color: var(--color-text-tertiary);
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition:
    opacity 0.2s,
    color 0.2s;
}

.delete-button:hover {
  color: var(--color-danger, #ff4d4f);
}

.ritual-item:hover .delete-button {
  opacity: 1;
}

.add-ritual-form {
  margin-bottom: 1rem;
}

.add-ritual-input {
  width: 100%;
  padding: 0.8rem;
  border: 1px dashed var(--color-border-default);
  border-radius: 0.6rem;
  background-color: transparent;
  color: var(--color-text-primary);
  font-size: 1.4rem;
  transition: all 0.2s;
}

.add-ritual-input:focus {
  outline: none;
  border-style: solid;
  border-color: var(--color-primary);
  background-color: var(--color-background-soft, #f9f9f9);
}

.add-ritual-input::placeholder {
  color: var(--color-text-tertiary);
}

.panel-footer {
  padding-top: 1rem;
  border-top: 1px solid var(--color-border-default);
  text-align: center;
}

.reset-info {
  font-size: 1.2rem;
  color: var(--color-text-tertiary);
  font-style: italic;
}
</style>
