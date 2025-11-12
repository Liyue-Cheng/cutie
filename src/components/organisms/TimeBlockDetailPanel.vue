<script setup lang="ts">
import { computed } from 'vue'
import { useTimeBlockStore } from '@/stores/timeblock'
import CuteIcon from '@/components/parts/CuteIcon.vue'

const props = defineProps<{
  timeBlockId: string | null
}>()

const emit = defineEmits<{
  close: []
}>()

const timeBlockStore = useTimeBlockStore()

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

// 格式化时间范围
function formatTimeRange(timeBlock: any) {
  if (timeBlock.is_all_day) {
    return '全天'
  }

  let startTime: string
  let endTime: string

  // 如果是浮动时间且有本地时间，使用本地时间
  if (
    timeBlock.time_type === 'FLOATING' &&
    timeBlock.start_time_local &&
    timeBlock.end_time_local
  ) {
    startTime = timeBlock.start_time_local.substring(0, 5) // HH:MM
    endTime = timeBlock.end_time_local.substring(0, 5) // HH:MM
  } else {
    // 否则使用UTC时间转换为本地时间显示
    const startDate = new Date(timeBlock.start_time)
    const endDate = new Date(timeBlock.end_time)
    startTime = `${startDate.getHours().toString().padStart(2, '0')}:${startDate.getMinutes().toString().padStart(2, '0')}`
    endTime = `${endDate.getHours().toString().padStart(2, '0')}:${endDate.getMinutes().toString().padStart(2, '0')}`
  }

  return `${startTime} - ${endTime}`
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
            {{ formatTimeRange(timeBlock) }}
          </span>
          <span v-if="timeBlock.time_type === 'FLOATING'" class="time-type-badge floating">
            浮动时间
          </span>
          <span v-else class="time-type-badge fixed"> 固定时间 </span>
        </div>
        <div v-if="timeBlock.title" class="info-row">
          <CuteIcon name="FileText" :size="16" />
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
            <div class="task-status" :class="{ completed: task.is_completed }">
              {{ task.is_completed ? '已完成' : '进行中' }}
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
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid #f0f0f0;
  transition: all 0.2s ease;
  cursor: pointer;
}

.task-item:hover {
  background-color: var(--color-background-hover);
}

.task-title {
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary);
}

.task-item:last-child {
  border-bottom: none;
}

.task-status {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 4px;
  background-color: #e3f2fd;
  color: #1976d2;
}

.task-status.completed {
  background-color: #e8f5e8;
  color: #2e7d32;
}

.time-type-badge {
  font-size: 10px;
  padding: 2px 6px;
  border-radius: 4px;
  margin-left: 8px;
  font-weight: 500;
}

.time-type-badge.floating {
  background-color: #e0f2fe;
  color: #0277bd;
}

.time-type-badge.fixed {
  background-color: #fff3e0;
  color: #f57c00;
}
</style>
