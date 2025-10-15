/**
 * 拖放策略系统类型定义
 *
 * 新架构特点：
 * - 声明式策略定义
 * - 条件组合匹配
 * - 完全类型安全
 * - 可组合和可测试
 */

import type { TaskCard } from '@/types/dtos'
import type { ViewType } from '@/types/drag'

// ==================== 拖放会话 ====================

/**
 * 拖放会话 - 描述一次完整的拖放操作
 */
export interface DragSession {
  id: string

  // 源信息
  source: {
    viewId: string
    viewType: ViewType
    viewKey: string
    elementId: string
  }

  // 被拖放对象
  object: {
    type: 'task' | 'time-block' | 'other'
    data: TaskCard // 任务数据快照
    originalIndex: number
  }

  // 拖放模式
  dragMode: 'normal' | 'copy' | 'scheduled'

  // 目标信息（当进入目标区域时填充）
  target?: {
    viewId: string
    viewType: ViewType
    viewKey: string
    dropIndex?: number
  }

  // 元数据
  startTime: number
  metadata?: Record<string, any>
}

// ==================== 策略条件 ====================

/**
 * 源视图条件
 */
export interface SourceCondition {
  // 视图类型匹配
  viewType?: ViewType | ViewType[]

  // 视图键匹配（支持字符串或正则）
  viewKey?: string | RegExp

  // 任务状态匹配
  taskStatus?: TaskCard['schedule_status'] | TaskCard['schedule_status'][]

  // 自定义检查函数
  customCheck?: (session: DragSession) => boolean
}

/**
 * 目标视图条件
 */
export interface TargetCondition {
  // 视图类型匹配
  viewType?: ViewType | ViewType[]

  // 视图键匹配（支持字符串或正则）
  viewKey?: string | RegExp

  // 接受的任务状态
  acceptsStatus?: TaskCard['schedule_status'][]

  // 自定义检查函数
  customCheck?: (targetZone: string, session: DragSession) => boolean
}

/**
 * 策略匹配条件
 */
export interface StrategyCondition {
  // 源视图条件
  source?: SourceCondition

  // 目标视图条件
  target?: TargetCondition

  // 拖放模式
  dragMode?: 'normal' | 'copy' | 'scheduled'

  // 优先级（数字越大优先级越高）
  priority?: number
}

// ==================== 策略执行 ====================

/**
 * 策略执行上下文
 *
 * 设计原则：策略是纯计算，所有数据由调用者（组件）提供
 * - ❌ 策略不应该查询 Store
 * - ✅ 组件通过 Context 传入所有必要数据
 *
 * V2 设计：灵活的 JSON 上下文
 * - sourceContext: 起始组件自由传入的数据（任意结构）
 * - targetContext: 结束组件自由传入的数据（任意结构）
 * - 策略自行解包需要的字段，类型安全由策略保证
 */
export interface StrategyContext {
  // 拖放会话
  session: DragSession

  // 目标区域
  targetZone: string

  // 便捷访问（从 session 中提取）
  sourceViewId: string
  sourceViewType: ViewType
  targetViewId: string
  targetViewType: ViewType
  task: TaskCard
  dropIndex?: number

  // 🔥 灵活的上下文数据（V2 设计）
  sourceContext: Record<string, any> // 起始组件传入的所有数据
  targetContext: Record<string, any> // 结束组件传入的所有数据

  // 元数据
  timestamp: number
}

/**
 * 常见的上下文数据结构（供参考，非强制）
 */
export interface CommonSourceContext {
  taskIds?: string[] // 任务ID列表
  displayTasks?: TaskCard[] // 完整的任务列表
  viewConfig?: Record<string, any> // 视图配置
  [key: string]: any // 允许任意扩展
}

export interface CommonTargetContext {
  taskIds?: string[] // 任务ID列表
  displayTasks?: TaskCard[] // 完整的任务列表
  dropIndex?: number // 插入位置
  viewConfig?: Record<string, any> // 视图配置
  [key: string]: any // 允许任意扩展
}

/**
 * 策略执行结果
 */
export interface StrategyResult {
  success: boolean
  message?: string
  error?: string

  // 受影响的视图（用于刷新）
  affectedViews?: string[]

  // 是否只是重排序（不改变任务属性）
  reorderOnly?: boolean

  // 额外数据
  metadata?: Record<string, any>
}

/**
 * 策略动作定义
 */
export interface StrategyAction {
  // 动作名称
  name: string

  // 动作描述
  description: string

  // 前置检查（可选）
  canExecute?: (ctx: StrategyContext) => Promise<boolean> | boolean

  // 执行逻辑（打印模式：只打印不执行）
  execute: (ctx: StrategyContext) => Promise<StrategyResult>

  // 回滚逻辑（可选，预留）
  rollback?: (ctx: StrategyContext) => Promise<void>
}

/**
 * 策略定义
 */
export interface Strategy {
  // 唯一标识
  id: string

  // 策略名称
  name: string

  // 匹配条件
  conditions: StrategyCondition

  // 执行动作
  action: StrategyAction

  // 标签（用于分类和调试）
  tags?: string[]

  // 是否启用
  enabled?: boolean
}

// ==================== 策略预览 ====================

/**
 * 策略预览（hover 时显示）
 */
export interface StrategyPreview {
  strategyId: string
  strategyName: string
  description: string
  canExecute: boolean
  estimatedEffect?: string
  warnings?: string[]
}

// ==================== 策略注册表 ====================

/**
 * 策略注册表统计信息
 */
export interface RegistryStats {
  totalStrategies: number
  enabledStrategies: number
  disabledStrategies: number
  strategiesByTag: Record<string, number>
}
