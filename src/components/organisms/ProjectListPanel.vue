<template>
  <TwoRowLayout class="project-list-panel">
    <template #top>
      <!-- 控制栏 -->
      <div class="control-bar">
        <h2 class="title">{{ $t('project.title.list') }}</h2>
        <button class="create-btn" @click="emit('create-project')">
          <CuteIcon name="Plus" :size="16" />
          <span>{{ $t('project.action.create') }}</span>
        </button>
      </div>
    </template>

    <template #bottom>
      <!-- 项目列表 -->
      <div class="project-list">
        <!-- 无项目选项（置顶） -->
        <div
          class="project-card"
          :class="{ active: selectedId === null }"
          @click="emit('select-project', null)"
        >
          <div class="project-row">
            <div class="project-left">
              <div class="no-project-icon">
                <CuteIcon name="Inbox" :size="16" />
              </div>
              <div class="project-name">{{ $t('project.title.noProject') }}</div>
            </div>

            <div class="project-right">
              <span class="task-count">
                {{ noProjectStats.completed }}/{{ noProjectStats.total }}
                {{ $t('task.count.tasks') }}
              </span>
            </div>
          </div>
        </div>

        <!-- 普通项目列表 -->
        <div
          v-for="project in projects"
          :key="project.id"
          class="project-card"
          :class="{
            active: selectedId === project.id,
            'drop-target': isDropTarget(getProjectViewKey(project.id)),
          }"
          :ref="(el) => setProjectDropzoneRef(project.id, el)"
          @click="emit('select-project', project.id)"
          @contextmenu="showContextMenu(project, $event)"
        >
          <div class="project-row">
            <div class="project-left">
              <CircularProgress
                :completed="getProjectStats(project.id).completed"
                :total="getProjectStats(project.id).total"
                size="small"
                hide-text
                class="progress"
              />
              <div class="project-name">{{ project.name }}</div>
            </div>

            <div class="project-right">
              <span
                v-if="projectColor(project)"
                class="area-dot"
                :style="{ backgroundColor: projectColor(project) }"
              ></span>
              <span class="task-count">
                {{ getProjectStats(project.id).completed }}/{{ getProjectStats(project.id).total }}
                {{ $t('task.count.tasks') }}
              </span>
              <span v-if="project.due_date" class="due-date">{{
                formatDate(project.due_date)
              }}</span>
              <span
                v-if="project.status === 'COMPLETED'"
                class="status-badge completed inline-badge"
                >{{ $t('project.status.completed') }}</span
              >
            </div>
          </div>
        </div>

        <!-- 空状态 -->
        <div v-if="projects.length === 0" class="empty-state">
          <p>{{ $t('project.empty.noProjects') }}</p>
          <p class="hint">{{ $t('project.empty.noProjectsHint') }}</p>
        </div>
      </div>
    </template>
  </TwoRowLayout>
</template>

<script setup lang="ts">
import { computed, onMounted, onBeforeUnmount } from 'vue'
import type { ProjectCard } from '@/types/dtos'
import { useProjectStore } from '@/stores/project'
import { useAreaStore } from '@/stores/area'
import { useTaskStore } from '@/stores/task'
import { useViewStore } from '@/stores/view'
import { useDragStrategy } from '@/composables/drag/useDragStrategy'
import { useContextMenu } from '@/composables/useContextMenu'
import { interactManager, dragPreviewState, type DragSession } from '@/infra/drag-interact'
import { logger, LogTags } from '@/infra/logging/logger'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import CircularProgress from '@/components/parts/CircularProgress.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import ProjectCardMenu from '@/components/assembles/ContextMenu/ProjectCardMenu.vue'

interface Props {
  selectedId?: string | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'select-project': [id: string | null]
  'create-project': []
  'edit-project': [id: string]
  'add-section': [projectId: string]
}>()

const projectStore = useProjectStore()
const areaStore = useAreaStore()
const taskStore = useTaskStore()
const viewStore = useViewStore()
const dragStrategy = useDragStrategy()
const contextMenu = useContextMenu()

const dropzoneElements = new Map<string, HTMLElement>()

