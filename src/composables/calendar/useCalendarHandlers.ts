/**
 * useCalendarHandlers - 日历事件处理器
 *
 * 处理用户创建、修改、右键点击日历事件的逻辑
 */

import { type Ref } from 'vue'
import type { EventInput, EventChangeArg, DateSelectArg, EventMountArg } from '@fullcalendar/core'
import { useTimeBlockStore } from '@/stores/timeblock'
import { useContextMenu } from '@/composables/useContextMenu'
import CalendarEventMenu from '@/components/parts/CalendarEventMenu.vue'

export function useCalendarHandlers(
  previewEvent: Ref<EventInput | null>,
  currentDateRef: Ref<string | undefined>
) {
  const timeBlockStore = useTimeBlockStore()
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
          dayEnd.setHours(0, 0, 0, 0)
          dayEnd.setDate(dayEnd.getDate() + 1)
          if (end.getTime() > dayEnd.getTime()) {
            end = dayEnd
          }
          startISO = start.toISOString()
          endISO = end.toISOString()
        }

        await timeBlockStore.createTimeBlock({
          title,
          start_time: startISO,
          end_time: endISO,
          is_all_day: isAllDay, // ✅ 传递全天标志
        })

        // 清除临时预览，真实事件会通过store更新显示
        previewEvent.value = null
      } catch (error) {
        console.error('Failed to create event:', error)

        // 清除临时预览
        previewEvent.value = null

        // 显示错误信息给用户
        let errorMessage = 'Could not create the event. It might be overlapping with another event.'
        if (error instanceof Error) {
          errorMessage = error.message
        } else if (typeof error === 'string') {
          errorMessage = error
        }

        console.error(`创建事件失败: ${errorMessage}`)
        alert(`创建事件失败: ${errorMessage}`)
      }
    }
  }

  /**
   * 处理事件变化 - 拖动或调整大小时间块
   */
  async function handleEventChange(changeInfo: EventChangeArg) {
    const { event, oldEvent } = changeInfo

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

      // 截断：不得跨天
      const dayEnd = new Date(start)
      dayEnd.setHours(0, 0, 0, 0)
      dayEnd.setDate(dayEnd.getDate() + 1)
      if (end.getTime() > dayEnd.getTime()) {
        end = dayEnd
      }

      startTime = start.toISOString()
      endTime = end.toISOString()

      console.log(
        `[Calendar] Converting all-day event to timed event (max 1 hour): ${startTime} - ${endTime}`
      )
    }

    // ✅ 从分时拖到全天：规整到日界
    if (!wasAllDay && isNowAllDay && event.start && event.end) {
      const startDate = new Date(event.start)
      startDate.setHours(0, 0, 0, 0)
      const endDate = new Date(event.end)
      endDate.setHours(0, 0, 0, 0)
      startTime = startDate.toISOString()
      endTime = endDate.toISOString()

      console.log(`[Calendar] Converting timed event to all-day event: ${startTime} - ${endTime}`)
    }

    // 统一截断：分时事件不得跨天（包括拖动/拉伸）
    if (!isNowAllDay && event.start && event.end) {
      let start = new Date(event.start)
      let end = new Date(event.end)

      // 检查是否在同一天（UTC日期比较）
      const startDay = start.toISOString().split('T')[0]
      const endDay = end.toISOString().split('T')[0]

      if (startDay !== endDay) {
        // 跨天了：根据当前日历视图日期决定保留哪一天
        const viewDate = currentDateRef.value || startDay // 默认保留start那天

        if (viewDate === endDay) {
          // 视图日期是end那天，截断start到end那天的开始
          const dayStart = new Date(end)
          dayStart.setHours(0, 0, 0, 0)
          start = dayStart
          startTime = start.toISOString()
          console.log(
            `[Calendar] Cross-day detected, keeping view date (${viewDate}), truncating start to ${startTime}`
          )
        } else {
          // 视图日期是start那天（或默认），截断到start那天的末尾
          const dayEnd = new Date(start)
          dayEnd.setHours(0, 0, 0, 0)
          dayEnd.setDate(dayEnd.getDate() + 1)
          end = dayEnd
          endTime = end.toISOString()
          console.log(
            `[Calendar] Cross-day detected, keeping view date (${viewDate}), truncating end to ${endTime}`
          )
        }
      }
    }

    try {
      await timeBlockStore.updateTimeBlock(event.id, {
        title: event.title,
        start_time: startTime,
        end_time: endTime,
        is_all_day: isNowAllDay, // ✅ 更新全天标志
      })
    } catch (error) {
      console.error('Failed to update event:', error)

      // 显示错误信息给用户
      let errorMessage = 'Could not update the event. It might be overlapping with another event.'
      if (error instanceof Error) {
        errorMessage = error.message
      } else if (typeof error === 'string') {
        errorMessage = error
      }

      console.error(`更新事件失败: ${errorMessage}`)
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

  return {
    handleDateSelect,
    handleEventChange,
    handleEventContextMenu,
  }
}
