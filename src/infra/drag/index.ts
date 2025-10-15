/**
 * 拖放策略系统 - 统一导出
 *
 * 新一代拖放策略系统，完全重写
 * - 声明式策略定义
 * - 条件匹配引擎
 * - 统一执行流程
 * - 完整追踪日志
 */

// 核心组件
export { strategyRegistry } from './strategy-registry'
export { strategyExecutor } from './strategy-executor'
export { matchStrategy, calculateMatchScore } from './strategy-matcher'

// 类型定义
export type {
  DragSession,
  StrategyCondition,
  SourceCondition,
  TargetCondition,
  StrategyContext,
  StrategyResult,
  StrategyAction,
  Strategy,
  StrategyPreview,
  RegistryStats,
} from './types'

// 策略集合
export * as strategies from './strategies'

// 初始化函数
import { strategyRegistry } from './strategy-registry'
import * as allStrategies from './strategies'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * 初始化拖放策略系统
 * 注册所有预定义策略
 */
export function initializeDragStrategies(): void {
  logger.info(LogTags.DRAG_STRATEGY, '🚀 Initializing drag strategy system...')

  // 注册所有策略
  const strategyList = Object.values(allStrategies)
  strategyRegistry.registerBatch(strategyList)

  const stats = strategyRegistry.getStats()

  logger.info(LogTags.DRAG_STRATEGY, '✅ Drag strategy system initialized', {
    totalStrategies: stats.totalStrategies,
    enabledStrategies: stats.enabledStrategies,
    strategiesByTag: stats.strategiesByTag,
  })

  // 开发环境：打印策略列表
  if (import.meta.env.DEV) {
    console.log('🎯 Registered Drag Strategies:')
    strategyRegistry.debug()
  }
}
