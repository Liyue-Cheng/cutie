/**
 * useCrossViewDrag - 跨看板拖放核心
 *
 * 提供统一的跨看板拖放协调功能
 */

import { computed } from 'vue'
import type { ViewMetadata, StrategyResult, DragStrategy } from '@/types/drag'
import type { TaskCard } from '@/types/dtos'
import { logger, LogTags } from '@/infra/logging/logger'
import { useDragContext } from './context'
import { findStrategy, hasStrategy, getStrategyPriority } from './finder'
import {
  registerStrategy as registerStrategyInternal,
  unregisterStrategy,
  getRegisteredStrategies,
} from './strategies'

/**
 * 跨看板拖放核心 Composable
 */
export function useCrossViewDrag() {
  const dragContext = useDragContext()

  // ==================== 计算属性 ====================

  /**
   * 是否处于吸附模式
   */
  const isSnapMode = computed(() => dragContext.currentMode.value === 'snap')

  /**
   * 是否处于普通拖放模式
   */
  const isNormalMode = computed(() => dragContext.currentMode.value === 'normal')

  // ==================== 拖放操作 ====================

  /**
   * 开始普通拖放
   * @param task - 被拖拽的任务
   * @param sourceView - 源看板元数据
   */
  function startNormalDrag(task: TaskCard, sourceView: ViewMetadata): void {
    dragContext.startNormalDrag(task, sourceView)
  }

  /**
   * 开始吸附式拖放
   * @param task - 被拖拽的任务
   * @param sourceView - 源看板元数据
   * @param activatedBy - 激活按钮的标识
   * @param params - 额外参数
   */
  function startSnapDrag(
    task: TaskCard,
    sourceView: ViewMetadata,
    activatedBy: string,
    params?: Record<string, any>
  ): void {
    dragContext.startSnapDrag(task, sourceView, activatedBy, params)
  }

  /**
   * 处理放置
   * @param targetView - 目标看板元数据
   * @param event - DragEvent（可选，用于从 dataTransfer 读取数据）
   * @returns 策略执行结果
   */
  async function handleDrop(targetView: ViewMetadata, event?: DragEvent): Promise<StrategyResult> {
    const context = dragContext.currentContext.value

    if (!context) {
      logger.error(
        LogTags.DRAG_CROSS_VIEW,
        'No active drag context',
        new Error('Drop attempted without active drag context')
      )
      return {
        success: false,
        error: '没有活动的拖拽上下文',
      }
    }

    // 🔍 检查点5：策略调用前的上下文
    logger.debug(LogTags.DRAG_CROSS_VIEW, 'handleDrop called', {
      context: {
        taskTitle: context.task.title,
        sourceType: context.sourceView.type,
        sourceId: context.sourceView.id,
      },
      targetView: {
        type: targetView.type,
        id: targetView.id,
      },
    })

    logger.info(LogTags.DRAG_CROSS_VIEW, 'Handling drop', {
      task: context.task.title,
      source: `${context.sourceView.type}:${context.sourceView.id}`,
      target: `${targetView.type}:${targetView.id}`,
      mode: context.dragMode.mode,
      duration: `${dragContext.getDragDuration()}ms`,
    })

    try {
      // 🆕 标记 drop 开始，避免外层 dragend 把上下文提前清理
      dragContext.setDropInProgress(true)
      // 查找并执行策略
      const strategy = findStrategy(context.sourceView.type, targetView.type, context.dragMode.mode)

      // 🔍 检查点5：策略查找结果
      logger.debug(LogTags.DRAG_CROSS_VIEW, 'Strategy found', {
        strategyPath: `${context.sourceView.type}->${targetView.type}`,
      })

      const result = await strategy(context, targetView)

      // 🔍 检查点5：策略执行结果
      logger.debug(LogTags.DRAG_CROSS_VIEW, 'Strategy executed', { result })

      logger.info(LogTags.DRAG_CROSS_VIEW, 'Drop handled', {
        success: result.success,
        message: result.message,
        error: result.error,
        reorderOnly: result.reorderOnly,
        affectedViews: result.affectedViews,
      })

      // 清除上下文（drop 完成后）
      dragContext.clearContext()

      return result
    } catch (error) {
      logger.error(
        LogTags.DRAG_CROSS_VIEW,
        'Drop failed',
        error instanceof Error ? error : new Error(String(error))
      )

      // 清除上下文
      dragContext.clearContext()

      return {
        success: false,
        error: error instanceof Error ? error.message : '未知错误',
      }
    } finally {
      // 🆕 无论成功失败，复位 drop 标记
      dragContext.setDropInProgress(false)
    }
  }

  /**
   * 取消拖放
   */
  function cancelDrag(): void {
    const context = dragContext.currentContext.value

    if (!context) {
      logger.warn(LogTags.DRAG_CROSS_VIEW, 'No active drag to cancel')
      return
    }

    logger.info(LogTags.DRAG_CROSS_VIEW, 'Drag cancelled', {
      task: context.task.title,
      mode: context.dragMode.mode,
      duration: `${dragContext.getDragDuration()}ms`,
    })

    dragContext.clearContext()
  }

  // ==================== 辅助功能 ====================

  /**
   * 检查是否可以放置
   * @param sourceView - 源看板元数据
   * @param targetView - 目标看板元数据
   * @returns 是否可以放置
   */
  function canDrop(sourceView: ViewMetadata, targetView: ViewMetadata): boolean {
    // 不能拖到自己
    if (sourceView.id === targetView.id) {
      return false
    }

    // 检查是否有对应的策略
    return hasStrategy(sourceView.type, targetView.type)
  }

  /**
   * 获取放置提示文字
   * @param sourceView - 源看板元数据
   * @param targetView - 目标看板元数据
   * @returns 提示文字
   */
  function getDropHint(sourceView: ViewMetadata, targetView: ViewMetadata): string {
    const exactKey = `${sourceView.type}->${targetView.type}`
    const sourceWildcard = `${sourceView.type}->*`
    const targetWildcard = `*->${targetView.type}`

    // 根据策略类型返回不同的提示
    const hints: Record<string, string> = {
      // 精确匹配
      'status->date': '放置后将设置排期',
      'date->date': '放置后将改期',
      'date->status': '放置后将取消排期',
      'project->project': '放置后将移动到此项目',

      // 通配符匹配
      '*->calendar': '放置后将创建时间块',
    }

    // 1. 优先精确匹配
    if (hints[exactKey]) {
      return hints[exactKey]!
    }

    // 2. 源通配符
    if (hints[sourceWildcard]) {
      return hints[sourceWildcard]!
    }

    // 3. 目标通配符
    if (hints[targetWildcard]) {
      return hints[targetWildcard]!
    }

    // 4. 默认
    return '放置后将移动任务'
  }

  /**
   * 获取策略优先级（调试用）
   * @param sourceView - 源看板元数据
   * @param targetView - 目标看板元数据
   * @returns 优先级名称
   */
  function getStrategyInfo(
    sourceView: ViewMetadata,
    targetView: ViewMetadata
  ): {
    exists: boolean
    priority: string
    key: string
  } {
    const key = `${sourceView.type}->${targetView.type}`
    const exists = hasStrategy(sourceView.type, targetView.type)
    const priority = getStrategyPriority(sourceView.type, targetView.type)

    return { exists, priority, key }
  }

  // ==================== 扩展功能 ====================

  /**
   * 注册自定义策略
   * @param key - 策略键（例如：'custom->date'）
   * @param strategy - 策略函数
   */
  function registerStrategy(key: string, strategy: DragStrategy): void {
    registerStrategyInternal(key, strategy)
  }

  /**
   * 注销策略
   * @param key - 策略键
   */
  function removeStrategy(key: string): void {
    unregisterStrategy(key)
  }

  /**
   * 获取所有已注册的策略
   * @returns 策略键列表
   */
  function listStrategies(): string[] {
    return getRegisteredStrategies()
  }

  // ==================== 返回 ====================

  return {
    // 状态（只读）
    currentContext: dragContext.currentContext,
    isDragging: dragContext.isDragging,
    currentMode: dragContext.currentMode,
    currentTask: dragContext.currentTask,
    sourceView: dragContext.sourceView,
    targetViewId: dragContext.targetViewId, // 🆕 导出目标看板ID
    isDropInProgress: dragContext.isDropInProgress, // 🆕 导出 drop 执行状态
    isSnapMode,
    isNormalMode,

    // 拖放操作
    startNormalDrag,
    startSnapDrag,
    handleDrop,
    cancelDrag,
    setTargetViewId: dragContext.setTargetViewId, // 🆕 导出设置方法

    // 辅助功能
    canDrop,
    getDropHint,
    getStrategyInfo,

    // 扩展功能
    registerStrategy,
    removeStrategy,
    listStrategies,

    // 调试功能
    getDragDuration: dragContext.getDragDuration,
  }
}
