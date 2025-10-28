<template>
  <div class="staging-view">
    <TwoRowLayout>
      <template #top>
        <div class="staging-header">
          <h2 class="staging-title">Staging</h2>
          <span class="task-count">{{ tasks.length }} 个任务</span>
        </div>
      </template>
      <template #bottom>
        <div class="task-list">
          <!-- Staging 任务栏 -->
          <TaskBar title="待安排任务" view-key="misc::staging" />
        </div>
      </template>
    </TwoRowLayout>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import TaskBar from '@/components/parts/TaskBar.vue'
import { useTaskStore } from '@/stores/task'
import { logger, LogTags } from '@/infra/logging/logger'

const taskStore = useTaskStore()

// 获取 staging 任务列表
const tasks = computed(() => {
  return taskStore.getTasksByViewKey_Mux('misc::staging')
})

onMounted(async () => {
  logger.info(LogTags.VIEW_HOME, 'Initializing StagingView component...')
  // 加载 staging 任务
  await taskStore.fetchAllIncompleteTasks_DMA()
})
</script>

<style scoped>
.staging-view {
  height: 100%;
  width: 100%;
  display: flex;
  flex-direction: column;
}

/* 头部 */
.staging-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 1.6rem;
}

.staging-title {
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.task-count {
  font-size: 1.3rem;
  color: var(--color-text-tertiary);
  font-weight: 500;
}

/* 任务列表 */
.task-list {
  height: 100%;
  overflow-y: auto;
  padding: 0;
}
</style>

