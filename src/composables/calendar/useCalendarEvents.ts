/**
 * useCalendarEvents - 日历事件数据管理
 *
 * 从 store 读取时间块数据并转换为 FullCalendar 事件格式
 */

import { computed, type Ref } from 'vue'
import type { EventInput } from '@fullcalendar/core'
import { useTimeBlockStore } from '@/stores/timeblock'
import { useTaskStore } from '@/stores/task'
import { useAreaStore } from '@/stores/area'

export function useCalendarEvents(
  previewEvent: Ref<EventInput | null>,
  viewType: Ref<'day' | 'week' | 'month'>
) {
  const timeBlockStore = useTimeBlockStore()
  const taskStore = useTaskStore()
  const areaStore = useAreaStore()

  /**
   * 日历事件列表（响应式）
   *
   * ✅ 正确做法：
   * - 使用 computed 包装，从 store getter 读取
   * - 显示时间块 + 已排期任务
   * - 任何对 store 的操作（create/update/delete）都会触发 UI 更新
   *
   * ❌ 常见错误：
   * - 不要缓存数据到本地 ref/reactive
   * - 不要在组件内维护列表的副本
   * - 所有操作必须通过 store，不要直接修改本地状态
   */
  const calendarEvents = computed((): EventInput[] => {
    const events: EventInput[] = []

    // 1. 添加时间块事件
    timeBlockStore.allTimeBlocks.forEach((timeBlock) => {
      // 颜色优先级：
      // 1. 如果有 area，使用 area 的颜色
      // 2. 如果没有 area 但有关联任务（从任务创建），使用灰色
      // 3. 如果没有 area 也没有关联任务（手动创建），使用青色
      let color = '#bceaee' // 默认青色（手动创建）
      // ✅ 通过 area_id 从 store 获取完整 area 信息
      const area = timeBlock.area_id ? areaStore.getAreaById(timeBlock.area_id) : null
      if (area) {
        color = area.color
      } else if (timeBlock.linked_tasks && timeBlock.linked_tasks.length > 0) {
        color = '#9ca3af' // 灰色（从无 area 任务创建）
      }

      // 计算显示时间
      let displayStartTime: string
      let displayEndTime: string

      if (
        timeBlock.time_type === 'FLOATING' &&
        timeBlock.start_time_local &&
        timeBlock.end_time_local
      ) {
        // 浮动时间：将本地时间应用到当前日期
        const baseDate = new Date(timeBlock.start_time) // 获取原始日期

        // 验证日期是否有效
        if (isNaN(baseDate.getTime())) {
          // 如果日期无效，跳过这个时间块
          return
        }

        const startTimeLocal = timeBlock.start_time_local // HH:MM:SS
        const endTimeLocal = timeBlock.end_time_local // HH:MM:SS

        // 解析本地时间
        const [startHour, startMin, startSec] = startTimeLocal.split(':').map((n) => Number(n) || 0)
        const [endHour, endMin, endSec] = endTimeLocal.split(':').map((n) => Number(n) || 0)

        // 创建显示时间（保持原日期，使用本地时间）
        const displayStart = new Date(baseDate)
        displayStart.setHours(startHour || 0, startMin || 0, startSec || 0, 0)

        const displayEnd = new Date(baseDate)
        displayEnd.setHours(endHour || 0, endMin || 0, endSec || 0, 0)

        // 再次验证计算后的日期
        if (isNaN(displayStart.getTime()) || isNaN(displayEnd.getTime())) {
          return
        }

        displayStartTime = displayStart.toISOString()
        displayEndTime = displayEnd.toISOString()
      } else {
        // 固定时间：直接使用UTC时间
        displayStartTime = timeBlock.start_time
        displayEndTime = timeBlock.end_time

        // 验证时间字符串是否有效
        const startDate = new Date(displayStartTime)
        const endDate = new Date(displayEndTime)
        if (isNaN(startDate.getTime()) || isNaN(endDate.getTime())) {
          return
        }
      }

      events.push({
        id: timeBlock.id, // ✅ 直接使用真实的 UUID，简化设计
        title: timeBlock.title ?? 'Time Block',
        start: displayStartTime,
        end: displayEndTime,
        allDay: timeBlock.is_all_day,
        color: color,
        extendedProps: {
          type: 'timeblock',
        },
      })
    })

    // 2. 添加已排期任务事件（仅在月视图）
    if (viewType.value === 'month') {
      const tasksWithTimeBlocks = new Set(
        timeBlockStore.allTimeBlocks.flatMap((tb) => (tb.linked_tasks || []).map((t) => t.id))
      )

      // 遍历所有已排期的任务
      taskStore.plannedTasks.forEach((task) => {
        // 跳过已完成的任务
        if (task.is_completed) return

        // 如果任务已经有时间块，不重复显示
        if (tasksWithTimeBlocks.has(task.id)) return

        // 遍历该任务的所有日程
        task.schedules?.forEach((schedule) => {
          const area = task.area_id ? areaStore.getAreaById(task.area_id) : null
          const color = area?.color || '#9ca3af'

          // 任务显示为全日事件
          const startDate = new Date(schedule.scheduled_day + 'T00:00:00')

          // 验证日期有效性
          if (isNaN(startDate.getTime())) {
            return
          }

          const endDate = new Date(startDate)
          endDate.setDate(endDate.getDate() + 1)

          events.push({
            id: `task-${task.id}-${schedule.scheduled_day}`,
            title: `📋 ${task.title}`,
            start: startDate.toISOString(),
            end: endDate.toISOString(),
            allDay: true,
            color: color,
            editable: false, // ✅ 任务事件也不可拖动（它们只是显示，不是时间块）
            classNames: ['task-event'],
            extendedProps: {
              type: 'task',
              taskId: task.id,
              scheduleDay: schedule.scheduled_day,
            },
          })
        })
      })
    }

    // 3. 添加截止日期事件（仅在月视图）
    if (viewType.value === 'month') {
      taskStore.allTasks.forEach((task) => {
        // 跳过已完成、已归档、已删除的任务
        if (task.is_completed || task.is_archived || task.is_deleted) return

        // 只显示有截止日期的任务
        if (!task.due_date) return

        // 截止日期使用特殊颜色：逾期=红色，未逾期=橙色
        const color = task.due_date.is_overdue ? '#ef4444' : '#f59e0b'

        // 截止日期显示为全日事件
        // ✅ due_date.date 是完整的 ISO 8601 字符串（DateTime<Utc>），提取日期部分
        const dueDateTime = new Date(task.due_date.date)

        // 验证日期有效性
        if (isNaN(dueDateTime.getTime())) {
          return
        }

        // 创建当天 00:00:00 的日期（全日事件）
        const startDate = new Date(dueDateTime)
        startDate.setHours(0, 0, 0, 0)

        const endDate = new Date(startDate)
        endDate.setDate(endDate.getDate() + 1)

        events.push({
          id: `due-${task.id}`,
          title: `⏰ ${task.title}`,
          start: startDate.toISOString(),
          end: endDate.toISOString(),
          allDay: true,
          color: color,
          editable: false, // ✅ 截止日期不可拖动
          classNames: ['due-date-event', task.due_date.is_overdue ? 'overdue' : ''],
          extendedProps: {
            type: 'due_date',
            taskId: task.id,
            isOverdue: task.due_date.is_overdue,
          },
        })
      })
    }

    // 4. 添加预览事件
    if (previewEvent.value) {
      events.push({
        id: previewEvent.value.id || 'preview-event',
        title: previewEvent.value.title || '预览',
        start: typeof previewEvent.value.start === 'string' ? previewEvent.value.start : '',
        end: typeof previewEvent.value.end === 'string' ? previewEvent.value.end : '',
        allDay: previewEvent.value.allDay || false,
        color: previewEvent.value.color || '#BCEAEE',
      })
    }

    return events
  })

  return {
    calendarEvents,
  }
}
