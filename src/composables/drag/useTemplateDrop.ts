/**
 * useTemplateDrop - 模板拖放处理
 *
 * 处理模板拖动到看板的逻辑
 */

import { useDragTransfer } from './useDragTransfer'
import type { ViewMetadata, DateViewConfig } from '@/types/drag'
import { logger, LogTags } from '@/infra/logging/logger'

export interface TemplateDropResult {
  /** 是否已处理 */
  handled: boolean
  /** 是否成功 */
  success: boolean
  /** 错误信息 */
  error?: string
  /** 创建的任务ID */
  taskId?: string
}

/**
 * 模板拖放处理器
 */
export function useTemplateDrop() {
  const dragTransfer = useDragTransfer()

  /**
   * 处理模板拖放到看板
   * @param event - DragEvent
   * @param targetView - 目标看板元数据
   * @returns 处理结果
   */
  async function handleTemplateDrop(
    event: DragEvent,
    targetView: ViewMetadata
  ): Promise<TemplateDropResult> {
    const dragData = dragTransfer.getDragData(event)

    // 不是模板拖放，返回未处理
    if (!dragData || dragData.type !== 'template') {
      return { handled: false, success: false }
    }

    logger.info(LogTags.DRAG_STRATEGY, 'Template drop detected', {
      templateId: dragData.templateId,
      templateName: dragData.templateName,
      targetView: targetView.id,
      targetType: targetView.type,
    })

    // 只在日期类型的看板中处理模板拖放
    if (targetView.type !== 'date') {
      logger.warn(LogTags.DRAG_STRATEGY, 'Template drop not supported for non-date views', {
        targetType: targetView.type,
      })
      return {
        handled: true,
        success: false,
        error: '模板只能拖放到日期看板',
      }
    }

    const viewConfig = targetView.config as DateViewConfig
    const targetDate = viewConfig.date

    if (!targetDate) {
      logger.error(LogTags.DRAG_STRATEGY, 'Target date not found in view config')
      return {
        handled: true,
        success: false,
        error: '目标日期无效',
      }
    }

    try {
      // 动态导入 stores
      const { useTemplateStore } = await import('@/stores/template')
      const { useTaskStore } = await import('@/stores/task')

      const templateStore = useTemplateStore()
      const taskStore = useTaskStore()

      // 从模板创建任务
      logger.info(LogTags.DRAG_STRATEGY, 'Creating task from template', {
        templateId: dragData.templateId,
      })

      const newTask = await templateStore.createTaskFromTemplate(dragData.templateId)

      // 详细日志：检查返回的数据
      logger.debug(LogTags.DRAG_STRATEGY, 'Received task from template', {
        newTask,
        hasTask: !!newTask,
        hasId: newTask ? !!newTask.id : false,
        taskId: newTask?.id,
        taskType: typeof newTask,
      })

      if (!newTask || !newTask.id) {
        logger.error(
          LogTags.DRAG_STRATEGY,
          'Task creation validation failed',
          new Error('No task or task ID'),
          {
            newTask,
            newTaskKeys: newTask ? Object.keys(newTask) : [],
          }
        )
        throw new Error('创建任务失败：未返回任务数据')
      }

      logger.info(LogTags.DRAG_STRATEGY, 'Task created from template', {
        taskId: newTask.id,
        templateId: dragData.templateId,
      })

      // 将新创建的任务添加到 taskStore
      taskStore.addOrUpdateTask(newTask)

      // 添加日程到目标日期
      logger.info(LogTags.DRAG_STRATEGY, 'Adding schedule to task', {
        taskId: newTask.id,
        date: targetDate,
      })

      await taskStore.addSchedule(newTask.id, targetDate)

      logger.info(LogTags.DRAG_STRATEGY, 'Template drop completed successfully', {
        taskId: newTask.id,
        date: targetDate,
      })

      return {
        handled: true,
        success: true,
        taskId: newTask.id,
      }
    } catch (error) {
      logger.error(
        LogTags.DRAG_STRATEGY,
        'Failed to create task from template',
        error instanceof Error ? error : new Error(String(error))
      )

      let errorMessage = '从模板创建任务失败'
      if (error instanceof Error) {
        errorMessage = error.message
      } else if (typeof error === 'string') {
        errorMessage = error
      }

      return {
        handled: true,
        success: false,
        error: errorMessage,
      }
    }
  }

  return {
    handleTemplateDrop,
  }
}
