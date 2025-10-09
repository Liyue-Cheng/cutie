import { computed, onMounted } from 'vue'
import type { TaskCard } from '@/types/dtos'
import { useTaskStore } from '@/stores/task'
import { useViewStore } from '@/stores/view'
import { logger, LogTags } from '@/services/logger'

/**
 * 根据 viewKey 自动获取和排序任务
 *
 * 支持的 viewKey 格式（遵循 VIEW_CONTEXT_KEY_SPEC）：
 * - misc::staging - 未安排的任务
 * - misc::planned - 已安排的任务
 * - misc::incomplete - 所有未完成任务
 * - misc::completed - 已完成任务
 * - misc::all - 所有任务
 * - daily::{YYYY-MM-DD} - 指定日期的任务
 * - area::{uuid} - 指定区域的任务
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

    const parts = viewKey.split('::')
    if (parts.length < 2) {
      logger.warn(LogTags.STORE_VIEW, 'Invalid viewKey format', { viewKey })
      return []
    }

    const [type, id] = parts
    let baseTasks: TaskCard[] = []

    try {
      switch (type) {
        case 'daily':
          baseTasks = taskStore.getTasksByDate(id)
          break
        case 'area':
          baseTasks = taskStore.getTasksByArea(id)
          break
        case 'project':
          baseTasks = taskStore.getTasksByProject(id)
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
      logger.debug(LogTags.STORE_VIEW, `${viewKey}: ${baseTasks.length} base → ${sortedTasks.length} sorted`, {
        baseCount: baseTasks.length,
        sortedCount: sortedTasks.length,
        viewKey
      })

      return sortedTasks
    } catch (error) {
      logger.error(LogTags.STORE_VIEW, 'Error processing viewKey', error, { viewKey })
      return []
    }
  })

  /**
   * 组件挂载时预加载排序配置
   */
  onMounted(async () => {
    if (viewKey) {
      try {
        await viewStore.fetchViewPreference(viewKey)
        logger.debug(LogTags.STORE_VIEW, 'Loaded sorting preference', { viewKey })
      } catch (error) {
        logger.error(LogTags.STORE_VIEW, 'Failed to load preference', error, { viewKey })
      }
    }
  })

  return {
    tasks,
  }
}
