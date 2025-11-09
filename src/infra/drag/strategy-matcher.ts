/**
 * 策略匹配算法
 *
 * 根据拖放会话和目标区域，判断策略是否匹配
 */

import type { DragSession, StrategyCondition, SourceCondition, TargetCondition } from './types'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * 判断策略是否匹配当前拖放操作
 */
export function matchStrategy(
  condition: StrategyCondition,
  session: DragSession,
  targetZone: string
): boolean {
  logger.debug(LogTags.DRAG_STRATEGY, 'Matching strategy condition', {
    condition,
    sessionSource: session.source.viewId,
    targetZone,
  })

  // 1. 匹配源视图
  if (condition.source) {
    if (!matchSource(condition.source, session)) {
      logger.debug(LogTags.DRAG_STRATEGY, 'Source condition not matched')
      return false
    }
  }

  // 2. 匹配目标视图
  if (condition.target) {
    if (!matchTarget(condition.target, targetZone, session)) {
      logger.debug(LogTags.DRAG_STRATEGY, 'Target condition not matched')
      return false
    }
  }

  // 3. 匹配拖放模式
  if (condition.dragMode && session.dragMode !== condition.dragMode) {
    logger.debug(LogTags.DRAG_STRATEGY, 'Drag mode not matched', {
      expected: condition.dragMode,
      actual: session.dragMode,
    })
    return false
  }

  logger.debug(LogTags.DRAG_STRATEGY, 'Strategy condition matched ✓')
  return true
}

/**
 * 匹配源视图条件
 */
function matchSource(condition: SourceCondition, session: DragSession): boolean {
  // 匹配视图类型
  if (condition.viewType) {
    const types = Array.isArray(condition.viewType) ? condition.viewType : [condition.viewType]
    if (!types.includes(session.source.viewType)) {
      logger.debug(LogTags.DRAG_STRATEGY, 'Source viewType not matched', {
        expected: types,
        actual: session.source.viewType,
      })
      return false
    }
  }

  // 匹配视图键
  if (condition.viewKey) {
    if (condition.viewKey instanceof RegExp) {
      if (!condition.viewKey.test(session.source.viewKey)) {
        logger.debug(LogTags.DRAG_STRATEGY, 'Source viewKey (regex) not matched', {
          pattern: condition.viewKey.source,
          actual: session.source.viewKey,
        })
        return false
      }
    } else {
      if (session.source.viewKey !== condition.viewKey) {
        logger.debug(LogTags.DRAG_STRATEGY, 'Source viewKey not matched', {
          expected: condition.viewKey,
          actual: session.source.viewKey,
        })
        return false
      }
    }
  }

  // 匹配对象类型
  if (condition.objectType) {
    const types = Array.isArray(condition.objectType)
      ? condition.objectType
      : [condition.objectType]
    if (!types.includes(session.object.type)) {
      logger.debug(LogTags.DRAG_STRATEGY, 'Source objectType not matched', {
        expected: types,
        actual: session.object.type,
      })
      return false
    }
  }

  // 匹配任务状态（仅当对象类型为 task 时有效）
  if (condition.taskStatus && session.object.type === 'task') {
    const statuses = Array.isArray(condition.taskStatus)
      ? condition.taskStatus
      : [condition.taskStatus]
    
    // 🔥 实时计算任务状态
    const task = session.object.data as any
    const today = new Date().toISOString().split('T')[0]!
    const hasFutureOrTodaySchedule =
      task.schedules?.some((schedule: any) => schedule.scheduled_day >= today) ?? false
    const actualStatus = hasFutureOrTodaySchedule ? 'scheduled' : 'staging'
    
    if (!statuses.includes(actualStatus)) {
      logger.debug(LogTags.DRAG_STRATEGY, 'Source taskStatus not matched', {
        expected: statuses,
        actual: actualStatus,
      })
      return false
    }
  }

  // 自定义检查
  if (condition.customCheck) {
    if (!condition.customCheck(session)) {
      logger.debug(LogTags.DRAG_STRATEGY, 'Source customCheck failed')
      return false
    }
  }

  return true
}

/**
 * 匹配目标视图条件
 */
function matchTarget(
  condition: TargetCondition,
  targetZone: string,
  session: DragSession
): boolean {
  // 匹配视图类型
  if (condition.viewType) {
    const types = Array.isArray(condition.viewType) ? condition.viewType : [condition.viewType]
    // 从 session.target 或 targetZone 推断目标类型
    const targetViewType = session.target?.viewType
    if (targetViewType && !types.includes(targetViewType)) {
      logger.debug(LogTags.DRAG_STRATEGY, 'Target viewType not matched', {
        expected: types,
        actual: targetViewType,
      })
      return false
    }
  }

  // 匹配视图键
  if (condition.viewKey) {
    if (condition.viewKey instanceof RegExp) {
      if (!condition.viewKey.test(targetZone)) {
        logger.debug(LogTags.DRAG_STRATEGY, 'Target viewKey (regex) not matched', {
          pattern: condition.viewKey.source,
          actual: targetZone,
        })
        return false
      }
    } else {
      if (targetZone !== condition.viewKey) {
        logger.debug(LogTags.DRAG_STRATEGY, 'Target viewKey not matched', {
          expected: condition.viewKey,
          actual: targetZone,
        })
        return false
      }
    }
  }

  // 匹配接受的任务状态（仅当对象类型为 task 时有效）
  if (condition.acceptsStatus && session.object.type === 'task') {
    // 🔥 实时计算任务状态
    const task = session.object.data as any
    const today = new Date().toISOString().split('T')[0]!
    const hasFutureOrTodaySchedule =
      task.schedules?.some((schedule: any) => schedule.scheduled_day >= today) ?? false
    const actualStatus = hasFutureOrTodaySchedule ? 'scheduled' : 'staging'
    
    if (!condition.acceptsStatus.includes(actualStatus)) {
      logger.debug(LogTags.DRAG_STRATEGY, 'Target acceptsStatus not matched', {
        acceptsStatus: condition.acceptsStatus,
        taskStatus: actualStatus,
      })
      return false
    }
  }

  // 自定义检查
  if (condition.customCheck) {
    if (!condition.customCheck(targetZone, session)) {
      logger.debug(LogTags.DRAG_STRATEGY, 'Target customCheck failed')
      return false
    }
  }

  return true
}

/**
 * 计算匹配得分（用于调试和优化）
 */
export function calculateMatchScore(
  condition: StrategyCondition,
  _session: DragSession,
  _targetZone: string
): number {
  let score = 0

  // 基础得分：优先级
  score += condition.priority ?? 0

  // 精确匹配加分
  if (condition.source?.viewKey && typeof condition.source.viewKey === 'string') {
    score += 10
  }
  if (condition.target?.viewKey && typeof condition.target.viewKey === 'string') {
    score += 10
  }

  // 类型匹配加分
  if (condition.source?.viewType) {
    score += 5
  }
  if (condition.target?.viewType) {
    score += 5
  }

  return score
}