const activeDropzoneId = computed(() => dragPreviewState.value?.raw.targetZoneId ?? null)

// 获取活跃项目列表
const projects = computed(() => projectStore.activeProjects)

// 获取项目统计（前端实时计算）
const getProjectStats = (projectId: string) => {
  return projectStore.getProjectStatsRealtime(projectId)
}

// 获取无项目任务的统计
// ⚠️ 重要：
// - 过滤已删除、EXPIRE 类型过期的循环任务
// - 对循环任务去重（每个循环规则只计一个未完成任务）
const noProjectStats = computed(() => {
  const today = new Date().toISOString().split('T')[0]!

  // 1. 基础过滤：无项目 + 未删除 + 未归档
  const noProjectTasks = taskStore.allTasks.filter(
    (task) => !task.project_id && !task.is_deleted && !task.is_archived
  )

  // 2. 过滤 EXPIRE 类型的过期循环任务
  const filteredTasks = noProjectTasks.filter(
    (task) => !taskStore.isExpiredRecurringTask(task, today)
  )

  // 3. 对循环任务去重（每个循环规则只计一个未完成任务）
  const deduplicatedTasks = taskStore.deduplicateRecurringTasks(filteredTasks)

  // 4. 统计完成和总数
  const completed = deduplicatedTasks.filter((task) => task.is_completed).length
  const total = deduplicatedTasks.length

  return { completed, total }
})

const getProjectViewKey = (projectId: string) => `project::${projectId}::section::all`

const isDropTarget = (zoneId: string) => activeDropzoneId.value === zoneId

const cleanupDropzone = (zoneId: string) => {
  const element = dropzoneElements.get(zoneId)
  if (element) {
    interactManager.unregisterDropzone(element)
    dropzoneElements.delete(zoneId)
  }
}

const setProjectDropzoneRef = (projectId: string, el: Element | null) => {
  const zoneId = getProjectViewKey(projectId)
  const element = el as HTMLElement | null

  if (!element) {
    cleanupDropzone(zoneId)
    return
  }

  const existing = dropzoneElements.get(zoneId)
  if (existing === element) {
    return
  }

  if (existing && existing !== element) {
    interactManager.unregisterDropzone(existing)
    dropzoneElements.delete(zoneId)
  }

  dropzoneElements.set(zoneId, element)

  interactManager.registerDropzone(element, {
    zoneId,
    type: 'kanban',
    onDrop: async (session: DragSession) => {
      await handleProjectDrop(zoneId, session)
    },
  })
}

const handleProjectDrop = async (zoneId: string, session: DragSession) => {
  const taskId = (session.object?.data as Record<string, any>)?.id

  try {
    const baseTasks = taskStore.getTasksByViewKey_Mux(zoneId) || []
    const sortedTasks = viewStore.applySorting(baseTasks, zoneId)
    const previewDropIndex = dragPreviewState.value?.computed.dropIndex
    const dropIndex =
      typeof previewDropIndex === 'number' && previewDropIndex >= 0
        ? previewDropIndex
        : sortedTasks.length

    const result = await dragStrategy.executeDrop(session, zoneId, {
      sourceContext: (session.metadata?.sourceContext as Record<string, any>) || {},
      targetContext: {
        taskIds: sortedTasks.map((task) => task.id),
        displayTasks: sortedTasks,
        dropIndex,
        viewKey: zoneId,
      },
    })

    if (!result.success) {
      logger.error(
        LogTags.DRAG_STRATEGY,
        'Project list drop failed',
        new Error(result.error || result.message || 'Unknown error'),
        {
          targetZone: zoneId,
          taskId,
          result,
        }
      )
    } else {
      logger.info(LogTags.DRAG_STRATEGY, 'Project list drop succeeded', {
        targetZone: zoneId,
        taskId,
      })
    }
  } catch (error) {
    logger.error(
      LogTags.DRAG_STRATEGY,
      'Project list drop threw an exception',
      error instanceof Error ? error : new Error(String(error)),
      {
        targetZone: zoneId,
        taskId,
      }
    )
  }
}

