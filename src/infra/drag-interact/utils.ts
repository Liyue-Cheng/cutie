/**
 * 拖放系统工具函数
 *
 * 提供拖放过程中需要的各种计算和辅助功能
 */

import type { Position } from './types'
import type { TaskCard } from '@/types/dtos'

// ==================== DOM 操作工具 ====================

/**
 * 计算看板列表中的插入位置（方向感知 + 邻居10%触发）
 *
 * 规则：
 * - 使用“邻居触发区”驱动步进：
 *   - 上移：当指针进入“上邻居”的顶部10%（且至少MIN像素）时，占位上移一位
 *   - 下移：当指针进入“下邻居”的底部10%（且至少MIN像素）时，占位下移一位
 * - 初次进入（无 lastDropIndex）不使用中线，改为按项 bottom 定位到更稳定的初始位置，避免起拖即跳变
 *
 * @param mouseY 鼠标Y坐标（页面坐标）
 * @param wrappers 任务卡片包装元素列表
 * @param lastDropIndex 上一次的插入位置（用于步进起点）
 * @returns 插入位置索引（0..wrappers.length）
 */
export function calculateDropIndex(
  mouseY: number,
  wrappers: HTMLElement[],
  lastDropIndex?: number
): number {
  if (wrappers.length === 0) return 0

  const ZONE_RATIO = 0.1 // 邻边触发区比例
  const MIN_ZONE_PX = 8 // 最小像素阈值，适配超小项

  const zonePx = (h: number) => Math.max(h * ZONE_RATIO, MIN_ZONE_PX)

  // =============== 情况 A：有历史占位索引 → 方向感知步进 ===============
  if (lastDropIndex !== undefined && lastDropIndex !== null) {
    let i = Math.max(0, Math.min(lastDropIndex, wrappers.length))

    // 允许一次跨越多项：循环消费触发区
    while (true) {
      let moved = false

      // 尝试上移：检查上一项的底部10%
      const prevIndex = i - 1
      if (prevIndex >= 0) {
        const prevEl = wrappers[prevIndex]
        if (!prevEl) break
        const prevRect = prevEl.getBoundingClientRect()
        const topEdge = prevRect.bottom - zonePx(prevRect.height)
        if (mouseY <= topEdge) {
          i = prevIndex
          moved = true
        }
      }

      // 若未上移，尝试下移：检查下邻居（当前位置 i+1 所指向的项）的顶部10%
      if (!moved) {
        const nextIndex = i + 1
        if (nextIndex < wrappers.length) {
          const nextEl = wrappers[nextIndex]
          if (!nextEl) break
          const nextRect = nextEl.getBoundingClientRect()
          const bottomEdge = nextRect.top + zonePx(nextRect.height)
          if (mouseY >= bottomEdge) {
            i = nextIndex + 1
            moved = true
          }
        }
      }

      if (!moved) break
    }

    return Math.max(0, Math.min(i, wrappers.length))
  }

  // =============== 情况 B：无历史索引 → 稳定初始定位 ===============
  // 采用“按项 bottom”定位：返回第一个 bottom >= mouseY 的元素索引
  // 好处：即便在当前项下半部起拖，也不会立刻判为“插到下一项之后”
  for (let i = 0; i < wrappers.length; i++) {
    const el = wrappers[i]
    if (!el) continue
    const rect = el.getBoundingClientRect()
    if (mouseY <= rect.bottom) {
      return i
    }
  }
  return wrappers.length
}

/**
 * 检查鼠标是否真的离开了容器（避免子元素触发 dragleave）
 * @param event DragEvent
 * @param container 容器元素
 * @returns 是否真的离开
 */
export function isReallyLeaving(event: DragEvent, container: HTMLElement): boolean {
  const rect = container.getBoundingClientRect()
  const x = event.clientX
  const y = event.clientY

  return x < rect.left || x > rect.right || y < rect.top || y > rect.bottom
}

/**
 * 获取元素的样式快照（用于幽灵元素）
 * @param element 源元素
 * @returns 样式快照对象
 */
