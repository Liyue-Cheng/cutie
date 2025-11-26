<template>
  <div class="project-detail-panel">
    <!-- 空状态（未选择任何项） -->
    <div v-if="projectId === undefined" class="empty-state">
      <CuteIcon name="Folder" :size="48" />
      <p>请选择一个项目</p>
    </div>

    <!-- 无项目视图 -->
    <div v-else-if="projectId === null" class="project-detail no-project-view">
      <!-- 头部 -->
      <div class="project-header">
        <div class="header-left">
          <h1 class="project-title">
            <CuteIcon name="Inbox" :size="24" />
            无项目
          </h1>
          <div class="project-description">显示所有未分配到任何项目的任务</div>
        </div>
      </div>

      <!-- 任务列表区域 -->
      <div class="tasks-area">
        <div class="task-section">
          <TaskList title="无项目任务" view-key="misc::no-project" />
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
            <TaskList
              :key="`project-${project.id}-no-section`"
              title="未分类任务"
              :view-key="`project::${project.id}::section::all`"
              title-color="var(--color-text-accent)"
            />
          </div>

          <!-- 各个 section 的任务 -->
          <div v-for="section in sections" :key="section.id" class="task-section">
            <TaskList
              :title="section.title"
              :view-key="`project::${project.id}::section::${section.id}`"
              title-color="var(--color-text-accent)"
            >
              <template #title-actions>
                <button class="icon-btn" @click="handleEditSection(section.id)">
                  <CuteIcon name="Pencil" :size="14" />
                </button>
              </template>
            </TaskList>
          </div>

          <!-- 空状态 -->
          <div v-if="!hasTasksWithoutSection && sections.length === 0" class="no-tasks">
            <p>暂无任务</p>
            <p class="hint">从其他视图拖动任务到此项目，或点击"添加章节"组织任务</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 项目不存在 -->
    <div v-else class="empty-state">
      <p>项目不存在</p>
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
import TaskList from '@/components/assembles/tasks/list/TaskList.vue'

interface Props {
  projectId?: string | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'edit-project': [id: string]
  'create-section': []
  'edit-section': [sectionId: string]
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
const projectStats = computed(() => {
  if (!props.projectId) return { completed: 0, total: 0 }
  const projectTasks = taskStore.allTasks.filter(
    (task) => task.project_id === props.projectId && !task.is_deleted
  )
  const completed = projectTasks.filter((task) => task.is_completed).length
  return { completed, total: projectTasks.length }
})

// 检查是否有无 section 的任务
const hasTasksWithoutSection = computed(() => {
  if (!props.projectId) return false
  const tasks = taskStore.allTasks.filter(
    (task) => task.project_id === props.projectId && !task.section_id && !task.is_deleted
  )
  return tasks.length > 0
})

// 格式化日期
const formatDate = (dateStr: string) => {
  const date = new Date(dateStr)
  return date.toLocaleDateString('zh-CN', { year: 'numeric', month: 'long', day: 'numeric' })
}

// 编辑章节
const handleEditSection = (sectionId: string) => {
  emit('edit-section', sectionId)
}

// 显示更多菜单（暂时触发编辑项目）
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
  background: var(--color-background-secondary, #f0f);
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
  padding: 7rem 1.6rem 4rem; /* 上边距增大，左右与 TaskList 对齐 */
}

.header-title-row {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.project-title {
  flex: 1;
  font-size: 2.2rem;
  font-weight: 700;
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
}

.tasks-area {
  flex: 1;
  overflow-y: auto;
  padding: 0; /* 移除 padding，由 TaskList 自己控制 */
}

.task-section {
  margin-bottom: 2.4rem;
}

.icon-btn {
  padding: 0.4rem;
  background: transparent;
  color: var(--color-text-secondary, #f0f);
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.2s;
}

.icon-btn:hover {
  background: var(--color-background-hover, #f0f);
  color: var(--color-text-primary, #f0f);
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
