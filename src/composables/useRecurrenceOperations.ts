import { useRecurrenceStore } from '@/stores/recurrence'
import { useTaskStore } from '@/stores/task'
import { useUIStore } from '@/stores/ui'
import type { TaskCard } from '@/types/dtos'
import { logger, LogTags } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'
import { getTodayDateString } from '@/infra/utils/dateUtils'

interface RecurrenceCleanupOptions {
  removeAfterDateExclusive?: string | null
  removeFromDateInclusive?: string | null
}

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
  const taskStore = useTaskStore()
  const uiStore = useUIStore()

  /**
   * 计算需要从 TaskStore 中移除的任务 ID
   */
  function collectRecurrenceTaskIds(
    recurrenceId: string,
    options: RecurrenceCleanupOptions = {}
  ): string[] {
    const { removeAfterDateExclusive, removeFromDateInclusive } = options
    const candidates = taskStore.allTasks.filter((task) => {
      if (task.recurrence_id !== recurrenceId) {
        return false
      }

      if (task.is_completed) {
        return false
      }

      if (task.is_deleted) {
        return true
      }

      const originalDate = task.recurrence_original_date

      if (removeAfterDateExclusive != null) {
        const cutoff = removeAfterDateExclusive
        if (!originalDate) {
          return true
        }
        if (originalDate > cutoff) {
          return true
        }
        return false
      }

      if (removeFromDateInclusive != null) {
        const cutoff = removeFromDateInclusive
        if (!originalDate) {
          return true
        }
        if (originalDate >= cutoff) {
          return true
        }
        return false
      }

      return true
    })

    return candidates.map((task) => task.id)
  }

  /**
   * 删除本地缓存中的循环任务实例，并刷新任务数据
   *
   * ⚠️ 注意：视图刷新已由 CPU 指令的 commit 阶段统一处理，这里只需要清理本地缓存
   */
  async function synchronizeAfterRecurrenceMutation(
    recurrenceId: string,
    options: RecurrenceCleanupOptions
  ) {
    const taskIdsToRemove = collectRecurrenceTaskIds(recurrenceId, options)

    if (taskIdsToRemove.length > 0) {
      taskStore.batchRemoveTasks_mut(taskIdsToRemove)
      logger.info(LogTags.COMPOSABLE_RECURRENCE, 'Removed recurrence tasks from store', {
        recurrenceId,
        count: taskIdsToRemove.length,
        options,
      })
    }

    try {
      await taskStore.fetchAllIncompleteTasks_DMA()
    } catch (error) {
      logger.error(
        LogTags.COMPOSABLE_RECURRENCE,
        'Failed to refetch incomplete tasks after recurrence mutation',
        error instanceof Error ? error : new Error(String(error)),
        { recurrenceId }
      )
    }

    // ✅ 视图刷新已由 CPU 指令的 commit 阶段统一处理，这里不再重复调用
  }

  /**
   * 停止重复（设置结束日期为当前任务的原始日期）
   */
  async function stopRepeating(recurrenceId: string, originalDate: string) {
    const confirmed = confirm(
      `确定停止此循环吗？\n将从 ${originalDate} 之后停止生成新任务。\n已生成的任务不会被删除。`
    )

    if (!confirmed) return

    try {
      await pipeline.dispatch('recurrence.update', {
        id: recurrenceId,
        end_date: originalDate,
      })

      await synchronizeAfterRecurrenceMutation(recurrenceId, {
        removeAfterDateExclusive: originalDate,
      })

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

      const taskDetail = await taskStore.fetchTaskDetail_DMA(sourceTask.id)

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
        await pipeline.dispatch('recurrence.fetch_all', {})
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

      // 3. 构造更新数据
      const updatePayload = {
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
            ...updatePayload,
            detail_note: updatePayload.detail_note
              ? `(${updatePayload.detail_note.length} chars)`
              : null,
            subtasks: updatePayload.subtasks ? `(${updatePayload.subtasks.length} items)` : null,
          },
        }
      )

      // 4. 使用CPU指令调用统一端点
      const result = await pipeline.dispatch('recurrence.update_template_and_instances', {
        recurrence_id: recurrenceId,
        ...updatePayload,
      })

      // 5. 检查结果
      const { template_updated, instances_updated_count } = result

      logger.info(LogTags.COMPOSABLE_RECURRENCE, 'Template and instances updated successfully', {
        recurrenceId,
        templateUpdated: template_updated,
        instancesUpdatedCount: instances_updated_count,
      })

      // ✅ 视图刷新已由 CPU 指令的 commit 阶段统一处理

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
      // 使用CPU指令删除循环规则，后端会自动清理所有未完成实例
      await pipeline.dispatch('recurrence.delete', { id: recurrenceId })

      await synchronizeAfterRecurrenceMutation(recurrenceId, {
        removeFromDateInclusive: getTodayDateString(),
      })

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
