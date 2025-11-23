/**
 * useCalendarHandlers - 日历事件处理器
 *
 * 🎯 核心职责：
 * 处理日历上的所有用户交互，包括：
 * - 时间段框选（通过自定义 overlay，不使用 FullCalendar 原生 select）
 * - 时间块拖动/调整大小
 * - 事件点击（打开详情面板）
 * - 右键菜单（任务菜单、时间块菜单）
 *
 * 🔑 重要概念：
 * - previewEvent：用于在用户操作过程中显示预览卡片（如框选时、拖拽时）
 * - 所有时间块相关操作通过 pipeline.dispatch 发送指令，走统一的命令系统
 *
 * 📌 注意：
 * - 本文件只处理"松手后"的逻辑（打开创建对话框）
 * - "拖动过程中"的预览由 CuteCalendar.vue 的 mouse 事件驱动
 */

import { type Ref } from 'vue'
import type { EventInput, EventChangeArg, EventMountArg, EventClickArg } from '@fullcalendar/core'
import { useContextMenu } from '@/composables/useContextMenu'
import CalendarEventMenu from '@/components/assembles/ContextMenu/CalendarEventMenu.vue'
import KanbanTaskCardMenu from '@/components/assembles/tasks/kanban/KanbanTaskCardMenu.vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'
import { useTaskStore } from '@/stores/task'
import { useUIStore } from '@/stores/ui'
import { getDefaultAreaColor } from '@/infra/utils/themeUtils'

