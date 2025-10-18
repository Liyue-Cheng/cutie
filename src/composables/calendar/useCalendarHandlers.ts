/**
 * useCalendarHandlers - 日历事件处理器
 *
 * 处理用户创建、修改、右键点击日历事件的逻辑
 */

import { type Ref } from 'vue'
import type {
  EventInput,
  EventChangeArg,
  DateSelectArg,
  EventMountArg,
  EventClickArg,
} from '@fullcalendar/core'
import { useContextMenu } from '@/composables/useContextMenu'
import CalendarEventMenu from '@/components/parts/CalendarEventMenu.vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'

export function useCalendarHandlers(
  previewEvent: Ref<EventInput | null>,
  currentDateRef: Ref<string | undefined>,
  selectedTimeBlockId: Ref<string | null>
) {
  const contextMenu = useContextMenu()

  /**
   * 处理日期选择 - 创建新的时间块
   */
  async function handleDateSelect(selectInfo: DateSelectArg) {
    const calendarApi = selectInfo.view.calendar
    calendarApi.unselect() // clear date selection

    const title = prompt('Please enter a new title for your time block')
    if (title) {
      // ✅ 根据选择区域判断是否为全天事件
      const isAllDay = selectInfo.allDay

      // 创建临时预览事件，减少视觉跳动
      const tempEvent = {
        id: 'temp-creating',
        title: title,
        start: selectInfo.start.toISOString(),
        end: selectInfo.end.toISOString(),
        allDay: isAllDay,
        color: '#BCEAEE',
        classNames: ['creating-event'],
      }

      // 添加临时预览
      previewEvent.value = tempEvent

      try {
        // 截断：非全天情况下不得跨天
        let startISO = selectInfo.start.toISOString()
        let endISO = selectInfo.end.toISOString()
        if (!isAllDay) {
          const start = new Date(selectInfo.start)
          let end = new Date(selectInfo.end)
          const dayEnd = new Date(start)
          dayEnd.setHours(23, 59, 59, 999) // 截断到当天最后一刻
          if (end.getTime() > dayEnd.getTime()) {
            end = dayEnd
          }
          startISO = start.toISOString()
          endISO = end.toISOString()
        }

        // 计算本地时间字符串
        let startTimeLocal: string | undefined
        let endTimeLocal: string | undefined

        if (isAllDay) {
          // 全天事件：使用 00:00:00 到 23:59:59
          startTimeLocal = '00:00:00'
          endTimeLocal = '23:59:59'
        } else {
          // 分时事件：提取时间部分
          const startDate = new Date(startISO)
          const endDate = new Date(endISO)
          startTimeLocal = startDate.toTimeString().split(' ')[0] // HH:MM:SS
          endTimeLocal = endDate.toTimeString().split(' ')[0] // HH:MM:SS
        }

        // ✅ 使用命令系统创建空时间块
        await pipeline.dispatch('time_block.create', {
          title,
          start_time: startISO,
          end_time: endISO,
          start_time_local: startTimeLocal,
          end_time_local: endTimeLocal,
          time_type: 'FLOATING', // 默认使用浮动时间
          creation_timezone: Intl.DateTimeFormat().resolvedOptions().timeZone, // 当前时区
          is_all_day: isAllDay, // ✅ 传递全天标志
        })

        // 清除临时预览，真实事件会通过store更新显示
        previewEvent.value = null
      } catch (error) {
        logger.error(
          LogTags.COMPONENT_CALENDAR,
          'Failed to create event',
          error instanceof Error ? error : new Error(String(error))
        )

        // 清除临时预览
        previewEvent.value = null

        // 显示错误信息给用户
        let errorMessage = 'Could not create the event. It might be overlapping with another event.'
        if (error instanceof Error) {
          errorMessage = error.message
        } else if (typeof error === 'string') {
          errorMessage = error
        }

        logger.error(LogTags.COMPONENT_CALENDAR, 'Event creation failed', new Error(errorMessage))
        alert(`创建事件失败: ${errorMessage}`)
      }
    }
  }

  /**
   * 处理事件变化 - 拖动或调整大小时间块
   */
  async function handleEventChange(changeInfo: EventChangeArg) {
    const { event, oldEvent } = changeInfo

    // ✅ 只处理真实的时间块事件，忽略虚拟事件（任务、截止日期等）
    const eventType = (event.extendedProps as any)?.type
    if (eventType !== 'timeblock') {
      logger.debug(LogTags.COMPONENT_CALENDAR, 'Ignoring event change for non-timeblock event', {
        eventId: event.id,
        eventType,
      })
      changeInfo.revert() // 恢复原状
      return
    }

    // ✅ 检查全天状态变化
    const wasAllDay = oldEvent.allDay
    const isNowAllDay = event.allDay
    const isNowTimed = !event.allDay

    let startTime = event.start?.toISOString()
    let endTime = event.end?.toISOString()

    // ✅ 从全天拖到分时：设置为 1 小时，并截断到日界
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

    // ✅ 从分时拖到全天：规整到日界
    if (!wasAllDay && isNowAllDay && event.start && event.end) {
      const startDate = new Date(event.start)
      startDate.setHours(0, 0, 0, 0)
      const endDate = new Date(event.end)
      endDate.setHours(0, 0, 0, 0)
      startTime = startDate.toISOString()
      endTime = endDate.toISOString()

      logger.debug(LogTags.COMPONENT_CALENDAR, 'Converting timed to all-day event', {
        startTime,
        endTime,
      })
    }

    // 统一截断：分时事件不得跨天（包括拖动/拉伸）
    if (!isNowAllDay && event.start && event.end) {
      let start = new Date(event.start)
      let end = new Date(event.end)

      // 使用本地日期比较（避免 UTC 偏移导致误判）
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
      contextMenu.show(CalendarEventMenu, { event: info.event }, e)
    })
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
    handleDateSelect,
    handleEventChange,
    handleEventContextMenu,
    handleEventClick,
  }
}
