<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useTimeBlockStore } from '@/stores/timeblock'
import { useAreaStore } from '@/stores/area'
import CuteIcon from './CuteIcon.vue'

const props = defineProps<{
  timeBlockId: string | null
}>()

const emit = defineEmits<{
  close: []
}>()

const timeBlockStore = useTimeBlockStore()
const areaStore = useAreaStore()

// 获取当前时间块
const timeBlock = computed(() => {
  if (!props.timeBlockId) return null
  return timeBlockStore.getTimeBlockById(props.timeBlockId)
})

// 获取链接的任务
const linkedTasks = computed(() => {
  if (!timeBlock.value) return []
  return timeBlock.value.linked_tasks || []
})

// 格式化时间
function formatTime(isoString: string) {
  const date = new Date(isoString)
  const hours = date.getHours().toString().padStart(2, '0')
  const minutes = date.getMinutes().toString().padStart(2, '0')
  return `${hours}:${minutes}`
}

// 格式化日期
function formatDate(isoString: string) {
  const date = new Date(isoString)
  const month = (date.getMonth() + 1).toString().padStart(2, '0')
  const day = date.getDate().toString().padStart(2, '0')
  return `${month}/${day}`
}
</script>

<template>
  <div v-if="timeBlock" class="time-block-detail-panel">
    <div class="panel-header">
      <h3>时间块详情</h3>
      <button class="close-btn" @click="emit('close')">
        <CuteIcon name="X" :size="16" />
      </button>
    </div>

    <div class="panel-content">
      <!-- 时间块基本信息 -->
      <div class="time-block-info">
        <div class="info-row">
          <CuteIcon name="Clock" :size="16" />
          <span class="time-range">
            {{ formatTime(timeBlock.start_time) }} - {{ formatTime(timeBlock.end_time) }}
          </span>
        </div>
        <div v-if="timeBlock.title" class="info-row">
          <CuteIcon name="Text" :size="16" />
          <span class="block-title">{{ timeBlock.title }}</span>
        </div>
      </div>

      <!-- 链接的任务列表 -->
      <div class="linked-tasks-section">
        <div class="section-header">
          <CuteIcon name="Link" :size="16" />
          <span>链接的任务</span>
          <span class="task-count">{{ linkedTasks.length }}</span>
        </div>

        <div v-if="linkedTasks.length === 0" class="empty-state">
          <p>暂无链接任务</p>
        </div>

        <div v-else class="tasks-list">
          <div v-for="task in linkedTasks" :key="task.id" class="task-item">
            <div class="task-title">{{ task.title }}</div>
            <div v-if="task.area_id" class="task-area">
              <span
                class="area-dot"
                :style="{ backgroundColor: areaStore.getAreaById(task.area_id)?.color || '#ccc' }"
              ></span>
              <span class="area-name">{{ areaStore.getAreaById(task.area_id)?.name }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.time-block-detail-panel {
  position: fixed;
  left: 1rem;
  top: 50%;
  transform: translateY(-50%);
  width: 28rem;
  max-height: 80vh;
  background-color: var(--color-background-content);
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
  box-shadow: 0 4px 16px rgb(0 0 0 / 15%);
  display: flex;
  flex-direction: column;
  z-index: 1000;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.5rem;
  border-bottom: 1px solid var(--color-border-default);
}

.panel-header h3 {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.close-btn {
  padding: 0.4rem;
  background: none;
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  color: var(--color-text-tertiary);
  transition: all 0.2s ease;
}

.close-btn:hover {
  background-color: var(--color-background-hover);
  color: var(--color-text-primary);
}

.panel-content {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.time-block-info {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

.info-row {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  color: var(--color-text-secondary);
  font-size: 1.4rem;
}

.time-range,
.block-title {
  color: var(--color-text-primary);
  font-weight: 500;
}

.linked-tasks-section {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  font-size: 1.4rem;
  font-weight: 600;
  color: var(--color-text-secondary);
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
  color: var(--color-text-tertiary);
  background-color: var(--color-background-hover);
  border-radius: 1rem;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
}

.empty-state p {
  font-size: 1.3rem;
  color: var(--color-text-tertiary);
}

.tasks-list {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

.task-item {
  padding: 1rem;
  background-color: var(--color-background);
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  transition: all 0.2s ease;
  cursor: pointer;
}

.task-item:hover {
  background-color: var(--color-background-hover);
  border-color: var(--color-border-hover);
}

.task-title {
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary);
}

.task-area {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  font-size: 1.2rem;
  color: var(--color-text-secondary);
}

.area-dot {
  width: 0.8rem;
  height: 0.8rem;
  border-radius: 50%;
}

.area-name {
  font-weight: 500;
}
</style>
