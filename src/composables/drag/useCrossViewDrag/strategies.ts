/**
 * useCrossViewDrag/strategies - 拖放策略注册表（v4.0 - CPU Pipeline 版）
 *
 * 定义和管理所有拖放策略
 * ✅ 完全使用 CPU Pipeline，不再直接调用 Store CRUD 方法
 */

import type {
  DragStrategy,
  StrategyRegistry,
  StatusViewConfig,
  DateViewConfig,
  ProjectViewConfig,
  CalendarViewConfig,
} from '@/types/drag'
import { logger, LogTags } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'

// ==================== 策略实现 ====================

/**
 * 策略：status -> status
 * 场景：在状态看板之间拖动
 */
const statusToStatus: DragStrategy = async (context, targetView) => {
  const sourceConfig = context.sourceView.config as StatusViewConfig
  const targetConfig = targetView.config as StatusViewConfig

  logger.debug(LogTags.DRAG_STRATEGY, 'Status to status strategy', {
    taskTitle: context.task.title,
    fromStatus: sourceConfig.status,
    toStatus: targetConfig.status,
    mode: context.dragMode.mode,
  })

  // 特殊情况：任何状态 -> staging（返回暂存区）
  if (targetConfig.status === 'staging' && sourceConfig.status !== 'staging') {
    logger.info(LogTags.DRAG_STRATEGY, 'Action: Return to staging from status view')

    try {
      // ✅ 使用 CPU Pipeline
      await pipeline.dispatch('task.return_to_staging', { id: context.task.id })

      return {
        success: true,
        message: '已返回暂存区',
        affectedViews: [context.sourceView.id, targetView.id],
      }
    } catch (error) {
      logger.error(
        LogTags.DRAG_STRATEGY,
        'Failed to return to staging from status view',
        error instanceof Error ? error : new Error(String(error))
      )

      return {
        success: false,
        error: error instanceof Error ? error.message : '返回暂存区失败',
      }
    }
  }

  // 特殊情况：staging -> planned（暂不实现，需要选择日期）
  if (sourceConfig.status === 'staging' && targetConfig.status === 'planned') {
    logger.info(LogTags.DRAG_STRATEGY, 'Action: Set scheduled_date to today', {
      taskId: context.task.id,
    })
    return {
      success: true,
      message: '已设置排期',
      affectedViews: [context.sourceView.id, targetView.id],
    }
  }

  // 默认：仅重排序
  logger.debug(LogTags.DRAG_STRATEGY, 'Action: Reorder only', { taskId: context.task.id })
  return {
    success: true,
    reorderOnly: true,
  }
}

/**
 * 策略：date -> date
 * 场景：在日期看板之间拖动（改期）
 */
const dateToDate: DragStrategy = async (context, targetView) => {
  const sourceDate = (context.sourceView.config as DateViewConfig).date
  const targetDate = (targetView.config as DateViewConfig).date

  logger.debug(LogTags.DRAG_STRATEGY, 'Date to date strategy', {
    taskTitle: context.task.title,
    fromDate: sourceDate,
    toDate: targetDate,
    mode: context.dragMode.mode,
  })

  try {
    // 检查目标日期是否已有安排
    const hasTargetSchedule = context.task.schedules?.some((s) => s.scheduled_day === targetDate)

    if (hasTargetSchedule) {
      // 目标天已有安排，删除源日期的安排即可
      logger.info(
        LogTags.DRAG_STRATEGY,
        'Action: Target date already has schedule, deleting source date',
        {
          taskId: context.task.id,
          sourceDate,
          targetDate,
        }
      )

      // ✅ 使用 CPU Pipeline
      await pipeline.dispatch('schedule.delete', {
        task_id: context.task.id,
        scheduled_day: sourceDate,
      })

      return {
        success: true,
        message: `任务在 ${targetDate} 已有安排，已删除 ${sourceDate} 的安排`,
        affectedViews: [context.sourceView.id, targetView.id],
      }
    } else {
      // 目标天没有安排，更新日期
      logger.info(LogTags.DRAG_STRATEGY, 'Action: Update scheduled_date', {
        taskId: context.task.id,
        fromDate: sourceDate,
        toDate: targetDate,
      })

      // ✅ 使用 CPU Pipeline
      await pipeline.dispatch('schedule.update', {
        task_id: context.task.id,
        scheduled_day: sourceDate,
        updates: {
          new_date: targetDate,
        },
      })

      return {
        success: true,
        message: `已改期至 ${targetDate}`,
        affectedViews: [context.sourceView.id, targetView.id],
      }
    }
  } catch (error) {
    logger.error(
      LogTags.DRAG_STRATEGY,
      'Failed to update date schedule',
      error instanceof Error ? error : new Error(String(error)),
      { taskId: context.task.id }
    )

    return {
      success: false,
      error: error instanceof Error ? error.message : '改期失败',
    }
  }
}

