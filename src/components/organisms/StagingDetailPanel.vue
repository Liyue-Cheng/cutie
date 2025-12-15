<template>
  <div class="staging-detail-panel">
    <!-- 空状态（未选择任何分类） -->
    <div v-if="categoryId === undefined" class="empty-state">
      <CuteIcon name="Layers" :size="48" />
      <p>{{ $t('view.staging.empty.selectCategory') }}</p>
    </div>

    <!-- 最近结转视图 -->
    <div v-else-if="categoryId === 'recent-carryover'" class="staging-detail">
      <div class="content-container">
        <div class="staging-header">
          <div class="header-title-row">
            <CuteIcon name="History" :size="24" />
            <h1 class="staging-title">{{ $t('task.label.recentCarryover') }}</h1>
          </div>
          <div class="staging-description">{{ $t('view.staging.desc.recentCarryover') }}</div>
        </div>
        <div class="tasks-area">
          <div class="task-section">
            <TaskList
              :title="$t('task.label.recentCarryover')"
              view-key="misc::staging::recent-carryover"
              :show-add-input="false"
              :hide-header="true"
              fill-remaining-space
            />
          </div>
        </div>
      </div>
    </div>

    <!-- 无区域视图：按项目分组 -->
    <div v-else-if="categoryId === 'no-area'" class="staging-detail">
      <div class="content-container">
        <div class="staging-header">
          <div class="header-title-row">
            <CuteIcon name="Inbox" :size="24" />
            <h1 class="staging-title">{{ $t('task.label.noArea') }}</h1>
          </div>
          <div class="staging-description">{{ $t('view.staging.desc.noArea') }}</div>
        </div>
        <div class="tasks-area">
          <!-- 无项目任务列表 -->
          <div class="task-section">
            <TaskList
              :title="$t('project.title.noProject')"
              view-key="misc::staging::no-project"
              :show-add-input="true"
              :collapsible="false"
            />
          </div>

          <!-- 各项目任务列表（无区域） -->
          <div v-for="group in noAreaProjectGroups" :key="group.projectId" class="task-section">
            <TaskList
              :title="group.projectName"
              :view-key="`misc::staging::project::${group.projectId}`"
              :show-add-input="true"
              :collapsible="true"
              :default-collapsed="false"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- 区域视图：按项目分组 -->
    <div v-else-if="area" :key="categoryId" class="staging-detail">
      <div class="content-container">
        <div class="staging-header">
          <div class="header-title-row">
            <CuteIcon
              name="Hash"
              :size="24"
              :color="area.color || 'var(--color-text-tertiary)'"
            />
            <h1 class="staging-title">{{ area.name }}</h1>
          </div>
          <div v-if="area.description" class="staging-description">{{ area.description }}</div>
        </div>
        <div class="tasks-area">
          <!-- 无项目任务列表（该区域） -->
          <div class="task-section">
            <TaskList
              :title="$t('project.title.noProject')"
              :view-key="`misc::staging::${categoryId}::no-project`"
              :show-add-input="true"
              :collapsible="false"
            />
          </div>

          <!-- 各项目任务列表（该区域） -->
          <div v-for="group in areaProjectGroups" :key="group.projectId" class="task-section">
            <TaskList
              :title="group.projectName"
              :view-key="`misc::staging::${categoryId}::project::${group.projectId}`"
              :show-add-input="true"
              :collapsible="true"
              :default-collapsed="false"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- 分类不存在 -->
    <div v-else class="empty-state">
      <p>{{ $t('view.staging.empty.notExist') }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useAreaStore } from '@/stores/area'
import { useTaskStore } from '@/stores/task'
import { useProjectStore } from '@/stores/project'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import TaskList from '@/components/assembles/tasks/list/TaskList.vue'

interface Props {
  categoryId?: string | null
}

const props = defineProps<Props>()

const areaStore = useAreaStore()
const taskStore = useTaskStore()
const projectStore = useProjectStore()

// 获取当前区域（如果选择的是区域）
const area = computed(() => {
  if (
    !props.categoryId ||
    props.categoryId === 'recent-carryover' ||
    props.categoryId === 'no-area'
  ) {
    return null
  }
  return areaStore.getAreaById(props.categoryId)
})

// 无区域的 staging 任务按项目分组
const noAreaProjectGroups = computed(() => {
  const stagingTasks = taskStore.stagingTasks
  const projectTasksMap = new Map<string, number>()

  // 统计无区域任务中每个项目的任务数量
  for (const task of stagingTasks) {
    if (!task.area_id && task.project_id) {
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

// 指定区域的 staging 任务按项目分组
const areaProjectGroups = computed(() => {
  if (
    !props.categoryId ||
    props.categoryId === 'recent-carryover' ||
    props.categoryId === 'no-area'
  ) {
    return []
  }

  const stagingTasks = taskStore.stagingTasks
  const projectTasksMap = new Map<string, number>()

  // 统计该区域任务中每个项目的任务数量
  for (const task of stagingTasks) {
    if (task.area_id === props.categoryId && task.project_id) {
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
.staging-detail-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-background-content, #f0f);
  overflow: hidden;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--color-text-secondary, #f0f);
  gap: 1.2rem;
}

.staging-detail {
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

/* 头部样式 */
.staging-header {
  flex-shrink: 0;
  padding: 7rem 1.6rem 4rem;
}

.header-title-row {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.staging-title {
  flex: 1;
  font-size: 2.2rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
  margin: 0;
  line-height: 1.4;
}

.staging-description {
  font-size: 1.4rem;
  color: var(--color-text-secondary, #f0f);
  margin-top: 0.8rem;
  line-height: 1.6;
  white-space: pre-wrap;
}

.tasks-area {
  flex: 1;
  overflow-y: auto;
  padding: 0 0 1rem;
}

.task-section {
  margin-bottom: 0.5rem;
}
</style>
