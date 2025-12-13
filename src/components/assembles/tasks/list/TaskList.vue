<template>
  <div class="task-bar" :class="taskBarClasses" ref="taskBarRef">
    <!-- 标题栏（可隐藏） -->
    <div
      v-if="!props.hideHeader"
      ref="headerRef"
      class="task-bar-header"
      :class="{ 'non-collapsible': !props.collapsible }"
      @click="props.collapsible ? toggleCollapse() : undefined"
    >
      <div class="header-left">
        <h3 class="task-bar-title" :style="titleStyle">{{ title }}</h3>
        <span class="task-count">{{ displayItems.length }}</span>
      </div>
      <div class="header-right">
        <!-- 标题栏操作按钮插槽 -->
        <slot name="title-actions" />
        <CuteIcon
          v-if="props.collapsible"
          name="ChevronDown"
          :size="16"
          class="collapse-icon"
          :class="{ rotated: isCollapsed }"
        />
      </div>
    </div>

    <!-- 内容区（可折叠） -->
    <div v-if="!isCollapsed" class="task-bar-content">
      <!-- 任务输入框 -->
      <div
        v-if="showAddInput"
        class="task-input-wrapper"
        :class="[`border-${props.inputBorderStyle}`, { focused: isInputFocused }]"
      >
        <input
          ref="taskInputRef"
          v-model="newTaskTitle"
          type="text"
          class="task-input"
          :placeholder="$t('task.action.addNewTask')"
          :disabled="isCreatingTask"
          @keydown.enter="addTask"
          @focus="isInputFocused = true"
          @blur="isInputFocused = false"
        />
        <button v-if="newTaskTitle && !isCreatingTask" class="add-task-btn" @click="addTask">
          <CuteIcon name="Plus" :size="16" />
        </button>
      </div>

      <!-- 副标题区域（当不显示输入框但有副标题时） -->
      <div
        v-else-if="subtitle"
        class="subtitle-wrapper"
        :class="`border-${props.inputBorderStyle}`"
      >
        <span class="subtitle-text">{{ subtitle }}</span>
      </div>

      <!-- 虚线分隔符（当不显示输入框、无副标题、但需要分隔符时） -->
      <div v-else-if="showDashedDivider" class="dashed-divider"></div>

      <!-- 任务纸条列表 -->
      <div ref="taskListRef" class="task-list-container">
        <TransitionGroup name="task-list" tag="div" class="task-list">
          <div
            v-for="task in displayItems"
            :key="task.id"
            :class="[
              'task-draggable',
              `task-draggable-${normalizedViewKey}`,
              {
                'is-preview': (task as any)._isPreview === true,
                'drag-compact': (task as any)._dragCompact === true,
                'fading-out': fadingTasks.has(task.id),
              },
            ]"
            :data-task-id="task.id"
          >
            <TaskStrip
              :task="task"
              :view-key="viewKey"
              :display-mode="displayMode"
              :show-estimated-duration="showEstimatedDuration"
              :read-only="props.readOnly"
              @toggle-complete="toggleTaskComplete(task.id)"
              @toggle-subtask="(subtaskId) => toggleSubtask(task.id, subtaskId)"
              @completing="onTaskCompleting"
            />
          </div>
          <div v-if="displayItems.length === 0" key="empty-state" class="empty-state">
            <p>{{ $t('task.label.noTasks') }}</p>
          </div>
        </TransitionGroup>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { ViewMetadata } from '@/types/drag'
import type { TaskCard } from '@/types/dtos'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import TaskStrip from './TaskStrip.vue'
import { useViewTasks } from '@/composables/useViewTasks'
import { useInteractDrag } from '@/composables/drag/useInteractDrag'
import { useDragStrategy } from '@/composables/drag/useDragStrategy'
import { dragPreviewState } from '@/infra/drag-interact'
import { deriveViewMetadata } from '@/services/viewAdapter'
import { pipeline } from '@/cpu'
import { logger, LogTags } from '@/infra/logging/logger'
import { useRecurrenceStore } from '@/stores/recurrence'
import { getTodayDateString } from '@/infra/utils/dateUtils'

