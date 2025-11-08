/**
 * useTimePosition - 从拖拽位置计算日历时间
 *
 * 将鼠标拖拽的屏幕坐标转换为日历时间，用于预览和创建时间块
 */

import { ref, type Ref } from 'vue'
import type FullCalendar from '@fullcalendar/vue3'
import { logger, LogTags } from '@/infra/logging/logger'

const UPDATE_THROTTLE = 16 // 约60fps

export function useTimePosition(calendarRef: Ref<InstanceType<typeof FullCalendar> | null>) {
  const cachedCalendarEl = ref<HTMLElement | null>(null)
  const cachedRect = ref<DOMRect | null>(null)
  const lastUpdateTime = ref(0)

  /**
   * 从拖拽位置计算时间
   * @param event 拖拽事件
   * @param currentTarget 当前目标元素（日历容器）
   * @returns 计算得到的时间，如果无法计算返回 null
   */
  function getTimeFromDropPosition(event: DragEvent, currentTarget: HTMLElement): Date | null {
    // 缓存DOM元素和位置信息，避免重复查询
    if (!cachedCalendarEl.value) {
      cachedCalendarEl.value = currentTarget.querySelector('.fc-timegrid-body')
    }
    if (!cachedCalendarEl.value) return null

    // 只在必要时重新计算位置
    const now = Date.now()
    if (!cachedRect.value || now - lastUpdateTime.value > UPDATE_THROTTLE) {
      cachedRect.value = cachedCalendarEl.value.getBoundingClientRect()
      lastUpdateTime.value = now
    }

    if (!calendarRef.value) return null
    const calendarApi = calendarRef.value.getApi()
    const currentView = calendarApi.view

    // 🔧 FIX: 根据视图类型确定日期
    let currentDate: Date
    if (currentView.type === 'timeGridWeek' || currentView.type === 'timeGrid3Days') {
      // 周视图或三天视图：找到鼠标实际所在的日期列
      // 注意：排除第一列（时间轴列），只查询有 data-date 属性的日期列
      const dayColumns = currentTarget.querySelectorAll('.fc-timegrid-col[data-date]')
      let dayIndex = -1

      // 遍历所有日期列，找到鼠标所在的列
      for (let i = 0; i < dayColumns.length; i++) {
        const col = dayColumns[i] as HTMLElement
        const colRect = col.getBoundingClientRect()
        
        // 检查鼠标X坐标是否在这一列的范围内
        if (event.clientX >= colRect.left && event.clientX <= colRect.right) {
          dayIndex = i
          break
        }
      }

      // 如果没找到（比如在边界外），使用fallback逻辑
      if (dayIndex === -1) {
        const relativeX = event.clientX - cachedRect.value.left
        const columnPercentage = relativeX / cachedRect.value.width
        const numDays = currentView.type === 'timeGridWeek' ? 7 : 3
        dayIndex = Math.floor(columnPercentage * numDays)
        dayIndex = Math.max(0, Math.min(dayIndex, numDays - 1))
      }

      // 获取视图起始日期
      const viewStart = new Date(currentView.activeStart)
      currentDate = new Date(viewStart)
      currentDate.setDate(viewStart.getDate() + dayIndex)
      currentDate.setHours(0, 0, 0, 0)
    } else if (currentView.type === 'dayGridMonth') {
      // 月视图：月视图通常不需要精确时间，这里返回当日0点
      // （月视图的拖放通常在 useCalendarInteractDrag 中通过 fc-daygrid-day 处理）
      currentDate = calendarApi.getDate()
      currentDate.setHours(0, 0, 0, 0)
    } else {
      // 单天视图：直接使用日历显示的日期
      currentDate = calendarApi.getDate()
      currentDate.setHours(0, 0, 0, 0)
    }

    // 计算 Y 坐标对应的时间
    const relativeY = event.clientY - cachedRect.value.top
    const percentage = relativeY / cachedRect.value.height

    // 计算时间（从0:00到24:00，共24小时）
    const step = 5 // 分钟步长
    let totalMinutes = percentage * 24 * 60
    // 防止越界：限制在 [0, 24h - step]
    totalMinutes = Math.max(0, Math.min(totalMinutes, 24 * 60 - step))
    const hours = Math.floor(totalMinutes / 60)
    const minutes = Math.floor((totalMinutes % 60) / step) * step // 5分钟间隔对齐

    const dropTime = new Date(currentDate)
    dropTime.setHours(hours, minutes, 0, 0)

    // 🔍 检查点3 & 4：日历日期同步 & 缓存（暂时禁用详细日志）
    // logger.debug(LogTags.COMPONENT_CALENDAR, 'Drop position calculated', {
    //   viewType: currentView.type,
    //   calendarDate: currentDate.toISOString().split('T')[0],
    //   dropTime: dropTime.toISOString(),
    //   clientX: event.clientX,
    //   clientY: event.clientY,
    //   cachedRectTop: cachedRect.value.top,
    //   cachedRectLeft: cachedRect.value.left,
    //   relativeY,
    //   percentage: percentage.toFixed(3),
    //   lastUpdateTime: now - lastUpdateTime.value,
    // })

    return dropTime
  }

  /**
   * 清除缓存
   */
  function clearCache() {
    cachedCalendarEl.value = null
    cachedRect.value = null
  }

  /**
   * 重置缓存（在日期切换等场景下使用）
   */
  function resetCache() {
    cachedCalendarEl.value = null
    cachedRect.value = null
    logger.debug(LogTags.COMPONENT_CALENDAR, 'Cache reset on drag enter')
  }

  return {
    getTimeFromDropPosition,
    clearCache,
    resetCache,
  }
}
