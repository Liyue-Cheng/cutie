/**
 * useCalendarOptions - FullCalendar 配置
 *
 * 配置 FullCalendar 插件、视图、时间槽等选项
 */

import { reactive, type ComputedRef, createApp } from 'vue'
import interactionPlugin from '@fullcalendar/interaction'
import timeGridPlugin from '@fullcalendar/timegrid'
import dayGridPlugin from '@fullcalendar/daygrid'
import type {
  EventInput,
  EventChangeArg,
  DateSelectArg,
  EventMountArg,
  EventClickArg,
  DatesSetArg,
  EventContentArg,
} from '@fullcalendar/core'
import CalendarTaskEventContent from '@/components/parts/calendar/CalendarTaskEventContent.vue'
import { useTaskStore } from '@/stores/task'

export function useCalendarOptions(
  calendarEvents: ComputedRef<EventInput[]>,
  handlers: {
    handleDateSelect: (selectInfo: DateSelectArg) => Promise<void>
    handleEventChange: (changeInfo: EventChangeArg) => Promise<void>
    handleEventContextMenu: (info: EventMountArg) => void
    handleEventClick: (clickInfo: EventClickArg) => void
    handleEventDidMount: (arg: EventMountArg) => void
  },
  viewType: 'day' | 'week' | 'month' = 'day', // ✅ 新增：视图类型参数，默认为单天
  handleDatesSet?: (dateInfo: DatesSetArg) => void, // 🆕 日期变化回调
  days: 1 | 3 | 5 | 7 = 1 // 🆕 显示天数（1天、3天、5天或7天）
) {
  const taskStore = useTaskStore()

  // ✅ 加载所有插件，支持动态切换视图
  const plugins = [interactionPlugin, timeGridPlugin, dayGridPlugin]

  let initialView: string
  if (viewType === 'day') {
    // 根据天数选择对应的视图
    if (days === 3) {
      initialView = 'timeGrid3Days'
    } else if (days === 5) {
      initialView = 'timeGrid5Days'
    } else if (days === 7) {
      initialView = 'timeGrid7Days'
    } else {
      initialView = 'timeGridDay'
    }
  } else if (viewType === 'week') {
    initialView = 'timeGridWeek'
  } else {
    initialView = 'dayGridMonth'
  }

  const calendarOptions = reactive({
    plugins,
    headerToolbar: false as false, // 移除标题栏
    dayHeaders: false, // 移除日期列头部
    dayHeaderFormat: {
      weekday: 'short' as const,
      month: 'numeric' as const,
      day: 'numeric' as const,
    }, // 🆕 日期头部格式
    initialView,
    firstDay: 1, // ✅ 一周从周一开始（0=周日, 1=周一）
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

    // 🆕 自定义视图：3天、5天、7天视图
    views: {
      timeGrid3Days: {
        type: 'timeGrid',
        duration: { days: 3 },
      },
      timeGrid5Days: {
        type: 'timeGrid',
        duration: { days: 5 },
      },
      timeGrid7Days: {
        type: 'timeGrid',
        duration: { days: 7 },
      },
    },

    // ✅ 月视图配置：固定格子高度，超出事件用 "+N more" 折叠
    dayMaxEvents: 4, // 每个格子最多显示4个事件，超过的折叠
    moreLinkClick: 'popover' as const, // 点击 "+N more" 时显示弹出框
    fixedWeekCount: false, // 不固定显示6周，根据实际月份调整

    events: calendarEvents,
    select: handlers.handleDateSelect,
    eventChange: handlers.handleEventChange,
    eventDidMount: handlers.handleEventDidMount,
    eventClick: handlers.handleEventClick,
    datesSet: handleDatesSet, // 🆕 日期变化回调

    // 🔥 自定义事件内容渲染（官方推荐方式）
    eventContent: (arg: EventContentArg) => {
      const extended = arg.event.extendedProps as {
        type?: string
        taskId?: string
        scheduleDay?: string
        scheduleOutcome?: string | null
        isCompleted?: boolean
        isPreview?: boolean
        [key: string]: any
      }

      // 只对月视图的任务事件进行自定义渲染
      if (extended?.type === 'task' && arg.view.type === 'dayGridMonth') {
        const container = document.createElement('div')
        container.style.width = '100%'
        container.style.height = '100%'

        // 获取最新的任务数据
        let isCompleted = extended.isCompleted ?? false
        let scheduleOutcome = extended.scheduleOutcome ?? null

        if (extended.taskId) {
          const task = taskStore.getTaskById_Mux(extended.taskId)
          if (task) {
            isCompleted = task.is_completed
            if (extended.scheduleDay) {
              const schedule = task.schedules?.find((s) => s.scheduled_day === extended.scheduleDay)
              if (schedule) {
                scheduleOutcome = schedule.outcome ?? scheduleOutcome
              }
            }
          }
        }

        // 使用 Vue 组件渲染
        const app = createApp(CalendarTaskEventContent, {
          taskId: extended.taskId,
          title: arg.event.title || '任务',
          scheduleDay: extended.scheduleDay,
          scheduleOutcome,
          isCompleted,
          isPreview: Boolean(extended.isPreview),
          isRecurring: Boolean(extended.isRecurring),
        })

        app.mount(container)

        // 返回自定义内容
        return { domNodes: [container] }
      }

      // 其他事件使用默认渲染
      return true
    },
  })

  return {
    calendarOptions,
  }
}
