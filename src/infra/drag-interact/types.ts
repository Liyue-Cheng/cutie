/**
 * Interact.js 拖放系统类型定义
 *
 * 基于增强版拖放功能需求说明书 V2
 * 支持双重视觉元素、非破坏性预览、越界即时回弹
 */

import type { TaskCard } from '@/types/dtos'
import type { ViewMetadata } from '@/types/drag'

// ==================== 基础类型 ====================

/**
 * 坐标位置
 */
export interface Position {
  x: number
  y: number
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
 * 在拖动开始时捕获完整快照，避免时序竞争
 */
export interface DragSession {
  /** 拖放源信息 */
  source: {
    viewType: string
    viewId: string
    date?: string
    areaId?: string
  }

  /** 被拖放物体信息 */
  object: {
    type: 'task'
    data: TaskCard // 完整的任务数据快照
    originalIndex: number // 在源列表中的位置
  }

  /** 拖放目标信息（动态更新） */
  target: {
    viewType?: string
    viewId?: string
    date?: string
    dropIndex?: number
    calendarMeta?: CalendarDropMeta
  } | null
}

/**
 * 日历拖放元数据
 */
export interface CalendarDropMeta {
  dropTime: Date
  isAllDay: boolean
  viewDate: string
}

// ==================== 预览状态 ====================

/**
 * 拖放预览状态
 * 驱动所有组件的响应式渲染
 */
export interface DragPreviewState {
  type: 'kanban' | 'calendar'

  /** 原始数据 */
  raw: {
    ghostTask: TaskCard // 被拖动的任务
    sourceZoneId: string // 拖动开始时的列表ID
    targetZoneId: string | null // 当前悬停的目标列表ID (null = 越界回弹)
    mousePosition: Position // 鼠标位置
  }

  /** 计算数据 */
  computed: {
    dropIndex?: number // 在目标列表中的插入位置
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
  getData: (element: HTMLElement) => DragData
}

/**
 * 拖放区配置
 */
export interface DropzoneOptions {
  /** 区域ID */
  zoneId: string

  /** 区域类型 */
  type: 'kanban' | 'calendar'

  /** 计算预览位置的函数（可选，由控制器提供标准实现） */
  computePreview?: (rawData: DragPreviewRawData, element: HTMLElement) => DragPreviewComputed

  /** 放置处理函数 */
  onDrop?: (session: DragSession) => Promise<void>
}

/**
 * 拖拽数据
 */
export interface DragData {
  type: 'task'
  task: TaskCard
  sourceView: ViewMetadata
  index: number
}

/**
 * 预览原始数据
 */
export interface DragPreviewRawData {
  mousePosition: Position
  ghostTask: TaskCard
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
