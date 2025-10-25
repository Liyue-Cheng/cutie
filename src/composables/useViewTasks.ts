import { computed, onMounted } from 'vue'
import type { TaskCard } from '@/types/dtos'
import { useTaskStore } from '@/stores/task'
import { useViewStore } from '@/stores/view'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * 根据 viewKey 自动获取和排序任务
 *
 * 支持的 viewKey 格式（遵循 VIEW_CONTEXT_KEY_SPEC）：
 * - misc::staging - 未安排的任务
 * - misc::staging::{areaId} - 指定区域的 staging 任务
 * - misc::planned - 已安排的任务
 * - misc::incomplete - 所有未完成任务
 * - misc::completed - 已完成任务
 * - misc::archive - 归档任务
 * - misc::all - 所有任务
 * - daily::{YYYY-MM-DD} - 指定日期的任务
 * - area::{uuid} - 指定区域的所有任务
 * - project::{uuid} - 指定项目的任务
 */
export function useViewTasks(viewKey: string) {
  const taskStore = useTaskStore()
  const viewStore = useViewStore()

  /**
   * 根据 viewKey 获取基础任务列表并应用排序
   */
  const tasks = computed<TaskCard[]>(() => {
    if (!viewKey) {
      logger.warn(LogTags.STORE_VIEW, 'ViewKey is empty, returning empty array')
      return []
    }

    // 🔥 首先尝试使用新的统一 viewKey mux（支持复杂格式）
    try {
      const baseTasks = taskStore.getTasksByViewKey_Mux(viewKey)
      if (baseTasks.length > 0 || isComplexViewKey(viewKey)) {
        // 应用排序
        const sortedTasks = viewStore.applySorting(baseTasks, viewKey)

        logger.debug(
          LogTags.STORE_VIEW,
          `${viewKey}: ${baseTasks.length} base → ${sortedTasks.length} sorted (via viewKey mux)`,
          {
            baseCount: baseTasks.length,
            sortedCount: sortedTasks.length,
            viewKey,
          }
        )

        return sortedTasks
      }
    } catch (error) {
      logger.warn(LogTags.STORE_VIEW, 'ViewKey mux failed, falling back to legacy logic', {
        viewKey,
        error,
      })
    }

    // 兜底：使用旧的分支逻辑（向后兼容）
    const parts = viewKey.split('::')
    if (parts.length < 2) {
      logger.warn(LogTags.STORE_VIEW, 'Invalid viewKey format', { viewKey })
      return []
    }

    const [type, id] = parts
    let baseTasks: TaskCard[] = []

    // 确保 id 存在
    if (!id) {
      logger.warn(LogTags.STORE_VIEW, 'Missing id in viewKey', { viewKey })
      return []
    }

    try {
      switch (type) {
        case 'daily':
          baseTasks = taskStore.getTasksByDate_Mux(id)
          break
        case 'area':
          baseTasks = taskStore.getTasksByArea_Mux(id)
          break
        case 'project':
          baseTasks = taskStore.getTasksByProject_Mux(id)
          break
        case 'misc':
          switch (id) {
            case 'all':
              baseTasks = taskStore.allTasks
              break
            case 'staging':
              baseTasks = taskStore.stagingTasks
              break
            case 'planned':
              baseTasks = taskStore.plannedTasks
              break
            case 'incomplete':
              baseTasks = taskStore.incompleteTasks
              break
            case 'completed':
              baseTasks = taskStore.completedTasks
              break
            case 'archive':
              baseTasks = taskStore.archivedTasks
              break
            default:
              logger.warn(LogTags.STORE_VIEW, 'Unknown misc viewKey', { viewKey })
              baseTasks = []
          }
          break
        default:
          logger.warn(LogTags.STORE_VIEW, 'Unknown viewKey type', { type, viewKey })
          baseTasks = []
      }

      // 应用排序
      const sortedTasks = viewStore.applySorting(baseTasks, viewKey)

      // 调试日志
      logger.debug(
        LogTags.STORE_VIEW,
        `${viewKey}: ${baseTasks.length} base → ${sortedTasks.length} sorted (via legacy logic)`,
        {
          baseCount: baseTasks.length,
          sortedCount: sortedTasks.length,
          viewKey,
        }
      )

      return sortedTasks
    } catch (error) {
      logger.error(
        LogTags.STORE_VIEW,
        'Error processing viewKey',
        error instanceof Error ? error : new Error(String(error)),
        { viewKey }
      )
      return []
    }
  })

  /**
   * 判断是否是复杂的 viewKey 格式（三段式或更多）
   */
  function isComplexViewKey(viewKey: string): boolean {
    const parts = viewKey.split('::')
    return (
      parts.length >= 3 ||
      (parts.length === 2 && ['misc::staging', 'misc::archive'].includes(viewKey))
    )
  }

  /**
   * 组件挂载时预加载排序配置和数据
   */
  onMounted(async () => {
    if (viewKey) {
      try {
        // 1. 加载排序配置
        await viewStore.fetchViewPreference(viewKey)
        logger.debug(LogTags.STORE_VIEW, 'Loaded sorting preference', { viewKey })

        // 2. 如果是日视图，调用专用端点获取任务（触发循环任务实例化）
        const parts = viewKey.split('::')
        if (parts.length >= 2 && parts[0] === 'daily' && parts[1]) {
          const date = parts[1]
          logger.info(LogTags.STORE_VIEW, 'Fetching daily tasks for date', { date, viewKey })
          await taskStore.fetchDailyTasks_DMA(date)
          logger.info(LogTags.STORE_VIEW, 'Daily tasks loaded', { date, viewKey })
        }
      } catch (error) {
        logger.error(
          LogTags.STORE_VIEW,
          'Failed to load view data',
          error instanceof Error ? error : new Error(String(error)),
          { viewKey }
        )
      }
    }
  })

  return {
    tasks,
  }
}
