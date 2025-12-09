<script setup lang="ts">
import { onMounted, ref, computed, nextTick } from 'vue'
import type { ViewMetadata } from '@/types/drag'
import SimpleKanbanColumn from '@/components/assembles/tasks/kanban/SimpleKanbanColumn.vue'
import { useAreaStore } from '@/stores/area'
import { useTaskStore } from '@/stores/task'
import { controllerDebugState } from '@/infra/drag-interact'
import { logger, LogTags } from '@/infra/logging/logger'

// ==================== Stores ====================
const areaStore = useAreaStore()
const taskStore = useTaskStore()

// ==================== 配置常量 ====================
const KANBAN_WIDTH = 23 // 每个看板宽度（rem）
const KANBAN_GAP = 0 // 看板间隔（rem）

// ==================== 状态 ====================
const scrollContainer = ref<HTMLElement | null>(null)

// 拖动滚动状态（看板横向拖动）
const isDragging = ref(false)
const dragStartX = ref(0)
const dragStartScrollLeft = ref(0)

// ✅ 使用 interact.js 的全局拖动状态来检测任务卡片是否正在拖动
const isTaskDragging = computed(() => {
  // 当拖动状态不是 IDLE 时，说明有任务正在被拖动
  return controllerDebugState.value.phase !== 'IDLE'
})

// ==================== Area 看板系统 ====================
interface AreaKanban {
  id: string // Area ID，'no-area' 表示无区域
  areaName: string
  areaColor: string
  viewKey: string // misc::staging::${areaId} 或 misc::staging (无区域)
  isNoArea?: boolean // 是否为"无区域"看板
}

// 计算属性：基于 Areas 生成看板列表，始终包含"无区域"看板
const kanbans = computed(() => {
  // 首先添加"无区域"看板
  const noAreaKanban: AreaKanban = {
    id: 'no-area',
    areaName: '无区域',
    areaColor: 'var(--color-text-tertiary)',
    viewKey: 'misc::staging::no-area',
    isNoArea: true,
  }

  // 然后添加各个 Area 的看板
  const areaKanbans: AreaKanban[] = areaStore.allAreas.map((area) => ({
    id: area.id,
    areaName: area.name,
    areaColor: area.color,
    viewKey: `misc::staging::${area.id}`,
  }))

  const allKanbans = [noAreaKanban, ...areaKanbans]

  logger.debug(LogTags.COMPONENT_KANBAN, 'Generated area kanbans', {
    count: allKanbans.length,
    areas: allKanbans.map((k) => ({ id: k.id, name: k.areaName })),
  })

  return allKanbans
})

// 为每个看板生成 ViewMetadata
function getKanbanMetadata(kanban: AreaKanban): ViewMetadata {
  if (kanban.isNoArea) {
    return {
      type: 'misc',
      id: kanban.viewKey,
      config: {},
      label: '无区域 - Staging',
    }
  }
  return {
    type: 'area',
    id: kanban.viewKey,
    config: { areaId: kanban.id },
    label: `${kanban.areaName} - Staging`,
  }
}

// 计算轨道总宽度
const trackWidth = computed(() => {
  return kanbans.value.length * KANBAN_WIDTH + Math.max(0, kanbans.value.length - 1) * KANBAN_GAP
})

// ==================== 拖动滚动功能 ====================
function handleMouseDown(event: MouseEvent) {
  // 只处理左键
  if (event.button !== 0) return

  // ✅ 核心修复：检测鼠标是否在任务卡片上
  const target = event.target as HTMLElement

  // 如果点击的是任务卡片或其内部元素，不启动看板拖动
  if (target.closest('.task-draggable')) {
    return
  }

  // 如果点击的是其他可交互元素（输入框、按钮等），也不启动看板拖动
  if (
    target.closest('input') ||
    target.closest('button') ||
    target.closest('textarea') ||
    target.closest('select')
  ) {
    return
  }

  // ✅ 额外检查：如果任务卡片已经在拖动中（防抖阈值期间），也不启动看板拖动
  if (isTaskDragging.value) {
    return
  }

  isDragging.value = true
  dragStartX.value = event.pageX
  dragStartScrollLeft.value = scrollContainer.value?.scrollLeft || 0

  // 改变光标样式：按下时显示grab
  if (scrollContainer.value) {
    scrollContainer.value.style.cursor = 'grab'
    scrollContainer.value.style.userSelect = 'none'
  }
}