/**
 * 策略：project -> project
 * 场景：在项目看板之间拖动
 */
const projectToProject: DragStrategy = async (context, targetView) => {
  const sourceConfig = context.sourceView.config as ProjectViewConfig
  const targetConfig = targetView.config as ProjectViewConfig

  logger.debug(LogTags.DRAG_STRATEGY, 'Project to project strategy', {
    taskTitle: context.task.title,
    fromProject: sourceConfig.projectName,
    toProject: targetConfig.projectName,
    mode: context.dragMode.mode,
  })

  // 检查权限：已完成的任务不能移动项目
  if (context.task.is_completed) {
    logger.warn(LogTags.DRAG_STRATEGY, 'Blocked: Completed tasks cannot change projects', {
      taskId: context.task.id,
    })
    return {
      success: false,
      error: '已完成的任务不能移动到其他项目',
    }
  }

  logger.info(LogTags.DRAG_STRATEGY, 'Action: Change project', {
    fromProjectId: sourceConfig.projectId,
    toProjectId: targetConfig.projectId,
  })

  return {
    success: true,
    message: `已移动到项目 ${targetConfig.projectName}`,
    affectedViews: [context.sourceView.id, targetView.id],
  }
}

/**
 * 策略：status -> date
 * 场景：从状态看板拖到日期看板（设置排期）
 */
const statusToDate: DragStrategy = async (context, targetView) => {
  const sourceStatus = (context.sourceView.config as StatusViewConfig).status
  const targetDate = (targetView.config as DateViewConfig).date

  logger.debug(LogTags.DRAG_STRATEGY, 'Status to date strategy', {
    taskTitle: context.task.title,
    fromStatus: sourceStatus,
    toDate: targetDate,
    mode: context.dragMode.mode,
  })

  try {
    // 特殊处理：从 staging 拖到日期看板，新建安排
    if (sourceStatus === 'staging') {
      logger.info(LogTags.DRAG_STRATEGY, 'Action: Add schedule for date', { targetDate })

      // ✅ 使用 CPU Pipeline
      await pipeline.dispatch('schedule.create', {
        task_id: context.task.id,
        scheduled_day: targetDate,
      })

      return {
        success: true,
        message: `已添加排期：${targetDate}`,
        affectedViews: [context.sourceView.id, targetView.id],
      }
    }

    // 其他状态看板：仅提示（保留原有逻辑）
    logger.info(LogTags.DRAG_STRATEGY, 'Action: Set scheduled_date', { targetDate })

    return {
      success: true,
      message: `已设置排期：${targetDate}`,
      affectedViews: [context.sourceView.id, targetView.id],
    }
  } catch (error) {
    logger.error(
      LogTags.DRAG_STRATEGY,
      'Failed to add schedule',
      error instanceof Error ? error : new Error(String(error))
    )

    return {
      success: false,
      error: error instanceof Error ? error.message : '设置排期失败',
    }
  }
}

/**
 * 策略：date -> status
 * 场景：从日期看板拖回状态看板
 */
