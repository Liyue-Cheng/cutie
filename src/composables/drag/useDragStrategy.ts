/**
 * useDragStrategy - 组件层拖放策略 API
 *
 * 为 Vue 组件提供简洁的策略系统接口
 */

import { computed, ref } from 'vue'
import type { DragSession, StrategyResult, Strategy } from '@/infra/drag/types'
import { strategyExecutor, strategyRegistry } from '@/infra/drag'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * 拖放策略 Composable
 */
export function useDragStrategy() {
  const isExecuting = ref(false)
  const lastResult = ref<StrategyResult | null>(null)
  const lastStrategy = ref<Strategy | null>(null)

  /**
   * 执行拖放策略
   *
   * @param session 拖放会话
   * @param targetZone 目标区域
   * @param contextData V2：灵活的上下文数据
   */
  async function executeDrop(
    session: DragSession,
    targetZone: string,
    contextData?: {
      sourceContext?: Record<string, any>
      targetContext?: Record<string, any>
    }
  ): Promise<StrategyResult> {
    if (isExecuting.value) {
      logger.warn(LogTags.DRAG_STRATEGY, 'Strategy execution already in progress')
      return {
        success: false,
        error: '策略正在执行中',
      }
    }

    isExecuting.value = true
    lastResult.value = null
    lastStrategy.value = null

    try {
      logger.info(LogTags.DRAG_STRATEGY, '▶️ Executing drag strategy', {
        sourceView: session.source.viewId,
        targetZone,
        taskId: session.object.data.id,
        hasSourceContext: !!contextData?.sourceContext,
        hasTargetContext: !!contextData?.targetContext,
      })

      const result = await strategyExecutor.execute(session, targetZone, contextData)

      lastResult.value = result
      lastStrategy.value = strategyRegistry.findMatch(session, targetZone)

      return result
    } catch (error) {
      logger.error(
        LogTags.DRAG_STRATEGY,
        'Strategy execution error in composable',
        error instanceof Error ? error : new Error(String(error))
      )

      const errorResult: StrategyResult = {
        success: false,
        error: error instanceof Error ? error.message : '未知错误',
      }

      lastResult.value = errorResult
      return errorResult
    } finally {
      isExecuting.value = false
    }
  }

  /**
   * 预览拖放策略（不执行）
   */
  function previewDrop(
    session: DragSession,
    targetZone: string
  ): {
    hasMatch: boolean
    strategy: Strategy | null
    description: string
  } {
    const preview = strategyExecutor.preview(session, targetZone)

    return {
      hasMatch: preview.hasMatch,
      strategy: preview.strategy,
      description: preview.strategy?.action.description || '无可用策略',
    }
  }

  /**
   * 检查是否有可用策略
   */
  function hasStrategy(session: DragSession, targetZone: string): boolean {
    return strategyRegistry.findMatch(session, targetZone) !== null
  }

  /**
   * 获取调试信息
   */
  function getDebugInfo(session: DragSession, targetZone: string) {
    return strategyExecutor.getDebugInfo(session, targetZone)
  }

  /**
   * 获取注册表统计
   */
  const registryStats = computed(() => strategyRegistry.getStats())

  return {
    // 状态
    isExecuting: computed(() => isExecuting.value),
    lastResult: computed(() => lastResult.value),
    lastStrategy: computed(() => lastStrategy.value),
    registryStats,

    // 方法
    executeDrop,
    previewDrop,
    hasStrategy,
    getDebugInfo,
  }
}