function handleMouseMove(event: MouseEvent) {
  // ✅ 如果任务正在拖动（通过 interact.js），立即停止看板拖动
  if (isTaskDragging.value && isDragging.value) {
    handleMouseUp()
    return
  }

  if (!isDragging.value || !scrollContainer.value) return

  event.preventDefault()

  // 开始拖动时，改变光标为grabbing
  if (scrollContainer.value.style.cursor !== 'grabbing') {
    scrollContainer.value.style.cursor = 'grabbing'
  }

  const deltaX = event.pageX - dragStartX.value
  scrollContainer.value.scrollLeft = dragStartScrollLeft.value - deltaX
}

function handleMouseUp() {
  if (!isDragging.value) return

  isDragging.value = false

  // 恢复光标样式为pointer
  if (scrollContainer.value) {
    scrollContainer.value.style.cursor = 'pointer'
    scrollContainer.value.style.userSelect = ''
  }
}

function handleMouseLeave() {
  if (isDragging.value) {
    handleMouseUp()
  }
}

// ==================== Props & Events ====================
const emit = defineEmits<{
  'kanban-count-change': [count: number]
}>()

// 暴露属性给父组件
defineExpose({
  kanbanCount: computed(() => kanbans.value.length),
})

// ==================== 初始化 ====================
onMounted(async () => {
  logger.info(LogTags.COMPONENT_KANBAN, 'Initializing area kanbans')

  // 确保 Areas 已加载
  if (areaStore.allAreas.length === 0) {
    await areaStore.fetchAll()
  }

  // 确保任务已加载
  await taskStore.fetchAllIncompleteTasks_DMA()

  // 发送看板数量变化事件
  emit('kanban-count-change', kanbans.value.length)

  logger.info(LogTags.COMPONENT_KANBAN, 'Area kanbans initialized', {
    areaCount: areaStore.allAreas.length,
    kanbanCount: kanbans.value.length,
    taskCount: taskStore.incompleteTasks.length,
  })
})

// 监听看板数量变化
const kanbanCountWatcher = computed(() => {
  const count = kanbans.value.length
  nextTick(() => {
    emit('kanban-count-change', count)
  })
  return count
})
kanbanCountWatcher.value // 触发初始计算
</script>

<template>
  <div
    ref="scrollContainer"
    class="area-kanban-scroll-container"
    @mousedown="handleMouseDown"
    @mousemove="handleMouseMove"
    @mouseup="handleMouseUp"
    @mouseleave="handleMouseLeave"
  >
    <div class="area-kanban-track" :style="{ width: `${trackWidth}rem` }">
      <SimpleKanbanColumn
        v-for="kanban in kanbans"
        :key="kanban.id"
        :title="kanban.areaName"
        :subtitle="kanban.isNoArea ? '待安排任务' : '待安排任务'"
        :view-key="kanban.viewKey"
        :view-metadata="getKanbanMetadata(kanban)"
        :show-add-input="true"
        :style="{
          width: `${KANBAN_WIDTH}rem`,
          flexShrink: 0,
        }"
      />
    </div>
  </div>
</template>

<style scoped>
.area-kanban-scroll-container {
  overflow: auto hidden;
  height: 100%;
  width: 100%;

  /* 关键：防止内容撑破容器 */
  min-width: 0;
  min-height: 0;

  /* 拖动滚动样式 */
  cursor: pointer;
  user-select: none;
}

.area-kanban-track {
  display: flex;
  justify-content: center; /* 居中排列 */
  gap: 0; /* 无间隔，由看板自身 padding 填补 */
  height: 100%;
  padding: 0 1rem;
  min-height: 100%;
  min-width: 100%; /* 确保至少占满容器宽度，使居中生效 */
}
</style>
