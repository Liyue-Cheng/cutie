/**
 * Interact.js 拖放系统类型定义
 *
 * 基于增强版拖放功能需求说明书 V2
 * 支持双重视觉元素、非破坏性预览、越界即时回弹
 */

import type { DragObject, DragObjectType } from '@/types/dtos'
import type { ViewMetadata } from '@/types/drag'
import type { DragSession } from '@/infra/drag/types'

// ==================== 基础类型 ====================

/**
 * 坐标位置
 */
export interface Position {
  x: number
  y: number
}

/**
 * dropzone 的矩形定义
 */
export interface DropzoneRect {
  left: number
  right: number
  top: number
  bottom: number
  width: number
  height: number
}

/**
 * 拖放阶段枚举
 */
export const DragPhase = {
  IDLE: 'IDLE',
  PREPARING: 'PREPARING',
  DRAGGING: 'DRAGGING',
  OVER_TARGET: 'OVER_TARGET',
  DROPPING: 'DROPPING',
} as const

export type DragPhase = (typeof DragPhase)[keyof typeof DragPhase]

// ==================== 拖放会话 ====================

/**
 * 拖放会话数据
 *
 * ⚠️ 统一使用新策略系统的类型定义
 * 从 @/infra/drag/types 导入 DragSession
 *
 * 这里重新导出以保持向后兼容
 */
export type { DragSession }

// ==================== 预览状态 ====================

/**
 * 拖放预览状态
 * 驱动所有组件的响应式渲染
 *
 * @template T 被拖放对象的类型，默认为 DragObject 联合类型
 */
export interface DragPreviewState<T = DragObject> {
  type: 'kanban' | 'calendar'

  /** 原始数据 */
  raw: {
    draggedObject: T // 被拖动的对象（泛型）
    objectType: DragObjectType // 对象类型
    sourceZoneId: string // 拖动开始时的列表ID
    targetZoneId: string | null // 当前悬停的目标列表ID (null = 越界回弹)
    mousePosition: Position // 鼠标位置
  }

  /** 计算数据 */
  computed: {
    dropIndex?: number // 在目标列表中的插入位置
    isCompact?: boolean // 是否启用截断预览
    calendarMeta?: {
      start: string
      end: string
      isAllDay: boolean
      title: string
      color: string
    }
  }
}

// ==================== 拖放管理器状态 ====================

/**
 * 拖放管理器内部状态
 */
export interface DragManagerState {
  phase: DragPhase
  session: DragSession | null
  targetZone: string | null
  dropIndex: number | null
}

// ==================== 配置选项 ====================

/**
 * 可拖拽元素配置
 */
export interface DraggableOptions {
  /** 获取拖拽数据的函数 */
  getData: (element: HTMLElement) => DragData<any>
}

/**
 * 拖放区配置
 */
export interface DropzoneOptions {
  /** 区域ID */
  zoneId: string

  /** 区域类型 */
  type: 'kanban' | 'calendar'

  /** 自定义矩形计算函数（可选） */
  rectChecker?: (element: HTMLElement) => DropzoneRect

  /** 计算预览位置的函数（可选，由控制器提供标准实现） */
  computePreview?: (rawData: DragPreviewRawData<any>, element: HTMLElement) => DragPreviewComputed

  /** 放置处理函数 */
  onDrop?: (session: DragSession<any>) => Promise<void>
}

/**
 * 拖拽数据
 *
 * @template T 被拖放对象的类型，默认为 DragObject 联合类型
 */
export interface DragData<T = DragObject> {
  type: DragObjectType
  data: T // 泛型数据，替代原来的 task 字段
  sourceView: ViewMetadata
  index: number
  // 🔥 V2: 源组件的灵活上下文数据
  sourceContext: Record<string, any>
}

/**
 * 预览原始数据
 *
 * @template T 被拖放对象的类型，默认为 DragObject 联合类型
 */
export interface DragPreviewRawData<T = DragObject> {
  mousePosition: Position
  draggedObject: T // 改为泛型
  objectType: DragObjectType
  targetZoneId: string
  sourceZoneId: string
}

/**
 * 预览计算数据
 */
export interface DragPreviewComputed {
  dropIndex?: number
  calendarMeta?: {
    start: string
    end: string
    isAllDay: boolean
    title: string
    color: string
  }
}

// ==================== 中断检测 ====================

/**
 * 中断检测器接口（预留）
 */
export interface InterruptionDetector {
  shouldInterrupt(session: DragSession): Promise<boolean>
  getInterruptionReason(): string
}
