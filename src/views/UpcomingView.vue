<!--
  UpcomingView.vue - 即将到来的任务视图（横向布局）

  功能：
  - 按时间范围（逾期、今日、本周、下周、本月、更远）展示任务
  - 任务分类（截止日期、循环任务、一般排期）
  - 支持任务拖拽、完成、编辑等操作

  数据加载：
  - onMounted 时加载所有未完成任务 (fetchAllIncompleteTasks_DMA)
  - 后端会自动实例化循环任务的未来实例
-->
<template>
  <div class="upcoming-view">
    <TwoRowLayout>
      <!-- 上栏：标题栏 -->
      <template #top>
        <div class="upcoming-header">
          <h2 class="header-title">{{ $t('upcoming.title.horizontal') }}</h2>
          <span class="task-count">{{ totalTaskCount }}</span>
        </div>
      </template>

      <!-- 下栏：6栏布局 -->
      <template #bottom>
        <UpcomingSixColumnLayout :columns="columnsData" @completing="handleTaskCompleting" />
      </template>
    </TwoRowLayout>

    <!-- 任务编辑器模态框 -->
    <TaskEditorModal
      v-if="uiStore.isEditorOpen"
      :task-id="uiStore.editorTaskId"
      :view-key="uiStore.editorViewKey ?? undefined"
      @close="uiStore.closeEditor"
    />
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useUIStore } from '@/stores/ui'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import UpcomingSixColumnLayout from '@/components/assembles/upcoming/UpcomingSixColumnLayout.vue'
import TaskEditorModal from '@/components/assembles/tasks/TaskEditorModal.vue'
import { useUpcomingTasks } from '@/composables/useUpcomingTasks'

const uiStore = useUIStore()

// 使用共享的 upcoming 任务逻辑
const { initialize, handleTaskCompleting, totalTaskCount, columnsData } = useUpcomingTasks()

// 初始化
onMounted(() => {
  initialize()
})
</script>

<style scoped>
.upcoming-view {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: var(--color-background-content);
  overflow: hidden;
}

/* 顶部标题栏 */
.upcoming-header {
  display: flex;
  align-items: center;
  gap: 1.2rem;
  padding: 1.2rem 2rem;
  background-color: var(--color-background-content);
  flex-shrink: 0;
}

.header-title {
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.task-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2.4rem;
  height: 2.4rem;
  padding: 0 0.8rem;
  font-size: 1.3rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  background-color: var(--color-background-secondary);
  border-radius: 1.2rem;
}
</style>
