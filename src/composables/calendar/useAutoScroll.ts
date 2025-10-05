/**
 * useAutoScroll - 日历拖拽自动滚动功能
 *
 * 在拖拽任务到日历时，如果鼠标靠近边缘，自动向上或向下滚动
 */

import { ref } from 'vue'

const SCROLL_ZONE_SIZE = 100 // 触发滚动的边缘区域大小（像素）
const SCROLL_SPEED = 5 // 滚动速度（像素/次）

export function useAutoScroll() {
  const scrollTimer = ref<number | null>(null)

  /**
   * 处理自动滚动逻辑
   * @param event 拖拽事件
   * @param calendarContainer 日历容器元素
   */
  function handleAutoScroll(event: DragEvent, calendarContainer: HTMLElement) {
    const scrollableEl = calendarContainer.querySelector('.fc-scroller') as HTMLElement

    if (!scrollableEl) return

    const rect = scrollableEl.getBoundingClientRect()
    const mouseY = event.clientY
    const relativeY = mouseY - rect.top

    let scrollDirection = 0

    // 检查是否在顶部滚动区域
    if (relativeY < SCROLL_ZONE_SIZE) {
      scrollDirection = -1 // 向上滚动
    }
    // 检查是否在底部滚动区域
    else if (relativeY > rect.height - SCROLL_ZONE_SIZE) {
      scrollDirection = 1 // 向下滚动
    }

    if (scrollDirection !== 0) {
      startAutoScroll(scrollableEl, scrollDirection)
    } else {
      stopAutoScroll()
    }
  }

  /**
   * 启动自动滚动
   * @param scrollableEl 可滚动元素
   * @param direction 滚动方向（-1 向上，1 向下）
   */
  function startAutoScroll(scrollableEl: HTMLElement, direction: number) {
    // 如果已经在滚动，就不重复启动
    if (scrollTimer.value !== null) return

    scrollTimer.value = window.setInterval(() => {
      const scrollAmount = SCROLL_SPEED * direction
      scrollableEl.scrollTop += scrollAmount

      // 检查是否已经到达边界
      if (direction < 0 && scrollableEl.scrollTop <= 0) {
        stopAutoScroll()
      } else if (
        direction > 0 &&
        scrollableEl.scrollTop >= scrollableEl.scrollHeight - scrollableEl.clientHeight
      ) {
        stopAutoScroll()
      }
    }, 16) // 约60fps
  }

  /**
   * 停止自动滚动
   */
  function stopAutoScroll() {
    if (scrollTimer.value !== null) {
      clearInterval(scrollTimer.value)
      scrollTimer.value = null
    }
  }

  return {
    handleAutoScroll,
    stopAutoScroll,
  }
}
