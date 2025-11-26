/**
 * 策略执行引擎
 *
 * 负责查找、验证和执行拖放策略
 * 当前为打印模式：只记录策略信息，不执行实际业务
 */

import type { DragSession, StrategyResult, StrategyContext, Strategy } from './types'
import { strategyRegistry } from './strategy-registry'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * 策略执行引擎
 */
class StrategyExecutor {
  /**
   * 执行拖放策略
   *
   * @param session 拖放会话
   * @param targetZone 目标区域
   * @param contextData 灵活的上下文数据（V2 设计）
   */
  async execute(
    session: DragSession,
    targetZone: string,
    contextData?: {
      sourceContext?: Record<string, any> // 起始组件传入的数据
      targetContext?: Record<string, any> // 结束组件传入的数据
    }
  ): Promise<StrategyResult> {
    // ✅ 移除旧的 tracker，现在由 CPU Pipeline 统一追踪

    try {
      // 1. 查找匹配的策略
      logger.info(LogTags.DRAG_STRATEGY, '🔍 查找匹配策略', {
        sourceView: session.source.viewId,
        targetZone,
        objectType: session.object.type,
        dragMode: session.dragMode,
      })

      const strategy = strategyRegistry.findMatch(session, targetZone)

      if (!strategy) {
        logger.warn(LogTags.DRAG_STRATEGY, '❌ 未找到匹配策略', {
          sourceView: session.source.viewId,
          targetZone,
          objectType: session.object.type,
          dragMode: session.dragMode,
        })

        return {
          success: false,
          error: '找不到合适的策略处理此拖放操作',
        }
      }

      logger.info(LogTags.DRAG_STRATEGY, '✅ 找到匹配策略', {
        strategyId: strategy.id,
        strategyName: strategy.name,
        priority: strategy.priority,
      })

      // 2. 构建执行上下文
      const context = this.buildContext(session, targetZone, strategy, contextData)

      // 3. 打印策略信息（不执行）
      this.printStrategyInfo(strategy, context)

      // 4. 前置检查（如果有）
      if (strategy.action.canExecute) {
        const canExecute = await strategy.action.canExecute(context)
        if (!canExecute) {
          logger.warn(LogTags.DRAG_STRATEGY, '⚠️ Strategy canExecute check failed', {
            strategyId: strategy.id,
            strategyName: strategy.name,
          })

          return {
            success: false,
            error: `策略 ${strategy.name} 不满足执行条件`,
          }
        }
      }

      // 5. 执行策略
      logger.info(LogTags.DRAG_STRATEGY, '🚀 开始执行策略', {
        strategyId: strategy.id,
        strategyName: strategy.name,
        actionName: strategy.action.name,
        sourceView: session.source.viewId,
        targetZone,
      })

      const result = await strategy.action.execute(context)

      if (result.success) {
        logger.info(LogTags.DRAG_STRATEGY, '✅ 策略执行成功', {
          strategyId: strategy.id,
          strategyName: strategy.name,
          message: result.message,
          affectedViews: result.affectedViews,
        })
      } else {
        logger.warn(LogTags.DRAG_STRATEGY, '⚠️ 策略执行失败', {
          strategyId: strategy.id,
          strategyName: strategy.name,
          error: result.error,
          message: result.message,
        })
      }

      return result
    } catch (error) {
      logger.error(
        LogTags.DRAG_STRATEGY,
        '❌ Strategy execution error',
        error instanceof Error ? error : new Error(String(error))
      )

      return {
        success: false,
        error: error instanceof Error ? error.message : '策略执行失败',
      }
    }
  }

  /**
   * 预览策略（不执行，只返回信息）
   */
  preview(
    session: DragSession,
    targetZone: string
  ): {
    hasMatch: boolean
    strategy: Strategy | null
    canExecute: boolean
  } {
    const strategy = strategyRegistry.findMatch(session, targetZone)

    if (!strategy) {
      return {
        hasMatch: false,
        strategy: null,
        canExecute: false,
      }
    }

    return {
      hasMatch: true,
      strategy,
      canExecute: true, // 简化：始终返回 true，实际检查在 execute 时
    }
  }

  /**
   * 构建策略执行上下文（V2：灵活的 JSON 上下文）
   */
  private buildContext(
    session: DragSession,
    targetZone: string,
    strategy: Strategy,
    contextData?: {
      sourceContext?: Record<string, any>
      targetContext?: Record<string, any>
    }
  ): StrategyContext {
    // 解析目标视图类型（从 targetZone 推断）
    const targetViewType = this.inferViewType(targetZone)

    // 🔥 V2：灵活的上下文数据
    const sourceContext = contextData?.sourceContext ?? {}
    const targetContext = contextData?.targetContext ?? {}

    // 如果没有传入上下文数据，记录警告
    if (Object.keys(sourceContext).length === 0 || Object.keys(targetContext).length === 0) {
      logger.warn(LogTags.DRAG_STRATEGY, '⚠️ Missing context data', {
        hasSourceContext: Object.keys(sourceContext).length > 0,
        hasTargetContext: Object.keys(targetContext).length > 0,
        strategyId: strategy.id,
      })
    }

    return {
      session,
      targetZone,
      sourceViewId: session.source.viewId,
      sourceViewType: session.source.viewType,
      targetViewId: targetZone,
      targetViewType,
      draggedObject: session.object.data,
      dropIndex: targetContext.dropIndex ?? session.target?.dropIndex,
      sourceContext,
      targetContext,
      timestamp: Date.now(),
    }
  }

  /**
   * 从 viewKey 推断视图类型
   */
  private inferViewType(viewKey: string): any {
    if (viewKey.startsWith('daily::')) return 'date'
    if (viewKey.startsWith('misc::')) return 'status'
    if (viewKey.startsWith('project::')) return 'project'
    if (viewKey.startsWith('calendar::')) return 'calendar'
    return 'unknown'
  }

  /**
   * 打印策略信息（核心：展示策略细节）
   */
  private printStrategyInfo(strategy: Strategy, context: StrategyContext): void {
    // ✅ 移除旧的控制台打印噪音，改用简洁日志
    const objectTitle = (context.draggedObject as any)?.title || 'Unknown'
    logger.debug(LogTags.DRAG_STRATEGY, '🎯 Strategy matched and ready', {
      strategyId: strategy.id,
      strategyName: strategy.name,
      actionName: strategy.action.name,
      sourceView: context.sourceViewId,
      targetView: context.targetViewId,
      objectTitle,
      objectType: context.session.object.type,
    })
  }

  /**
   * 获取调试信息
   */
  getDebugInfo(
    session: DragSession,
    targetZone: string
  ): {
    allMatches: Strategy[]
    bestMatch: Strategy | null
    registryStats: any
  } {
    return {
      allMatches: strategyRegistry.findAllMatches(session, targetZone),
      bestMatch: strategyRegistry.findMatch(session, targetZone),
      registryStats: strategyRegistry.getStats(),
    }
  }
}

// 导出全局单例
export const strategyExecutor = new StrategyExecutor()

// 开发环境：暴露到 window
if (import.meta.env.DEV) {
  ;(window as any).strategyExecutor = strategyExecutor
}
