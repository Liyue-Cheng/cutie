<script setup lang="ts">
import { onMounted, computed, ref } from 'vue'
import type { TaskCard } from '@/types/dtos'
import SimpleKanbanColumn from '@/components/parts/kanban/SimpleKanbanColumn.vue'
import KanbanTaskEditorModal from '@/components/parts/kanban/KanbanTaskEditorModal.vue'
import CuteCalendar from '@/components/parts/CuteCalendar.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import CuteButton from '@/components/parts/CuteButton.vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import { useTaskStore } from '@/stores/task'
import { useViewStore } from '@/stores/view'
import { useViewOperations } from '@/composables/useViewOperations'
import { useTaskOperations } from '@/composables/useTaskOperations'

const taskStore = useTaskStore()
const viewStore = useViewStore()
const viewOps = useViewOperations()
const taskOps = useTaskOperations()
const isEditorOpen = ref(false)
const selectedTaskId = ref<string | null>(null)

// ✅ 新架构：过滤（TaskStore）+ 排序（ViewStore）
// ✅ 完全自动的实时更新：任务状态改变立即反映

const allTasks = computed(() => {
  return viewStore.applySorting(taskStore.allTasks, 'all')
})

const incompleteTasks = computed(() => {
  return viewStore.applySorting(taskStore.incompleteTasks, 'incomplete')
})

const stagingTasks = computed(() => {
  return viewStore.applySorting(taskStore.stagingTasks, 'staging')
})

const plannedTasks = computed(() => {
  return viewStore.applySorting(taskStore.plannedTasks, 'planned')
})

function handleOpenEditor(task: TaskCard) {
  selectedTaskId.value = task.id
  isEditorOpen.value = true
}

async function handleAddTask(title: string) {
  // ✅ 使用 TaskOperations 创建任务
  const taskId = await taskOps.createTask({ title })
  if (taskId) {
    console.log('[HomeView] Task created:', taskId)
    // ✅ 新架构：无需手动添加，任务会自动出现在 stagingTasks 中
  }
}

// 处理拖拽排序
async function handleReorder(viewKey: string, newOrder: string[]) {
  console.log(`[HomeView] 重新排序 ${viewKey}:`, newOrder)
  await viewStore.updateSorting(viewKey, newOrder)
}

// 以下方法已废弃，保留用于向后兼容
async function handleMoveTaskUp(viewKey: string, taskId: string) {
  const currentTasks = getTasksForView(viewKey)
  const index = currentTasks.findIndex((t) => t.id === taskId)

  if (index <= 0) return // 已经在最上面

  // 交换位置
  const newOrder = [...currentTasks]
  const temp = newOrder[index - 1]
  if (temp) {
    newOrder[index - 1] = newOrder[index]!
    newOrder[index] = temp
  }

  // 保存新顺序
  await viewStore.updateSorting(
    viewKey,
    newOrder.map((t) => t.id)
  )
}

// 处理任务向下移动
async function handleMoveTaskDown(viewKey: string, taskId: string) {
  const currentTasks = getTasksForView(viewKey)
  const index = currentTasks.findIndex((t) => t.id === taskId)

  if (index < 0 || index >= currentTasks.length - 1) return // 已经在最下面

  // 交换位置
  const newOrder = [...currentTasks]
  const temp = newOrder[index + 1]
  if (temp) {
    newOrder[index + 1] = newOrder[index]!
    newOrder[index] = temp
  }

  // 保存新顺序
  await viewStore.updateSorting(
    viewKey,
    newOrder.map((t) => t.id)
  )
}

// 辅助函数：根据 viewKey 获取对应的任务列表
function getTasksForView(viewKey: string): TaskCard[] {
  switch (viewKey) {
    case 'all':
      return allTasks.value
    case 'incomplete':
      return incompleteTasks.value
    case 'staging':
      return stagingTasks.value
    case 'planned':
      return plannedTasks.value
    default:
      return []
  }
}

onMounted(async () => {
  // ✅ 加载任务数据和排序配置
  try {
    // 1. 加载任务数据
    await Promise.all([
      viewOps.loadAllTasks(),
      viewOps.loadPlannedTasks(),
      viewOps.loadStagingTasks(),
    ])

    // 2. TODO: 加载排序配置（等后端 API 完成）
    // const sortPrefs = await fetchAllViewPreferences()
    // sortPrefs.forEach(pref => {
    //   const taskIds = JSON.parse(pref.sorted_task_ids)
    //   viewStore.loadSorting(pref.context_key, taskIds)
    // })

    console.log('[HomeView] Loaded all task views')
  } catch (error) {
    console.error('[HomeView] Failed to fetch tasks:', error)
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
              view-key="all"
              :tasks="allTasks"
              @open-editor="handleOpenEditor"
              @reorder-tasks="(order) => handleReorder('all', order)"
            />
            <SimpleKanbanColumn
              title="Incomplete"
              subtitle="未完成"
              view-key="incomplete"
              :tasks="incompleteTasks"
              @open-editor="handleOpenEditor"
              @reorder-tasks="(order) => handleReorder('incomplete', order)"
            />
            <SimpleKanbanColumn
              title="Staging"
              subtitle="未排期"
              view-key="staging"
              :tasks="stagingTasks"
              :show-add-input="true"
              @open-editor="handleOpenEditor"
              @add-task="handleAddTask"
              @reorder-tasks="(order) => handleReorder('staging', order)"
            />
            <SimpleKanbanColumn
              title="Planned"
              subtitle="已排期"
              view-key="planned"
              :tasks="plannedTasks"
              @open-editor="handleOpenEditor"
              @reorder-tasks="(order) => handleReorder('planned', order)"
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
    <KanbanTaskEditorModal
      v-if="isEditorOpen"
      :task-id="selectedTaskId"
      @close="isEditorOpen = false"
    />
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
  flex: 1;
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
  background-color: #4a90e2; /* A nice blue */
  color: #fff; /* White text */
  border-color: transparent;
}

:deep(.top-row .cute-button:hover) {
  background-color: #357abd; /* A darker blue for hover */
}
</style>
