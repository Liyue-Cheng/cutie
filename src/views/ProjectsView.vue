<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import ProjectsPanel from '@/components/organisms/ProjectsPanel.vue'
import ArchiveColumn from '@/components/assembles/tasks/kanban/ArchiveColumn.vue'
import DoubleRowTimeline from '@/components/parts/timeline/DoubleRowTimeline.vue'
import VerticalToolbar from '@/components/functional/VerticalToolbar.vue'
import TaskEditorModal from '@/components/assembles/tasks/TaskEditorModal.vue'
import GlobalRecurrenceEditDialog from '@/components/parts/recurrence/GlobalRecurrenceEditDialog.vue'
import { useRegisterStore } from '@/stores/register'
import { useUIStore } from '@/stores/ui'
import { logger, LogTags } from '@/infra/logging/logger'

// ==================== 视图类型 ====================
type RightPaneView = 'timeline' | 'archive'

// ==================== Stores ====================
const registerStore = useRegisterStore()
const uiStore = useUIStore()

// ==================== 状态 ====================
const currentRightPaneView = ref<RightPaneView | null>(null)

import { useI18n } from 'vue-i18n'

const { t } = useI18n()

// 右侧面板视图配置
const rightPaneViewConfig = computed(() => ({
  timeline: { icon: 'Clock' as const, label: t('toolbar.timeline') },
  archive: { icon: 'Archive' as const, label: t('toolbar.archive') },
}))

// ==================== 初始化 ====================
onMounted(() => {
  logger.info(LogTags.VIEW_HOME, 'ProjectsView mounted')
  registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_VIEW, 'projects')
})

// ==================== 事件处理 ====================
function switchRightPaneView(view: string | null) {
  logger.debug(LogTags.VIEW_HOME, 'Switching right pane view', { view })
  currentRightPaneView.value = view as RightPaneView | null
}
</script>

<template>
  <div class="projects-view-container">
    <!-- 主内容区域：项目面板 -->
    <div class="main-content-pane">
      <ProjectsPanel />
    </div>

    <!-- 右边栏：控制选项 -->
    <div v-if="currentRightPaneView" class="right-control-pane">
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
      :allow-collapse="true"
      :default-view="null"
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
.projects-view-container {
  display: flex;
  height: 100%;
  width: 100%;
  background-color: var(--color-background-content, #f0f);
  overflow: hidden;
}

/* ==================== 主内容区域 ==================== */
.main-content-pane {
  flex: 1;
  min-width: 0;
  border-right: 1px solid var(--color-border-adaptive-light-normal-dark-none, #f0f);
  position: relative;
  overflow: hidden;
}

/* ==================== 右边栏：控制面板 ==================== */
.right-control-pane {
  width: 28rem;
  min-width: 28rem;
  border-right: 1px solid var(--color-border-adaptive-light-normal-dark-none, #f0f);
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
