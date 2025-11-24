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
import CalendarTaskEventContent from '@/components/assembles/calender/CalendarTaskEventContent.vue'
import CalendarTimeBlockEventContent from '@/components/assembles/calender/CalendarTimeBlockEventContent.vue'
import CalendarTimeGridEventContent from '@/components/assembles/calender/CalendarTimeGridEventContent.vue'
import CalendarDueDateEventContent from '@/components/assembles/calender/CalendarDueDateEventContent.vue'
import CalendarSelectionPreview from '@/components/assembles/calender/CalendarSelectionPreview.vue'
import { useTaskStore } from '@/stores/task'
import { toLocalISOString } from '@/infra/utils/dateUtils'
import { getDefaultAreaColor } from '@/infra/utils/themeUtils'

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
    selectMirror: true, // 启用选区镜像预览（类似 Google Calendar）
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
        areaColor?: string
        [key: string]: any
      }

      const isTimeGridView = arg.view.type.startsWith('timeGrid')

      // 🎯 selectMirror 选区预览渲染（仅显示时间，不显示标题）
      if (arg.isMirror && isTimeGridView && !arg.event.allDay) {
        const container = document.createElement('div')
        container.style.width = '100%'
        container.style.height = '100%'

        const startTime = arg.event.start ? toLocalISOString(arg.event.start) : ''
        const endTime = arg.event.end ? toLocalISOString(arg.event.end) : ''

        // 使用自定义选区预览组件
        const app = createApp(CalendarSelectionPreview, {
          startTime,
          endTime,
        })

        app.mount(container)
        return { domNodes: [container] }
      }

      const isPreviewEvent = Boolean(extended?.isPreview)
      const isTimeBlockEvent = extended?.type === 'timeblock'

      // TimeGrid 视图的时间块事件（以及拖拽预览）自定义渲染
      if (!arg.event.allDay && isTimeGridView && (isTimeBlockEvent || isPreviewEvent)) {
        const container = document.createElement('div')
        container.style.width = '100%'
        container.style.height = '100%'

        const areaColorCandidate =
          extended.areaColor || extended.previewColor || arg.event.backgroundColor
        const areaColor = areaColorCandidate || getDefaultAreaColor()
        const startTime = arg.event.start ? toLocalISOString(arg.event.start) : ''
        const endTime = arg.event.end ? toLocalISOString(arg.event.end) : ''
        const taskId = extended.taskId as string | undefined
        const isCompleted = extended.isCompleted as boolean | undefined
        const scheduleOutcome = extended.scheduleOutcome as string | null | undefined
        const scheduleDay = extended.scheduleDay as string | undefined

        // 使用 Vue 组件渲染
        const app = createApp(CalendarTimeGridEventContent, {
          title: arg.event.title || 'Time Block',
          areaColor,
          startTime,
          endTime,
          taskId,
          isCompleted,
          scheduleOutcome,
          scheduleDay,
        })

        app.mount(container)

        // 返回自定义内容
        return { domNodes: [container] }
      }

      // 月视图的任务事件自定义渲染
      if (extended?.type === 'task' && arg.view.type === 'dayGridMonth') {
        const container = document.createElement('div')
        container.style.width = '100%'
        container.style.height = '100%'

        // 获取最新的任务数据
        let isCompleted = extended.isCompleted ?? false
        let scheduleOutcome = extended.scheduleOutcome ?? null
        let hasDueFlag = Boolean(extended.hasDueFlag)
        let isDueOverdue = Boolean(extended.isDueOverdue)

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

            if (task.due_date && extended.scheduleDay) {
              // ✅ due_date.date 现在是 YYYY-MM-DD 格式，直接使用
              const dueDateDay = task.due_date.date
              if (dueDateDay && dueDateDay === extended.scheduleDay) {
                hasDueFlag = true
                isDueOverdue = task.due_date.is_overdue
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
          hasDueFlag,
          isDueOverdue,
        })

        app.mount(container)

        // 返回自定义内容
        return { domNodes: [container] }
      }

      // 月视图的全天时间块事件自定义渲染
      if (extended?.type === 'timeblock' && arg.view.type === 'dayGridMonth') {
        const container = document.createElement('div')
        container.style.width = '100%'
        container.style.height = '100%'

        const areaColor = extended.areaColor || getDefaultAreaColor()

        // 使用 Vue 组件渲染
        const app = createApp(CalendarTimeBlockEventContent, {
          title: arg.event.title || 'Time Block',
          areaColor,
        })

        app.mount(container)

        // 返回自定义内容
        return { domNodes: [container] }
      }

      // 月视图的截止日期事件自定义渲染
      if (extended?.type === 'due_date' && arg.view.type === 'dayGridMonth') {
        const container = document.createElement('div')
        container.style.width = '100%'
        container.style.height = '100%'

        const app = createApp(CalendarDueDateEventContent, {
          title: arg.event.title || '任务',
          isOverdue: Boolean(extended.isOverdue),
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
