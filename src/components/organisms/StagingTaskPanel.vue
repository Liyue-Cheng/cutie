<template>
  <div class="staging-view">
    <TwoRowLayout>
      <template #top>
        <div class="staging-header">
          <!-- 筛选按钮 -->
          <button class="filter-btn" @click="handleFilter">
            <CuteIcon name="ListFilter" :size="16" />
            <span>筛选</span>
          </button>
        </div>
      </template>
      <template #bottom>
        <div class="task-list">
          <!-- Staging 任务栏 -->
          <TaskList title="待安排任务" view-key="misc::staging" fill-remaining-space />
        </div>
      </template>
    </TwoRowLayout>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import TaskList from '@/components/assembles/tasks/list/TaskList.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import { useTaskStore } from '@/stores/task'
import { logger, LogTags } from '@/infra/logging/logger'

const taskStore = useTaskStore()

// 获取 staging 任务列表
const tasks = computed(() => {
  return taskStore.getTasksByViewKey_Mux('misc::staging')
})

// 筛选功能（暂未实现）
function handleFilter() {
  logger.debug(LogTags.VIEW_HOME, 'Filter button clicked (not implemented yet)')
}

onMounted(async () => {
  logger.info(LogTags.VIEW_HOME, 'Initializing StagingTaskPanel component...')
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
  padding: 0 1.6rem;
}

/* 筛选按钮 */
.filter-btn {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  height: 3.6rem;
  padding: 0 1.2rem;
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary);
  background-color: var(--color-background-secondary, #f0f);
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.filter-btn:hover {
  background-color: var(--color-background-hover, #f0f);
  border-color: var(--color-border-hover);
}

.filter-btn:active {
  transform: scale(0.98);
}

/* 任务列表 */
.task-list {
  height: 100%;
  overflow-y: auto;
  padding: 0;
  display: flex;
  flex-direction: column;
}

/* 最后一个TaskList延展到底部，避免拖动到底部空白区域时闪烁 */
.task-list > :deep(:last-child) {
  flex: 1;
  min-height: auto;
}
</style>
