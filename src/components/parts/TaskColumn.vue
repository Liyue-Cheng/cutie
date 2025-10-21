<script setup lang="ts">
import { ref, computed } from 'vue'
import type { ViewMetadata } from '@/types/drag'
import { useViewTasks } from '@/composables/useViewTasks'
import { deriveViewMetadata } from '@/services/viewAdapter'
import KanbanTaskCard from './kanban/KanbanTaskCard.vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'
import { useInteractDrag } from '@/composables/drag/useInteractDrag'
import { useDragStrategy } from '@/composables/drag/useDragStrategy'

const props = defineProps<{
  viewKey: string // 必需：视图唯一标识
  viewMetadata?: ViewMetadata // 可选：可自动推导
  placeholder?: string // 输入框占位符
}>()

// ==================== 数据源管理 ====================
const { tasks: effectiveTasks } = useViewTasks(props.viewKey)

// 统一的 ViewMetadata
const effectiveViewMetadata = computed<ViewMetadata>(() => {
  if (props.viewMetadata) {
    return props.viewMetadata
  }

  const derived = deriveViewMetadata(props.viewKey)
  if (derived) {
    return derived
  }

  // 兜底：提供最小可用元数据
  return {
    id: props.viewKey,
    type: 'custom',
    label: 'Tasks',
    config: {},
  } as ViewMetadata
})

// ==================== 添加任务 ====================
const newTaskTitle = ref('')

async function handleAddTask() {
  const title = newTaskTitle.value.trim()
  if (!title) return

  logger.info(LogTags.COMPONENT_KANBAN, 'Adding task to view', {
    viewKey: props.viewKey,
    title,
  })

  try {
    // 检查是否是日期视图（daily::YYYY-MM-DD）
    const viewMetadata = effectiveViewMetadata.value
    const isDateView = viewMetadata.type === 'date'

    if (isDateView) {
      // 日期视图：使用合并端点一次性创建任务并添加日程
      const dateConfig = viewMetadata.config as import('@/types/drag').DateViewConfig
      const date = dateConfig.date // YYYY-MM-DD

      await pipeline.dispatch('task.create_with_schedule', {
        title,
        estimated_duration: 60, // 默认 60 分钟
        scheduled_day: date,
      })

      logger.info(LogTags.COMPONENT_KANBAN, 'Task added with schedule', {
        viewKey: props.viewKey,
        title,
        date,
      })
    } else {
      // 非日期视图：只创建任务
      await pipeline.dispatch('task.create', {
        title,
      })

      logger.info(LogTags.COMPONENT_KANBAN, 'Task added successfully', {
        viewKey: props.viewKey,
        title,
      })
    }

    // 清空输入框
    newTaskTitle.value = ''
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_KANBAN,
      'Failed to add task',
      error instanceof Error ? error : new Error(String(error))
    )
  }
}

// ==================== 拖放系统 ====================
const taskColumnRef = ref<HTMLElement | null>(null)
const dragStrategy = useDragStrategy()

const { displayItems } = useInteractDrag({
  viewMetadata: effectiveViewMetadata,
  items: effectiveTasks,
  containerRef: taskColumnRef,
  draggableSelector: `.task-card-wrapper-${props.viewKey.replace(/::/g, '--')}`,
  objectType: 'task',
  getObjectId: (task) => task.id,
  onDrop: async (session) => {
    // 执行拖放策略
    const result = await dragStrategy.executeDrop(session, props.viewKey, {
      sourceContext: (session.metadata?.sourceContext as Record<string, any>) || {},
      targetContext: {
        taskIds: displayItems.value.map((t) => t.id),
        displayTasks: displayItems.value,
      },
    })

    if (!result.success) {
      logger.error(
        LogTags.COMPONENT_KANBAN,
        'Drop failed',
        new Error(result.error || 'Unknown error')
      )
    }
  },
})
</script>

<template>
  <div class="task-column" ref="taskColumnRef">
    <!-- 顶部输入框 -->
    <div class="input-section">
      <input
        v-model="newTaskTitle"
        type="text"
        class="task-input"
        :placeholder="placeholder || '添加任务...'"
        @keydown.enter="handleAddTask"
      />
    </div>

    <!-- 任务列表 -->
    <div class="task-list">
      <div
        v-for="task in displayItems"
        :key="task.id"
        :class="`task-card-wrapper-${viewKey.replace(/::/g, '--')}`"
        :data-task-id="task.id"
      >
        <KanbanTaskCard :task="task" :view-key="viewKey" :view-metadata="effectiveViewMetadata" />
      </div>

      <!-- 空状态 -->
      <div v-if="displayItems.length === 0" class="empty-state">
        <p>暂无任务</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.task-column {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  overflow: hidden;
}

/* ==================== 输入框区域 ==================== */
.input-section {
  flex-shrink: 0;
  padding: 1.2rem 1.6rem;
}

.task-input {
  width: 100%;
  padding: 1rem 1.2rem;
  font-size: 1.4rem;
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  background-color: var(--color-background-primary);
  color: var(--color-text-primary);
  transition: all 0.2s ease;
}

.task-input::placeholder {
  color: var(--color-text-tertiary);
}

.task-input:focus {
  outline: none;
  border-color: var(--rose-pine-foam, #56949f);
  box-shadow: 0 0 0 3px rgb(86 148 159 / 10%);
}

/* ==================== 任务列表 ==================== */
.task-list {
  flex: 1;
  overflow-y: auto;
  padding: 1.2rem 1.6rem;
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

/* 任务卡片包装器 */
.task-list > [class^='task-card-wrapper-'] {
  flex-shrink: 0;
}

/* ==================== 空状态 ==================== */
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4rem 2rem;
  color: var(--color-text-tertiary);
}

.empty-state p {
  margin: 0;
  font-size: 1.4rem;
  text-align: center;
}

/* ==================== 滚动条样式 ==================== */
.task-list::-webkit-scrollbar {
  width: 8px;
}

.task-list::-webkit-scrollbar-track {
  background: transparent;
}

.task-list::-webkit-scrollbar-thumb {
  background-color: rgb(0 0 0 / 20%);
  border-radius: 4px;
  transition: background-color 0.2s;
}

.task-list::-webkit-scrollbar-thumb:hover {
  background-color: rgb(0 0 0 / 30%);
}
</style>
