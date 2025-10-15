/**
 * 策略注册中心
 *
 * 管理所有拖放策略的注册、查找和生命周期
 */

import type { Strategy, DragSession, RegistryStats } from './types'
import { matchStrategy, calculateMatchScore } from './strategy-matcher'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * 策略注册中心
 */
class StrategyRegistry {
  private strategies: Map<string, Strategy> = new Map()
  private sortedStrategies: Strategy[] = []

  /**
   * 注册策略
   */
  register(strategy: Strategy): void {
    if (this.strategies.has(strategy.id)) {
      logger.warn(LogTags.DRAG_STRATEGY, `Strategy already registered: ${strategy.id}, overwriting`)
    }

    this.strategies.set(strategy.id, strategy)
    this.rebuildSortedList()

    logger.debug(LogTags.DRAG_STRATEGY, 'Strategy registered', {
      id: strategy.id,
      name: strategy.name,
      priority: strategy.conditions.priority ?? 0,
      tags: strategy.tags,
    })
  }

  /**
   * 批量注册策略
   */
  registerBatch(strategies: Strategy[]): void {
    strategies.forEach((s) => this.strategies.set(s.id, s))
    this.rebuildSortedList()

    logger.info(LogTags.DRAG_STRATEGY, `Batch registered ${strategies.length} strategies`)
  }

  /**
   * 注销策略
   */
  unregister(id: string): void {
    if (!this.strategies.has(id)) {
      logger.warn(LogTags.DRAG_STRATEGY, `Strategy not found for unregister: ${id}`)
      return
    }

    this.strategies.delete(id)
    this.rebuildSortedList()

    logger.debug(LogTags.DRAG_STRATEGY, 'Strategy unregistered', { id })
  }

  /**
   * 查找匹配的策略
   * @returns 第一个匹配的策略（按优先级排序）
   */
  findMatch(session: DragSession, targetZone: string): Strategy | null {
    logger.debug(LogTags.DRAG_STRATEGY, 'Finding matching strategy', {
      sourceView: session.source.viewId,
      targetZone,
      dragMode: session.dragMode,
      taskStatus: session.object.data.schedule_status,
    })

    for (const strategy of this.sortedStrategies) {
      // 跳过禁用的策略
      if (strategy.enabled === false) {
        continue
      }

      if (matchStrategy(strategy.conditions, session, targetZone)) {
        logger.info(LogTags.DRAG_STRATEGY, 'Strategy matched ✓', {
          strategyId: strategy.id,
          strategyName: strategy.name,
          priority: strategy.conditions.priority ?? 0,
          tags: strategy.tags,
        })
        return strategy
      }
    }

    logger.warn(LogTags.DRAG_STRATEGY, 'No matching strategy found', {
      sourceView: session.source.viewId,
      targetZone,
      availableStrategies: this.sortedStrategies.length,
    })

    return null
  }

  /**
   * 查找所有匹配的策略（用于调试）
   */
  findAllMatches(session: DragSession, targetZone: string): Strategy[] {
    return this.sortedStrategies
      .filter((s) => s.enabled !== false)
      .filter((s) => matchStrategy(s.conditions, session, targetZone))
  }

  /**
   * 获取策略
   */
  get(id: string): Strategy | undefined {
    return this.strategies.get(id)
  }

  /**
   * 检查策略是否存在
   */
  has(id: string): boolean {
    return this.strategies.has(id)
  }

  /**
   * 获取所有策略
   */
  getAll(): Strategy[] {
    return Array.from(this.strategies.values())
  }

  /**
   * 获取已排序的策略列表
   */
  getSorted(): Strategy[] {
    return [...this.sortedStrategies]
  }

  /**
   * 按标签查找策略
   */
  findByTag(tag: string): Strategy[] {
    return Array.from(this.strategies.values()).filter((s) => s.tags?.includes(tag))
  }

  /**
   * 获取统计信息
   */
  getStats(): RegistryStats {
    const all = Array.from(this.strategies.values())
    const enabled = all.filter((s) => s.enabled !== false)
    const disabled = all.filter((s) => s.enabled === false)

    const strategiesByTag: Record<string, number> = {}
    all.forEach((s) => {
      s.tags?.forEach((tag) => {
        strategiesByTag[tag] = (strategiesByTag[tag] || 0) + 1
      })
    })

    return {
      totalStrategies: all.length,
      enabledStrategies: enabled.length,
      disabledStrategies: disabled.length,
      strategiesByTag,
    }
  }

  /**
   * 清空所有策略
   */
  clear(): void {
    this.strategies.clear()
    this.sortedStrategies = []
    logger.info(LogTags.DRAG_STRATEGY, 'All strategies cleared')
  }

  /**
   * 重建排序列表
   * 按优先级降序排序
   */
  private rebuildSortedList(): void {
    this.sortedStrategies = Array.from(this.strategies.values()).sort((a, b) => {
      const priorityA = a.conditions.priority ?? 0
      const priorityB = b.conditions.priority ?? 0
      return priorityB - priorityA
    })

    logger.debug(LogTags.DRAG_STRATEGY, 'Sorted strategy list rebuilt', {
      count: this.sortedStrategies.length,
      priorities: this.sortedStrategies.map((s) => ({
        id: s.id,
        priority: s.conditions.priority ?? 0,
      })),
    })
  }

  /**
   * 启用策略
   */
  enable(id: string): void {
    const strategy = this.strategies.get(id)
    if (!strategy) {
      logger.warn(LogTags.DRAG_STRATEGY, `Strategy not found for enable: ${id}`)
      return
    }

    strategy.enabled = true
    logger.debug(LogTags.DRAG_STRATEGY, 'Strategy enabled', { id })
  }

  /**
   * 禁用策略
   */
  disable(id: string): void {
    const strategy = this.strategies.get(id)
    if (!strategy) {
      logger.warn(LogTags.DRAG_STRATEGY, `Strategy not found for disable: ${id}`)
      return
    }

    strategy.enabled = false
    logger.debug(LogTags.DRAG_STRATEGY, 'Strategy disabled', { id })
  }

  /**
   * 调试：打印所有策略
   */
  debug(): void {
    console.group('🎯 Drag Strategy Registry')
    console.log('Total strategies:', this.strategies.size)
    console.table(
      this.sortedStrategies.map((s) => ({
        ID: s.id,
        Name: s.name,
        Priority: s.conditions.priority ?? 0,
        Enabled: s.enabled !== false ? '✓' : '✗',
        Tags: s.tags?.join(', ') || '-',
      }))
    )
    console.groupEnd()
  }
}

// 导出全局单例
export const strategyRegistry = new StrategyRegistry()

// 开发环境：暴露到 window
if (import.meta.env.DEV) {
  ;(window as any).strategyRegistry = strategyRegistry
}
