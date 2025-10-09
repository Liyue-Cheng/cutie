/**
 * 拖放系统类型定义
 *
 * 基于 DRAG_DROP_SYSTEM_DESIGN.md 设计文档
 */

import type { TaskCard } from './dtos'

// ==================== 看板元数据 ====================

/**
 * 看板类型
 */
export type ViewType = 'status' | 'date' | 'project' | 'priority' | 'area' | 'calendar' | 'custom'

/**
 * 看板元数据
 * 描述看板的类型、配置和身份
 */
export interface ViewMetadata {
  /** 看板类型 */
  type: ViewType

  /** 唯一标识符 */
  id: string

  /** 类型特定的配置 */
  config: ViewConfig

  /** 可选：显示名称 */
  label?: string

  /** 可选：图标 */
  icon?: string
}

// ==================== 看板配置（联合类型） ====================

/**
 * 看板配置（联合类型）
 */
export type ViewConfig =
  | StatusViewConfig
  | DateViewConfig
  | ProjectViewConfig
  | PriorityViewConfig
  | AreaViewConfig
  | CalendarViewConfig
  | CustomViewConfig

export interface StatusViewConfig {
  status: 'staging' | 'planned' | 'completed'
}

export interface DateViewConfig {
  /** ISO 8601 格式：YYYY-MM-DD */
  date: string
}

export interface ProjectViewConfig {
  projectId: string
  projectName: string
}

export interface PriorityViewConfig {
  priority: 'high' | 'medium' | 'low'
}

export interface AreaViewConfig {
  areaId: string
  areaName: string
  color: string
}

export interface CalendarViewConfig {
  /** ISO 8601 UTC 格式 */
  startTime: string
  /** ISO 8601 UTC 格式 */
  endTime: string
  /** 是否为全天事件 */
  isAllDay?: boolean
}

export interface CustomViewConfig {
  filter: (task: TaskCard) => boolean
  metadata: Record<string, any>
}

// ==================== 拖放模式 ====================

/**
 * 拖放模式
 */
export type DragMode = 'normal' | 'snap'

export interface NormalDragMode {
  mode: 'normal'
}

export interface SnapDragMode {
  mode: 'snap'
  /** 激活按钮的上下文 */
  activatedBy: string
  /** 额外的模式参数 */
  params?: Record<string, any>
}

// ==================== 拖拽上下文 ====================

/**
 * 拖拽上下文
 * 携带拖拽过程中的所有信息
 */
export interface DragContext {
  /** 被拖拽的任务 */
  task: TaskCard

  /** 源看板元数据 */
  sourceView: ViewMetadata

  /** 拖放模式 */
  dragMode: NormalDragMode | SnapDragMode

  /** 拖拽开始时间（用于性能追踪） */
  startTime: number

  /** 附加数据（可选） */
  metadata?: Record<string, any>
}

// ==================== 策略执行结果 ====================

/**
 * 策略执行结果
 */
export interface StrategyResult {
  /** 是否成功 */
  success: boolean

  /** 错误信息 */
  error?: string

  /** 是否仅重排序（不修改任务数据） */
  reorderOnly?: boolean

  /** 需要更新的视图列表 */
  affectedViews?: string[]

  /** 用户提示消息 */
  message?: string

  /** 更新后的任务（可选） */
  updatedTask?: TaskCard
}

// ==================== 拖放策略 ====================

/**
 * 拖放策略函数
 */
export type DragStrategy = (
  context: DragContext,
  targetView: ViewMetadata
) => Promise<StrategyResult>

/**
 * 策略键：source.type -> target.type
 */
export type StrategyKey = `${ViewType}->${ViewType}` | '*->*' | `${ViewType}->*` | `*->${ViewType}`

/**
 * 策略注册表
 */
export type StrategyRegistry = {
  [key in StrategyKey]?: DragStrategy
}

// ==================== 辅助类型 ====================

/**
 * 自动滚动选项
 */
export interface AutoScrollOptions {
  /** 边缘触发距离（像素） */
  edgeSize?: number
  /** 滚动速度（像素/帧） */
  speed?: number
  /** 最大速度（像素/帧） */
  maxSpeed?: number
}

/**
 * HTML5 拖拽数据传输格式
 */
export type DragTransferData = TaskDragData | TemplateDragData

/**
 * 任务拖放数据
 */
export interface TaskDragData {
  type: 'task'
  task: TaskCard
  sourceView: ViewMetadata
  dragMode: NormalDragMode | SnapDragMode
}

/**
 * 模板拖放数据
 */
export interface TemplateDragData {
  type: 'template'
  templateId: string
  templateName: string
}