export function captureElementSnapshot(element: HTMLElement) {
  const computedStyle = window.getComputedStyle(element)
  const rect = element.getBoundingClientRect()

  return {
    width: rect.width,
    height: rect.height,
    innerHTML: element.innerHTML,
    boundingRect: {
      left: rect.left,
      top: rect.top,
      width: rect.width,
      height: rect.height,
    },
    computedStyle: {
      backgroundColor: computedStyle.backgroundColor,
      color: computedStyle.color,
      fontSize: computedStyle.fontSize,
      fontFamily: computedStyle.fontFamily,
      borderRadius: computedStyle.borderRadius,
      padding: computedStyle.padding,
      border: computedStyle.border,
      boxShadow: computedStyle.boxShadow,
    },
  }
}

// ==================== 几何计算工具 ====================

/**
 * 计算两点之间的距离
 * @param pos1 位置1
 * @param pos2 位置2
 * @returns 距离（像素）
 */
export function getDistance(pos1: Position, pos2: Position): number {
  const dx = pos2.x - pos1.x
  const dy = pos2.y - pos1.y
  return Math.sqrt(dx * dx + dy * dy)
}

/**
 * 检查点是否在矩形内
 * @param point 点坐标
 * @param rect 矩形区域
 * @returns 是否在矩形内
 */
export function isPointInRect(point: Position, rect: DOMRect): boolean {
  return (
    point.x >= rect.left && point.x <= rect.right && point.y >= rect.top && point.y <= rect.bottom
  )
}

// ==================== 任务数据工具 ====================

/**
 * 从 DOM 元素中提取任务ID
 * @param element DOM元素
 * @returns 任务ID，如果未找到返回null
 */
export function extractTaskId(element: HTMLElement): string | null {
  return (
    element.getAttribute('data-task-id') ||
    element.closest('[data-task-id]')?.getAttribute('data-task-id') ||
    null
  )
}

/**
 * 获取区域颜色（从现有实现中复用）
 * @param areaId 区域ID
 * @returns 颜色值，默认为灰色
 */
export function getAreaColor(areaId: string | null): string {
  // TODO: 从 areaStore 获取颜色，现在先返回默认值
  if (!areaId) {
    return '#6b7280' // gray-500
  }

  // 简单的颜色映射，实际应该从 store 获取
  const colorMap: Record<string, string> = {
    work: '#3b82f6', // blue-500
    personal: '#10b981', // emerald-500
    health: '#f59e0b', // amber-500
    learning: '#8b5cf6', // violet-500
  }

  return colorMap[areaId] || '#6b7280'
}

// ==================== 日历相关工具 ====================

/**
 * 计算任务在日历中的时长
 * @param task 任务数据
 * @returns 时长（分钟）
 */
export function calculateTaskDuration(task: TaskCard): number {
  // 如果是 tiny 任务（estimated_duration 为 0 或 null），使用 15 分钟
  const duration = task.estimated_duration
  if (duration === null || duration === 0) {
    return 15
  }
  return duration
}

/**
 * 检查时间是否在全天区域
 * @param mouseY 鼠标Y坐标
 * @param calendarElement 日历元素
 * @returns 是否在全天区域
 */
export function isInAllDayZone(mouseY: number, calendarElement: HTMLElement): boolean {
  const allDayZone = calendarElement.querySelector('.fc-daygrid-body')
  if (!allDayZone) {
    return false
  }

  const rect = allDayZone.getBoundingClientRect()
  return mouseY >= rect.top && mouseY <= rect.bottom
}

// ==================== 错误处理工具 ====================

/**
 * 显示错误提示（使用现有的提示系统）
 * @param message 错误消息
 */
export function showErrorMessage(message: string): void {
  // 使用简单的 alert，实际项目中应该使用统一的 toast 系统
  console.error('[DragSystem]', message)
  // TODO: 集成项目的 toast 系统
  alert(message)
}

/**
 * 显示警告提示
 * @param message 警告消息
 */
export function showWarningMessage(message: string): void {
  console.warn('[DragSystem]', message)
  // TODO: 集成项目的 toast 系统
  alert(message)
}

// ==================== 调试工具 ====================

/**
 * 生成元素的哈希值（用于缓存优化）
 * @param container 容器元素
 * @returns 哈希字符串
 */
export function hashElements(container: HTMLElement): string {
  const wrappers = container.querySelectorAll('.task-card-wrapper')
  // 简单哈希：元素数量 + 第一个和最后一个元素的位置
  const count = wrappers.length
  const firstTop = wrappers[0]?.getBoundingClientRect().top || 0
  const lastTop = wrappers[count - 1]?.getBoundingClientRect().top || 0

  return `${count}-${Math.round(firstTop)}-${Math.round(lastTop)}`
}
