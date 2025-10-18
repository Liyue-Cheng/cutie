<!--
  新拖放系统测试页面
  
  使用真实的 SimpleKanbanColumn 组件和数据
  测试 interact.js 拖放系统
-->

<template>
  <div class="interact-test-view">
    <div class="test-header">
      <h1>🧪 新拖放系统测试</h1>
      <p>基于 interact.js 的拖放系统，使用真实的 SimpleKanbanColumn 组件</p>
    </div>

    <div class="test-layout">
      <!-- 左侧三个看板 -->
      <div class="kanban-section">
        <!-- Staging 看板 -->
        <InteractKanbanColumn
          view-key="misc::staging"
          title="📥 Staging"
          subtitle="待安排任务"
          :show-add-input="true"
        />

        <!-- Today 看板 -->
        <InteractKanbanColumn
          :view-key="todayViewKey"
          title="📅 Today"
          subtitle="今日任务"
          :show-add-input="true"
        />

        <!-- Tomorrow 看板 -->
        <InteractKanbanColumn
          :view-key="tomorrowViewKey"
          title="🚀 Tomorrow"
          subtitle="明日任务"
          :show-add-input="true"
        />
      </div>

      <!-- 右侧数据面板 -->
      <div class="data-panel">
        <InteractDataPanel />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, computed } from 'vue'
import InteractKanbanColumn from '@/components/test/InteractKanbanColumn.vue'
import InteractDataPanel from '@/components/test/InteractDataPanel.vue'
import { useTaskStore } from '@/stores/task'
import { logger, LogTags } from '@/infra/logging/logger'

const taskStore = useTaskStore()

// ✅ 动态计算日期，避免硬编码
const today = computed(() => {
  const date = new Date()
  return date.toISOString().split('T')[0] // YYYY-MM-DD 格式
})

const tomorrow = computed(() => {
  const date = new Date()
  date.setDate(date.getDate() + 1)
  return date.toISOString().split('T')[0] // YYYY-MM-DD 格式
})

const todayViewKey = computed(() => `daily::${today.value}`)
const tomorrowViewKey = computed(() => `daily::${tomorrow.value}`)

// ==================== 初始化 ====================
onMounted(async () => {
  logger.info(LogTags.VIEW_HOME, 'Initializing InteractTestView, loading incomplete tasks...')
  // ✅ 使用 fetchAllIncompleteTasks_DMA 替代已删除的 fetchAllTasks_DMA
  await taskStore.fetchAllIncompleteTasks_DMA()
  logger.info(LogTags.VIEW_HOME, 'Loaded tasks for InteractTestView', {
    count: taskStore.allTasks.length,
    todayViewKey: todayViewKey.value,
    tomorrowViewKey: tomorrowViewKey.value,
  })
})
</script>

<style scoped>
.interact-test-view {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
}

.test-header {
  padding: 1.5rem 2rem;
  background: var(--color-card-available);
  border-bottom: 1px solid var(--color-border-default);
  box-shadow: 0 1px 3px rgb(0 0 0 / 10%);
}

.test-header h1 {
  margin: 0 0 0.5rem;
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.test-header p {
  margin: 0;
  color: var(--color-text-secondary);
  font-size: 0.875rem;
}

.test-layout {
  flex: 1;
  display: flex;
  gap: 1rem;
  padding: 1rem;
  min-height: 0;
}

.kanban-section {
  flex: 1;
  display: flex;
  gap: 1rem;
  min-height: 0;
}

.data-panel {
  width: 320px;
  min-height: 0;
}

/* 响应式设计 */
@media (width <= 1200px) {
  .test-layout {
    flex-direction: column;
  }

  .data-panel {
    width: 100%;
    height: 300px;
  }

  .kanban-section {
    flex-direction: column;
    height: 400px;
  }
}
</style>