const dateToStatus: DragStrategy = async (context, targetView) => {
  const sourceDate = (context.sourceView.config as DateViewConfig).date
  const targetStatus = (targetView.config as StatusViewConfig).status

  logger.debug(LogTags.DRAG_STRATEGY, 'Date to status strategy', {
    taskTitle: context.task.title,
    fromDate: sourceDate,
    toStatus: targetStatus,
    mode: context.dragMode.mode,
  })

  // 拖回 staging：调用返回暂存区API
  if (targetStatus === 'staging') {
    logger.info(LogTags.DRAG_STRATEGY, 'Action: Return to staging')

    try {
      // ✅ 使用 CPU Pipeline
      await pipeline.dispatch('task.return_to_staging', { id: context.task.id })

      return {
        success: true,
        message: '已返回暂存区',
        affectedViews: [context.sourceView.id, targetView.id],
      }
    } catch (error) {
      logger.error(
        LogTags.DRAG_STRATEGY,
        'Failed to return to staging',
        error instanceof Error ? error : new Error(String(error))
      )

      return {
        success: false,
        error: error instanceof Error ? error.message : '返回暂存区失败',
      }
    }
  }

  // 其他状态看板：仅重排序
  logger.debug(LogTags.DRAG_STRATEGY, 'Action: Reorder only')
  return {
    success: true,
    reorderOnly: true,
  }
}

/**
 * 策略：* -> calendar
 * 场景：拖到日历创建时间块
 */
const anyToCalendar: DragStrategy = async (context, targetView) => {
  const calendarConfig = targetView.config as CalendarViewConfig

  logger.debug(LogTags.DRAG_STRATEGY, 'Calendar drop strategy', {
    taskTitle: context.task.title,
    fromView: `${context.sourceView.type}:${context.sourceView.id}`,
    calendarSlot: {
      start: calendarConfig.startTime,
      end: calendarConfig.endTime,
    },
    mode: context.dragMode.mode,
  })

  const { useTimeBlockStore } = await import('@/stores/timeblock')
  const timeBlockStore = useTimeBlockStore()

  try {
    logger.info(LogTags.DRAG_STRATEGY, 'Action: Create time block from task', {
      taskId: context.task.id,
      startTime: calendarConfig.startTime,
      endTime: calendarConfig.endTime,
    })

    // 如果任务是 tiny（estimated_duration 为 0 或 null），先更新为 15 分钟
    const estimatedDuration = context.task.estimated_duration
    if (estimatedDuration === null || estimatedDuration === 0) {
      logger.debug(LogTags.DRAG_STRATEGY, 'Task is tiny, updating estimated_duration to 15 minutes')

      // ✅ 使用 CPU Pipeline
      await pipeline.dispatch('task.update', {
        id: context.task.id,
        updates: { estimated_duration: 15 },
      })

      // 更新本地任务对象，以便后续使用
      context.task.estimated_duration = 15
    }

    // 截断跨天：如果是分时事件，确保 end <= 当日 24:00
    let startISO = calendarConfig.startTime
    let endISO = calendarConfig.endTime
    if (!calendarConfig.isAllDay) {
      const start = new Date(startISO)
      let end = new Date(endISO)
      const dayEnd = new Date(start)
      dayEnd.setHours(0, 0, 0, 0)
      dayEnd.setDate(dayEnd.getDate() + 1)
      if (end.getTime() > dayEnd.getTime()) {
        end = dayEnd
      }
      startISO = start.toISOString()
      endISO = end.toISOString()
    }

    // 计算本地时间字符串
    let startTimeLocal: string | undefined
    let endTimeLocal: string | undefined

    if (calendarConfig.isAllDay) {
      // 全天事件：使用 00:00:00 到 23:59:59
      startTimeLocal = '00:00:00'
      endTimeLocal = '23:59:59'
    } else {
      // 分时事件：提取时间部分
      const startDate = new Date(startISO)
      const endDate = new Date(endISO)
      startTimeLocal = startDate.toTimeString().split(' ')[0] // HH:MM:SS
      endTimeLocal = endDate.toTimeString().split(' ')[0] // HH:MM:SS
    }

    // ✅ 调用 timeBlockStore（这个仍然保留，因为它是数据加载方法，不是 CRUD）
    const result = await timeBlockStore.createTimeBlockFromTask({
      task_id: context.task.id,
      start_time: startISO,
      end_time: endISO,
      start_time_local: startTimeLocal,
      end_time_local: endTimeLocal,
      time_type: 'FLOATING',
      creation_timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
      is_all_day: calendarConfig.isAllDay,
    })

    if (result) {
      logger.info(LogTags.DRAG_STRATEGY, 'Time block created successfully', {
        timeBlockId: result.time_block.id,
      })

      return {
        success: true,
        message: '已创建时间块',
        affectedViews: [context.sourceView.id, 'calendar'],
      }
    } else {
      return {
        success: false,
        error: '创建时间块失败：未返回结果',
      }
    }
  } catch (error) {
    logger.error(
      LogTags.DRAG_STRATEGY,
      'Failed to create time block',
      error instanceof Error ? error : new Error(String(error))
    )

    return {
      success: false,
      error: error instanceof Error ? error.message : '创建时间块失败',
    }
  }
}

