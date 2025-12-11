<template>
  <div class="home-staging-panel">
    <TwoRowLayout>
      <template #top>
        <div class="panel-controls">
          <!-- 左侧：标题 -->
          <div class="controls-left">
            <div class="panel-title-wrapper">
              <span class="panel-title-text">{{ $t('task.label.scheduled') }}</span>
            </div>
          </div>

          <!-- 中间：占位 -->
          <div class="spacer"></div>

          <!-- 右侧控制组（预留） -->
          <div class="controls-right"></div>
        </div>
      </template>

      <template #bottom>
        <div class="staging-groups-container">
          <!-- 最近结转任务列表 -->
          <TaskList
            v-if="recentCarryoverTasks.length > 0"
            :title="$t('task.label.recentCarryover')"
            view-key="misc::staging::recent-carryover"
            :show-add-input="false"
            :show-dashed-divider="true"
            :collapsible="true"
            :default-collapsed="false"
          />

          <!-- 无项目任务列表（始终显示，作为默认添加入口） -->
          <TaskList
            :title="$t('project.title.noProject')"
            view-key="misc::staging::no-project"
            :show-add-input="true"
            :collapsible="false"
            :hide-header="false"
          />

          <!-- 各项目任务列表 -->
          <div
            v-for="group in projectGroups"
            :key="group.projectId"
            class="project-task-group"
          >
            <TaskList
              :title="group.projectName"
              :view-key="`misc::staging::project::${group.projectId}`"
              :show-add-input="false"
              :collapsible="true"
              :default-collapsed="false"
            />
          </div>
        </div>
      </template>
    </TwoRowLayout>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import TaskList from '@/components/assembles/tasks/list/TaskList.vue'
import { useTaskStore } from '@/stores/task'
import { useProjectStore } from '@/stores/project'

const taskStore = useTaskStore()
const projectStore = useProjectStore()

// 最近结转的 staging 任务（过去5天内有排期）
const recentCarryoverTasks = computed(() => {
  return taskStore.getTasksByViewKey_Mux('misc::staging::recent-carryover')
})

// 无项目的 staging 任务
const noProjectTasks = computed(() => {
  return taskStore.stagingTasks.filter((task) => !task.project_id)
})

// 按项目分组的 staging 任务
const projectGroups = computed(() => {
  const stagingTasks = taskStore.stagingTasks
  const projectTasksMap = new Map<string, number>()

  // 统计每个项目的 staging 任务数量
  for (const task of stagingTasks) {
    if (task.project_id) {
      const count = projectTasksMap.get(task.project_id) || 0
      projectTasksMap.set(task.project_id, count + 1)
    }
  }

  // 转换为数组并获取项目名称
  const groups: Array<{ projectId: string; projectName: string; taskCount: number }> = []
  for (const [projectId, taskCount] of projectTasksMap) {
    const project = projectStore.getProjectById(projectId)
    groups.push({
      projectId,
      projectName: project?.name || '未知项目',
      taskCount,
    })
  }

  // 按项目名称排序
  return groups.sort((a, b) => a.projectName.localeCompare(b.projectName))
})
</script>

<style scoped>
.home-staging-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ==================== 控制栏 ==================== */
.panel-controls {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1.2rem;
  padding: 1.2rem 0.8rem 1.2rem 1.6rem;
  background-color: transparent;
}

.controls-left {
  position: relative;
  display: flex;
  align-items: center;
  gap: 1.2rem;
}

.controls-right {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

/* ==================== 标题样式 ==================== */
.panel-title-wrapper {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.panel-title-text {
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
  line-height: 1.4;
  white-space: nowrap;
}

/* 占位 */
.spacer {
  flex: 1;
}

/* ==================== 分组容器 ==================== */
.staging-groups-container {
  display: flex;
  flex-direction: column;
  gap: 0;
  overflow-y: auto;
  height: 100%;
  padding: 1rem;
}

.project-task-group {
  /* 项目分组之间无额外间距，由 TaskList 自身控制 */
}
</style>