interface Props {
  title: string
  viewKey: string // 🔥 必需：遵循 VIEW_CONTEXT_KEY_SPEC 规范
  subtitle?: string // 副标题（当 showAddInput=false 时显示，样式与输入框区域一致）
  defaultCollapsed?: boolean
  showAddInput?: boolean // 是否显示添加任务输入框
  showDashedDivider?: boolean // 是否显示虚线分隔符（当 showAddInput=false 时使用）
  fillRemainingSpace?: boolean // 是否占满父容器剩余空间
  collapsible?: boolean // 是否可折叠
  hideDailyRecurringTasks?: boolean // 是否隐藏每日循环任务
  hideCompleted?: boolean // 是否隐藏已完成任务
  disableDrag?: boolean // 禁用拖拽（只读展示）
  readOnly?: boolean // 禁用复选框/子任务勾选（只读展示）
  inputBorderStyle?: 'dashed' | 'solid' | 'none' // 输入框底部边框样式
  titleColor?: string // 标题颜色（CSS 颜色值或 CSS 变量）
  displayMode?: 'simple' | 'full' // 显示模式：简单/完整
  showEstimatedDuration?: boolean // 是否显示预期时间指示器
  hideHeader?: boolean // 是否隐藏标题栏（用于外部自定义标题）
}

const props = withDefaults(defineProps<Props>(), {
  defaultCollapsed: false,
  showAddInput: true,
  showDashedDivider: false,
  fillRemainingSpace: false,
  collapsible: true,
  hideDailyRecurringTasks: false,
  hideCompleted: false,
  disableDrag: false,
  readOnly: false,
  inputBorderStyle: 'dashed',
  titleColor: '',
  displayMode: 'full',
  showEstimatedDuration: true,
  hideHeader: false,
})

// Emits
const emit = defineEmits<{
  'add-task': [title: string]
}>()

// 🔥 使用 useViewTasks 获取任务数据
const { tasks } = useViewTasks(props.viewKey)

// 获取循环规则 store
const recurrenceStore = useRecurrenceStore()

// 🔥 淡出任务缓存：用于在任务消失后仍能显示淡出动画
// 利用 sort_positions 来保持正确的排序位置
interface FadingTask {
  task: TaskCard // 任务快照（包含 sort_positions）
}
const fadingTasks = ref<Map<string, FadingTask>>(new Map())

// 过滤任务：根据配置过滤已完成和每日循环任务
const filteredTasks = computed(() => {
  let result = [...tasks.value]

  // 1. 添加淡出任务（如果不在原始列表中）
  for (const [taskId, { task }] of fadingTasks.value) {
    if (!result.find((t) => t.id === taskId)) {
      result.push(task)
    }
  }

  // 2. 按 sort_positions 排序（利用现有排序系统保持位置）
  result.sort((a, b) => {
    const posA = a.sort_positions?.[props.viewKey] || ''
    const posB = b.sort_positions?.[props.viewKey] || ''
    return posA.localeCompare(posB)
  })

  // 3. 过滤已完成任务（但保留淡出中的任务）
  if (props.hideCompleted) {
    result = result.filter((task) => {
      // 如果任务未完成，保留
      if (!task.is_completed) return true
      // 如果任务正在淡出，也暂时保留
      if (fadingTasks.value.has(task.id)) return true
      // 其他已完成任务，过滤掉
      return false
    })
  }

  // 4. 过滤每日循环任务（但今天的不隐藏）
  if (props.hideDailyRecurringTasks) {
    // 从 viewKey 提取日期（格式：daily::YYYY-MM-DD）
    const viewDate = props.viewKey.startsWith('daily::') ? props.viewKey.split('::')[1] : null
    const today = getTodayDateString()
    const isToday = viewDate === today

    // 如果是今天，不过滤每日循环任务
    if (!isToday) {
      result = result.filter((task) => {
        // 如果任务没有循环规则，保留
        if (!task.recurrence_id) {
          return true
        }

        // 获取循环规则
        const recurrence = recurrenceStore.getRecurrenceById(task.recurrence_id)
        if (!recurrence) {
          return true // 如果找不到规则，保留任务（安全起见）
        }

        // 检查是否是每日循环（FREQ=DAILY）
        const isDailyRecurrence = recurrence.rule.includes('FREQ=DAILY')

        // 如果是每日循环，过滤掉（返回 false）；否则保留
        return !isDailyRecurrence
      })
    }
  }

  return result
})

