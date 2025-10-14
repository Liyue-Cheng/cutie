/**
 * 事务处理器 (Transaction Processor)
 *
 * 统一处理 HTTP 响应和 SSE 事件，基于后端的统一事务结果结构。
 *
 * 核心功能：
 * 1. 接收来自 HTTP 和 SSE 的统一数据结构
 * 2. 基于 correlation ID 或 event ID 去重
 * 3. 应用主资源和所有副作用到 Store
 *
 * 架构：
 * HTTP Response ---> TransactionProcessor ---> Store Mutations
 * SSE Event     -/                        \---> Deduplication
 *
 * CPU 类比：
 * - Reorder Buffer（重排序缓冲）
 * - Commit Unit（提交单元）
 */

import { logger, LogTags } from '@/infra/logging'
import { useTaskStore } from '@/stores/task'
import { useTimeBlockStore } from '@/stores/timeblock'
import type { TaskCard, TimeBlockView } from '@/types/dtos'

/**
 * 任务事务结果（与后端 TaskTransactionResult 对应）
 */
export interface TaskTransactionResult {
  task: TaskCard
  side_effects?: {
    deleted_time_blocks?: TimeBlockView[]
    truncated_time_blocks?: TimeBlockView[]
    updated_time_blocks?: TimeBlockView[]
    created_time_blocks?: TimeBlockView[]
    updated_tasks?: TaskCard[]
  }
}

/**
 * 时间块事务结果（与后端 TimeBlockTransactionResult 对应）
 */
export interface TimeBlockTransactionResult {
  time_block: TimeBlockView
  side_effects?: {
    updated_tasks?: TaskCard[]
    updated_time_blocks?: TimeBlockView[]
  }
}

/**
 * 删除时间块的响应（特殊情况，因为时间块已删除）
 */
export interface DeleteTimeBlockResponse {
  time_block_id: string
  side_effects?: {
    updated_tasks?: TaskCard[]
    updated_time_blocks?: TimeBlockView[]
  }
}

/**
 * 事务元数据
 */
export interface TransactionMeta {
  correlation_id?: string
  event_id?: string
  source: 'http' | 'sse'
}

/**
 * 事务处理器类
 */
class TransactionProcessor {
  private processed = new Set<string>()
  private readonly TTL = 10000 // 10秒后清理

  /**
   * 应用任务事务结果
   */
  async applyTaskTransaction(result: TaskTransactionResult, meta: TransactionMeta): Promise<void> {
    const key = this.getTransactionKey(meta)

    if (this.processed.has(key)) {
      logger.debug(LogTags.SYSTEM_API, 'Transaction already processed (Task)', {
        key,
        source: meta.source,
        taskId: result.task.id,
      })
      return
    }

    logger.info(LogTags.SYSTEM_API, 'Applying task transaction', {
      taskId: result.task.id,
      source: meta.source,
      hasSideEffects: !!result.side_effects,
      correlationId: meta.correlation_id,
    })

    // 1. 更新主资源（任务）
    const taskStore = useTaskStore()
    taskStore.addOrUpdateTask_mut(result.task)

    // 2. 应用副作用
    if (result.side_effects) {
      await this.applyTaskSideEffects(result.side_effects)
    }

    // 3. 记录已处理
    this.markProcessed(key)
  }

  /**
   * 应用时间块事务结果
   */
  async applyTimeBlockTransaction(
    result: TimeBlockTransactionResult,
    meta: TransactionMeta
  ): Promise<void> {
    const key = this.getTransactionKey(meta)

    if (this.processed.has(key)) {
      logger.debug(LogTags.SYSTEM_API, 'Transaction already processed (TimeBlock)', {
        key,
        source: meta.source,
        timeBlockId: result.time_block.id,
      })
      return
    }

    logger.info(LogTags.SYSTEM_API, 'Applying time block transaction', {
      timeBlockId: result.time_block.id,
      source: meta.source,
      hasSideEffects: !!result.side_effects,
    })

    // 1. 更新主资源（时间块）
    const timeBlockStore = useTimeBlockStore()
    timeBlockStore.addOrUpdateTimeBlock_mut(result.time_block)

    // 2. 应用副作用
    if (result.side_effects) {
      await this.applyTimeBlockSideEffects(result.side_effects)
    }

    // 3. 记录已处理
    this.markProcessed(key)
  }

