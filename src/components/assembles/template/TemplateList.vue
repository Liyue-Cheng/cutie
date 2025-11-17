<template>
  <div class="template-list">
    <div class="template-header">
      <div class="header-title">
        <h3>模板</h3>
        <span class="template-count">{{ displayItems.length }}</span>
      </div>
      <button class="add-template-button" @click="isCreateModalOpen = true">
        <span class="plus-icon">＋</span>
        <span>新建模板</span>
      </button>
    </div>

    <!-- 模板列表 -->
    <div ref="templateContainerRef" class="template-list-scroll-area">
      <div
        v-for="template in displayItems"
        :key="template.id"
        :class="`template-card-wrapper template-card-wrapper-${VIEW_KEY_CLASS} template-strip-wrapper template-strip-wrapper-${VIEW_KEY_CLASS}`"
        :data-object-id="template.id"
      >
        <TemplateStrip :template="template" @open-editor="handleOpenEditor(template.id)" />
      </div>

      <div v-if="displayItems.length === 0" class="empty-state">暂无模板</div>
    </div>

    <!-- 模板编辑器 -->
    <TemplateEditorModal
      v-if="isEditorOpen"
      :template-id="selectedTemplateId"
      @close="isEditorOpen = false"
    />
    <!-- 模板创建弹窗 -->
    <TemplateCreateModal v-if="isCreateModalOpen" @close="isCreateModalOpen = false" />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useTemplateStore } from '@/stores/template'
import { useViewStore } from '@/stores/view'
import type { ViewMetadata } from '@/types/drag'
import TemplateStrip from './TemplateStrip.vue'
import TemplateEditorModal from './TemplateEditorModal.vue'
import TemplateCreateModal from './TemplateCreateModal.vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { useInteractDrag } from '@/composables/drag/useInteractDrag'
import { useDragStrategy } from '@/composables/drag/useDragStrategy'
import { dragPreviewState } from '@/infra/drag-interact/preview-state'

const templateStore = useTemplateStore()
const viewStore = useViewStore()

const selectedTemplateId = ref<string | null>(null)
const isEditorOpen = ref(false)
const isCreateModalOpen = ref(false)

// 模板看板的 viewKey 和 metadata
const VIEW_KEY = 'misc::template'
const VIEW_KEY_CLASS = VIEW_KEY.replace(/::/g, '--')
const viewMetadata = computed<ViewMetadata>(
  () =>
    ({
      id: VIEW_KEY,
      type: 'status',
      label: '模板',
    }) as ViewMetadata
)

// 加载所有模板和视图偏好
onMounted(async () => {
  try {
    // 1. 加载视图偏好排序
    await viewStore.fetchViewPreference(VIEW_KEY)
    logger.debug(LogTags.COMPONENT_KANBAN_COLUMN, 'Template view preference loaded', {
      viewKey: VIEW_KEY,
    })

    // 2. 加载模板数据
    await templateStore.fetchAllTemplates()
    logger.info(LogTags.COMPONENT_KANBAN_COLUMN, 'Templates loaded', {
      count: templateStore.generalTemplates.length,
    })
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_KANBAN_COLUMN,
      'Failed to load templates',
      error instanceof Error ? error : new Error(String(error))
    )
  }
})

// 原始模板列表（仅通用模板 + 应用排序）
const originalTemplates = computed(() => {
  const baseTemplates = templateStore.generalTemplates

  // 应用视图偏好排序
  const weights = viewStore.sortWeights.get(VIEW_KEY)
  if (!weights || weights.size === 0) {
    // 没有排序信息，保持原顺序
    return baseTemplates
  }

  // 手动应用排序
  const sorted = [...baseTemplates].sort((a, b) => {
    const weightA = weights.get(a.id) ?? Infinity
    const weightB = weights.get(b.id) ?? Infinity
    return weightA - weightB
  })

  return sorted
})

// 拖放系统集成
const templateContainerRef = ref<HTMLElement | null>(null)
const dragStrategy = useDragStrategy()

const { displayItems } = useInteractDrag({
  viewMetadata,
  items: originalTemplates,
  containerRef: templateContainerRef,
  draggableSelector: `.template-card-wrapper-${VIEW_KEY_CLASS}`,
  objectType: 'template',
  getObjectId: (template) => template.id,
  onDrop: async (session) => {
    console.group('🎯 Template Drop Event')
    console.log('Session:', session)
    console.log('Target ViewKey:', VIEW_KEY)
    console.log('Templates:', originalTemplates.value.length)
    console.groupEnd()

    // 执行拖放策略
    const result = await dragStrategy.executeDrop(session, VIEW_KEY, {
      sourceContext: (session.metadata?.sourceContext as Record<string, any>) || {},
      targetContext: {
        itemIds: originalTemplates.value.map((t) => t.id),
        displayItems: displayItems.value,
        dropIndex: dragPreviewState.value?.computed.dropIndex,
        viewKey: VIEW_KEY,
      },
    })

    if (!result.success) {
      logger.error(
        LogTags.COMPONENT_KANBAN_COLUMN,
        'Template drop failed',
        new Error(result.message || 'Unknown error'),
        { result, session }
      )
    }
  },
})

function handleOpenEditor(templateId: string) {
  selectedTemplateId.value = templateId
  isEditorOpen.value = true
  logger.info(LogTags.COMPONENT_KANBAN_COLUMN, 'Opening template editor', { templateId })
}
</script>

<style scoped>
.template-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--color-background-content);
  overflow-y: auto;
}

.template-header {
  padding: 1.2rem 1.6rem 1rem;
  border-bottom: 1px solid var(--color-border-default);
  background-color: var(--color-background-content);
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.header-title h3 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.template-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2.4rem;
  height: 2.4rem;
  padding: 0 0.6rem;
  font-size: 1.2rem;
  font-weight: 600;
  color: var(--color-text-tertiary);
  background-color: var(--color-background-hover);
  border-radius: 1.2rem;
}

.add-template-button {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.6rem 1.2rem;
  border-radius: 0.8rem;
  border: 1px solid var(--color-border-default);
  background-color: var(--color-background-secondary);
  color: var(--color-text-primary);
  font-size: 1.3rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.add-template-button:hover {
  border-color: var(--color-primary, #4a90e2);
  color: var(--color-primary, #4a90e2);
}

.plus-icon {
  font-size: 1.4rem;
  line-height: 1;
}

.template-list-scroll-area {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem 1.6rem 1.6rem;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 10rem;
  color: var(--color-text-tertiary);
  font-size: 1.4rem;
}

/* 拖拽相关样式 */
.template-strip-wrapper {
  position: relative;
  cursor: grab;
  transition: transform 0.2s ease;
  margin-bottom: 0.8rem;
}

.template-strip-wrapper:active {
  cursor: grabbing;
}

.template-strip-wrapper:last-child {
  margin-bottom: 0;
}

/* 滚动条样式 */
.template-list-scroll-area::-webkit-scrollbar {
  width: 6px;
}

.template-list-scroll-area::-webkit-scrollbar-track {
  background: transparent;
}

.template-list-scroll-area::-webkit-scrollbar-thumb {
  background: var(--color-border-default);
  border-radius: 3px;
}

.template-list-scroll-area::-webkit-scrollbar-thumb:hover {
  background: var(--color-text-tertiary);
}
</style>
