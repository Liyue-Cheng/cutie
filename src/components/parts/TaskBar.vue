<template>
  <div class="task-bar" :class="{ collapsed: isCollapsed }" ref="taskBarRef">
    <!-- 标题栏（可点击折叠） -->
    <div class="task-bar-header" @click="toggleCollapse">
      <div class="header-left">
        <h3 class="task-bar-title">{{ title }}</h3>
        <span class="task-count">{{ displayItems.length }}</span>
      </div>
      <CuteIcon
        name="ChevronDown"
        :size="16"
        class="collapse-icon"
        :class="{ rotated: isCollapsed }"
      />
    </div>

    <!-- 内容区（可折叠） -->
    <div v-if="!isCollapsed" class="task-bar-content">
      <!-- 任务输入框 -->
      <div v-if="showAddInput" class="task-input-wrapper">
        <input
          v-model="newTaskTitle"
          type="text"
          class="task-input"
          placeholder="添加新任务..."
          :disabled="isCreatingTask"
          @keydown.enter="addTask"
        />
        <button v-if="newTaskTitle && !isCreatingTask" class="add-task-btn" @click="addTask">
          <CuteIcon name="Plus" :size="16" />
        </button>
      </div>

      <!-- 任务纸条列表 -->
      <TransitionGroup name="task-list" tag="div" class="task-list">
        <div
          v-for="task in displayItems"
          :key="task.id"
          :class="['task-card-wrapper', `task-strip-wrapper-${normalizedViewKey}`]"
          :data-task-id="task.id"
        >
          <TaskStrip
            :task="task"
            :view-key="viewKey"
            @toggle-complete="toggleTaskComplete(task.id)"
            @toggle-subtask="(subtaskId) => toggleSubtask(task.id, subtaskId)"
          />
        </div>
        <div v-if="displayItems.length === 0" key="empty-state" class="empty-state">
          <p>暂无任务</p>
        </div>
      </TransitionGroup>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { ViewMetadata } from '@/types/drag'
import CuteIcon from './CuteIcon.vue'
import TaskStrip from './TaskStrip.vue'
import { useViewTasks } from '@/composables/useViewTasks'
import { useInteractDrag } from '@/composables/drag/useInteractDrag'
import { useDragStrategy } from '@/composables/drag/useDragStrategy'
import { dragPreviewState } from '@/infra/drag-interact'
import { deriveViewMetadata } from '@/services/viewAdapter'
import { pipeline } from '@/cpu'
import { logger, LogTags } from '@/infra/logging/logger'

interface Props {
  title: string
  viewKey: string // 🔥 必需：遵循 VIEW_CONTEXT_KEY_SPEC 规范
  defaultCollapsed?: boolean
  showAddInput?: boolean // 是否显示添加任务输入框
}

const props = withDefaults(defineProps<Props>(), {
  defaultCollapsed: false,
  showAddInput: true,
})

// Emits
const emit = defineEmits<{
  'add-task': [title: string]
}>()

// 🔥 使用 useViewTasks 获取任务数据
const { tasks } = useViewTasks(props.viewKey)

// State
const isCollapsed = ref(props.defaultCollapsed)
const newTaskTitle = ref('')
const isCreatingTask = ref(false)
const taskBarRef = ref<HTMLElement | null>(null)

// ==================== ViewMetadata 推导 ====================
const effectiveViewMetadata = computed<ViewMetadata>(() => {
  const derived = deriveViewMetadata(props.viewKey)
  if (derived) {
    return derived
  }

  // 兜底：提供最小可用元数据
  return {
    id: props.viewKey,
    type: 'custom',
    label: props.title,
    config: {},
  } as ViewMetadata
})

// ==================== 拖放系统集成 ====================
const dragStrategy = useDragStrategy()

// 标准化 viewKey 作为 CSS class（:: 替换为 --）
const normalizedViewKey = computed(() => props.viewKey.replace(/::/g, '--'))

