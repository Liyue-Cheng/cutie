<template>
  <div class="project-detail-panel">
    <!-- 空状态（未选择任何项） -->
    <div v-if="projectId === undefined" class="empty-state">
      <CuteIcon name="Folder" :size="48" />
      <p>{{ $t('project.empty.selectProject') }}</p>
    </div>

    <!-- 无项目视图 -->
    <div v-else-if="projectId === null" class="project-detail">
      <!-- 内容容器（限制宽度） -->
      <div class="content-container">
        <!-- 头部 -->
        <div class="project-header">
          <div class="header-title-row">
            <CuteIcon name="Inbox" :size="24" />
            <h1 class="project-title">{{ $t('project.title.noProject') }}</h1>
          </div>
          <div class="project-description">{{ $t('project.label.unassignedToProject') }}</div>
        </div>

        <!-- 任务列表区域 -->
        <div class="tasks-area">
          <div class="task-section">
            <ProjectTaskList :title="$t('project.title.noProject')" view-key="misc::no-project" />
          </div>
        </div>
      </div>
    </div>

    <!-- 项目详情 -->
    <div v-else-if="project" class="project-detail">
      <!-- 内容容器（限制宽度） -->
      <div class="content-container">
        <!-- 项目头部 - Things 3 风格 -->
        <div class="project-header">
          <!-- 第一行：进度环 + 标题 + 三点菜单 -->
          <div class="header-title-row">
            <CircularProgress
              :completed="projectStats.completed"
              :total="projectStats.total"
              size="small"
              hide-text
            />
            <h1 class="project-title">{{ project.name }}</h1>
            <button class="more-btn" @click="showMoreMenu">
              <CuteIcon name="Ellipsis" :size="20" />
            </button>
          </div>

          <!-- 第二行：描述 -->
          <div v-if="project.description" class="project-description">
            {{ project.description }}
          </div>
        </div>

        <!-- 任务列表区域 -->
        <div class="tasks-area">
          <!-- 无 section 的任务（即使没有任务也要显示） -->
          <div class="task-section">
            <ProjectTaskList
              :key="`project-${project.id}-no-section`"
              :title="$t('project.label.uncategorized')"
              :view-key="`project::${project.id}::section::all`"
              title-color="var(--color-text-accent)"
            />
          </div>

          <!-- 各个 section 的任务 -->
          <div v-for="section in sections" :key="section.id" class="task-section">
            <ProjectTaskList
              :title="section.title"
              :view-key="`project::${project.id}::section::${section.id}`"
              title-color="var(--color-text-accent)"
            />
          </div>

          <!-- 空状态 -->
          <div v-if="!hasTasksWithoutSection && sections.length === 0" class="no-tasks">
            <p>{{ $t('project.empty.noTasks') }}</p>
            <p class="hint">{{ $t('project.empty.noTasksHint') }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 项目不存在 -->
    <div v-else class="empty-state">
      <p>{{ $t('project.empty.notExist') }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useProjectStore } from '@/stores/project'
import { useAreaStore } from '@/stores/area'
import { useTaskStore } from '@/stores/task'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CircularProgress from '@/components/parts/CircularProgress.vue'
import ProjectTaskList from '@/components/assembles/tasks/list/ProjectTaskList.vue'

interface Props {
  projectId?: string | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'edit-project': [id: string]
}>()

const projectStore = useProjectStore()
const areaStore = useAreaStore()
const taskStore = useTaskStore()

// 获取当前项目
const project = computed(() => {
  if (!props.projectId) return null
  return projectStore.getProjectById(props.projectId)
})

// 获取项目的 area 标签
const areaTag = computed(() => {
  if (!project.value?.area_id) return null
  return areaStore.getAreaById(project.value.area_id)
})

// 获取项目的章节列表
const sections = computed(() => {
  if (!props.projectId) return []
  return projectStore.getSectionsByProject(props.projectId)
})

// 计算项目进度统计
// ⚠️ 使用 projectStore.getProjectStatsRealtime 统一计算，确保过滤和去重一致
const projectStats = computed(() => {
  if (!props.projectId) return { completed: 0, total: 0 }
  return projectStore.getProjectStatsRealtime(props.projectId)
})

// 检查是否有无 section 的任务
// ⚠️ 过滤 EXPIRE 过期任务并去重循环任务
const hasTasksWithoutSection = computed(() => {
  if (!props.projectId) return false
  const today = new Date().toISOString().split('T')[0]!

  // 1. 基础过滤
  const tasks = taskStore.allTasks.filter(
    (task) =>
      task.project_id === props.projectId &&
      !task.section_id &&
      !task.is_deleted &&
      !task.is_archived
  )

  // 2. 过滤 EXPIRE 过期任务
  const filtered = tasks.filter((task) => !taskStore.isExpiredRecurringTask(task, today))

  // 3. 去重循环任务
  const deduplicated = taskStore.deduplicateRecurringTasks(filtered)

  return deduplicated.length > 0
})

// 显示更多菜单（触发编辑项目）
const showMoreMenu = () => {
  if (project.value) {
    emit('edit-project', project.value.id)
  }
}
</script>

<style scoped>
.project-detail-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-background-content, #f0f);
  overflow: hidden;

  /* 默认：更紧凑的头部间距（DailyPlanning 等复用场景更合适）
     如需“大留白”风格（ProjectsView），在外层视图中覆盖该变量即可 */
  --project-detail-header-padding: 2.4rem 1.6rem 2.4rem;
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

.project-detail {
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

/* Things 3 风格头部 */
.project-header {
  flex-shrink: 0;
  padding: var(--project-detail-header-padding);
}

.header-title-row {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.project-title {
  flex: 1;
  font-size: 2.2rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
  margin: 0;
  line-height: 1.4;
}

.more-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3.2rem;
  height: 3.2rem;
  background: transparent;
  color: var(--color-text-tertiary, #f0f);
  border: none;
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s;
}

.more-btn:hover {
  background: var(--color-background-hover, #f0f);
  color: var(--color-text-primary, #f0f);
}

.project-description {
  font-size: 1.4rem;
  color: var(--color-text-secondary, #f0f);
  margin-top: 0.8rem;
  line-height: 1.6;
  white-space: pre-wrap;
}

.tasks-area {
  flex: 1;
  overflow-y: auto;
  padding: 0; /* 移除 padding，由 TaskList 自己控制 */
}

.task-section {
  margin-bottom: 2.4rem;
}

.no-tasks {
  text-align: center;
  padding: 4rem 2rem;
  color: var(--color-text-secondary, #f0f);
}

.no-tasks p {
  margin: 0.8rem 0;
}

.no-tasks .hint {
  font-size: 1.2rem;
  color: var(--color-text-tertiary, #f0f);
}
</style>
