/**
 * 拖放系统工具函数
 *
 * 提供拖放过程中需要的各种计算和辅助功能
 */

import type { Position } from './types'
import type { TaskCard } from '@/types/dtos'

// ==================== DOM 操作工具 ====================

/**
 * 计算看板列表中的插入位置
 *
 * 使用施密特触发器（迟滞比较器）避免边界抖动：
 * - 向下移动：需要越过下沿 (centerY + 20% height)
 * - 向上移动：需要越过上沿 (centerY - 20% height)
 *
 * @param mouseY 鼠标Y坐标
 * @param wrappers 任务卡片包装元素列表
 * @param lastDropIndex 上一次的插入位置（用于判断移动方向）
 * @returns 插入位置索引
 */
export function calculateDropIndex(
  mouseY: number,
  wrappers: HTMLElement[],
  lastDropIndex?: number
): number {
  if (wrappers.length === 0) {
    return 0
  }

  // 🔥 施密特触发器参数
  const HYSTERESIS = 0.25 // 25% 迟滞区间

  for (let i = 0; i < wrappers.length; i++) {
    const wrapper = wrappers[i]
    if (!wrapper) continue

    const rect = wrapper.getBoundingClientRect()
    const height = rect.height
    const centerY = rect.top + height / 2

    // 计算上下沿（带迟滞）
    const upperThreshold = centerY - height * HYSTERESIS // 上沿：中心线上方 25%
    const lowerThreshold = centerY + height * HYSTERESIS // 下沿：中心线下方 25%

    // 🔥 施密特触发器逻辑
    if (lastDropIndex !== undefined) {
      // 有历史位置，使用迟滞比较
      if (lastDropIndex <= i) {
        // 向下移动或保持：需要越过下沿
        if (mouseY < lowerThreshold) {
          return i
        }
      } else {
        // 向上移动：需要越过上沿
        if (mouseY < upperThreshold) {
          return i
        }
      }
    } else {
      // 没有历史位置（首次计算），使用中心线
      if (mouseY < centerY) {
        return i
      }
    }
  }

  // 如果鼠标在所有元素下方，插入到末尾
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
