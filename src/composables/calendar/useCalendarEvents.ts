/**
 * useCalendarEvents - 日历事件数据管理
 *
 * 从 store 读取时间块数据并转换为 FullCalendar 事件格式
 */

import { computed, type Ref } from 'vue'
import type { EventInput } from '@fullcalendar/core'
import { useTimeBlockStore } from '@/stores/timeblock'
import { useAreaStore } from '@/stores/area'

export function useCalendarEvents(previewEvent: Ref<EventInput | null>) {
  const timeBlockStore = useTimeBlockStore()
  const areaStore = useAreaStore()

  /**
   * 日历事件列表（响应式）
   *
   * ✅ 正确做法：
   * - 使用 computed 包装，从 store.allTimeBlocks getter 读取
   * - allTimeBlocks 是 computed，当 store.timeBlocks 变化时自动重新计算
   * - 任何对 store 的操作（create/update/delete）都会触发 UI 更新
   *
   * ❌ 常见错误：
   * - 不要缓存 timeBlocks 到本地 ref/reactive
   * - 不要在组件内维护时间块列表的副本
   * - 所有操作必须通过 store，不要直接修改本地状态
   */
  const calendarEvents = computed((): EventInput[] => {
    // ✅ 直接从 store 的 computed getter 读取，确保响应式更新
    const events = timeBlockStore.allTimeBlocks.map((timeBlock) => {
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
        const startTimeLocal = timeBlock.start_time_local // HH:MM:SS
        const endTimeLocal = timeBlock.end_time_local // HH:MM:SS

        // 解析本地时间
        const [startHour, startMin, startSec] = startTimeLocal.split(':').map(Number)
        const [endHour, endMin, endSec] = endTimeLocal.split(':').map(Number)

        // 创建显示时间（保持原日期，使用本地时间）
        const displayStart = new Date(baseDate)
        displayStart.setHours(startHour, startMin, startSec || 0, 0)

        const displayEnd = new Date(baseDate)
        displayEnd.setHours(endHour, endMin, endSec || 0, 0)

        displayStartTime = displayStart.toISOString()
        displayEndTime = displayEnd.toISOString()
      } else {
        // 固定时间：直接使用UTC时间
        displayStartTime = timeBlock.start_time
        displayEndTime = timeBlock.end_time
      }

      return {
        id: timeBlock.id,
        title: timeBlock.title ?? 'Time Block',
        start: displayStartTime,
        end: displayEndTime,
        allDay: timeBlock.is_all_day, // ✅ 使用后端返回的 is_all_day 字段
        color: color,
      }
    })

    // 添加预览事件
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
