/**
 * Recurrence Store - Event Handlers
 *
 * 职责：处理 SSE 事件，更新本地状态
 */

import { logger, LogTags } from '@/infra/logging/logger'
import * as core from './core'
import type { TaskRecurrence } from '@/types/dtos'

/**
 * 初始化 Recurrence 相关的 SSE 事件订阅
 *
 * v4.0: 所有事件通过 INT（中断管理器）注册
 */
export function initEventSubscriptions() {
  import('@/cpu/interrupt/InterruptHandler').then(({ interruptHandler }) => {
    // 🔥 注册到 INT（中断管理器）

    // Recurrence 创建事件
    interruptHandler.on('recurrence.created', (event) => {
      const data = event.payload as TaskRecurrence
      logger.debug(LogTags.STORE_RECURRENCE, 'SSE: recurrence.created', {
        recurrenceId: data.id,
      })
      core.addOrUpdateRecurrence(data)
    })

    // Recurrence 更新事件
    interruptHandler.on('recurrence.updated', (event) => {
      const data = event.payload as TaskRecurrence
      logger.debug(LogTags.STORE_RECURRENCE, 'SSE: recurrence.updated', {
        recurrenceId: data.id,
      })
      core.addOrUpdateRecurrence(data)
    })

    // Recurrence 删除事件
    interruptHandler.on('recurrence.deleted', (event) => {
      const data = event.payload as { id: string }
      logger.debug(LogTags.STORE_RECURRENCE, 'SSE: recurrence.deleted', {
        recurrenceId: data.id,
      })
      core.removeRecurrence(data.id)
    })

    logger.info(
      LogTags.STORE_RECURRENCE,
      'Recurrence event subscriptions initialized (v4.0 - via INT)'
    )
  })
}
