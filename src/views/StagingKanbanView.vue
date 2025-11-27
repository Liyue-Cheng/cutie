<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import InfiniteAreaKanban from '@/components/organisms/InfiniteAreaKanban.vue'
import ArchiveColumn from '@/components/assembles/tasks/kanban/ArchiveColumn.vue'
import DoubleRowTimeline from '@/components/parts/timeline/DoubleRowTimeline.vue'
import VerticalToolbar from '@/components/functional/VerticalToolbar.vue'
import TaskEditorModal from '@/components/assembles/tasks/TaskEditorModal.vue'
import GlobalRecurrenceEditDialog from '@/components/parts/recurrence/GlobalRecurrenceEditDialog.vue'
import { useAreaStore } from '@/stores/area'
import { useTaskStore } from '@/stores/task'
import { useUIStore } from '@/stores/ui'
import { logger, LogTags } from '@/infra/logging/logger'

// ==================== 视图类型 ====================
type RightPaneView = 'archive' | 'timeline'

// ==================== Stores ====================
const areaStore = useAreaStore()
const taskStore = useTaskStore()
const uiStore = useUIStore()

// ==================== 初始化 ====================
onMounted(async () => {
  logger.info(LogTags.VIEW_STAGING, 'Initializing staging view, loading data...')
  // 加载必要的数据
  await Promise.all([areaStore.fetchAll(), taskStore.fetchAllIncompleteTasks_DMA()])
  logger.info(LogTags.VIEW_STAGING, 'Staging view data loaded', {
    areaCount: areaStore.allAreas.length,
    taskCount: taskStore.incompleteTasks.length,
  })
})

// ==================== 状态 ====================
const kanbanRef = ref<InstanceType<typeof InfiniteAreaKanban> | null>(null)
const currentRightPaneView = ref<RightPaneView>('timeline') // 右侧面板当前视图
const kanbanCount = ref(0) // 看板数量

// 获取看板数量
const displayKanbanCount = computed(() => kanbanRef.value?.kanbanCount ?? kanbanCount.value)

// 右侧面板视图配置
const rightPaneViewConfig = {
  timeline: { icon: 'Clock', label: '时间线' },
  archive: { icon: 'Archive', label: '已归档' },
} as const

// ==================== 事件处理 ====================
function switchRightPaneView(view: string | null) {
  if (!view) return
  logger.debug(LogTags.VIEW_STAGING, 'Switching right pane view', { view })
  currentRightPaneView.value = view as RightPaneView
}

function handleKanbanCountChange(count: number) {
  kanbanCount.value = count
  logger.debug(LogTags.VIEW_STAGING, 'Kanban count changed', { count })
}
</script>

<template>
  <div class="staging-view-container">
    <!-- 主内容区域：Area 看板 -->
    <div class="main-content-pane">
      <TwoRowLayout>
        <template #top>
          <div class="kanban-header">
            <h2>Staging 看板</h2>
            <span class="kanban-count">{{ displayKanbanCount }} 个区域</span>
          </div>
        </template>
        <template #bottom>
          <InfiniteAreaKanban ref="kanbanRef" @kanban-count-change="handleKanbanCountChange" />
        </template>
      </TwoRowLayout>
    </div>

    <!-- 右边栏：控制选项 -->
    <div class="right-control-pane">
      <TwoRowLayout>
        <template #top>
          <div class="right-pane-header">
            <h3>{{ rightPaneViewConfig[currentRightPaneView].label }}</h3>
          </div>
        </template>
        <template #bottom>
          <!-- 已归档视图 -->
          <ArchiveColumn v-if="currentRightPaneView === 'archive'" />
          <!-- 时间线视图（单栏模式） -->
          <DoubleRowTimeline v-else-if="currentRightPaneView === 'timeline'" layout-mode="single" />
        </template>
      </TwoRowLayout>
    </div>

    <!-- 右边栏工具栏 -->
    <VerticalToolbar
      :view-config="rightPaneViewConfig"
      :current-view="currentRightPaneView"
      @view-change="switchRightPaneView"
    />

    <!-- 全局模态框 -->
    <TaskEditorModal
      v-if="uiStore.isEditorOpen"
      :task-id="uiStore.editorTaskId"
      :view-key="uiStore.editorViewKey ?? undefined"
      @close="uiStore.closeEditor"
    />
    <GlobalRecurrenceEditDialog />
  </div>
</template>

<style scoped>
.staging-view-container {
  display: flex;
  height: 100%;
  width: 100%;
  background-color: var(--color-background-content, #f0f);
  border: 1px solid var(--color-border-default, #f0f);
  border-radius: 0.8rem;
}

/* ==================== 主内容区域 ==================== */
.main-content-pane {
  flex: 1;
  min-width: 0;
  border-right: 1px solid var(--color-border-default, #f0f);
  position: relative;
}

.kanban-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 0 1rem;
  gap: 1rem;
}

.kanban-header h2 {
  margin: 0;
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
}

.kanban-count {
  font-size: 1.3rem;
  color: var(--color-text-tertiary, #f0f);
}

/* ==================== 右边栏：控制面板 ==================== */
.right-control-pane {
  width: 28rem;
  min-width: 28rem;
  border-right: 1px solid var(--color-border-default, #f0f);
}

.right-pane-header {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  padding: 0 1rem;
}

.right-pane-header h3 {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
  text-align: center;
}
</style>
