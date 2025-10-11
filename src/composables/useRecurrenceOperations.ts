import { useRecurrenceStore } from '@/stores/recurrence'
import { useViewStore } from '@/stores/view'
import { useTaskStore } from '@/stores/task'
import { useUIStore } from '@/stores/ui'
import type { TaskCard } from '@/types/dtos'
import { logger, LogTags } from '@/services/logger'
import { waitForApiReady } from '@/composables/useApiConfig'

/**
 * 循环任务操作 Composable
 *
 * 提供循环任务相关的操作功能：
 * - 停止重复
 * - 更改重复频率
 * - 批量更新所有实例
 * - 删除所有实例并停止重复
 */
export function useRecurrenceOperations() {
  const recurrenceStore = useRecurrenceStore()
  const viewStore = useViewStore()
  const taskStore = useTaskStore()
  const uiStore = useUIStore()

  /**
   * 停止重复（设置结束日期为当前任务的原始日期）
   */
  async function stopRepeating(recurrenceId: string, originalDate: string) {
    const confirmed = confirm(
      `确定停止此循环吗？\n将从 ${originalDate} 之后停止生成新任务。\n已生成的任务不会被删除。`
    )

    if (!confirmed) return

    try {
      await recurrenceStore.updateRecurrence(recurrenceId, {
        end_date: originalDate,
      })

      // 刷新所有已挂载的日视图
      await viewStore.refreshAllMountedDailyViews()

      logger.info(LogTags.COMPOSABLE_RECURRENCE, 'Successfully stopped repeating', {
        recurrenceId,
        endDate: originalDate,
      })
    } catch (error) {
      logger.error(
        LogTags.COMPOSABLE_RECURRENCE,
        'Failed to stop repeating',
        error instanceof Error ? error : new Error(String(error)),
        { recurrenceId, originalDate }
      )
      throw error
    }
  }

  /**
   * 打开编辑循环规则对话框
   *
   * 通过 UI Store 打开全局的循环规则编辑对话框
   * RecurrenceBoard 组件会监听 UI Store 的状态并显示对话框
   */
  function openEditDialog(recurrenceId: string) {
    logger.info(LogTags.COMPOSABLE_RECURRENCE, 'Opening edit dialog for recurrence', {
      recurrenceId,
    })

    uiStore.openRecurrenceEditDialog(recurrenceId)
  }

  /**
   * 更新所有未完成实例以匹配当前任务
   *
   * @param recurrenceId 循环规则ID
   * @param sourceTask 源任务（TaskCard），用于获取 taskId
   */
  async function updateAllInstances(recurrenceId: string, sourceTask: TaskCard) {
    const confirmed = confirm(
      `确定将所有未完成的循环任务实例更新为与当前任务相同吗？\n` +
        `这将更新标题、笔记、预期时长、子任务、区域等信息。\n` +
        `同时也会更新循环模板，影响未来生成的新实例。\n` +
        `已完成的任务不会被影响。`
    )

    if (!confirmed) return

    try {
      // 1. 先获取完整的任务详情（包含 detail_note、subtasks 等）
      logger.info(LogTags.COMPOSABLE_RECURRENCE, 'Fetching task detail for batch update', {
        taskId: sourceTask.id,
        recurrenceId,
      })

      const taskDetail = await taskStore.fetchTaskDetail(sourceTask.id)

      if (!taskDetail) {
        throw new Error('无法获取任务详情')
      }

      // 2. 获取循环规则信息（用于找到模板ID）
      let recurrence = recurrenceStore.getRecurrenceById(recurrenceId)
      if (!recurrence) {
        // 如果本地 store 没有数据，先从后端获取
        logger.info(
          LogTags.COMPOSABLE_RECURRENCE,
          'Recurrence not found in store, fetching from backend',
          {
            recurrenceId,
          }
        )
        await recurrenceStore.fetchAllRecurrences()
        recurrence = recurrenceStore.getRecurrenceById(recurrenceId)

        if (!recurrence) {
          throw new Error('无法找到循环规则')
        }
      }

      logger.debug(LogTags.COMPOSABLE_RECURRENCE, 'Task detail and recurrence fetched', {
        taskId: sourceTask.id,
        title: taskDetail.title,
        hasDetailNote: !!taskDetail.detail_note,
        subtasksCount: taskDetail.subtasks?.length || 0,
        templateId: recurrence.template_id,
      })

      // 3. 构造请求体（基于 TaskDetail）
      const instancePayload = {
        title: taskDetail.title,
        glance_note: taskDetail.glance_note,
        detail_note: taskDetail.detail_note,
        estimated_duration: taskDetail.estimated_duration,
        area_id: taskDetail.area_id,
        subtasks: taskDetail.subtasks, // 新增：同步子任务
      }

      // 4. 🔥 使用新的统一端点，在同一事务中更新模板和实例
      const payload = {
        title: taskDetail.title,
        glance_note: taskDetail.glance_note,
        detail_note: taskDetail.detail_note,
        estimated_duration: taskDetail.estimated_duration,
        area_id: taskDetail.area_id,
        subtasks: taskDetail.subtasks,
      }

      logger.info(
        LogTags.COMPOSABLE_RECURRENCE,
        'Updating template and instances in single transaction',
        {
          recurrenceId,
          payload: {
            ...payload,
            detail_note: payload.detail_note ? `(${payload.detail_note.length} chars)` : null,
            subtasks: payload.subtasks ? `(${payload.subtasks.length} items)` : null,
          },
        }
      )

      // 5. 调用新的统一端点
      const response = await fetch(
        `${await waitForApiReady()}/recurrences/${recurrenceId}/template-and-instances`,
        {
          method: 'PATCH',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload),
        }
      )

      // 6. 检查结果
      if (!response.ok) {
        const errorText = await response.text()
        logger.error(
          LogTags.COMPOSABLE_RECURRENCE,
          'Batch update template and instances failed',
          new Error(`HTTP ${response.status}`),
          { errorText }
        )
        throw new Error(`批量更新失败: HTTP ${response.status}: ${errorText}`)
      }

      const result = await response.json()
      const { template_updated, instances_updated_count } = result.data || result

      logger.info(LogTags.COMPOSABLE_RECURRENCE, 'Template and instances updated successfully', {
        recurrenceId,
        templateUpdated: template_updated,
        instancesUpdatedCount: instances_updated_count,
      })

      // 7. 刷新所有已挂载的日视图
      await viewStore.refreshAllMountedDailyViews()

      alert(
        `成功更新了模板${template_updated ? '和' : '，'}${instances_updated_count} 个未完成的任务实例。\n未来生成的新实例也会使用更新后的内容。`
      )
    } catch (error) {
      logger.error(
        LogTags.COMPOSABLE_RECURRENCE,
        'Failed to update template and instances',
        error instanceof Error ? error : new Error(String(error)),
        { recurrenceId, sourceTaskId: sourceTask.id }
      )
      alert('批量更新失败，请查看控制台日志或重试。')
      throw error
    }
  }

  /**
   * 删除所有未完成实例并停止重复
   */
  async function deleteAllInstancesAndStop(recurrenceId: string) {
    try {
      // 直接删除循环规则，后端会自动清理所有未完成实例
      await recurrenceStore.deleteRecurrence(recurrenceId)

      // 刷新所有已挂载的日视图
      await viewStore.refreshAllMountedDailyViews()

      logger.info(
        LogTags.COMPOSABLE_RECURRENCE,
        'Successfully deleted all instances and stopped repeating',
        {
          recurrenceId,
        }
      )
    } catch (error) {
      logger.error(
        LogTags.COMPOSABLE_RECURRENCE,
        'Failed to delete all instances and stop repeating',
        error instanceof Error ? error : new Error(String(error)),
        { recurrenceId }
      )
      alert('删除失败，请重试。')
      throw error
    }
  }

  return {
    stopRepeating,
    openEditDialog,
    updateAllInstances,
    deleteAllInstancesAndStop,
  }
}
