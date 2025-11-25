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
      <!-- 项目头部 -->
      <div class="project-header">
        <div class="header-left">
          <h1 class="project-title">{{ project.name }}</h1>
          <div v-if="project.description" class="project-description">
            {{ project.description }}
          </div>
          <div class="project-tags">
            <span v-if="areaTag" class="area-tag" :style="{ backgroundColor: areaTag.color }">
              {{ areaTag.name }}
            </span>
            <span v-if="project.due_date" class="due-date-tag">
              <CuteIcon name="Calendar" :size="14" />
              {{ formatDate(project.due_date) }}
            </span>
          </div>
        </div>
        <div class="header-actions">
          <button class="action-btn" @click="emit('edit-project', project.id)">
            <CuteIcon name="Pencil" :size="16" />
            编辑项目
          </button>
          <button class="action-btn" @click="emit('create-section')">
            <CuteIcon name="Plus" :size="16" />
            添加章节
          </button>
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
          />
        </div>

        <!-- 各个 section 的任务 -->
        <div v-for="section in sections" :key="section.id" class="task-section">
          <TaskList
            :title="section.title"
            :view-key="`project::${project.id}::section::${section.id}`"
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
</script>

<style scoped>
.project-detail-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-background-secondary);
  overflow: hidden;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--color-text-secondary);
  gap: 1.2rem;
}

.project-detail {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.project-header {
  flex-shrink: 0;
  padding: 2rem;
  border-bottom: 1px solid var(--color-border-default);
}

.header-left {
  margin-bottom: 1.6rem;
}

.project-title {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  font-size: 2.4rem;
  font-weight: 700;
  color: var(--color-text-primary);
  margin: 0 0 0.8rem;
}

.project-description {
  font-size: 1.4rem;
  color: var(--color-text-secondary);
  margin-bottom: 1.2rem;
  line-height: 1.6;
}

.project-tags {
  display: flex;
  gap: 0.8rem;
  flex-wrap: wrap;
}

.area-tag {
  padding: 0.4rem 0.8rem;
  border-radius: 0.4rem;
  font-size: 1.2rem;
  color: var(--color-text-on-accent);
  font-weight: 500;
}

.due-date-tag {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.4rem 0.8rem;
  border-radius: 0.4rem;
  font-size: 1.2rem;
  background: var(--color-background-content);
  color: var(--color-text-secondary);
}

.header-actions {
  display: flex;
  gap: 0.8rem;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.8rem 1.2rem;
  background: var(--color-background-secondary);
  color: var(--color-text-primary);
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  cursor: pointer;
  font-size: 1.4rem;
  transition: all 0.2s;
}

.action-btn:hover {
  background: var(--color-background-hover);
}

.tasks-area {
  flex: 1;
  overflow-y: auto;
  padding: 1.6rem;
}

.task-section {
  margin-bottom: 2.4rem;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1.2rem;
}

.section-title {
  font-size: 1.6rem;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.section-actions {
  display: flex;
  gap: 0.4rem;
}

.icon-btn {
  padding: 0.4rem;
  background: transparent;
  color: var(--color-text-secondary);
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.2s;
}

.icon-btn:hover {
  background: var(--color-background-hover);
  color: var(--color-text-primary);
}

.no-tasks {
  text-align: center;
  padding: 4rem 2rem;
  color: var(--color-text-secondary);
}

.no-tasks p {
  margin: 0.8rem 0;
}

.no-tasks .hint {
  font-size: 1.2rem;
  color: var(--color-text-tertiary);
}
</style>
