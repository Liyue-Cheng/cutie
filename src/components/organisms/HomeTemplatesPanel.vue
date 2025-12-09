<template>
  <div class="home-templates-panel">
    <TwoRowLayout>
      <template #top>
        <div class="panel-controls">
          <!-- 左侧：标题 -->
          <div class="controls-left">
            <div class="title-wrapper">
              <span class="title-text">{{ $t('nav.templates') }}</span>
              <span class="template-count">{{ templateCount }}</span>
            </div>
          </div>

          <!-- 右侧控制组 -->
          <div class="controls-right">
            <!-- 新建模板按钮 -->
            <button class="add-btn" @click="openCreateModal">
              <CuteIcon name="Plus" :size="16" />
              <span>新建</span>
            </button>
          </div>
        </div>
      </template>

      <template #bottom>
        <div ref="templateContainerRef" class="template-list">
          <div
            v-for="template in displayItems"
            :key="template.id"
            :class="`template-draggable template-draggable-${VIEW_KEY_CLASS}`"
            :data-object-id="template.id"
          >
            <TemplateStrip :template="template" @open-editor="handleOpenEditor(template.id)" />
          </div>

          <!-- 空状态 -->
          <div v-if="displayItems.length === 0" class="empty-state">
            <CuteIcon name="FileText" :size="48" />
            <p>暂无模板</p>
            <button class="create-first-btn" @click="openCreateModal">创建第一个模板</button>
          </div>
        </div>
      </template>
    </TwoRowLayout>

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
import { computed, onMounted, ref, watch } from 'vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import TemplateStrip from '@/components/assembles/template/TemplateStrip.vue'
import TemplateEditorModal from '@/components/assembles/template/TemplateEditorModal.vue'
import TemplateCreateModal from '@/components/assembles/template/TemplateCreateModal.vue'
import { useTemplateStore } from '@/stores/template'
import type { ViewMetadata } from '@/types/drag'
import { logger, LogTags } from '@/infra/logging/logger'
import { useInteractDrag } from '@/composables/drag/useInteractDrag'
import { useDragStrategy } from '@/composables/drag/useDragStrategy'
import { dragPreviewState } from '@/infra/drag-interact/preview-state'
import { pipeline } from '@/cpu'

const templateStore = useTemplateStore()
const pendingInit = ref(new Set<string>())

const selectedTemplateId = ref<string | null>(null)
const isEditorOpen = ref(false)
const isCreateModalOpen = ref(false)

// 模板数量
const templateCount = computed(() => displayItems.value.length)

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

// 原始模板列表
const originalTemplates = computed(() => templateStore.generalTemplates)

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

// 拖放系统集成
const templateContainerRef = ref<HTMLElement | null>(null)
const dragStrategy = useDragStrategy()

const { displayItems } = useInteractDrag({
  viewMetadata,
  items: originalTemplates,
  containerRef: templateContainerRef,
  draggableSelector: `.template-draggable-${VIEW_KEY_CLASS}`,
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

function openCreateModal() {
  isCreateModalOpen.value = true
}
</script>

<style scoped>
.home-templates-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ==================== 控制栏 ==================== */
.panel-controls {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1.2rem;
  padding: 1.2rem 0.8rem 1.2rem 1.6rem;
  background-color: transparent;
}

.controls-left {
  display: flex;
  align-items: center;
  gap: 1.2rem;
}

.controls-right {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

/* ==================== 标题样式 ==================== */
.title-wrapper {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.title-text {
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
  line-height: 1.4;
  white-space: nowrap;
}

.template-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2.4rem;
  height: 2.4rem;
  padding: 0 0.8rem;
  font-size: 1.3rem;
  font-weight: 600;
  color: var(--color-text-secondary, #f0f);
  background-color: var(--color-background-secondary, #f0f);
  border-radius: 1.2rem;
}

/* ==================== 新建按钮 ==================== */
.add-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.6rem 1.2rem;
  border-radius: 0.6rem;
  border: 1px solid var(--color-border-default, #f0f);
  background-color: var(--color-background-secondary, #f0f);
  color: var(--color-text-primary, #f0f);
  font-size: 1.3rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.add-btn:hover {
  border-color: var(--color-border-hover, #f0f);
  background-color: var(--color-background-hover, #f0f);
}

.add-btn:active {
  transform: scale(0.98);
}

/* ==================== 模板列表 ==================== */
.template-list {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem 1.6rem 1.6rem;
}

/* 拖拽相关样式 */
.template-draggable {
  position: relative;
  cursor: grab;
  transition: transform 0.2s ease;
  margin-bottom: 0.8rem;
}

.template-draggable:active {
  cursor: grabbing;
}

.template-draggable:last-child {
  margin-bottom: 0;
}

/* ==================== 空状态 ==================== */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 4rem 2rem;
  color: var(--color-text-tertiary, #f0f);
  gap: 1.2rem;
}

.empty-state p {
  margin: 0;
  font-size: 1.4rem;
  line-height: 1.4;
}

.create-first-btn {
  margin-top: 0.8rem;
  padding: 0.8rem 1.6rem;
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-on-accent, #f0f);
  background-color: var(--color-background-accent, #f0f);
  border: none;
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.create-first-btn:hover {
  opacity: 0.9;
}

.create-first-btn:active {
  transform: scale(0.98);
}

/* 滚动条样式 */
.template-list::-webkit-scrollbar {
  width: 6px;
}

.template-list::-webkit-scrollbar-track {
  background: transparent;
}

.template-list::-webkit-scrollbar-thumb {
  background: var(--color-border-default, #f0f);
  border-radius: 3px;
}

.template-list::-webkit-scrollbar-thumb:hover {
  background: var(--color-text-tertiary, #f0f);
}
</style>
