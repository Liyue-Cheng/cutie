/**
 * useTimePosition - 从拖拽位置计算日历时间
 *
 * 🎯 核心功能：
 * 将鼠标的屏幕坐标（clientX, clientY）转换为日历时间（Date 对象）
 *
 * 🔑 使用场景：
 * 1. 拖拽任务到日历：计算任务应该放在哪个时间点
 * 2. 自定义框选预览：mousemove 时计算当前鼠标对应的时间
 * 3. 时间块拖动/调整：FullCalendar 拖动事件的时间校准
 *
 * ⚡ 性能优化：
 * - 使用 DOMRect 缓存，避免频繁 getBoundingClientRect()
 * - 节流更新（16ms ≈ 60fps），减少布局抖动
 *
 * 🎯 精度优化：
 * - 优先读取鼠标下方 .fc-timegrid-slot[data-time] 的精确时间
 * - 回退使用线性插值计算（基于整体高度比例）
 * - 自动对齐到 5 分钟网格（与 FullCalendar snapDuration 一致）
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
   *
   * 🎯 算法流程：
   * 1. 精确模式：读取鼠标下方 slot 的 data-time 属性（整点时间）
   * 2. 回退模式：使用 Y 坐标比例 × 24 小时计算（亚像素精度）
   * 3. 对齐网格：最终结果对齐到 5 分钟步长
   * 4. 多列处理：周视图/多日视图时先确定鼠标在哪一列（日期）
   *
   * @param event 拖拽事件（需要 clientX/clientY）
   * @param currentTarget 当前目标元素（日历容器 DOM）
   * @returns 计算得到的时间（Date 对象），无法计算返回 null
   *
   * 📌 注意：返回的时间已设置为本地时间（非 UTC）
   */
  function getTimeFromDropPosition(event: DragEvent, currentTarget: HTMLElement): Date | null {
    // 🔄 缓存 DOM 元素，避免重复查询
    // .fc-timegrid-body 是 FullCalendar 时间网格的主体容器
    if (!cachedCalendarEl.value) {
      cachedCalendarEl.value = currentTarget.querySelector('.fc-timegrid-body')
    }
    if (!cachedCalendarEl.value) return null

    // ⚡ 节流优化：只在必要时重新计算位置（16ms ≈ 60fps）
    // getBoundingClientRect() 会触发浏览器回流，频繁调用会拖慢交互
    const now = Date.now()
    if (!cachedRect.value || now - lastUpdateTime.value > UPDATE_THROTTLE) {
      cachedRect.value = cachedCalendarEl.value.getBoundingClientRect()
      lastUpdateTime.value = now
    }

    if (!calendarRef.value) return null
    const calendarApi = calendarRef.value.getApi()
    const currentView = calendarApi.view

    // 📅 第一步：根据视图类型确定"哪一天"
    // 单日视图：直接用 calendarApi.getDate()
    // 多日/周视图：需要根据鼠标 X 坐标判断落在哪一列（哪一天）
    let currentDate: Date
    if (currentView.type === 'timeGridWeek' || currentView.type === 'timeGrid3Days') {
      // 📍 多日视图：找到鼠标实际所在的日期列
      // ⚠️ 注意：排除第一列（时间轴列），只查询有 data-date 属性的日期列
      // 例如：周视图有 7 列，三天视图有 3 列
      const dayColumns = currentTarget.querySelectorAll('.fc-timegrid-col[data-date]')
      let dayIndex = -1

      // 🔍 遍历所有日期列，检查鼠标 X 坐标落在哪一列
      for (let i = 0; i < dayColumns.length; i++) {
        const col = dayColumns[i] as HTMLElement
        const colRect = col.getBoundingClientRect()

        // 判断鼠标是否在这一列的左右边界内
        if (event.clientX >= colRect.left && event.clientX <= colRect.right) {
          dayIndex = i
          break
        }
      }

      // 🔄 Fallback：如果没找到列（比如鼠标在边界外）
      // 使用线性插值：X 坐标比例 × 总天数 = 列索引
      if (dayIndex === -1) {
        const relativeX = event.clientX - cachedRect.value.left
        const columnPercentage = relativeX / cachedRect.value.width
        const numDays = currentView.type === 'timeGridWeek' ? 7 : 3
        dayIndex = Math.floor(columnPercentage * numDays)
        dayIndex = Math.max(0, Math.min(dayIndex, numDays - 1)) // 限制在有效范围
      }

      // 📅 计算目标日期：视图起始日期 + 列偏移
      // activeStart 是 FullCalendar 视图的第一天（可能是周日或周一）
      const viewStart = new Date(currentView.activeStart)
      currentDate = new Date(viewStart)
      currentDate.setDate(viewStart.getDate() + dayIndex)
      currentDate.setHours(0, 0, 0, 0) // 重置到当天 00:00
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

    // 优先根据鼠标所在的 slot 元素获取精确时间
    const slotElement = document
      .elementFromPoint(event.clientX, event.clientY)
      ?.closest('.fc-timegrid-slot[data-time]') as HTMLElement | null

    if (slotElement) {
      const timeAttr = slotElement.getAttribute('data-time')
      if (timeAttr) {
        const [hourStr, minuteStr, secondStr] = timeAttr.split(':')
        const dropTime = new Date(currentDate)
        dropTime.setHours(Number(hourStr) || 0, Number(minuteStr) || 0, Number(secondStr) || 0, 0)
        return dropTime
      }
    }

    // 计算 Y 坐标对应的时间（回退）
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
   *
   * 🔄 使用场景：
   * - 日历日期切换（切换到另一天/周/月）
   * - 视图类型切换（单日 ↔ 周 ↔ 月）
   * - 缩放等级变化（1x ↔ 2x ↔ 3x）
   *
   * 清除后，下次调用 getTimeFromDropPosition 会重新查询 DOM
   */
  function clearCache() {
    cachedCalendarEl.value = null
    cachedRect.value = null
  }

  /**
   * 重置缓存（在拖拽开始时使用）
   *
   * 📌 与 clearCache 的区别：
   * - clearCache：静默清除，不输出日志
   * - resetCache：带日志的清除，用于拖拽进入时强制刷新
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
