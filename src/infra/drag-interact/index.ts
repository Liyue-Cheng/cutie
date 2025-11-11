/**
 * Interact.js 拖放系统统一导出
 */

// 核心管理器 & 调试状态
export { interactManager, controllerDebugState } from './drag-controller'

// 预览状态
export {
  dragPreviewState,
  hasPreview,
  previewType,
  isRebounding,
  previewMousePosition,
  getPreviewDebugInfo,
  dragPreviewActions, // 🔥 导出 actions（包含 forceReset）
} from './preview-state'

// 类型定义
export type * from './types'

// 工具函数
export * from './utils'