  /**
   * 应用删除时间块的响应
   */
  async applyDeleteTimeBlock(
    response: DeleteTimeBlockResponse,
    meta: TransactionMeta
  ): Promise<void> {
    const key = this.getTransactionKey(meta)

    if (this.processed.has(key)) {
      logger.debug(LogTags.SYSTEM_API, 'Transaction already processed (Delete TimeBlock)', {
        key,
        source: meta.source,
      })
      return
    }

    logger.info(LogTags.SYSTEM_API, 'Applying delete time block', {
      timeBlockId: response.time_block_id,
      source: meta.source,
    })

    // 1. 删除时间块
    const timeBlockStore = useTimeBlockStore()
    timeBlockStore.removeTimeBlock_mut(response.time_block_id)

    // 2. 应用副作用（更新受影响的任务）
    if (response.side_effects) {
      await this.applyTimeBlockSideEffects(response.side_effects)
    }

    // 3. 记录已处理
    this.markProcessed(key)
  }

  /**
   * 应用任务副作用
   */
  private async applyTaskSideEffects(
    sideEffects: TaskTransactionResult['side_effects']
  ): Promise<void> {
    if (!sideEffects) return

    const timeBlockStore = useTimeBlockStore()
    const taskStore = useTaskStore()

    // 删除的时间块
    if (sideEffects.deleted_time_blocks) {
      for (const block of sideEffects.deleted_time_blocks) {
        timeBlockStore.removeTimeBlock_mut(block.id)
      }
      logger.debug(LogTags.SYSTEM_API, 'Deleted time blocks', {
        count: sideEffects.deleted_time_blocks.length,
      })
    }

    // 截断的时间块
    if (sideEffects.truncated_time_blocks) {
      for (const block of sideEffects.truncated_time_blocks) {
        timeBlockStore.addOrUpdateTimeBlock_mut(block)
      }
      logger.debug(LogTags.SYSTEM_API, 'Truncated time blocks', {
        count: sideEffects.truncated_time_blocks.length,
      })
    }

    // 更新的时间块
    if (sideEffects.updated_time_blocks) {
      for (const block of sideEffects.updated_time_blocks) {
        timeBlockStore.addOrUpdateTimeBlock_mut(block)
      }
      logger.debug(LogTags.SYSTEM_API, 'Updated time blocks', {
        count: sideEffects.updated_time_blocks.length,
      })
    }

    // 创建的时间块
    if (sideEffects.created_time_blocks) {
      for (const block of sideEffects.created_time_blocks) {
        timeBlockStore.addOrUpdateTimeBlock_mut(block)
      }
      logger.debug(LogTags.SYSTEM_API, 'Created time blocks', {
        count: sideEffects.created_time_blocks.length,
      })
    }

    // 更新的其他任务
    if (sideEffects.updated_tasks) {
      for (const task of sideEffects.updated_tasks) {
        taskStore.addOrUpdateTask_mut(task)
      }
      logger.debug(LogTags.SYSTEM_API, 'Updated other tasks', {
        count: sideEffects.updated_tasks.length,
      })
    }
  }

  /**
   * 应用时间块副作用
   */
  private async applyTimeBlockSideEffects(
    sideEffects:
      | TimeBlockTransactionResult['side_effects']
      | DeleteTimeBlockResponse['side_effects']
  ): Promise<void> {
    if (!sideEffects) return

    const taskStore = useTaskStore()
    const timeBlockStore = useTimeBlockStore()

    // 更新的任务
    if (sideEffects.updated_tasks) {
      for (const task of sideEffects.updated_tasks) {
        taskStore.addOrUpdateTask_mut(task)
      }
      logger.debug(LogTags.SYSTEM_API, 'Updated tasks from time block operation', {
        count: sideEffects.updated_tasks.length,
      })
    }

    // 更新的时间块
    if (sideEffects.updated_time_blocks) {
      for (const block of sideEffects.updated_time_blocks) {
        timeBlockStore.addOrUpdateTimeBlock_mut(block)
      }
      logger.debug(LogTags.SYSTEM_API, 'Updated time blocks from time block operation', {
        count: sideEffects.updated_time_blocks.length,
      })
    }
  }

  /**
   * 生成事务键（用于去重）
   */
  private getTransactionKey(meta: TransactionMeta): string {
    if (meta.correlation_id) {
      return `corr:${meta.correlation_id}`
    }
    if (meta.event_id) {
      return `event:${meta.event_id}`
    }
    // 降级方案：使用时间戳（不保证去重）
    return `time:${Date.now()}`
  }

  /**
   * 标记事务已处理，并设置 TTL 自动清理
   */
  private markProcessed(key: string): void {
    this.processed.add(key)

    // TTL 后自动清理
    setTimeout(() => {
      this.processed.delete(key)
    }, this.TTL)
  }

  /**
   * 清理所有已处理记录（用于测试或重置）
   */
  clear(): void {
    this.processed.clear()
  }

  /**
   * 获取统计信息（用于调试）
   */
  getStats() {
    return {
      processedCount: this.processed.size,
      processedKeys: Array.from(this.processed),
    }
  }
}

// 导出单例
export const transactionProcessor = new TransactionProcessor()