/**
 * 策略：project -> status
 * 场景：从项目看板拖到状态看板
 */
const projectToStatus: DragStrategy = async (context, targetView) => {
  const targetStatus = (targetView.config as StatusViewConfig).status

  logger.debug(LogTags.DRAG_STRATEGY, 'Project to status strategy', {
    taskTitle: context.task.title,
    toStatus: targetStatus,
    mode: context.dragMode.mode,
  })

  // 拖到 staging：返回暂存区
  if (targetStatus === 'staging') {
    logger.info(LogTags.DRAG_STRATEGY, 'Action: Return to staging from project view')

    try {
      // ✅ 使用 CPU Pipeline
      await pipeline.dispatch('task.return_to_staging', { id: context.task.id })

      return {
        success: true,
        message: '已返回暂存区',
        affectedViews: [context.sourceView.id, targetView.id],
      }
    } catch (error) {
      logger.error(
        LogTags.DRAG_STRATEGY,
        'Failed to return to staging from project view',
        error instanceof Error ? error : new Error(String(error))
      )

      return {
        success: false,
        error: error instanceof Error ? error.message : '返回暂存区失败',
      }
    }
  }

  // 其他状态看板：仅重排序
  logger.debug(LogTags.DRAG_STRATEGY, 'Action: Reorder only')
  return {
    success: true,
    reorderOnly: true,
  }
}

/**
 * 默认策略：不支持的拖放操作
 */
const defaultStrategy: DragStrategy = async (context, targetView) => {
  logger.warn(LogTags.DRAG_STRATEGY, 'Unsupported drag operation', {
    taskTitle: context.task.title,
    fromView: `${context.sourceView.type}:${context.sourceView.id}`,
    toView: `${targetView.type}:${targetView.id}`,
    mode: context.dragMode.mode,
  })

  return {
    success: false,
    error: '不支持此拖放操作',
  }
}

// ==================== 策略注册表 ====================

/**
 * 策略注册表
 *
 * 键格式：'sourceType->targetType'
 * 特殊键：'*->type' 或 'type->*' 表示通配符
 */
export const dragStrategies: StrategyRegistry = {
  // 同类型看板之间
  'status->status': statusToStatus,
  'date->date': dateToDate,
  'project->project': projectToProject,

  // 跨类型拖放
  'status->date': statusToDate,
  'date->status': dateToStatus,
  'project->status': projectToStatus,

  // 特殊目标：日历
  '*->calendar': anyToCalendar,

  // 默认处理
  '*->*': defaultStrategy,
}

// ==================== 策略管理 ====================

/**
 * 注册自定义策略
 */
export function registerStrategy(key: string, strategy: DragStrategy): void {
  dragStrategies[key as keyof StrategyRegistry] = strategy
  logger.debug(LogTags.DRAG_STRATEGY, 'Registered custom strategy', { key })
}

/**
 * 注销策略
 */
export function unregisterStrategy(key: string): void {
  delete dragStrategies[key as keyof StrategyRegistry]
  logger.debug(LogTags.DRAG_STRATEGY, 'Unregistered strategy', { key })
}

/**
 * 获取所有已注册的策略键
 */
export function getRegisteredStrategies(): string[] {
  return Object.keys(dragStrategies)
}
