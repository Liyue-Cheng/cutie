/**
 * Drag & Drop Composables
 *
 * 统一导出所有拖放相关的 Composables
 */

// ==================== 轻量工具包 ====================
export { useDragTransfer } from './useDragTransfer'
export { useAutoScroll } from './useAutoScroll'
export { useThrottledDragOver, useThrottledCallback } from './useThrottledDragOver'
export { useDragState } from './useDragState'
export { useTemplateDrop } from './useTemplateDrop'

// ==================== 跨看板拖放核心 ====================
export { useCrossViewDrag } from './useCrossViewDrag'
export { useSameViewDrag } from './useSameViewDrag'
export { useCrossViewDragTarget } from './useCrossViewDragTarget'

// ==================== 上下文管理（高级用法） ====================
export { useDragContext } from './useCrossViewDrag/context'

// ==================== 策略管理（高级用法） ====================
export {
  dragStrategies,
  registerStrategy,
  unregisterStrategy,
  getRegisteredStrategies,
} from './useCrossViewDrag/strategies'

export { findStrategy, hasStrategy, getStrategyPriority } from './useCrossViewDrag/finder'

// ==================== 类型导出 ====================
export type * from '@/types/drag'
