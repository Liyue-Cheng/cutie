<template>
  <FullCalendar :options="calendarOptions" />
</template>

<script setup lang="ts">
import FullCalendar from '@fullcalendar/vue3'
import interactionPlugin from '@fullcalendar/interaction'
import timeGridPlugin from '@fullcalendar/timegrid'
import { reactive, onMounted, computed } from 'vue'
import { useActivityStore } from '@/stores/activity'
import type { EventInput, EventChangeArg, DateSelectArg, EventMountArg } from '@fullcalendar/core'
import { useContextMenu } from '@/composables/useContextMenu'
import CalendarEventMenu from '@/components/parts/CalendarEventMenu.vue'

const activityStore = useActivityStore()
const contextMenu = useContextMenu()

onMounted(() => {
  activityStore.fetchActivities()
})

const calendarEvents = computed((): EventInput[] => {
  return activityStore.allActivities.map((activity) => ({
    id: activity.id,
    title: activity.title ?? 'Untitled',
    start: activity.start_time,
    end: activity.end_time,
    allDay: activity.is_all_day,
    color: activity.color ?? undefined,
  }))
})

async function handleDateSelect(selectInfo: DateSelectArg) {
  const calendarApi = selectInfo.view.calendar
  calendarApi.unselect() // clear date selection

  const title = prompt('Please enter a new title for your event')
  if (title) {
    try {
      await activityStore.createActivity({
        title,
        start_time: selectInfo.start.toISOString(),
        end_time: selectInfo.end.toISOString(),
        is_all_day: selectInfo.allDay,
      })
    } catch (error) {
      console.error('Failed to create event:', error)
      alert(`Error: Could not create the event. It might be overlapping with another event.`)
      // No need to manually revert, as it was never added to the store successfully
    }
  }
}

async function handleEventChange(changeInfo: EventChangeArg) {
  const { event, oldEvent } = changeInfo

  // Check if this is a drag from all-day to timed event
  const wasAllDay = oldEvent.allDay
  const isNowTimed = !event.allDay

  let startTime = event.start?.toISOString()
  let endTime = event.end?.toISOString()

  // If dragging from all-day to timed, set duration to 1 hour
  if (wasAllDay && isNowTimed && event.start) {
    const start = new Date(event.start)
    const end = new Date(start.getTime() + 60 * 60 * 1000) // Add 1 hour
    startTime = start.toISOString()
    endTime = end.toISOString()

    console.log(
      `[Calendar] Converting all-day event to 1-hour timed event: ${startTime} - ${endTime}`
    )
  }

  try {
    await activityStore.updateActivity(event.id, {
      title: event.title,
      start_time: startTime,
      end_time: endTime,
      is_all_day: event.allDay,
    })
  } catch (error) {
    console.error('Failed to update event:', error)
    alert(`Error: Could not update the event. It might be overlapping with another event.`)
    changeInfo.revert() // Revert the change on the calendar
  }
}

function handleEventContextMenu(info: EventMountArg) {
  info.el.addEventListener('contextmenu', (e: MouseEvent) => {
    contextMenu.show(CalendarEventMenu, { event: info.event }, e)
  })
}

const calendarOptions = reactive({
  plugins: [interactionPlugin, timeGridPlugin],
  headerToolbar: false as const,
  dayHeaders: false,
  initialView: 'timeGridDay',
  slotLabelFormat: {
    hour: '2-digit' as const,
    minute: '2-digit' as const,
    hour12: false,
  },
  height: '100%',
  weekends: true,
  editable: true,
  selectable: true,
  events: calendarEvents,
  select: handleDateSelect,
  eventChange: handleEventChange,
  eventDidMount: handleEventContextMenu,
})
</script>

<style>
/*
 * ===============================================
 * FullCalendar 自定义样式
 * ===============================================
 * 
 * 本文件包含对 FullCalendar 组件的所有自定义样式修改，
 * 按功能模块分组，便于维护和理解。
 */

/* ===============================================
 * 1. 今日高亮样式
 * =============================================== */
.fc .fc-day-today {
  background-color: transparent !important; /* 移除今日的默认蓝色背景 */
}

/* ===============================================
 * 2. 时间标签样式修复
 * =============================================== */

/* 时间标签垂直居中 */
.fc .fc-timegrid-slot-label {
  transform: translateY(-50%);
}

/* 移除时间槽边框 */
.fc .fc-timegrid-slot-label,
.fc .fc-timegrid-slot-minor {
  border: none !important;
}

/* 为时间标签容器添加上边距，防止 translateY(-50%) 导致的裁切问题 */
.fc .fc-timegrid-slots {
  padding-top: 1rem !important;
}

/* ===============================================
 * 3. 滚动条样式美化
 * =============================================== */

/* 隐藏默认滚动条 */
.fc .fc-scroller::-webkit-scrollbar {
  width: 8px;
  background-color: transparent;
}

/* 滚动条轨道样式 */
.fc .fc-scroller::-webkit-scrollbar-track {
  background-color: transparent;
}

/* 滚动条滑块样式 */
.fc .fc-scroller::-webkit-scrollbar-thumb {
  background-color: var(--color-border-hover);
  border-radius: 4px;
}

/* ===============================================
 * 4. 时间网格分隔线样式
 * =============================================== */
.fc .fc-timegrid-divider {
  padding: 1rem !important; /* 增加分隔线区域的内边距 */
  background-color: transparent !important; /* 设置透明背景 */
}

/* ===============================================
 * 5. 边框移除 - 解决多余边框显示问题
 * =============================================== */

/* 移除主网格边框 */
.fc-theme-standard .fc-scrollgrid {
  border: none !important;
}

/* 移除表格单元格右边框 */
.fc-theme-standard td,
.fc-theme-standard th {
  border-right: none !important;
}

/* 移除特定容器的边框 */
.fc .fc-scrollgrid-section-liquid > td {
  border: none !important;
}

/* ===============================================
 * 6. 事件样式自定义
 * =============================================== */

/* 事件边框和视觉效果 */
.fc-event,
.fc-timegrid-event {
  border-color: #ddd !important; /* 设置事件边框为灰色 */
  box-shadow: none !important; /* 移除默认阴影效果 */
}
</style>
