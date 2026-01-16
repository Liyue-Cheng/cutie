<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import CuteDualModeCheckbox from '@/components/parts/CuteDualModeCheckbox.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import { pipeline } from '@/cpu'
import { logger, LogTags } from '@/infra/logging/logger'

export type CheckboxState = null | 'completed' | 'present'

interface Props {
  taskId?: string
  title: string
  scheduleDay?: string
  scheduleOutcome?: string | null
  isCompleted?: boolean
  isPreview?: boolean
  isRecurring?: boolean
  hasDueFlag?: boolean
  isDueOverdue?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  taskId: undefined,
  scheduleDay: undefined,
  scheduleOutcome: null,
  isCompleted: false,
  isPreview: false,
  isRecurring: false,
  hasDueFlag: false,
  isDueOverdue: false,
})

const derivedState = () => {
  if (props.isCompleted) {
    return 'completed' as CheckboxState
  }
  if (props.scheduleOutcome === 'presence_logged') {
    return 'present' as CheckboxState
  }
  return null
}

const localState = ref<CheckboxState>(derivedState())

watch(
  () => [props.isCompleted, props.scheduleOutcome],
  () => {
    localState.value = derivedState()
  }
)

const isInteractive = computed(() => !props.isPreview && !!props.taskId && !!props.scheduleDay)
const checkboxInteractionKey = computed(() => {
  if (!props.taskId) return undefined
  const scheduleDay = props.scheduleDay
  return scheduleDay ? `month-task::${props.taskId}::${scheduleDay}` : `month-task::${props.taskId}`
})

const titleClass = computed(() => ({
  completed: localState.value === 'completed',
}))

const showRecurringIcon = computed(() => Boolean(props.isRecurring))
const showDueFlag = computed(() => Boolean(props.hasDueFlag))
const dueFlagColor = computed(() => (props.isDueOverdue ? '#ef4444' : '#9ca3af'))

async function updateTaskCompleted(taskId: string, completed: boolean) {
  if (completed) {
    await pipeline.dispatch('task.complete', { id: taskId })
  } else {
    await pipeline.dispatch('task.reopen', { id: taskId })
  }
}

async function updateScheduleOutcome(
  taskId: string,
  scheduleDay: string,
  outcome: 'PRESENCE_LOGGED' | 'PLANNED'
) {
  await pipeline.dispatch('schedule.update', {
    task_id: taskId,
    scheduled_day: scheduleDay,
    updates: { outcome },
  })
}

async function handleStateChange(nextState: CheckboxState) {
  if (!isInteractive.value || !props.taskId || !props.scheduleDay) {
    return
  }

  const previousState = localState.value
  if (previousState === nextState) {
    return
  }

  localState.value = nextState

  try {
    if (nextState === 'completed') {
      await updateTaskCompleted(props.taskId, true)
    } else if (nextState === 'present') {
      await updateScheduleOutcome(props.taskId, props.scheduleDay, 'PRESENCE_LOGGED')
    } else {
      if (previousState === 'completed') {
        await updateTaskCompleted(props.taskId, false)
      } else if (previousState === 'present') {
        await updateScheduleOutcome(props.taskId, props.scheduleDay, 'PLANNED')
      }
    }
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_CALENDAR,
      'Failed to update calendar task checkbox state',
      error instanceof Error ? error : new Error(String(error))
    )
    localState.value = previousState
  }
}
</script>

<template>
  <div class="calendar-task-event-content" :class="{ 'is-preview': !isInteractive }">
    <!--
      🔒 checkbox-wrapper 隔离层（故意不加任何 CSS）
      
      原理：空的 <div> 默认是 display: block，会创建块级格式化上下文（BFC），
      天然隔离内部 inline-flex checkbox 的行框影响（baseline、line-height 等），
      防止 checkbox 状态变化时导致外层容器高度抖动。
      
      ⚠️ 不要给这个 div 加 display: flex！flex 容器反而会让子元素的布局特性传递出去。
    -->
    <div class="checkbox-wrapper">
      <CuteDualModeCheckbox
        class="calendar-task-checkbox"
        size="1.6rem"
        :state="localState"
        :interaction-key="checkboxInteractionKey"
        @update:state="handleStateChange"
        @click.stop
      />
    </div>
    <span class="calendar-task-title" :class="titleClass">{{ title }}</span>
    <div class="calendar-task-icons">
      <CuteIcon
        v-if="showRecurringIcon"
        name="RefreshCcw"
        :size="13"
        class="calendar-task-recurring-icon"
      />
      <CuteIcon
        v-if="showDueFlag"
        name="Flag"
        :size="14"
        :color="dueFlagColor"
        class="calendar-task-due-icon"
      />
    </div>
  </div>
</template>

<style scoped>
.calendar-task-event-content {
  display: inline-flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.1rem 0.2rem 0.1rem 0.1rem;
  width: 100%;
  box-sizing: border-box;
  pointer-events: auto;
}

.calendar-task-event-content.is-preview {
  pointer-events: none;
  opacity: 0.8;
}

.calendar-task-checkbox {
  flex: 0 0 auto;
}

.calendar-task-title {
  flex: 1 1 auto;
  font-size: 1.2rem;
  line-height: 1.4rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.calendar-task-title.completed {
  text-decoration: line-through;
  opacity: 0.65;
}

.calendar-task-icons {
  flex: 0 0 auto;
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
}

.calendar-task-recurring-icon {
  color: var(--color-text-secondary, #f0f);
}

.calendar-task-due-icon {
  color: var(--color-text-secondary, #f0f);
}
</style>
