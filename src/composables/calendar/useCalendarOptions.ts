/**
 * useCalendarOptions - FullCalendar 配置
 *
 * 🎯 核心功能：
 * - 生成 FullCalendar 的完整配置对象（calendarOptions）
 * - 集成所有事件处理器（handlers）
 * - 配置自定义事件渲染（eventContent）
 *
 * 🔑 关键配置：
 * - plugins：interactionPlugin、timeGridPlugin、dayGridPlugin
 * - views：自定义 3 天/5 天/7 天视图
 * - slotDuration：5 分钟槽位（精细化时间控制）
 * - eventContent：使用 Vue 组件渲染所有事件（任务、时间块、截止日期）
 *
 * 🎨 自定义渲染策略：
 * - TimeGrid 视图：使用 CalendarTimeGridEventContent（带时间范围和复选框）
 * - DayGrid（月视图）：
 *   - 任务：CalendarTaskEventContent
 *   - 时间块：CalendarTimeBlockEventContent
 *   - 截止日期：CalendarDueDateEventContent
 *
 * 📌 重要：
 * - 已禁用 FullCalendar 原生的 select（改用自定义框选）
 * - eventContent 返回 { domNodes: [container] } 挂载 Vue 组件
 */

import { reactive, type ComputedRef, createApp } from 'vue'
import interactionPlugin from '@fullcalendar/interaction'
import timeGridPlugin from '@fullcalendar/timegrid'
import dayGridPlugin from '@fullcalendar/daygrid'
import type {
  EventInput,
  EventChangeArg,
  EventMountArg,
  EventClickArg,
  DatesSetArg,
  EventContentArg,
} from '@fullcalendar/core'
import CalendarTaskEventContent from '@/components/assembles/calender/CalendarTaskEventContent.vue'
import CalendarTimeBlockEventContent from '@/components/assembles/calender/CalendarTimeBlockEventContent.vue'
import CalendarTimeGridEventContent from '@/components/assembles/calender/CalendarTimeGridEventContent.vue'
import CalendarDueDateEventContent from '@/components/assembles/calender/CalendarDueDateEventContent.vue'
import { useTaskStore } from '@/stores/task'
import { toLocalISOString } from '@/infra/utils/dateUtils'
import { getDefaultAreaColor } from '@/infra/utils/themeUtils'

export function useCalendarOptions(
  calendarEvents: ComputedRef<EventInput[]>,
  handlers: {
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
    selectable: false,
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
    eventChange: handlers.handleEventChange,
    eventDidMount: handlers.handleEventDidMount,
    eventClick: handlers.handleEventClick,
    datesSet: handleDatesSet, // 🆕 日期变化回调

    /**
     * 🎨 自定义事件内容渲染（FullCalendar 官方推荐方式）
     *
     * 🔄 渲染流程：
     * 1. 检查事件类型（type: 'task' | 'timeblock' | 'due_date'）
     * 2. 检查视图类型（timeGrid | dayGrid）
     * 3. 创建 Vue 组件实例
     * 4. 挂载到 DOM 容器
     * 5. 返回 { domNodes: [container] }
     *
     * 🎯 组件映射：
     * - timeGrid + timeblock/preview → CalendarTimeGridEventContent
     * - dayGrid + task → CalendarTaskEventContent
     * - dayGrid + timeblock → CalendarTimeBlockEventContent
     * - dayGrid + due_date → CalendarDueDateEventContent
     *
     * 📌 注意：
     * - isPreview = true 时，CalendarTimeGridEventContent 不显示标题
     * - 所有组件都通过 createApp 动态创建，避免全局注册
     */
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

        // 预览事件使用空标题，避免显示“Time Block”占位
        const displayTitle = isPreviewEvent ? '' : arg.event.title || 'Time Block'

        // 使用 Vue 组件渲染
        const app = createApp(CalendarTimeGridEventContent, {
          title: displayTitle,
          areaColor,
          startTime,
          endTime,
          taskId,
          isCompleted,
          scheduleOutcome,
          scheduleDay,
          isPreview: isPreviewEvent,
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
