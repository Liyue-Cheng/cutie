<template>
  <div class="staging-task-groups">
    <!-- 最近结转任务列表 -->
    <TaskList
      v-if="recentCarryoverTasks.length > 0"
      :title="$t('task.label.recentCarryover')"
      :subtitle="$t('view.staging.desc.recentCarryover')"
      view-key="misc::staging::recent-carryover"
      :show-add-input="false"
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
      :fill-remaining-space="projectGroups.length === 0"
    />

    <!-- 各项目任务列表 -->
    <TaskList
      v-for="(group, index) in projectGroups"
      :key="group.projectId"
      :title="group.projectName"
      :view-key="`misc::staging::project::${group.projectId}`"
      :show-add-input="false"
      :show-dashed-divider="true"
      :collapsible="true"
      :default-collapsed="false"
      :fill-remaining-space="index === projectGroups.length - 1"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import TaskList from '@/components/assembles/tasks/list/TaskList.vue'
import { useTaskStore } from '@/stores/task'
import { useProjectStore } from '@/stores/project'

const taskStore = useTaskStore()
const projectStore = useProjectStore()

// 最近结转的 staging 任务（过去5天内有排期）
const recentCarryoverTasks = computed(() => {
  return taskStore.getTasksByViewKey_Mux('misc::staging::recent-carryover')
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
.staging-task-groups {
  display: flex;
  flex-direction: column;
  gap: 0;
  overflow-y: auto;
  height: 100%;
}
</style>

