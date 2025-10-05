/**
 * useTimePosition - 从拖拽位置计算日历时间
 *
 * 将鼠标拖拽的屏幕坐标转换为日历时间，用于预览和创建时间块
 */

import { ref, type Ref } from 'vue'
import type FullCalendar from '@fullcalendar/vue3'

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

    const relativeY = event.clientY - cachedRect.value.top

    // 计算相对于日历顶部的百分比
    const percentage = relativeY / cachedRect.value.height

    // 🔧 FIX: 获取日历当前显示的日期（而不是系统今天）
    if (!calendarRef.value) return null
    const calendarApi = calendarRef.value.getApi()
    const currentDate = calendarApi.getDate() // 获取日历当前显示的日期
    currentDate.setHours(0, 0, 0, 0)

    // 计算时间（从0:00到24:00，共24小时）
    const totalMinutes = percentage * 24 * 60
    const hours = Math.floor(totalMinutes / 60)
    const minutes = Math.floor((totalMinutes % 60) / 10) * 10 // 10分钟间隔对齐

    const dropTime = new Date(currentDate)
    dropTime.setHours(hours, minutes, 0, 0)

    // 🔍 检查点3 & 4：日历日期同步 & 缓存
    console.log('[CHK-3] Drop position calculated:', {
      calendarDate: currentDate.toISOString().split('T')[0],
      dropTime: dropTime.toISOString(),
      clientY: event.clientY,
      cachedRectTop: cachedRect.value.top,
      relativeY,
      percentage: percentage.toFixed(3),
      lastUpdateTime: now - lastUpdateTime.value,
    })

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
    console.log('[CHK-4] dragenter: reset cache')
  }

  return {
    getTimeFromDropPosition,
    clearCache,
    resetCache,
  }
}
