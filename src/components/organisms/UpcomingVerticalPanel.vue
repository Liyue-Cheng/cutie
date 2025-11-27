<!--
  UpcomingVerticalPanel.vue - 即将到来的任务竖排视图面板

  功能：
  - 类似 project-detail 的竖排布局
  - 按时间范围（逾期、今日、本周、下周、本月、更远）展示任务
  - 任务分类（截止日期、循环任务、一般排期）
-->
<template>
  <div class="upcoming-vertical-panel">
    <!-- 内容容器（限制宽度） -->
    <div class="content-container">
      <!-- 头部 -->
      <div class="panel-header">
        <div class="header-title-row">
          <h1 class="panel-title">Upcoming</h1>
          <span class="task-count">{{ totalTaskCount }}</span>
        </div>
      </div>

      <!-- 任务列表区域 -->
      <div class="tasks-area">
        <!-- 各时间范围的任务 section -->
        <template v-for="section in sectionsData" :key="section.key">
          <div v-if="section.totalCount > 0" class="task-section">
            <div class="section-header">
              <span>{{ section.title }}</span>
              <span class="section-count">{{ section.totalCount }}</span>
            </div>
            <div class="section-tasks">
              <TaskStrip
                v-for="task in section.tasks"
                :key="task.id"
                :task="task"
                :view-key="`upcoming-vertical::${section.key}`"
                display-mode="simple"
                @completing="handleTaskCompleting"
              />
            </div>
          </div>
        </template>

        <!-- 空状态 -->
        <div v-if="totalTaskCount === 0" class="empty-state">
          <CuteIcon name="Check" :size="48" />
          <p>暂无即将到来的任务</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import TaskStrip from '@/components/assembles/tasks/list/TaskStrip.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import { useUpcomingTasks } from '@/composables/useUpcomingTasks'

// 使用共享的 upcoming 任务逻辑
const { initialize, handleTaskCompleting, totalTaskCount, sectionsData } = useUpcomingTasks()

// 初始化
onMounted(() => {
  initialize()
})
</script>

<style scoped>
.upcoming-vertical-panel {
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
  max-width: 60rem;
  height: 100%;
  margin: 0 auto;
  overflow: hidden;
}

/* 头部 */
.panel-header {
  flex-shrink: 0;
  padding: 4rem 1.6rem;
}

.header-title-row {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.panel-title {
  flex: 1;
  font-size: 2.2rem;
  font-weight: 700;
  color: var(--color-text-primary, #f0f);
  margin: 0;
  line-height: 1.4;
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
  color: var(--color-text-secondary, #f0f);
  background-color: var(--color-background-secondary, #f0f);
  border-radius: 1.2rem;
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
  gap: 1.2rem;
}

.empty-state p {
  margin: 0;
  font-size: 1.4rem;
}
</style>
