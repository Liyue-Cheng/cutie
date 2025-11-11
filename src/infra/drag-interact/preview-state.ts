/**
 * 拖放预览状态管理
 *
 * 提供响应式的预览状态，驱动所有组件的渲染
 * 核心特性：
 * - 单一数据源
 * - 响应式更新
 * - 越界回弹支持 (targetZoneId = null)
 */

import { ref, shallowRef, computed, readonly } from 'vue'
import type { DragPreviewState, Position } from './types'
import type { DragObject, DragObjectType } from '@/types/dtos'

// ==================== 内部状态 ====================

/**
 * 内部预览状态（可变）
 */
const _previewState = ref<DragPreviewState<any> | null>(null)
const _mousePosition = shallowRef<Position | null>(null)

const hasWindow = typeof window !== 'undefined'
let pendingMousePosition: Position | null = null
let mousePositionRaf: number | null = null

// ==================== 导出的只读状态 ====================

/**
 * 只读的预览状态（组件订阅）
 */
export const dragPreviewState = readonly(_previewState)
export const previewMousePosition = readonly(_mousePosition)

/**
 * 派生状态：是否有预览
 */
export const hasPreview = computed(() => _previewState.value !== null)

/**
 * 派生状态：预览类型
 */
export const previewType = computed(() => _previewState.value?.type)

/**
 * 派生状态：是否在回弹状态
 */
export const isRebounding = computed(() => {
  const preview = _previewState.value
  return preview !== null && preview.raw.targetZoneId === null
})

// ==================== 状态操作 API ====================

/**
 * 预览状态操作接口
 * 仅供拖放管理器使用
 */
export const dragPreviewActions = {
  /**
   * 设置看板预览
   */
  setKanbanPreview<T = DragObject>(data: {
    draggedObject: T
    objectType: DragObjectType
    sourceZoneId: string
    targetZoneId: string
    mousePosition: Position
    dropIndex?: number
    isCompact?: boolean
  }) {
    const isCompact = data.isCompact === true
    cancelPendingMouseUpdate()

    const initialPosition = { ...data.mousePosition }

    _previewState.value = {
      type: 'kanban',
      raw: {
        draggedObject: data.draggedObject,
        objectType: data.objectType,
        sourceZoneId: data.sourceZoneId,
        targetZoneId: data.targetZoneId,
        mousePosition: initialPosition,
      },
      computed: {
        dropIndex: data.dropIndex,
        isCompact,
      },
    }

    _mousePosition.value = initialPosition
  },

  /**
   * 设置日历预览
   */
  setCalendarPreview<T = DragObject>(data: {
    draggedObject: T
    objectType: DragObjectType
    sourceZoneId: string
    mousePosition: Position
    calendarMeta: {
      start: string
      end: string
      isAllDay: boolean
      title: string
      color: string
    }
  }) {
    cancelPendingMouseUpdate()

    const initialPosition = { ...data.mousePosition }

    _previewState.value = {
      type: 'calendar',
      raw: {
        draggedObject: data.draggedObject,
        objectType: data.objectType,
        sourceZoneId: data.sourceZoneId,
        targetZoneId: 'calendar',
        mousePosition: initialPosition,
      },
      computed: {
        calendarMeta: data.calendarMeta,
      },
    }

    _mousePosition.value = initialPosition
  },

  /**
   * 更新看板预览的插入位置
   */
  updateDropIndex(dropIndex: number) {
    if (_previewState.value?.type === 'kanban') {
      _previewState.value = {
        ..._previewState.value,
        computed: {
          ..._previewState.value.computed,
          dropIndex,
        },
      }
    }
  },

  /**
   * 更新鼠标位置
   */
  updateMousePosition(position: Position) {
    if (!_previewState.value) {
      return
    }

    const lastPosition = _mousePosition.value
    if (lastPosition && lastPosition.x === position.x && lastPosition.y === position.y) {
      return
    }

    pendingMousePosition = { ...position }

    if (!hasWindow) {
      commitMousePosition()
      return
    }

    if (mousePositionRaf !== null) {
      return
    }

    mousePositionRaf = window.requestAnimationFrame(() => {
      mousePositionRaf = null
      commitMousePosition()
    })
  },

  /**
   * 触发越界回弹
   * 关键功能：将 targetZoneId 设置为 null，触发所有组件回弹
   */
  triggerRebound() {
    if (_previewState.value) {
      _previewState.value = {
        ..._previewState.value,
        raw: {
          ..._previewState.value.raw,
          targetZoneId: null, // 🔥 关键：设置为 null，触发回弹
        },
      }
    }
  },

  /**
   * 清除预览（拖动结束）
   */
  clear() {
    cancelPendingMouseUpdate()
    _previewState.value = null
    _mousePosition.value = null
  },

  /**
   * 🔥 安全重置：强制清理所有状态，用于错误恢复
   */
  forceReset() {
    cancelPendingMouseUpdate()
    _previewState.value = null
    _mousePosition.value = null
    pendingMousePosition = null
    if (mousePositionRaf !== null && hasWindow) {
      window.cancelAnimationFrame(mousePositionRaf)
      mousePositionRaf = null
    }
  },
}

// ==================== 调试辅助 ====================

/**
 * 获取调试信息
 */
export function getPreviewDebugInfo() {
  const preview = _previewState.value
  if (!preview) {
    return { status: 'no-preview' }
  }

  // 安全地获取标题（支持多种对象类型）
  const objectTitle = (preview.raw.draggedObject as any)?.title || 'Unknown'

  return {
    status: 'active',
    type: preview.type,
    objectType: preview.raw.objectType,
    sourceZoneId: preview.raw.sourceZoneId,
    targetZoneId: preview.raw.targetZoneId,
    isRebounding: preview.raw.targetZoneId === null,
    dropIndex: preview.computed.dropIndex,
    objectTitle,
    mousePosition: preview.raw.mousePosition,
  }
}

function commitMousePosition() {
  if (!_previewState.value || !pendingMousePosition) {
    return
  }

  const nextPosition = pendingMousePosition
  pendingMousePosition = null

  _previewState.value.raw.mousePosition = nextPosition
  _mousePosition.value = nextPosition
}

function cancelPendingMouseUpdate() {
  pendingMousePosition = null
  if (mousePositionRaf !== null && hasWindow) {
    window.cancelAnimationFrame(mousePositionRaf)
  }
  mousePositionRaf = null
}

/**
 * 开发环境下的状态监听（可选）
 */
if (import.meta.env.DEV) {
  // 在开发环境下可以监听状态变化
  // watchEffect(() => {
  //   const info = getPreviewDebugInfo()
  //   console.debug('[DragPreview]', info)
  // })
}
