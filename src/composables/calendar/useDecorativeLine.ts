/**
 * useDecorativeLine - 日历装饰竖线管理
 *
 * 在当前日期列显示一条装饰性的竖线，跨越整个底部区域
 */

import { ref, watch, nextTick, type Ref } from 'vue'
import type FullCalendar from '@fullcalendar/vue3'

export function useDecorativeLine(
  calendarRef: Ref<InstanceType<typeof FullCalendar> | null>,
  currentDate: Ref<string | undefined>
) {
  const position = ref<number | null>(null)
  const top = ref<number | null>(null)
  const height = ref<number | null>(null)

  /**
   * 更新装饰线位置
   */
  function updatePosition() {
    if (!calendarRef.value) return

    // 获取当前显示的日期字符串（YYYY-MM-DD）
    const displayDate = currentDate.value || new Date().toISOString().split('T')[0]

    // 查找当前日期的单元格
    const calendarEl = calendarRef.value.$el as HTMLElement
    const dateCell = calendarEl.querySelector(
      `.fc-daygrid-day[data-date="${displayDate}"]`
    ) as HTMLElement

    if (dateCell) {
      // 获取外层 TwoRowLayout 的可视容器（以它为参考，避免 padding 影响）
      const layoutEl = calendarEl.closest('.two-row-layout') as HTMLElement
      if (!layoutEl) return

      // 仅覆盖 TwoRowLayout 的下半部分（.bottom-row）
      const bottomRowEl = layoutEl.querySelector('.bottom-row') as HTMLElement | null
      if (!bottomRowEl) return

      const bottomRowRect = bottomRowEl.getBoundingClientRect()
      const cellRect = dateCell.getBoundingClientRect()

      // 使用 viewport 坐标（position: fixed）
      position.value = cellRect.left
      top.value = bottomRowRect.top
      height.value = bottomRowRect.height
    } else {
      position.value = null
      top.value = null
      height.value = null
    }
  }

  /**
   * 初始化 - 设置监听器
   */
  function initialize() {
    // 监听日历视图变化，重新计算竖线位置
    watch(
      currentDate,
      () => {
        nextTick(() => {
          updatePosition()
        })
      },
      { immediate: false }
    )
  }

  return {
    position,
    top,
    height,
    updatePosition,
    initialize,
  }
}
