/**
 * useCrossViewDrag/strategies - 拖放策略注册表
 *
 * 定义和管理所有拖放策略
 * 🚧 当前阶段：仅打印控制台日志，不执行实际业务逻辑
 */

import type {
  DragStrategy,
  StrategyRegistry,
  StatusViewConfig,
  DateViewConfig,
  ProjectViewConfig,
  CalendarViewConfig,
} from '@/types/drag'

// ==================== 策略实现 ====================

/**
 * 策略：status -> status
 * 场景：在状态看板之间拖动
 */
const statusToStatus: DragStrategy = async (context, targetView) => {
  const sourceConfig = context.sourceView.config as StatusViewConfig
  const targetConfig = targetView.config as StatusViewConfig

  console.log('[Strategy] 📊 status -> status', {
    task: context.task.title,
    from: sourceConfig.status,
    to: targetConfig.status,
    mode: context.dragMode.mode,
  })

  // 特殊情况：staging -> planned
  if (sourceConfig.status === 'staging' && targetConfig.status === 'planned') {
    console.log('  ➡️ Action: Set scheduled_date to today')
    return {
      success: true,
      message: '已设置排期',
      affectedViews: [context.sourceView.id, targetView.id],
    }
  }

  // 默认：仅重排序
  console.log('  ➡️ Action: Reorder only')
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

  console.log('[Strategy] 📅 date -> date', {
    task: context.task.title,
    from: sourceDate,
    to: targetDate,
    mode: context.dragMode.mode,
  })

  // 导入 taskStore
  const { useTaskStore } = await import('@/stores/task')
  const taskStore = useTaskStore()

  try {
    // 检查目标日期是否已有安排
    const hasTargetSchedule = context.task.schedules?.some((s) => s.scheduled_day === targetDate)

    if (hasTargetSchedule) {
      // 目标天已有安排，删除源日期的安排即可
      console.log(
        `  ➡️ Action: Target date already has schedule, deleting source date ${sourceDate}`
      )

      await taskStore.deleteSchedule(context.task.id, sourceDate)

      return {
        success: true,
        message: `任务在 ${targetDate} 已有安排，已删除 ${sourceDate} 的安排`,
        affectedViews: [context.sourceView.id, targetView.id],
      }
    } else {
      // 目标天没有安排，更新日期
      console.log(`  ➡️ Action: Update scheduled_date from ${sourceDate} to ${targetDate}`)

      const updatedTask = await taskStore.updateSchedule(context.task.id, sourceDate, {
        new_date: targetDate,
      })

      if (!updatedTask) {
        return {
          success: false,
          error: '更新日程失败',
        }
      }

      return {
        success: true,
        message: `已改期至 ${targetDate}`,
        affectedViews: [context.sourceView.id, targetView.id],
        updatedTask,
      }
    }
  } catch (error) {
    console.error('  ❌ Failed to update date schedule:', error)

    let errorMessage = '改期失败'
    if (error instanceof Error) {
      errorMessage = error.message
    } else if (typeof error === 'string') {
      errorMessage = error
    }

    return {
      success: false,
      error: errorMessage,
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

  console.log('[Strategy] 📁 project -> project', {
    task: context.task.title,
    from: sourceConfig.projectName,
    to: targetConfig.projectName,
    mode: context.dragMode.mode,
  })

  // 检查权限：已完成的任务不能移动项目
  if (context.task.is_completed) {
    console.log('  ❌ Blocked: Completed tasks cannot change projects')
    return {
      success: false,
      error: '已完成的任务不能移动到其他项目',
    }
  }

  console.log(
    `  ➡️ Action: Change project from ${sourceConfig.projectId} to ${targetConfig.projectId}`
  )

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

  console.log('[Strategy] 📊➡️📅 status -> date', {
    task: context.task.title,
    from: sourceStatus,
    to: targetDate,
    mode: context.dragMode.mode,
  })

  // 导入 taskStore
  const { useTaskStore } = await import('@/stores/task')
  const taskStore = useTaskStore()

  try {
    // 特殊处理：从 staging 拖到日期看板，新建安排
    if (sourceStatus === 'staging') {
      console.log(`  ➡️ Action: Add schedule for ${targetDate}`)

      const updatedTask = await taskStore.addSchedule(context.task.id, targetDate)

      if (!updatedTask) {
        return {
          success: false,
          error: '添加日程失败',
        }
      }

      return {
        success: true,
        message: `已添加排期：${targetDate}`,
        affectedViews: [context.sourceView.id, targetView.id],
        updatedTask,
      }
    }

    // 其他状态看板：仅提示（保留原有逻辑）
    console.log(`  ➡️ Action: Set scheduled_date to ${targetDate}`)

    return {
      success: true,
      message: `已设置排期：${targetDate}`,
      affectedViews: [context.sourceView.id, targetView.id],
    }
  } catch (error) {
    console.error('  ❌ Failed to add schedule:', error)

    let errorMessage = '设置排期失败'
    if (error instanceof Error) {
      errorMessage = error.message
    } else if (typeof error === 'string') {
      errorMessage = error
    }

    return {
      success: false,
      error: errorMessage,
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

  console.log('[Strategy] 📅➡️📊 date -> status', {
    task: context.task.title,
    from: sourceDate,
    to: targetStatus,
    mode: context.dragMode.mode,
  })

  // 拖回 staging：取消排期
  if (targetStatus === 'staging') {
    console.log('  ➡️ Action: Clear scheduled_date (return to staging)')
    return {
      success: true,
      message: '已取消排期',
      affectedViews: [context.sourceView.id, targetView.id],
    }
  }

  // 其他状态看板：仅重排序
  console.log('  ➡️ Action: Reorder only')
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

  // 🔍 检查点5：策略入口
  console.log('[CHK-5] ✅ anyToCalendar strategy invoked!')

  console.log('[Strategy] 🗓️ * -> calendar', {
    task: context.task.title,
    from: `${context.sourceView.type}:${context.sourceView.id}`,
    calendarSlot: {
      start: calendarConfig.startTime,
      end: calendarConfig.endTime,
    },
    mode: context.dragMode.mode,
  })

  // ✅ 实际调用 timeBlockStore（需要在策略外部注入）
  // 这里先导入必要的模块
  const { useTimeBlockStore } = await import('@/stores/timeblock')
  const { useTaskStore } = await import('@/stores/task')

  const timeBlockStore = useTimeBlockStore()
  const taskStore = useTaskStore()

  try {
    console.log('  ➡️ Action: Create time block from task')
    console.log('    - task_id:', context.task.id)
    console.log('    - start_time:', calendarConfig.startTime)
    console.log('    - end_time:', calendarConfig.endTime)

    // 如果任务是 tiny（estimated_duration 为 0 或 null），先更新为 15 分钟
    const estimatedDuration = context.task.estimated_duration
    if (estimatedDuration === null || estimatedDuration === 0) {
      console.log('  ⏱️ Task is tiny, updating estimated_duration to 15 minutes')
      await taskStore.updateTask(context.task.id, { estimated_duration: 15 } as any)
      // 更新本地任务对象，以便后续使用
      context.task.estimated_duration = 15
    }

    // 🔍 检查点5：即将调用 timeBlockStore
    console.log('[CHK-5] About to call timeBlockStore.createTimeBlockFromTask')

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

    const result = await timeBlockStore.createTimeBlockFromTask({
      task_id: context.task.id,
      start_time: startISO,
      end_time: endISO,
      is_all_day: calendarConfig.isAllDay, // ✅ 传递全天事件标记
    })

    // 🔍 检查点5：timeBlockStore 返回结果
    console.log('[CHK-5] timeBlockStore.createTimeBlockFromTask result=', result)

    if (result) {
      console.log('  ✅ Time block created:', result.time_block.id)

      // 更新任务到 store
      taskStore.addOrUpdateTask(result.updated_task)

      return {
        success: true,
        message: '已创建时间块',
        affectedViews: [context.sourceView.id, 'calendar'],
        updatedTask: result.updated_task,
      }
    } else {
      console.log('[CHK-5] ❌ No result returned from timeBlockStore')
      return {
        success: false,
        error: '创建时间块失败：未返回结果',
      }
    }
  } catch (error) {
    console.error('  ❌ Failed to create time block:', error)
    console.error('[CHK-5] ❌ Exception:', error)

    let errorMessage = '创建时间块失败'
    if (error instanceof Error) {
      errorMessage = error.message
    } else if (typeof error === 'string') {
      errorMessage = error
    }

    return {
      success: false,
      error: errorMessage,
    }
  }
}

/**
 * 默认策略：不支持的拖放操作
 */
const defaultStrategy: DragStrategy = async (context, targetView) => {
  console.log('[Strategy] ❌ Unsupported operation', {
    task: context.task.title,
    from: `${context.sourceView.type}:${context.sourceView.id}`,
    to: `${targetView.type}:${targetView.id}`,
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

  // 特殊目标：日历
  '*->calendar': anyToCalendar,

  // 默认处理
  '*->*': defaultStrategy,
}

// ==================== 策略管理 ====================

/**
 * 注册自定义策略
 * @param key - 策略键
 * @param strategy - 策略函数
 */
export function registerStrategy(key: string, strategy: DragStrategy): void {
  dragStrategies[key as keyof StrategyRegistry] = strategy

  console.log('[Strategies] ✅ Registered custom strategy:', key)
}

/**
 * 注销策略
 * @param key - 策略键
 */
export function unregisterStrategy(key: string): void {
  delete dragStrategies[key as keyof StrategyRegistry]

  console.log('[Strategies] ❌ Unregistered strategy:', key)
}

/**
 * 获取所有已注册的策略键
 */
export function getRegisteredStrategies(): string[] {
  return Object.keys(dragStrategies)
}
