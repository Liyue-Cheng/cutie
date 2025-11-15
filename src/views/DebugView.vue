<script setup lang="ts">
import { onMounted, computed, ref } from 'vue'
import type { TaskCard } from '@/types/dtos'
import SimpleKanbanColumn from '@/components/assembles/tasks/kanban/SimpleKanbanColumn.vue'
import TaskEditorModal from '@/components/assembles/tasks/TaskEditorModal.vue'
import CuteCalendar from '@/components/assembles/calender/CuteCalendar.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CuteButton from '@/components/parts/CuteButton.vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import { useTaskStore } from '@/stores/task'
import { useViewStore } from '@/stores/view'
import { useViewOperations } from '@/composables/useViewOperations'
import { logger, LogTags } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'

const taskStore = useTaskStore()
const viewStore = useViewStore()
const viewOps = useViewOperations()
const isEditorOpen = ref(false)
const selectedTaskId = ref<string | null>(null)

// ✅ 新架构：过滤（TaskStore）+ 排序（ViewStore）
// ✅ 完全自动的实时更新：任务状态改变立即反映

const allTasks = computed(() => {
  return viewStore.applySorting(taskStore.allTasks, 'misc::all')
})

const incompleteTasks = computed(() => {
  return viewStore.applySorting(taskStore.incompleteTasks, 'misc::incomplete')
})

const stagingTasks = computed(() => {
  return viewStore.applySorting(taskStore.stagingTasks, 'misc::staging')
})

const plannedTasks = computed(() => {
  return viewStore.applySorting(taskStore.plannedTasks, 'misc::planned')
})

function handleOpenEditor(task: TaskCard) {
  selectedTaskId.value = task.id
  isEditorOpen.value = true
}

async function handleAddTask(title: string) {
  // ✅ 使用 CPU Pipeline 创建任务
  try {
    await pipeline.dispatch('task.create', {
      title,
      estimated_duration: 60, // ✅ 默认1小时
    })
    logger.info(LogTags.VIEW_HOME, 'Task created in debug view', { title })
    // ✅ 新架构：无需手动添加，任务会自动通过 SSE 出现在 stagingTasks 中
  } catch (error) {
    logger.error(LogTags.VIEW_HOME, 'Failed to create task in debug view', error as Error)
  }
}

// 处理拖拽排序
async function handleReorder(viewKey: string, newOrder: string[]) {
  logger.debug(LogTags.VIEW_HOME, 'Reordering tasks in debug view', { viewKey, newOrder })
  await viewStore.updateSorting(viewKey, newOrder)
}

onMounted(async () => {
  // ✅ 职责分离：
  // - 父组件：加载业务数据（任务列表）
  // - 子组件：加载视图配置（排序设置）
  try {
    await Promise.all([
      viewOps.loadAllTasks(),
      viewOps.loadPlannedTasks(),
      viewOps.loadStagingTasks(),
    ])

    logger.info(LogTags.VIEW_HOME, 'Loaded all task data in debug view')
    // 注意：排序配置由 SimpleKanbanColumn 自己加载
  } catch (error) {
    logger.error(
      LogTags.VIEW_HOME,
      'Failed to fetch tasks in debug view',
      error instanceof Error ? error : new Error(String(error))
    )
  }
})
</script>

<template>
  <div class="home-view-container">
    <div class="main-content-pane">
      <TwoRowLayout>
        <template #top>
          <CuteButton>Test Button 1</CuteButton>
        </template>
        <template #bottom>
          <div class="task-view-pane">
            <SimpleKanbanColumn
              title="All"
              subtitle="所有任务"
              view-key="misc::all"
              :tasks="allTasks"
              @open-editor="handleOpenEditor"
              @reorder-tasks="(order) => handleReorder('misc::all', order)"
            />
            <SimpleKanbanColumn
              title="Incomplete"
              subtitle="未完成"
              view-key="misc::incomplete"
              :tasks="incompleteTasks"
              @open-editor="handleOpenEditor"
              @reorder-tasks="(order) => handleReorder('misc::incomplete', order)"
            />
            <SimpleKanbanColumn
              title="Staging"
              subtitle="未排期"
              view-key="misc::staging"
              :tasks="stagingTasks"
              :show-add-input="true"
              @open-editor="handleOpenEditor"
              @add-task="handleAddTask"
              @reorder-tasks="(order) => handleReorder('misc::staging', order)"
            />
            <SimpleKanbanColumn
              title="Planned"
              subtitle="已排期"
              view-key="misc::planned"
              :tasks="plannedTasks"
              @open-editor="handleOpenEditor"
              @reorder-tasks="(order) => handleReorder('misc::planned', order)"
            />
          </div>
        </template>
      </TwoRowLayout>
    </div>
    <div class="calendar-pane">
      <TwoRowLayout>
        <template #top>
          <CuteButton>Test Button 2</CuteButton>
        </template>
        <template #bottom>
          <CuteCalendar />
        </template>
      </TwoRowLayout>
    </div>
    <div class="toolbar-pane">
      <TwoRowLayout>
        <template #top>
          <CuteButton>Test</CuteButton>
        </template>
        <template #bottom>
          <div class="toolbar-icons">
            <CuteIcon name="Calendar" :size="28" />
            <CuteIcon name="Theater" :size="28" />
          </div>
        </template>
      </TwoRowLayout>
    </div>
    <TaskEditorModal v-if="isEditorOpen" :task-id="selectedTaskId" @close="isEditorOpen = false" />
  </div>
</template>

<style scoped>
.home-view-container {
  display: flex;
  height: 100%;
  width: 100%;
  background-color: var(--color-background-content);
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
}

.main-content-pane {
  flex: 1;
  min-width: 0;
  border-right: 1px solid var(--color-border-default);
  box-shadow: inset -4px 0 12px -2px rgb(0 0 0 / 5%);
  position: relative;
}

.calendar-pane {
  width: 30rem;
  min-width: 0;
  border-right: 1px solid var(--color-border-default);
}

.toolbar-pane {
  width: 6rem; /* 96px */
  min-width: 6rem;
}

.toolbar-icons {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  align-items: center;
  padding-top: 1rem;
}

.task-view-pane {
  display: flex;
  gap: 1rem;
  height: 100%;
}

:deep(.top-row .cute-button) {
  background-color: var(--color-background-accent);
  color: var(--color-text-on-accent);
  border-color: transparent;
}

:deep(.top-row .cute-button:hover) {
  background-color: var(--color-background-accent);
  filter: brightness(0.9);
}
</style>
