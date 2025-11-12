/**
 * Area Store - Event Handlers
 *
 * 职责：处理 SSE 事件，更新本地状态
 */

import { logger, LogTags } from '@/infra/logging/logger'
import * as core from './core'
import type { Area } from './core'

/**
 * 初始化 Area 相关的 SSE 事件订阅
 *
 * v4.0: 所有事件通过 INT（中断管理器）注册
 */
export function initEventSubscriptions() {
  import('@/cpu/interrupt/InterruptHandler').then(({ interruptHandler }) => {
    // 🔥 注册到 INT（中断管理器）

    // Area 创建事件
    interruptHandler.on('area.created', (event) => {
      const data = event.payload as Area
      logger.debug(LogTags.STORE_AREA, 'SSE: area.created', { areaId: data.id })
      core.addOrUpdate_mut(data)
    })

    // Area 更新事件
    interruptHandler.on('area.updated', (event) => {
      const data = event.payload as Area
      logger.debug(LogTags.STORE_AREA, 'SSE: area.updated', { areaId: data.id })
      core.addOrUpdate_mut(data)
    })

    // Area 删除事件
    interruptHandler.on('area.deleted', (event) => {
      const data = event.payload as { id: string }
      logger.debug(LogTags.STORE_AREA, 'SSE: area.deleted', { areaId: data.id })
      core.remove_mut(data.id)
    })

    logger.info(LogTags.STORE_AREA, 'Area event subscriptions initialized (v4.0 - via INT)')
  })
}