const { displayItems } = useInteractDrag({
  viewMetadata: effectiveViewMetadata,
  items: tasks,
  containerRef: taskBarRef,
  draggableSelector: `.task-strip-wrapper-${normalizedViewKey.value}`,
  objectType: 'task',
  getObjectId: (task) => task.id,
  onDrop: async (session) => {
    logger.debug(LogTags.COMPONENT_TASK_BAR, 'TaskBar drop event', {
      session,
      targetViewKey: props.viewKey,
      displayItems: displayItems.value.length,
      dropIndex: dragPreviewState.value?.computed.dropIndex,
    })

    // 🎯 执行拖放策略
    const result = await dragStrategy.executeDrop(session, props.viewKey, {
      sourceContext: (session.metadata?.sourceContext as Record<string, any>) || {},
      targetContext: {
        taskIds: displayItems.value.map((t) => t.id),
        displayTasks: displayItems.value,
        dropIndex: dragPreviewState.value?.computed.dropIndex,
        viewKey: props.viewKey,
      },
    })

    if (!result.success) {
      const errorMessage = result.message || result.error || 'Unknown error'
      logger.error(LogTags.COMPONENT_TASK_BAR, 'TaskBar drop failed', new Error(errorMessage), {
        result,
        session,
      })
    } else {
      logger.info(LogTags.COMPONENT_TASK_BAR, 'TaskBar drop succeeded', {
        taskId: session.object.id,
        targetViewKey: props.viewKey,
      })
    }
  },
})

// Methods
function toggleCollapse() {
  isCollapsed.value = !isCollapsed.value
}

async function addTask() {
  const title = newTaskTitle.value.trim()
  if (!title || isCreatingTask.value) return

  isCreatingTask.value = true
  newTaskTitle.value = ''

  try {
    // 解析 viewKey
    const parts = props.viewKey.split('::')
    const [type, identifier, thirdPart] = parts

    if (type === 'daily' && identifier) {
      // 日期视图：使用合并端点一次性创建任务并添加日程
      logger.info(LogTags.COMPONENT_TASK_BAR, 'Creating task with schedule', {
        title,
        date: identifier,
        viewKey: props.viewKey,
      })

      await pipeline.dispatch('task.create_with_schedule', {
        title,
        estimated_duration: 60, // 默认 60 分钟
        scheduled_day: identifier, // YYYY-MM-DD
      })
    } else {
      // 非日期视图：只创建任务，需要根据 viewKey 提取上下文信息
      const taskData: any = {
        title,
        estimated_duration: 60, // 默认 60 分钟
      }

      // 🔥 根据 viewKey 提取上下文信息
      if (type === 'misc' && identifier === 'staging' && thirdPart) {
        // misc::staging::${areaId} - 指定 area 的 staging 任务
        taskData.area_id = thirdPart
        logger.debug(LogTags.COMPONENT_TASK_BAR, 'Creating task with area context', {
          areaId: thirdPart,
          viewKey: props.viewKey,
        })
      } else if (type === 'area' && identifier) {
        // area::${areaId} - 指定 area 的所有任务
        taskData.area_id = identifier
        logger.debug(LogTags.COMPONENT_TASK_BAR, 'Creating task with area context', {
          areaId: identifier,
          viewKey: props.viewKey,
        })
      } else if (type === 'project' && identifier) {
        // project::${projectId} - 指定项目的任务
        taskData.project_id = identifier
        logger.debug(LogTags.COMPONENT_TASK_BAR, 'Creating task with project context', {
          projectId: identifier,
          viewKey: props.viewKey,
        })
      }

      logger.info(LogTags.COMPONENT_TASK_BAR, 'Creating task', {
        title,
        viewKey: props.viewKey,
        taskData,
      })

      await pipeline.dispatch('task.create', taskData)
    }

    emit('add-task', title)
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_TASK_BAR,
      'Failed to create task',
      error instanceof Error ? error : new Error(String(error)),
      { title, viewKey: props.viewKey }
    )
  } finally {
    isCreatingTask.value = false
  }
}

async function toggleTaskComplete(taskId: string) {
  try {
    // 获取当前任务的完成状态
    const task = displayItems.value.find((t) => t.id === taskId)
    if (!task) return

    logger.info(LogTags.COMPONENT_TASK_BAR, 'Toggling task completion', {
      taskId,
      currentStatus: task.is_completed,
      viewKey: props.viewKey,
    })

    if (task.is_completed) {
      // 重新打开任务
      await pipeline.dispatch('task.reopen', { id: taskId })
    } else {
      // 完成任务
      await pipeline.dispatch('task.complete', { id: taskId })
    }

    logger.info(LogTags.COMPONENT_TASK_BAR, 'Task completion toggled', {
      taskId,
      newStatus: !task.is_completed,
    })
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_TASK_BAR,
      'Failed to toggle task completion',
      error instanceof Error ? error : new Error(String(error)),
      { taskId, viewKey: props.viewKey }
    )
  }
}