export function useCalendarHandlers(
  previewEvent: Ref<EventInput | null>,
  currentDateRef: Ref<string | undefined>,
  selectedTimeBlockId: Ref<string | null>
) {
  const contextMenu = useContextMenu()
  const taskStore = useTaskStore()
  const uiStore = useUIStore()

  /**
   * 处理时间格框选 - 打开创建对话框并显示预览
   */
  async function handleTimeGridSelection(payload: {
    start: Date
    end: Date
    isAllDay?: boolean
    anchorTop?: number
    anchorLeft?: number
  }) {
    previewEvent.value = null

    const isAllDay = payload.isAllDay ?? false
    let normalizedStart = new Date(payload.start)
    let normalizedEnd = new Date(payload.end)

    if (normalizedEnd.getTime() < normalizedStart.getTime()) {
      const temp = normalizedStart
      normalizedStart = normalizedEnd
      normalizedEnd = temp
    }

    if (isAllDay) {
      normalizedStart.setHours(0, 0, 0, 0)
      normalizedEnd.setHours(23, 59, 59, 999)
    } else {
      const dayEnd = new Date(normalizedStart)
      dayEnd.setHours(23, 59, 59, 999)
      if (normalizedEnd.getTime() > dayEnd.getTime()) {
        normalizedEnd = dayEnd
      }

      // 至少保留 15 分钟
      if (normalizedEnd.getTime() === normalizedStart.getTime()) {
        const adjusted = new Date(normalizedStart.getTime() + 15 * 60 * 1000)
        normalizedEnd = adjusted.getTime() > dayEnd.getTime() ? dayEnd : adjusted
      }
    }

    const startISO = normalizedStart.toISOString()
    const endISO = normalizedEnd.toISOString()

    let startTimeLocal: string | undefined
    let endTimeLocal: string | undefined

    if (isAllDay) {
      startTimeLocal = '00:00:00'
      endTimeLocal = '23:59:59'
    } else {
      const startDate = new Date(startISO)
      const endDate = new Date(endISO)
      startTimeLocal = startDate.toTimeString().split(' ')[0]
      endTimeLocal = endDate.toTimeString().split(' ')[0]
    }

    uiStore.openTimeBlockCreateDialog({
      startISO,
      endISO,
      startTimeLocal,
      endTimeLocal,
      isAllDay,
      anchorTop: payload.anchorTop,
      anchorLeft: payload.anchorLeft,
    })

    previewEvent.value = {
      id: 'preview-event',
      title: '',
      start: startISO,
      end: endISO,
      allDay: isAllDay,
      color: 'transparent',
      backgroundColor: 'transparent',
      borderColor: 'transparent',
      classNames: ['preview-event'],
      display: 'block',
      extendedProps: {
        type: 'timeblock',
        isPreview: true,
        areaColor: getDefaultAreaColor(),
      },
    }
  }

  /**
   * 处理事件变化 - 拖动或调整大小时间块
   *
   * 🎯 触发时机：
   * - 用户拖动已有的时间块到新位置
   * - 用户调整时间块的开始/结束时间（拖动上下边缘）
   *
   * 🔄 处理流程：
   * 1. 过滤：只处理 type='timeblock' 的真实时间块（忽略任务、截止日期）
   * 2. 全天 ↔ 分时转换：自动调整时间格式
   * 3. 跨天截断：分时事件不允许跨天，自动截断到当天末尾
   * 4. 发送更新指令：通过 pipeline.dispatch('time_block.update') 更新后端
   *
   * 📌 注意：
   * - 乐观更新已在 timeblock-isa.ts 中实现，UI 会立即响应
   * - 失败时会 revert 日历显示并 alert 错误
   */
  async function handleEventChange(changeInfo: EventChangeArg) {
    const { event, oldEvent } = changeInfo

    // ✅ 过滤：只处理真实的时间块事件
    // 日历上还会显示"任务"、"截止日期"等虚拟事件，这些不允许拖动
    const eventType = (event.extendedProps as any)?.type
    if (eventType !== 'timeblock') {
      logger.debug(LogTags.COMPONENT_CALENDAR, 'Ignoring event change for non-timeblock event', {
        eventId: event.id,
        eventType,
      })
      changeInfo.revert() // 恢复原状
      return
    }

    // 🔄 检查全天 ↔ 分时状态变化
    // FullCalendar 允许用户把"全天事件"拖到"分时区域"，反之亦然
    const wasAllDay = oldEvent.allDay
    const isNowAllDay = event.allDay
    const isNowTimed = !event.allDay

    let startTime = event.start?.toISOString()
    let endTime = event.end?.toISOString()

    // 📅 → ⏰ 从全天拖到分时：默认设为 1 小时，并截断到当天末尾
    if (wasAllDay && isNowTimed && event.start) {
      const start = new Date(event.start)
      let end = new Date(start.getTime() + 60 * 60 * 1000) // Add 1 hour

      // 截断：不得跨天（使用当天最后一刻）
      const dayEnd = new Date(start)
      dayEnd.setHours(23, 59, 59, 999)
      if (end.getTime() > dayEnd.getTime()) {
        end = dayEnd
      }

      startTime = start.toISOString()
      endTime = end.toISOString()

      logger.debug(LogTags.COMPONENT_CALENDAR, 'Converting all-day to timed event', {
        startTime,
        endTime,
      })
    }

    // ⏰ → 📅 从分时拖到全天：规整到日界（00:00 - 00:00）
    if (!wasAllDay && isNowAllDay && event.start && event.end) {
      const startDate = new Date(event.start)
      startDate.setHours(0, 0, 0, 0) // 开始时间设为当天 00:00
      const endDate = new Date(event.end)
      endDate.setHours(0, 0, 0, 0) // 结束时间设为次日 00:00
      startTime = startDate.toISOString()
      endTime = endDate.toISOString()

      logger.debug(LogTags.COMPONENT_CALENDAR, 'Converting timed to all-day event', {
        startTime,
        endTime,
      })
    }

    // 🔪 统一截断：分时事件不得跨天（包括拖动/拉伸）
    // ⚠️ 重要：必须使用本地时间比较，不能直接比较 ISO 字符串
    // 原因：UTC 时间可能跨天，但本地时间未跨天（或反之）
    if (!isNowAllDay && event.start && event.end) {
      let start = new Date(event.start)
      let end = new Date(event.end)

      // 🌍 本地日期提取器：使用本地时间避免 UTC 偏移误判
      const toLocalYMD = (d: Date) => {
        const y = d.getFullYear()
        const m = `${d.getMonth() + 1}`.padStart(2, '0')
        const da = `${d.getDate()}`.padStart(2, '0')
        return `${y}-${m}-${da}`
      }

      const startLocalDay = toLocalYMD(start)
      const endLocalDay = toLocalYMD(end)

      if (startLocalDay !== endLocalDay) {
        // 跨天了：根据"当前日历视图日期"（本地）决定保留哪一天
        const viewLocalDate = currentDateRef.value || startLocalDay

        if (viewLocalDate === endLocalDay) {
          // 视图日期是结束那天：将开始截断到该天的本地 00:00
          const dayStart = new Date(end)
          dayStart.setHours(0, 0, 0, 0)
          start = dayStart
          startTime = start.toISOString()
          logger.debug(LogTags.COMPONENT_CALENDAR, 'Cross-day detected, truncate start', {
            viewLocalDate,
            startTime,
          })
        } else {
          // 默认：视图日期为开始那天：将结束截断到开始那天的 23:59:59.999
          const dayEnd = new Date(start)
          dayEnd.setHours(23, 59, 59, 999)
          end = dayEnd
          endTime = end.toISOString()
          logger.debug(LogTags.COMPONENT_CALENDAR, 'Cross-day detected, truncate end', {
            viewLocalDate,
            endTime,
          })
        }
      }
    }

    try {
      // 计算本地时间字符串
      let startTimeLocal: string | undefined
      let endTimeLocal: string | undefined

      if (isNowAllDay) {
        // 全天事件：使用 00:00:00 到 23:59:59
        startTimeLocal = '00:00:00'
        endTimeLocal = '23:59:59'
      } else if (startTime && endTime) {
        // 分时事件：提取时间部分
        const startDate = new Date(startTime)
        const endDate = new Date(endTime)
        startTimeLocal = startDate.toTimeString().split(' ')[0] // HH:MM:SS
        endTimeLocal = endDate.toTimeString().split(' ')[0] // HH:MM:SS
      }

      // ✅ 使用命令系统更新时间块（event.id 现在就是真实的 UUID）
      await pipeline.dispatch('time_block.update', {
        id: event.id,
        updates: {
          title: event.title,
          start_time: startTime,
          end_time: endTime,
          start_time_local: startTimeLocal,
          end_time_local: endTimeLocal,
          time_type: 'FLOATING', // 保持浮动时间类型
          is_all_day: isNowAllDay, // ✅ 更新全天标志
        },
      })
    } catch (error) {
      logger.error(
        LogTags.COMPONENT_CALENDAR,
        'Failed to update event',
        error instanceof Error ? error : new Error(String(error))
      )

      // 显示错误信息给用户
      let errorMessage = 'Could not update the event. It might be overlapping with another event.'
      if (error instanceof Error) {
        errorMessage = error.message
      } else if (typeof error === 'string') {
        errorMessage = error
      }

      logger.error(LogTags.COMPONENT_CALENDAR, 'Event update failed', new Error(errorMessage))
      alert(`更新事件失败: ${errorMessage}`)

      changeInfo.revert() // Revert the change on the calendar
    }
  }

  /**
   * 处理事件右键菜单
   */
  function handleEventContextMenu(info: EventMountArg) {
    info.el.addEventListener('contextmenu', (e: MouseEvent) => {
      const extended = info.event.extendedProps as {
        type?: string
        taskId?: string
        scheduleDay?: string
      }

      // 截止日期事件不提供右键菜单
      if (extended?.type === 'due_date') {
        e.preventDefault()
        return
      }

      if (extended?.type === 'task' && extended.taskId) {
        const task = taskStore.getTaskById_Mux(extended.taskId)

        if (task) {
          const viewKey = extended.scheduleDay ? `daily::${extended.scheduleDay}` : undefined
          contextMenu.show(KanbanTaskCardMenu, { task, viewKey }, e)
          return
        } else {
          logger.warn(LogTags.COMPONENT_CALENDAR, 'Task not found for calendar event', {
            taskId: extended.taskId,
          })
        }
      }

      contextMenu.show(CalendarEventMenu, { event: info.event }, e)
    })
  }

  /**
   * 处理事件挂载 - 只用于注册右键菜单
   */
  function handleEventDidMount(info: EventMountArg) {
    handleEventContextMenu(info)

    const extended = info.event.extendedProps as {
      isPreview?: boolean
      previewColor?: string
    }

    if (info.event.id === 'preview-event' && extended?.previewColor) {
      info.el.style.setProperty('--preview-bg', extended.previewColor)
      info.el.style.setProperty('--preview-border', extended.previewColor)
    }
  }

  /**
   * 处理事件点击 - 显示时间块详情面板
   */
  function handleEventClick(clickInfo: EventClickArg) {
    const eventId = clickInfo.event.id
    // 不处理预览事件和创建中事件
    if (eventId === 'preview-event' || eventId === 'temp-creating') {
      return
    }
    logger.debug(LogTags.COMPONENT_CALENDAR, 'Event clicked', { eventId })
    selectedTimeBlockId.value = eventId
  }

  return {
    handleTimeGridSelection,
    handleEventChange,
    handleEventContextMenu,
    handleEventClick,
    handleEventDidMount,
  }
}
