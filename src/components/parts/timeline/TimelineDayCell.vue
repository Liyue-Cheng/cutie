<script setup lang="ts">
import { computed, onMounted, onBeforeUnmount, ref } from 'vue'
import type { TaskCard, TimeBlockView } from '@/types/dtos'
import CalendarTaskEventContent from '@/components/parts/calendar/CalendarTaskEventContent.vue'
import CalendarDueDateEventContent from '@/components/parts/calendar/CalendarDueDateEventContent.vue'
import CalendarTimeBlockEventContent from '@/components/parts/calendar/CalendarTimeBlockEventContent.vue'
import { useUIStore } from '@/stores/ui'
import { useContextMenu } from '@/composables/useContextMenu'
import KanbanTaskCardMenu from '@/components/parts/kanban/KanbanTaskCardMenu.vue'
import CalendarEventMenu from '@/components/parts/CalendarEventMenu.vue'
import { interactManager } from '@/infra/drag-interact'
import { useDragStrategy } from '@/composables/drag/useDragStrategy'
import type { DragSession } from '@/infra/drag-interact/types'
import { logger, LogTags } from '@/infra/logging/logger'

interface Props {
  date: string // YYYY-MM-DD
  dayNumber: number
  tasks: TaskCard[]
  dueDates: TaskCard[]
  allDayEvents: TimeBlockView[]
  isToday: boolean
  isWeekend: boolean
}

const props = defineProps<Props>()

const uiStore = useUIStore()
const contextMenu = useContextMenu()
const dragStrategy = useDragStrategy()

const cellRef = ref<HTMLElement | null>(null)

const hasContent = computed(() => {
  return props.tasks.length > 0 || props.dueDates.length > 0 || props.allDayEvents.length > 0
})

function handleTaskClick(taskId: string) {
  uiStore.openEditor(taskId, `daily::${props.date}`)
}

function handleTaskContextMenu(event: MouseEvent, task: TaskCard) {
  event.preventDefault()
  event.stopPropagation()
  contextMenu.show(KanbanTaskCardMenu, { task, viewKey: `daily::${props.date}` }, event)
}

function handleDueDateClick(taskId: string) {
  uiStore.openEditor(taskId, `daily::${props.date}`)
}

function handleEventContextMenu(event: MouseEvent, timeBlock: TimeBlockView) {
  event.preventDefault()
  event.stopPropagation()
  contextMenu.show(CalendarEventMenu, { event: { id: timeBlock.id } }, event)
}

// 拖放支持
onMounted(() => {
  if (!cellRef.value) return

  const zoneId = `timeline::${props.date}`
  cellRef.value.setAttribute('data-zone-id', zoneId)

  interactManager.registerDropzone(cellRef.value, {
    zoneId,
    type: 'kanban',
    computePreview: () => ({
      dropIndex: 0, // 总是放在最前面
    }),
    onDrop: async (session: DragSession) => {
      logger.info(LogTags.COMPONENT_CALENDAR, 'Drop task on timeline cell', {
        taskId: (session.object.data as any)?.id,
        targetDate: props.date,
      })

      // 构造日期视图的 viewKey
      const viewKey = `daily::${props.date}`

      // 执行拖放策略
      const result = await dragStrategy.executeDrop(session, viewKey, {
        sourceContext: session.metadata?.sourceContext || {},
        targetContext: {
          taskIds: [], // 空列表表示放在最前面
          displayTasks: [],
        },
      })

      if (!result.success) {
        logger.error(
          LogTags.COMPONENT_CALENDAR,
          'Failed to drop task on timeline cell',
          new Error(result.error || 'Unknown error')
        )
      }
    },
  })

  logger.debug(LogTags.COMPONENT_CALENDAR, 'Timeline cell dropzone registered', {
    date: props.date,
    zoneId,
  })
})

onBeforeUnmount(() => {
  if (cellRef.value) {
    interactManager.unregisterDropzone(cellRef.value)
  }
})
</script>

<template>
  <div
    ref="cellRef"
    class="timeline-day-cell"
    :class="{
      'is-today': isToday,
      'is-weekend': isWeekend,
      'has-content': hasContent,
    }"
    :data-date="date"
  >
    <div class="day-header">
      <span class="day-number">{{ dayNumber }}</span>
    </div>

    <div class="day-content">
      <!-- 任务列表 -->
      <div v-if="tasks.length > 0" class="content-section tasks-section">
        <div
          v-for="task in tasks"
          :key="`task-${task.id}-${date}`"
          class="timeline-item task-item"
          @click="handleTaskClick(task.id)"
          @contextmenu="handleTaskContextMenu($event, task)"
        >
          <CalendarTaskEventContent
            :task-id="task.id"
            :title="task.title"
            :schedule-day="date"
            :schedule-outcome="
              task.schedules?.find((s) => s.scheduled_day === date)?.outcome ?? null
            "
            :is-completed="task.is_completed"
            :is-recurring="!!task.recurrence_id"
            :has-due-flag="task.due_date?.date?.slice(0, 10) === date"
            :is-due-overdue="task.due_date?.is_overdue ?? false"
          />
        </div>
      </div>

      <!-- 截止日期列表 -->
      <div v-if="dueDates.length > 0" class="content-section due-dates-section">
        <div
          v-for="dueTask in dueDates"
          :key="`due-${dueTask.id}`"
          class="timeline-item due-item"
          @click="handleDueDateClick(dueTask.id)"
        >
          <CalendarDueDateEventContent
            :title="dueTask.title"
            :is-overdue="dueTask.due_date?.is_overdue ?? false"
          />
        </div>
      </div>

      <!-- 全天事件列表 -->
      <div v-if="allDayEvents.length > 0" class="content-section events-section">
        <div
          v-for="event in allDayEvents"
          :key="`event-${event.id}`"
          class="timeline-item event-item"
          @contextmenu="handleEventContextMenu($event, event)"
        >
          <CalendarTimeBlockEventContent
            :title="event.title || 'Time Block'"
            :area-color="event.area_id ? '#9ca3af' : '#9ca3af'"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.timeline-day-cell {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--color-border-default);
  border-radius: 6px;
  background: var(--color-background-primary);
  overflow: hidden;
  transition: all 0.15s ease;
  min-height: 80px;
}

.timeline-day-cell:hover {
  border-color: var(--color-border-hover);
  box-shadow: 0 2px 4px rgb(0 0 0 / 5%);
}

.timeline-day-cell.is-today {
  background: var(--color-primary-bg, #e3f2fd);
  border-color: var(--color-primary, #4a90e2);
}

.timeline-day-cell.is-weekend {
  background: var(--color-background-secondary);
}

.day-header {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0.6rem;
  border-bottom: 1px solid var(--color-border-default);
  background: var(--color-background-secondary);
}

.timeline-day-cell.is-today .day-header {
  background: var(--color-primary, #4a90e2);
}

.day-number {
  font-size: 1.4rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.timeline-day-cell.is-today .day-number {
  color: #fff;
}

.day-content {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  padding: 0.6rem;
  overflow-y: auto;
  flex: 1;
}

.content-section {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}

.timeline-item {
  cursor: pointer;
  border-radius: 4px;
  transition: background-color 0.15s ease;
}

.timeline-item:hover {
  background: var(--color-background-hover);
}

.task-item,
.due-item,
.event-item {
  font-size: 1.2rem;
}
</style>