async function toggleSubtask(taskId: string, subtaskId: string) {
  try {
    // 获取当前任务
    const task = displayItems.value.find((t) => t.id === taskId)
    if (!task || !task.subtasks) return

    // 找到要切换的子任务
    const subtask = task.subtasks.find((st) => st.id === subtaskId)
    if (!subtask) return

    logger.info(LogTags.COMPONENT_TASK_BAR, 'Toggling subtask completion', {
      taskId,
      subtaskId,
      currentStatus: subtask.is_completed,
      viewKey: props.viewKey,
    })

    // 更新子任务状态
    const updatedSubtasks = task.subtasks.map((st) =>
      st.id === subtaskId ? { ...st, is_completed: !st.is_completed } : st
    )

    // 使用 pipeline 更新任务
    await pipeline.dispatch('task.update', {
      id: taskId,
      updates: { subtasks: updatedSubtasks },
    })

    logger.info(LogTags.COMPONENT_TASK_BAR, 'Subtask completion toggled', {
      taskId,
      subtaskId,
      newStatus: !subtask.is_completed,
    })
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_TASK_BAR,
      'Failed to toggle subtask completion',
      error instanceof Error ? error : new Error(String(error)),
      { taskId, subtaskId, viewKey: props.viewKey }
    )
  }
}
</script>

<style scoped>
.task-bar {
  background-color: transparent;
  margin-bottom: 1.6rem;
}

/* 标题栏 */
.task-bar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.6rem;
  cursor: pointer;
  user-select: none;
  transition: background-color 0.2s;
  border-radius: 0.6rem;
}

.task-bar-header:hover {
  background-color: rgb(0 0 0 / 3%);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.collapse-icon {
  color: var(--color-text-secondary);
  transition: transform 0.2s ease;
}

.collapse-icon.rotated {
  transform: rotate(-90deg);
}

.task-bar-title {
  font-size: 1.6rem;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.task-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2rem;
  height: 2rem;
  padding: 0 0.6rem;
  font-size: 1.2rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  background-color: var(--color-background-secondary, #f5f5f5);
  border-radius: 1rem;
}

/* 内容区 */
.task-bar-content {
  padding: 0;
}

/* 任务输入框 */
.task-input-wrapper {
  position: relative;
  margin-bottom: 0;
  border-bottom: 2px dashed rgb(0 0 0 / 15%);
}

.task-input {
  width: 100%;
  padding: 1.2rem 1.6rem;
  padding-right: 4rem;
  font-size: 1.4rem;
  color: var(--color-text-primary);
  background-color: transparent;
  border: none;
  border-radius: 0;
  outline: none;
  transition: all 0.2s ease;
  box-sizing: border-box;
}

.task-input::placeholder {
  color: var(--color-text-tertiary);
}

.task-input:focus {
  background-color: var(--color-background-hover, rgb(0 0 0 / 2%));
}

.task-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.add-task-btn {
  position: absolute;
  right: 0.6rem;
  top: 50%;
  transform: translateY(-50%);
  width: 3rem;
  height: 3rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-primary, #4a90e2);
  color: white;
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.add-task-btn:hover {
  background-color: var(--color-primary-hover, #357abd);
}

.add-task-btn:active {
  transform: translateY(-50%) scale(0.95);
}

/* 任务列表 */
.task-list {
  display: flex;
  flex-direction: column;
  position: relative;
}

/* 任务列表动画 */
.task-list-move {
  transition: transform 0.15s cubic-bezier(0.4, 0, 0.2, 1);
}

.task-list-enter-active {
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
}

.task-list-leave-active {
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
  position: absolute;
  width: 100%;
}

.task-list-enter-from {
  opacity: 0;
  transform: translateY(-10px);
}

.task-list-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

/* 空状态 */
.empty-state {
  padding: 0.8rem 1.6rem;
  text-align: center;
}

.empty-state p {
  font-size: 1.4rem;
  color: var(--color-text-tertiary);
  margin: 0;
}

/* 折叠状态 */
.task-bar.collapsed .task-bar-content {
  display: none;
}
</style>
