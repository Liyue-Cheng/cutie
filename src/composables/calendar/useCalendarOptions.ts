/**
 * useCalendarOptions - FullCalendar 配置
 *
 * 配置 FullCalendar 插件、视图、时间槽等选项
 */

import { reactive, type ComputedRef } from 'vue'
import interactionPlugin from '@fullcalendar/interaction'
import timeGridPlugin from '@fullcalendar/timegrid'
import type {
  EventInput,
  EventChangeArg,
  DateSelectArg,
  EventMountArg,
  EventClickArg,
} from '@fullcalendar/core'

export function useCalendarOptions(
  calendarEvents: ComputedRef<EventInput[]>,
  handlers: {
    handleDateSelect: (selectInfo: DateSelectArg) => Promise<void>
    handleEventChange: (changeInfo: EventChangeArg) => Promise<void>
    handleEventContextMenu: (info: EventMountArg) => void
    handleEventClick: (clickInfo: EventClickArg) => void
  }
) {
  const calendarOptions = reactive({
    plugins: [interactionPlugin, timeGridPlugin],
    headerToolbar: false as const,
    dayHeaders: false,
    initialView: 'timeGridDay',
    allDaySlot: true, // ✅ 启用全日槽位
    slotLabelFormat: {
      hour: '2-digit' as const,
      minute: '2-digit' as const,
      hour12: false,
    },
    slotMinTime: '00:00:00', // 从0:00开始显示
    slotMaxTime: '24:00:00', // 到24:00结束
    slotDuration: '00:05:00', // 5分钟时间槽
    slotLabelInterval: '00:30:00', // 每30分钟显示一个时间标签
    snapDuration: '00:05:00', // 5分钟对齐精度
    nowIndicator: true, // 显示当前时间指示器
    height: '100%',
    weekends: true,
    editable: true,
    selectable: true,
    eventResizableFromStart: true, // 允许从开始时间调整大小
    events: calendarEvents,
    select: handlers.handleDateSelect,
    eventChange: handlers.handleEventChange,
    eventDidMount: handlers.handleEventContextMenu,
    eventClick: handlers.handleEventClick,
  })

  return {
    calendarOptions,
  }
}
