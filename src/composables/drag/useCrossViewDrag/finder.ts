/**
 * useCrossViewDrag/finder - 策略查找算法
 *
 * 根据源看板和目标看板类型，查找合适的策略
 */

import type { DragStrategy, ViewType, DragMode } from '@/types/drag'
import { dragStrategies } from './strategies'

/**
 * 查找策略
 * @param sourceType - 源看板类型
 * @param targetType - 目标看板类型
 * @param dragMode - 拖放模式（可选，用于日志）
 * @returns 匹配的策略函数
 */
export function findStrategy(
  sourceType: ViewType,
  targetType: ViewType,
  dragMode?: DragMode
): DragStrategy {
  console.log('[StrategyFinder] 🔍 Finding strategy:', {
    sourceType,
    targetType,
    dragMode,
  })

  // 1. 精确匹配
  const exactKey = `${sourceType}->${targetType}`
  if (dragStrategies[exactKey as keyof typeof dragStrategies]) {
    console.log(`  ✅ Found exact match: ${exactKey}`)
    return dragStrategies[exactKey as keyof typeof dragStrategies]!
  }

  // 2. 源通配符：sourceType->*
  const sourceWildcard = `${sourceType}->*`
  if (dragStrategies[sourceWildcard as keyof typeof dragStrategies]) {
    console.log(`  ✅ Found source wildcard: ${sourceWildcard}`)
    return dragStrategies[sourceWildcard as keyof typeof dragStrategies]!
  }

  // 3. 目标通配符：*->targetType
  const targetWildcard = `*->${targetType}`
  if (dragStrategies[targetWildcard as keyof typeof dragStrategies]) {
    console.log(`  ✅ Found target wildcard: ${targetWildcard}`)
    return dragStrategies[targetWildcard as keyof typeof dragStrategies]!
  }

  // 4. 默认策略：*->*
  console.log('  ⚠️ Using default strategy: *->*')
  return dragStrategies['*->*']!
}

/**
 * 检查策略是否存在
 * @param sourceType - 源看板类型
 * @param targetType - 目标看板类型
 * @returns 是否有有效的策略
 */
export function hasStrategy(sourceType: ViewType, targetType: ViewType): boolean {
  const exactKey = `${sourceType}->${targetType}`
  const sourceWildcard = `${sourceType}->*`
  const targetWildcard = `*->${targetType}`

  return !!(
    dragStrategies[exactKey as keyof typeof dragStrategies] ||
    dragStrategies[sourceWildcard as keyof typeof dragStrategies] ||
    dragStrategies[targetWildcard as keyof typeof dragStrategies] ||
    dragStrategies['*->*']
  )
}

/**
 * 获取策略的优先级（用于调试）
 * @param sourceType - 源看板类型
 * @param targetType - 目标看板类型
 * @returns 优先级名称
 */
export function getStrategyPriority(sourceType: ViewType, targetType: ViewType): string {
  const exactKey = `${sourceType}->${targetType}`
  const sourceWildcard = `${sourceType}->*`
  const targetWildcard = `*->${targetType}`

  if (dragStrategies[exactKey as keyof typeof dragStrategies]) {
    return 'exact'
  }
  if (dragStrategies[sourceWildcard as keyof typeof dragStrategies]) {
    return 'source-wildcard'
  }
  if (dragStrategies[targetWildcard as keyof typeof dragStrategies]) {
    return 'target-wildcard'
  }
  return 'default'
}


