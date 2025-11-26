<template>
  <div class="project-detail-panel">
    <!-- 空状态（未选择任何项） -->
    <div v-if="projectId === undefined" class="empty-state">
      <CuteIcon name="Folder" :size="48" />
      <p>请选择一个项目</p>
    </div>

    <!-- 无项目视图 -->
    <div v-else-if="projectId === null" class="project-detail">
      <!-- 内容容器（限制宽度） -->
      <div class="content-container">
        <!-- 头部 -->
        <div class="project-header">
          <div class="header-title-row">
            <CuteIcon name="Inbox" :size="24" />
            <h1 class="project-title">无项目</h1>
          </div>
          <div class="project-description">显示所有未分配到任何项目的任务</div>
        </div>

        <!-- 任务列表区域 -->
        <div class="tasks-area">
          <div class="task-section">
            <TaskList title="无项目任务" view-key="misc::no-project" />
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
        <div class="tasks-area" @dragover="onContainerDragOver">
          <!-- 无 section 的任务（即使没有任务也要显示） -->
          <div class="task-section">
            <TaskList
              :key="`project-${project.id}-no-section`"
              title="未分类任务"
              :view-key="`project::${project.id}::section::all`"
              title-color="var(--color-text-accent)"
            />
          </div>

          <!-- 各个 section 的任务（支持拖放排序） -->
          <template v-for="(section, index) in sections" :key="section.id">
            <!-- 拖放指示线（在元素之前） -->
            <div v-if="dropTargetIndex === index" class="section-drop-indicator" />

            <div
              class="task-section is-draggable"
              :class="{ 'is-dragging': draggingSection?.id === section.id }"
              draggable="true"
              @dragstart="onDragStart(section, index, $event)"
              @dragover="onSectionDragOver($event, index)"
              @dragleave="onSectionDragLeave($event)"
            >
              <TaskList
                :title="section.title"
                :view-key="`project::${project.id}::section::${section.id}`"
                title-color="var(--color-text-accent)"
              >
                <template #title-actions>
                  <button class="icon-btn drag-handle" @mousedown.stop>
                    <CuteIcon name="GripVertical" :size="14" />
                  </button>
                  <button class="icon-btn" @click="handleEditSection(section.id)">
                    <CuteIcon name="Pencil" :size="14" />
                  </button>
                </template>
              </TaskList>
            </div>
          </template>

          <!-- 末尾拖放指示线 -->
          <div v-if="dropTargetIndex === sections.length" class="section-drop-indicator" />

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
import { useSectionDrag } from '@/composables/drag/useSectionDrag'
import { pipeline } from '@/cpu'

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

// Section 拖放排序
const {
  draggingSection,
  dropTargetIndex,
  onDragStart,
  onSectionDragOver,
  onSectionDragLeave,
  onContainerDragOver,
} = useSectionDrag({
  sections,
  onReorder: async (sectionId, prevId, nextId) => {
    if (!props.projectId) return

    await pipeline.dispatch('project_section.reorder', {
      project_id: props.projectId,
      section_id: sectionId,
      prev_section_id: prevId,
      next_section_id: nextId,
    })
  },
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

/* 拖拽把手 */
.icon-btn.drag-handle {
  cursor: grab;
  opacity: 0.5;
}

.icon-btn.drag-handle:hover {
  opacity: 1;
}

.icon-btn.drag-handle:active {
  cursor: grabbing;
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
