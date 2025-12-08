<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTemplateStore } from '@/stores/template'
import type { ViewMetadata } from '@/types/drag'
import CutePane from '@/components/alias/CutePane.vue'
import TemplateCard from './TemplateCard.vue'
import TemplateEditorModal from './TemplateEditorModal.vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'
import { useInteractDrag } from '@/composables/drag/useInteractDrag'
import { useDragStrategy } from '@/composables/drag/useDragStrategy'
import { dragPreviewState } from '@/infra/drag-interact/preview-state'

const templateStore = useTemplateStore()
const pendingInit = ref(new Set<string>())

const selectedTemplateId = ref<string | null>(null)
const isEditorOpen = ref(false)
const newTemplateName = ref('')

// 🔥 模板看板的 viewKey 和 metadata
const VIEW_KEY = 'misc::template'
const viewMetadata = computed<ViewMetadata>(
  () =>
    ({
      id: VIEW_KEY,
      type: 'status',
      label: '模板',
    }) as ViewMetadata
)

// 加载所有模板
onMounted(async () => {
  try {
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

// 原始模板列表（遵循 LexoRank 排序）
const originalTemplates = computed(() => templateStore.generalTemplates)

// 检测缺少 rank 的模板并触发批量初始化
watch(
  () => originalTemplates.value,
  (templates) => {
    const missing = templates
      .filter((template) => !template.sort_rank && !pendingInit.value.has(template.id))
      .map((template) => template.id)

    if (missing.length === 0) {
      return
    }

    missing.forEach((id) => pendingInit.value.add(id))
    pipeline
      .dispatch('template.batch_init_ranks', {
        template_ids: missing,
      })
      .catch((error) => {
        logger.error(
          LogTags.COMPONENT_KANBAN_COLUMN,
          'Failed to batch initialize template ranks',
          error instanceof Error ? error : new Error(String(error))
        )
        missing.forEach((id) => pendingInit.value.delete(id))
      })
  },
  { immediate: true }
)

// ==================== 拖放系统集成 ====================

const kanbanContainerRef = ref<HTMLElement | null>(null)
const dragStrategy = useDragStrategy()

const { displayItems } = useInteractDrag({
  viewMetadata,
  items: originalTemplates,
  containerRef: kanbanContainerRef,
  draggableSelector: `.template-card-wrapper-${VIEW_KEY.replace(/::/g, '--')}`,
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

// ✅ displayItems 已经是 Template[] 类型，无需转换！

function handleOpenEditor(templateId: string) {
  selectedTemplateId.value = templateId
  isEditorOpen.value = true
  logger.info(LogTags.COMPONENT_KANBAN_COLUMN, 'Opening template editor', { templateId })
}

async function handleCreateTemplate() {
  const title = newTemplateName.value.trim()
  if (!title) return

  try {
    // 先重置表单，给用户即时反馈
    newTemplateName.value = ''

    await pipeline.dispatch('template.create', {
      title: title,
    })

    logger.info(LogTags.COMPONENT_KANBAN_COLUMN, 'Template created successfully', { title })
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_KANBAN_COLUMN,
      'Failed to create template',
      error instanceof Error ? error : new Error(String(error))
    )
    alert(t('template.message.createFailed'))
  }
}

const { t } = useI18n()
</script>

<template>
  <CutePane class="template-kanban-column">
    <!-- 🔥 关键：kanbanContainerRef 必须指向一个 HTMLElement，不能直接指向 CutePane 组件 -->
    <div ref="kanbanContainerRef" class="kanban-dropzone-wrapper">
      <!-- Header -->
      <div class="header">
        <div class="title-section">
          <h2 class="title">{{ $t('template.title.templates') }}</h2>
          <p class="subtitle">{{ $t('template.title.subtitle') }}</p>
        </div>
        <div class="task-count">
          <span class="count">{{ displayItems.length }}</span>
        </div>
      </div>

      <!-- 创建模板表单 -->
      <div class="add-task-wrapper">
        <input
          v-model="newTemplateName"
          type="text"
          :placeholder="$t('template.placeholder.name')"
          class="add-task-input"
          @keyup.enter="handleCreateTemplate"
        />
      </div>

      <!-- 模板列表 -->
      <div class="task-list-scroll-area">
        <div
          v-for="template in displayItems"
          :key="template.id"
          :class="`template-card-wrapper template-card-wrapper-${VIEW_KEY.replace(/::/g, '--')}`"
          :data-object-id="template.id"
        >
          <TemplateCard :template="template" @open-editor="handleOpenEditor(template.id)" />
        </div>

        <div v-if="displayItems.length === 0" class="empty-state">{{ $t('template.label.noTemplates') }}</div>
      </div>
    </div>
  </CutePane>

  <!-- 模板编辑器 -->
  <TemplateEditorModal
    v-if="isEditorOpen"
    :template-id="selectedTemplateId"
    @close="isEditorOpen = false"
  />
</template>

<style scoped>
/* 复制 SimpleKanbanColumn 的样式 */
.template-kanban-column {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--color-background-content);
  width: 100%;
  flex-shrink: 0;
}

/* 🔥 dropzone wrapper 必须占满整个高度 */
.kanban-dropzone-wrapper {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
}

.header {
  padding: 1rem 1rem 0.5rem;
  border-bottom: 1px solid var(--color-border-default);
}

.title-section {
  margin-bottom: 0.5rem;
}

.title {
  font-size: 2.2rem;
  font-weight: 600;
  margin: 0;
  color: var(--color-text-primary);
}

.subtitle {
  font-size: 1.2rem;
  color: var(--color-text-secondary);
  margin: 0.25rem 0 0;
}

.task-count {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  font-size: 1.4rem;
  font-weight: 500;
}

.task-count .count {
  color: var(--color-text-secondary);
}

.add-task-wrapper {
  padding: 1rem 1rem 0.5rem;
}

.add-task-input {
  width: 100%;
  padding: 0.75rem;
  border: 1px solid var(--color-border-default);
  border-radius: 8px;
  background-color: var(--color-card-available);
  color: var(--color-text-primary);
  font-size: 1.5rem;
  transition: all 0.2s ease;
}

.add-task-input:focus {
  outline: none;
  border-color: var(--color-border-focus, #f0f);
  box-shadow: var(--shadow-focus, #f0f);
}

.add-task-input::placeholder {
  color: var(--color-text-secondary);
}

.add-task-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.creating-indicator {
  font-size: 1.2rem;
  color: var(--color-text-secondary);
  padding: 0.5rem 0.75rem;
  font-style: italic;
}

.task-list-scroll-area {
  flex-grow: 1;
  overflow-y: auto;
  padding: 0.5rem 1rem 1rem;
  min-height: 100px;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 10rem;
  color: var(--color-text-tertiary);
  font-size: 1.4rem;
}

/* 滚动条样式 */
.task-list-scroll-area::-webkit-scrollbar {
  width: 6px;
}

.task-list-scroll-area::-webkit-scrollbar-track {
  background: transparent;
}

.task-list-scroll-area::-webkit-scrollbar-thumb {
  background: var(--color-border-default);
  border-radius: 3px;
}

.task-list-scroll-area::-webkit-scrollbar-thumb:hover {
  background: var(--color-text-tertiary);
}

/* 拖拽相关样式 */
.template-card-wrapper {
  position: relative;
  cursor: grab;
  transition: transform 0.2s ease;
  margin-bottom: 1rem;
}

.template-card-wrapper:active {
  cursor: grabbing;
}

.template-card-wrapper:last-child {
  margin-bottom: 0;
}
</style>
