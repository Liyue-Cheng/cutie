/**
 * Template Store 事件处理器 (v4.0)
 *
 * 职责：
 * - 处理 SSE 推送的模板领域事件
 * - 使用 InterruptHandler 统一处理
 * - 与 Task store 保持一致的架构
 *
 * 架构升级：
 * - v1.0: 使用全局 window.__eventBus__
 * - v4.0: 使用 InterruptHandler（与 Task store 一致）
 */

import type { Template } from '@/types/dtos'
import * as core from './core'
import { logger, LogTags } from '@/infra/logging/logger'

// ==================== Event Handlers ====================

/**
 * 处理模板创建事件
 */
export function handleTemplateCreated(data: Template) {
  logger.info(LogTags.STORE_TEMPLATE, 'Template created event received', {
    templateId: data.id,
    title: data.title,
  })
  core.addOrUpdateTemplate_mut(data)
}

/**
 * 处理模板更新事件
 */
export function handleTemplateUpdated(data: Template) {
  logger.info(LogTags.STORE_TEMPLATE, 'Template updated event received', {
    templateId: data.id,
    title: data.title,
  })
  core.addOrUpdateTemplate_mut(data)
}

/**
 * 处理模板删除事件
 */
export function handleTemplateDeleted(data: { id: string }) {
  logger.info(LogTags.STORE_TEMPLATE, 'Template deleted event received', {
    templateId: data.id,
  })
  core.removeTemplate_mut(data.id)
}

// ==================== Event Subscriptions (v4.0) ====================

/**
 * 初始化事件订阅（v4.0 架构）
 *
 * 通过 InterruptHandler 注册，与 Task store 保持一致
 */
export function initEventSubscriptions() {
  import('@/cpu/interrupt/InterruptHandler').then(({ interruptHandler }) => {
    // 🔥 注册到 INT（中断管理器）
    interruptHandler.on('template.created', handleTemplateEvent)
    interruptHandler.on('template.updated', handleTemplateEvent)
    interruptHandler.on('template.deleted', handleTemplateEvent)

    logger.info(LogTags.STORE_TEMPLATE, 'Template event subscriptions initialized (v4.0 - via INT)')
  })
}

/**
 * 统一的模板事件处理器
 * v4.0: 接收 InterruptEvent 格式
 */
async function handleTemplateEvent(event: any) {
  try {
    const eventType = event.eventType
    const payload = event.payload

    switch (eventType) {
      case 'template.created':
        handleTemplateCreated(payload)
        break
      case 'template.updated':
        handleTemplateUpdated(payload)
        break
      case 'template.deleted':
        handleTemplateDeleted(payload)
        break
      default:
        logger.warn(LogTags.STORE_TEMPLATE, 'Unknown template event type', { eventType })
    }
  } catch (error) {
    logger.error(
      LogTags.STORE_TEMPLATE,
      'Failed to process template event',
      error instanceof Error ? error : new Error(String(error)),
      {
        eventType: event.eventType,
        correlationId: event.correlationId,
      }
    )
  }
}
