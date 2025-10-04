/**
 * useAutoScroll - 拖拽自动滚动工具
 *
 * 当拖拽到容器边缘时自动滚动
 */

import { ref } from 'vue'
import type { AutoScrollOptions } from '@/types/drag'

const DEFAULT_OPTIONS: Required<AutoScrollOptions> = {
  edgeSize: 50, // 边缘触发距离（像素）
  speed: 5, // 基础滚动速度（像素/帧）
  maxSpeed: 20, // 最大滚动速度（像素/帧）
}

/**
 * 自动滚动工具
 */
export function useAutoScroll(options: AutoScrollOptions = {}) {
  const config = { ...DEFAULT_OPTIONS, ...options }

  const scrollTimer = ref<number | null>(null)
  const isScrolling = ref(false)

  /**
   * 开始自动滚动
   * @param container - 滚动容器
   * @param direction - 滚动方向和速度（正数向下/右，负数向上/左）
   * @param axis - 滚动轴（'x' 或 'y'）
   */
  function startAutoScroll(container: HTMLElement, direction: number, axis: 'x' | 'y' = 'y'): void {
    // 如果已经在滚动，不重复启动
    if (scrollTimer.value !== null) return

    isScrolling.value = true

    scrollTimer.value = window.setInterval(() => {
      if (axis === 'y') {
        container.scrollTop += direction
      } else {
        container.scrollLeft += direction
      }
    }, 16) // ~60fps

    console.log('[useAutoScroll] Started:', { direction, axis })
  }

  /**
   * 停止自动滚动
   */
  function stopAutoScroll(): void {
    if (scrollTimer.value !== null) {
      clearInterval(scrollTimer.value)
      scrollTimer.value = null
      isScrolling.value = false

      console.log('[useAutoScroll] Stopped')
    }
  }

  /**
   * 处理拖拽事件的自动滚动
   * @param event - DragEvent
   * @param container - 滚动容器（可选，默认查找最近的可滚动父元素）
   */
  function handleAutoScroll(event: DragEvent, container?: HTMLElement): void {
    const target = container || findScrollableContainer(event.currentTarget as HTMLElement)
    if (!target) {
      stopAutoScroll()
      return
    }

    const rect = target.getBoundingClientRect()
    const mouseY = event.clientY
    const mouseX = event.clientX

    // 检查垂直滚动
    if (target.scrollHeight > target.clientHeight) {
      const distanceToTop = mouseY - rect.top
      const distanceToBottom = rect.bottom - mouseY

      if (distanceToTop < config.edgeSize) {
        // 向上滚动
        const speed = calculateSpeed(distanceToTop, config.edgeSize, config.speed, config.maxSpeed)
        startAutoScroll(target, -speed, 'y')
        return
      } else if (distanceToBottom < config.edgeSize) {
        // 向下滚动
        const speed = calculateSpeed(
          distanceToBottom,
          config.edgeSize,
          config.speed,
          config.maxSpeed
        )
        startAutoScroll(target, speed, 'y')
        return
      }
    }

    // 检查水平滚动
    if (target.scrollWidth > target.clientWidth) {
      const distanceToLeft = mouseX - rect.left
      const distanceToRight = rect.right - mouseX

      if (distanceToLeft < config.edgeSize) {
        // 向左滚动
        const speed = calculateSpeed(distanceToLeft, config.edgeSize, config.speed, config.maxSpeed)
        startAutoScroll(target, -speed, 'x')
        return
      } else if (distanceToRight < config.edgeSize) {
        // 向右滚动
        const speed = calculateSpeed(
          distanceToRight,
          config.edgeSize,
          config.speed,
          config.maxSpeed
        )
        startAutoScroll(target, speed, 'x')
        return
      }
    }

    // 不在边缘，停止滚动
    stopAutoScroll()
  }

  /**
   * 查找可滚动容器
   */
  function findScrollableContainer(element: HTMLElement | null): HTMLElement | null {
    let current = element

    while (current && current !== document.body) {
      const style = window.getComputedStyle(current)
      const overflow = style.overflow + style.overflowY + style.overflowX

      if (overflow.includes('scroll') || overflow.includes('auto')) {
        // 检查是否真的可滚动
        if (
          current.scrollHeight > current.clientHeight ||
          current.scrollWidth > current.clientWidth
        ) {
          return current
        }
      }

      current = current.parentElement
    }

    return null
  }

  /**
   * 计算滚动速度（距离越近速度越快）
   */
  function calculateSpeed(
    distance: number,
    edgeSize: number,
    baseSpeed: number,
    maxSpeed: number
  ): number {
    // 距离越近，speed 越大
    const ratio = 1 - distance / edgeSize
    return Math.min(baseSpeed + ratio * (maxSpeed - baseSpeed), maxSpeed)
  }

  return {
    isScrolling,
    startAutoScroll,
    stopAutoScroll,
    handleAutoScroll,
  }
}
