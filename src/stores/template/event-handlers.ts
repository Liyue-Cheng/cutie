import type { Template } from '@/types/dtos'
import * as core from './core'
import { logger, LogTags } from '@/services/logger'

// ==================== Event Handlers ====================

/**
 * 处理模板创建事件
 */
export function handleTemplateCreated(data: Template) {
  logger.info(LogTags.STORE_TEMPLATE, 'Template created event received', {
    templateId: data.id,
    title: data.title
  })
  core.addOrUpdateTemplate(data)
}

/**
 * 处理模板更新事件
 */
export function handleTemplateUpdated(data: Template) {
  logger.info(LogTags.STORE_TEMPLATE, 'Template updated event received', {
    templateId: data.id,
    title: data.title
  })
  core.addOrUpdateTemplate(data)
}

/**
 * 处理模板删除事件
 */
export function handleTemplateDeleted(data: { id: string }) {
  logger.info(LogTags.STORE_TEMPLATE, 'Template deleted event received', {
    templateId: data.id
  })
  core.removeTemplate(data.id)
}

// ==================== Event Subscriptions ====================

/**
 * 初始化 SSE 事件订阅
 */
export function initEventSubscriptions() {
  const { eventBus } = useEventBus()

  // 订阅模板创建事件
  eventBus.on('template.created', (data: Template) => {
    handleTemplateCreated(data)
  })

  // 订阅模板更新事件
  eventBus.on('template.updated', (data: Template) => {
    handleTemplateUpdated(data)
  })

  // 订阅模板删除事件
  eventBus.on('template.deleted', (data: { id: string }) => {
    handleTemplateDeleted(data)
  })

  logger.info(LogTags.STORE_TEMPLATE, 'Template SSE event subscriptions initialized')
}

// ==================== Helper ====================

function useEventBus() {
  // 从全局获取 eventBus
  const eventBus = (window as any).__eventBus__
  if (!eventBus) {
    logger.error(LogTags.STORE_TEMPLATE, 'EventBus not found on window')
    throw new Error('EventBus not initialized')
  }
  return { eventBus }
}