onBeforeUnmount(() => {
  dropzoneElements.forEach((element) => {
    interactManager.unregisterDropzone(element)
  })
  dropzoneElements.clear()
})

onMounted(() => {
  console.log('ProjectListPanel mounted, projects:', projects.value.length)
})

// 获取项目颜色（从 area 继承）
const projectColor = (project: ProjectCard) => {
  if (!project.area_id) return null
  const area = areaStore.getAreaById(project.area_id)
  return area?.color || null
}

// 格式化日期
const formatDate = (dateStr: string) => {
  const date = new Date(dateStr)
  return date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' })
}

// 右键菜单
const showContextMenu = (project: ProjectCard, event: MouseEvent) => {
  contextMenu.show(
    ProjectCardMenu,
    {
      project,
      onEdit: () => emit('edit-project', project.id),
      onAddSection: () => emit('add-section', project.id),
    },
    event
  )
}
</script>

<style scoped>
.project-list-panel {
  background: var(--color-background-content, #f0f);
}

.control-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 0 1.2rem 0 2.4rem; /* 左侧与卡片内图标对齐 (project-list 1.2rem + project-card 1.2rem) */
}

.title {
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
  line-height: 1.4;
}

.create-btn {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.6rem 1.2rem;
  background: var(--color-button-primary-bg);
  color: var(--color-button-primary-text);
  border: none;
  border-radius: 0.6rem;
  cursor: pointer;
  font-size: 1.4rem;
  transition: opacity 0.2s;
  flex-shrink: 0;
}

.create-btn:hover {
  opacity: 0.9;
}

.project-list {
  padding: 1.2rem;
}

.project-card {
  position: relative;
  display: flex;
  align-items: center;
  padding: 1.2rem;
  margin-bottom: 0.8rem;
  background: var(--color-background-content, #f0f);
  border: 2px solid transparent;
  border-radius: 0.8rem;
  cursor: pointer;
  transition: all 0.2s;
}

.project-card:hover {
  background: var(--color-background-hover);
}

.project-card.active {
  background: var(--color-background-active);
}

.project-card.drop-target {
  border-color: var(--color-text-accent);
  background: var(--color-background-hover);
  box-shadow: var(--shadow-focus);
}

.project-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  gap: 1.2rem;
}

.project-left {
  display: flex;
  align-items: center;
  gap: 1.2rem;
  min-width: 0;
}

.project-right {
  display: flex;
  align-items: center;
  gap: 1rem;
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  white-space: nowrap;
}

/* 无项目图标（与 CircularProgress small 尺寸对齐） */
.no-project-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.4rem;
  height: 2.4rem;
  color: var(--color-text-primary);
  flex-shrink: 0;
}

.hint-text {
  color: var(--color-text-tertiary);
  font-size: 1.2rem;
}

.progress {
  flex-shrink: 0;
}

.project-info {
  flex: 1;
  min-width: 0;
}

.project-name {
  font-size: 1.6rem;
  font-weight: 500;
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.4;
}

.project-card.drop-target .project-name {
  color: var(--color-text-accent);
}

.project-meta {
  display: flex;
  gap: 1.2rem;
  font-size: 1.2rem;
  color: var(--color-text-secondary);
}

.task-count {
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.due-date {
  font-size: 1.3rem;
  color: var(--color-text-secondary);
}

.status-badge {
  flex-shrink: 0;
  padding: 0.4rem 0.8rem;
  border-radius: 0.4rem;
  font-size: 1.2rem;
  font-weight: 500;
}

.inline-badge {
  font-size: 1.1rem;
  padding: 0.2rem 0.6rem;
}

.status-badge.completed {
  background: var(--color-success-light);
  color: var(--color-success-text);
}

/* 区域彩色圆点 */
.area-dot {
  width: 0.8rem;
  height: 0.8rem;
  border-radius: 50%;
  flex-shrink: 0;
}

.empty-state {
  text-align: center;
  padding: 4rem 2rem;
  color: var(--color-text-secondary);
}

.empty-state p {
  margin: 0.8rem 0;
}

.empty-state .hint {
  font-size: 1.2rem;
  color: var(--color-text-tertiary);
}
</style>