// State
const isCollapsed = ref(props.defaultCollapsed)
const newTaskTitle = ref('')
const isCreatingTask = ref(false)
const taskBarRef = ref<HTMLElement | null>(null)
const taskListRef = ref<HTMLElement | null>(null)
const taskInputRef = ref<HTMLInputElement | null>(null)
const isInputFocused = ref(false)
const headerRef = ref<HTMLElement | null>(null)

// 暴露标题栏 ref 给父组件（用于 Section 拖拽）
defineExpose({
  headerRef,
  taskCount: computed(() => displayItems.value.length),
})

const taskBarClasses = computed(() => ({
  collapsed: isCollapsed.value,
  'fill-vertical': props.fillRemainingSpace && !isCollapsed.value,
}))

// 标题样式
const titleStyle = computed(() => {
  if (!props.titleColor) return {}
  return { color: props.titleColor }
})

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

const dragApi = (() => {
  if (props.disableDrag) return null
  return useInteractDrag({
    viewMetadata: effectiveViewMetadata,
    items: filteredTasks,
    containerRef: taskBarRef,
    draggableSelector: `.task-draggable-${normalizedViewKey.value}`,
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
})()

const displayItems = dragApi ? dragApi.displayItems : computed(() => filteredTasks.value)

// Methods
function toggleCollapse() {
  isCollapsed.value = !isCollapsed.value
}

// 🔥 处理任务完成事件：缓存任务快照并延迟消失
function onTaskCompleting(taskId: string) {
  // 找到任务
  const task = tasks.value.find((t) => t.id === taskId)

  if (task) {
    // 缓存任务快照（包含 sort_positions，用于保持排序位置）
    const newMap = new Map(fadingTasks.value)
    newMap.set(taskId, {
      task: { ...task, is_completed: true },
    })
    fadingTasks.value = newMap
  }

  // 延迟后从缓存中移除，任务会自然消失
  setTimeout(() => {
    const newMap = new Map(fadingTasks.value)
    newMap.delete(taskId)
    fadingTasks.value = newMap
  }, 800)
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
        // misc::staging::no-area - 无区域的 staging 任务（不设置 area_id）
        // misc::staging::no-project - 无项目的 staging 任务（不设置 project_id）
        // misc::staging::recent-carryover - 最近结转任务（不设置任何上下文）
        // misc::staging::project::${projectId} - 指定项目的 staging 任务
        // misc::staging::${areaId} - 指定 area 的 staging 任务
        // misc::staging::${areaId}::no-project - 指定区域的无项目 staging 任务
        // misc::staging::${areaId}::project::${projectId} - 指定区域的指定项目 staging 任务
        const fourthPart = parts[3]
        const fifthPart = parts[4]

        if (
          thirdPart === 'no-area' ||
          thirdPart === 'no-project' ||
          thirdPart === 'recent-carryover'
        ) {
          // 不设置任何上下文，创建普通 staging 任务
          logger.debug(LogTags.COMPONENT_TASK_BAR, 'Creating staging task without context', {
            viewKey: props.viewKey,
          })
        } else if (thirdPart === 'project' && fourthPart) {
          // misc::staging::project::${projectId}
          taskData.project_id = fourthPart
          logger.debug(LogTags.COMPONENT_TASK_BAR, 'Creating task with project context', {
            projectId: fourthPart,
            viewKey: props.viewKey,
          })
        } else if (fourthPart === 'no-project') {
          // misc::staging::${areaId}::no-project - 指定区域的无项目任务
          taskData.area_id = thirdPart
          logger.debug(LogTags.COMPONENT_TASK_BAR, 'Creating task with area context (no project)', {
            areaId: thirdPart,
            viewKey: props.viewKey,
          })
        } else if (fourthPart === 'project' && fifthPart) {
          // misc::staging::${areaId}::project::${projectId} - 指定区域的指定项目任务
          taskData.area_id = thirdPart
          taskData.project_id = fifthPart
          logger.debug(LogTags.COMPONENT_TASK_BAR, 'Creating task with area and project context', {
            areaId: thirdPart,
            projectId: fifthPart,
            viewKey: props.viewKey,
          })
        } else {
          // misc::staging::${areaId} - 指定 area 的 staging 任务
          taskData.area_id = thirdPart
          logger.debug(LogTags.COMPONENT_TASK_BAR, 'Creating task with area context', {
            areaId: thirdPart,
            viewKey: props.viewKey,
          })
        }
      } else if (type === 'area' && identifier) {
        // area::${areaId} - 指定 area 的所有任务
        taskData.area_id = identifier
        logger.debug(LogTags.COMPONENT_TASK_BAR, 'Creating task with area context', {
          areaId: identifier,
          viewKey: props.viewKey,
        })
      } else if (type === 'project' && identifier) {
        // project::${projectId}::section::${sectionId} - 指定章节的任务
        // project::${projectId}::section::all - 项目无分类任务
        // project::${projectId} - 指定项目的任务
        taskData.project_id = identifier

        if (thirdPart === 'section' && parts[3]) {
          const sectionId = parts[3]
          if (sectionId !== 'all') {
            taskData.section_id = sectionId
            logger.debug(LogTags.COMPONENT_TASK_BAR, 'Creating task with project section context', {
              projectId: identifier,
              sectionId,
              viewKey: props.viewKey,
            })
          } else {
            logger.debug(
              LogTags.COMPONENT_TASK_BAR,
              'Creating task with project (no section) context',
              {
                projectId: identifier,
                viewKey: props.viewKey,
              }
            )
          }
        } else {
          logger.debug(LogTags.COMPONENT_TASK_BAR, 'Creating task with project context', {
            projectId: identifier,
            viewKey: props.viewKey,
          })
        }
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
    // 重新聚焦到输入框
    setTimeout(() => {
      taskInputRef.value?.focus()
    }, 0)
  }
}

async function toggleTaskComplete(taskId: string) {
  try {
    if (props.readOnly) return
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
    if (props.readOnly) return
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
  margin-bottom: 0;
  padding-bottom: 1.6rem;
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
  background-color: var(--color-overlay-light);
}

/* 不可折叠的标题栏 */
.task-bar-header.non-collapsible {
  cursor: default;
}

.task-bar-header.non-collapsible:hover {
  background-color: transparent;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 0.4rem;
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
  font-weight: 500;
  color: var(--color-text-primary);
  margin: 0;
  line-height: 1.4; /* 固定行高，避免中英文高度差异 */
}

.task-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2rem;
  height: 2rem;
  padding: 0 0.6rem;
  font-size: 1.2rem;
  font-weight: 400;
  line-height: 1; /* 固定行高 */
  color: var(--color-text-secondary);
  background-color: var(--color-background-secondary);
  border-radius: 1rem;
}

/* 内容区 */
.task-bar-content {
  padding: 0;
}

/* 任务输入框 */
.task-input-wrapper {
  position: relative;
  margin: 0 1.6rem 1rem; /* 左右 margin 与标题 padding 对齐 */
}

/* 边框样式变体 */
.task-input-wrapper.border-dashed {
  border-bottom: 2px dashed var(--color-input-underline, #f0f);
}

.task-input-wrapper.border-solid {
  border-bottom: 2px solid var(--color-input-underline, #f0f);
}

.task-input-wrapper.border-none {
  border-bottom: none;
}

.task-input {
  width: 100%;
  padding: 0.8rem 0; /* 移除左右 padding，由 wrapper 的 margin 控制对齐 */
  padding-right: 3.4rem; /* 为按钮留空间 */
  font-size: 1.5rem;
  line-height: 1.4; /* 固定行高，避免中英文高度差异 */
  color: var(--color-text-primary, #f0f);
  background-color: transparent;
  border: none;
  border-radius: 0;
  outline: none;
  transition: all 0.2s ease;
  box-sizing: border-box;
}

.task-input::placeholder {
  color: var(--color-text-tertiary, #f0f);
}

.task-input:focus {
  background-color: var(--color-background-input-focus, #f0f);
}

.task-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.add-task-btn {
  position: absolute;
  right: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3rem;
  height: 3rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: transparent;
  color: var(--color-text-accent, #f0f);
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.add-task-btn:hover {
  background-color: var(--color-background-accent-light, #f0f);
}

.add-task-btn:active {
  transform: translateY(-50%) scale(0.95);
}

/* 副标题区域（与输入框区域高度一致） */
.subtitle-wrapper {
  position: relative;
  margin: 0 1.6rem 1rem; /* 与 task-input-wrapper 一致 */
  padding: 0.8rem 0; /* 与 task-input 一致 */
}

.subtitle-wrapper.border-dashed {
  border-bottom: 2px dashed var(--color-input-underline, #f0f);
}

.subtitle-wrapper.border-solid {
  border-bottom: 2px solid var(--color-input-underline, #f0f);
}

.subtitle-wrapper.border-none {
  border-bottom: none;
}

.subtitle-text {
  font-size: 1.5rem;
  line-height: 1.4; /* 与 task-input 一致 */
  color: var(--color-text-tertiary, #f0f);
  font-style: italic;
}

/* 虚线分隔符（高度与输入框区域一致，确保分割线对齐） */
.dashed-divider {
  margin: 0 1.6rem 1rem; /* 与 task-input-wrapper 一致 */
  padding: 0.8rem 0; /* 与 task-input 一致 */
  min-height: calc(1.5rem * 1.4); /* 与 task-input 文字高度一致 (font-size * line-height) */
  box-sizing: border-box;
  border-bottom: 2px dashed var(--color-input-underline, #f0f);
}

/* 任务列表容器（拖放接收区） */
.task-list-container {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

/* 任务列表 */
.task-list {
  display: flex;
  flex-direction: column;
  position: relative;
}

.task-bar.fill-vertical {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

.task-bar.fill-vertical .task-bar-content {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

.task-bar.fill-vertical .task-list {
  flex: 1;
  min-height: 0;
}

/* 任务列表动画 */
.task-list-move {
  transition: transform 0.15s cubic-bezier(0.4, 0, 0.2, 1);
  will-change: transform;
  backface-visibility: hidden;
  contain: paint;
}

.task-list-enter-active {
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
}

.task-list-leave-active {
  display: none;
}

.task-list-enter-from {
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
  line-height: 2.35;

  /* 确保与 task-strip 的最小高度一致 */

  /* task-strip: padding 0.8rem + checkbox/title 2.1rem + padding 0.8rem = 3.7rem */

  /* empty-state: padding 0.8rem + text (1.4rem * 1.5 = 2.1rem) + padding 0.8rem = 3.7rem */
}

/* 折叠状态 */
.task-bar.collapsed .task-bar-content {
  display: none;
}
</style>
