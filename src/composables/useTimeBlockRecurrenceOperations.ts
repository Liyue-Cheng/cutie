import { useTimeBlockStore } from '@/stores/timeblock'
import { useViewStore } from '@/stores/view'
import { useUIStore } from '@/stores/ui'
import { logger, LogTags } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'
import { dialog } from '@/composables/useDialog'

/**
 * 时间块循环操作 Composable
 *
 * 提供时间块循环相关的操作功能：
 * - 停止循环（在指定日期停止，删除之后的实例）
 * - 继续循环（恢复已停止的循环）
 * - 更改重复频率
 */
export function useTimeBlockRecurrenceOperations() {
  const timeBlockStore = useTimeBlockStore()
  const viewStore = useViewStore()
  const uiStore = useUIStore()

  /**
   * 停止循环（在指定日期停止，并删除之后的所有实例）
   *
   * @param recurrenceId 循环规则ID
   * @param stopDate 停止日期（YYYY-MM-DD），该日期之后的实例将被删除
   */
  async function stopRepeating(recurrenceId: string, stopDate: string) {
    const confirmed = await dialog.confirm(
      `确定停止此时间块循环吗？\n将从 ${stopDate} 之后停止生成新时间块，并删除之后的实例。`
    )

    if (!confirmed) return

    try {
      await pipeline.dispatch('timeblock-recurrence.stop', {
        id: recurrenceId,
        stop_date: stopDate,
      })

      logger.info(LogTags.COMPOSABLE_RECURRENCE, 'Successfully stopped time block recurrence', {
        recurrenceId,
        stopDate,
      })
    } catch (error) {
      logger.error(
        LogTags.COMPOSABLE_RECURRENCE,
        'Failed to stop time block recurrence',
        error instanceof Error ? error : new Error(String(error)),
        { recurrenceId, stopDate }
      )
      await dialog.alert('停止循环失败，请重试。')
      throw error
    }
  }

  /**
   * 继续循环（恢复已停止的循环，清除结束日期）
   *
   * @param recurrenceId 循环规则ID
   */
  async function resumeRecurrence(recurrenceId: string) {
    const confirmed = await dialog.confirm('确定继续此时间块循环吗？\n将恢复生成新的时间块实例。')

    if (!confirmed) return

    try {
      await pipeline.dispatch('timeblock-recurrence.resume', {
        id: recurrenceId,
      })

      logger.info(LogTags.COMPOSABLE_RECURRENCE, 'Successfully resumed time block recurrence', {
        recurrenceId,
      })
    } catch (error) {
      logger.error(
        LogTags.COMPOSABLE_RECURRENCE,
        'Failed to resume time block recurrence',
        error instanceof Error ? error : new Error(String(error)),
        { recurrenceId }
      )
      await dialog.alert('继续循环失败，请重试。')
      throw error
    }
  }

  /**
   * 打开编辑循环规则对话框
   * TODO: 需要实现时间块循环编辑对话框
   */
  function openEditDialog(recurrenceId: string) {
    logger.info(LogTags.COMPOSABLE_RECURRENCE, 'Opening edit dialog for time block recurrence', {
      recurrenceId,
    })

    uiStore.openTimeBlockRecurrenceEditDialog(recurrenceId)
  }

  /**
   * 删除循环规则及所有未来实例
   */
  async function deleteRecurrence(recurrenceId: string) {
    try {
      // 使用 CPU 指令删除循环规则，后端会自动清理相关链接和未来实例
      await pipeline.dispatch('timeblock-recurrence.delete', { id: recurrenceId })

      logger.info(LogTags.COMPOSABLE_RECURRENCE, 'Successfully deleted time block recurrence', {
        recurrenceId,
      })
    } catch (error) {
      logger.error(
        LogTags.COMPOSABLE_RECURRENCE,
        'Failed to delete time block recurrence',
        error instanceof Error ? error : new Error(String(error)),
        { recurrenceId }
      )
      await dialog.alert('删除失败，请重试。')
      throw error
    }
  }

  return {
    stopRepeating,
    resumeRecurrence,
    openEditDialog,
    deleteRecurrence,
  }
}
