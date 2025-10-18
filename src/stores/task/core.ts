import { ref, computed } from 'vue'
import type { TaskCard, TaskDetail } from '@/types/dtos'
import { updateMapItem, removeMapItem, createLoadingState } from '@/stores/shared'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * Task Store 核心状态管理
 *
 * 职责：
 * - 管理任务数据的单一数据源
 * - 提供基础的状态操作方法
 * - 提供计算属性和过滤器
 */

/**
 * 创建任务核心状态
 */
export function createTaskCore() {
  // ============================================================
  // STATE - 只存储最原始、最规范化的数据
  // ============================================================

  /**
   * 任务映射表 (单一数据源)
   * key: task_id
   * value: TaskCard | TaskDetail (总是保存当前最完整的信息)
   *
   * 说明：TaskDetail extends TaskCard，所以可以安全地存储两种类型
   * 当获取详情时，会用 TaskDetail 覆盖原有的 TaskCard
   */
  const tasks = ref(new Map<string, TaskCard | TaskDetail>())

  /**
   * 加载状态管理
   */
  const { isLoading, error, withLoading } = createLoadingState()

  // ============================================================
  // GETTERS - 动态过滤（所有视图的数据源）
  // ============================================================

  /**
   * 基础数组缓存层（性能优化）
   * ✅ 只转换一次 Map → Array，所有其他 getter 复用此数组
   */
  const allTasksArray = computed(() => {
    return Array.from(tasks.value.values())
  })

  /**
   * 获取所有任务（数组形式）
   */
  const allTasks = computed(() => {
    return allTasksArray.value
  })

  /**
   * Staging 任务（未安排且未完成）
   * ✅ 动态过滤：任务完成后自动消失
   * ✅ 性能优化：复用 allTasksArray
   * ✅ 过滤规则：
   *    - schedule_status === 'staging'（后端计算的状态）
   *    - !is_completed（未完成）
   *    - !is_archived（未归档）
   *    - !is_deleted（未删除）
   *    - 无当前或未来日程（防御性检查，避免前后端状态不同步）
   */
  const stagingTasks = computed(() => {
    const today = new Date().toISOString().split('T')[0]

    return allTasksArray.value.filter((task) => {
      // 基础状态检查
      if (
        task.schedule_status !== 'staging' ||
        task.is_completed ||
        task.is_archived ||
        task.is_deleted
      ) {
        return false
      }

      // 🔥 防御性检查：确保没有当前或未来的日程
      // 即使 schedule_status 是 'staging'，也要确保没有 >= today 的 schedule
      const hasFutureOrTodaySchedule =
        task.schedules?.some((schedule) => schedule.scheduled_day >= today) ?? false

      return !hasFutureOrTodaySchedule
    })
  })

  /**
   * Planned 任务（已安排且未完成）
   * ✅ 动态过滤：任务完成后自动消失
   * ✅ 性能优化：复用 allTasksArray
   * ✅ 排除已删除的任务：删除后立即消失
   */
  const plannedTasks = computed(() => {
    return allTasksArray.value.filter(
      (task) => task.schedule_status === 'scheduled' && !task.is_completed && !task.is_deleted
    )
  })

  /**
   * 未完成的任务（所有状态）
   * ✅ 动态过滤：任务完成后自动消失
   * ✅ 性能优化：复用 allTasksArray
   * ✅ 排除已删除的任务：删除后立即消失
   */
  const incompleteTasks = computed(() => {
    return allTasksArray.value.filter((task) => !task.is_completed && !task.is_deleted)
  })

  /**
   * 已完成的任务
   * ✅ 性能优化：复用 allTasksArray
   * ✅ 排除已删除的任务：删除后立即消失
   */
  const completedTasks = computed(() => {
    return allTasksArray.value.filter((task) => task.is_completed && !task.is_deleted)
  })

  /**
   * 已归档的任务
   * ✅ 性能优化：复用 allTasksArray
   * ✅ 排除已删除的任务：归档中删除后立即消失
   */
  const archivedTasks = computed(() => {
    return allTasksArray.value.filter((task) => task.is_archived && !task.is_deleted)
  })

  /**
   * 已安排的任务（包括已完成和未完成）
   * ✅ 排除已删除的任务：删除后立即消失
   * @deprecated 使用 plannedTasks（只含未完成）
   */
  const scheduledTasks = computed(() => {
    return allTasksArray.value.filter((task) => task.schedule_status === 'scheduled' && !task.is_deleted)
  })

  /**
   * Mux: 根据 ID 获取任务（多路复用器）
   * 纯函数，不调用 API
   */
  function getTaskById_Mux(id: string): TaskCard | TaskDetail | undefined {
    return tasks.value.get(id)
  }

  /**
   * Mux: 获取指定日期的任务列表（多路复用器）
   * ✅ 单一数据源：从 TaskStore 过滤，自动响应变化
   * ✅ 性能优化：复用 allTasksArray
   * ✅ 过滤归档和已删除任务：这些任务不会出现在日期看板
   * ✅ 纯函数，不调用 API
   */
  const getTasksByDate_Mux = computed(() => (date: string) => {
    const result = allTasksArray.value.filter((task) => {
      // 🔍 调试：打印每个任务的 schedules 信息
      // if (task.schedules && task.schedules.length > 0) {
      //   console.log('[getTasksByDate] Task:', task.id, 'schedules:', task.schedules)
      // }

      // 排除归档和已删除的任务
      if (task.is_archived || task.is_deleted) {
        return false
      }

      // 检查任务是否有该日期的 schedule
      const hasSchedule = task.schedules?.some((schedule) => schedule.scheduled_day === date)

      // if (hasSchedule) {
      //   console.log(`[getTasksByDate] ✅ Task ${task.id} matches date ${date}`)
      // }

      return hasSchedule ?? false
    })

    // console.log(
    //   `[getTasksByDate] Date: ${date}, Total tasks: ${allTasksArray.value.length}, Matched: ${result.length}`
    // )
    return result
  })

  /**
   * Mux: 根据项目 ID 获取任务列表（多路复用器）
   * ✅ 性能优化：复用 allTasksArray
   * ✅ 排除已删除的任务：删除后立即消失
   * ✅ 纯函数，不调用 API
   */
  const getTasksByProject_Mux = computed(() => {
    return (projectId: string) => {
      return allTasksArray.value.filter((task) => task.project_id === projectId && !task.is_deleted)
    }
  })

  /**
   * Mux: 根据区域 ID 获取任务列表（多路复用器）
   * ✅ 性能优化：复用 allTasksArray
   * ✅ 排除已删除的任务：删除后立即消失
   * ✅ 纯函数，不调用 API
   */
  const getTasksByArea_Mux = computed(() => {
    return (areaId: string) => {
      return allTasksArray.value.filter((task) => task.area_id === areaId && !task.is_deleted)
    }
  })

  /**
   * Mux: 根据 viewkey 获取任务列表（多路复用器）
   * ✅ 性能优化：复用 allTasksArray
   * ✅ 支持多种 viewkey 格式：
   *     - misc::staging::${areaId} → 该 area 的 staging 任务
   *     - misc::staging → 全部 staging 任务
   *     - misc::archive → 归档任务
   *     - daily::${date} → 指定日期任务
   * ✅ 纯函数，不调用 API
   */
  const getTasksByViewKey_Mux = computed(() => {
    return (viewKey: string) => {
      const parts = viewKey.split('::')
      const [type, subtype, identifier] = parts

      logger.debug(LogTags.STORE_TASKS, 'getTasksByViewKey_Mux called', {
        viewKey,
        parts,
        totalTasks: allTasksArray.value.length
      })

      switch (type) {
        case 'misc':
          if (subtype === 'staging') {
            if (identifier) {
              // misc::staging::${areaId} - 指定 area 的 staging 任务
              const filteredTasks = allTasksArray.value.filter((task) => {
                const match = (
                  task.area_id === identifier &&
                  task.schedule_status === 'staging' &&
                  !task.is_completed &&
                  !task.is_archived &&
                  !task.is_deleted
                )
                if (task.area_id === identifier) {
                  logger.debug(LogTags.STORE_TASKS, 'Task area match check', {
                    taskId: task.id,
                    taskTitle: task.title,
                    taskAreaId: task.area_id,
                    targetAreaId: identifier,
                    scheduleStatus: task.schedule_status,
                    isCompleted: task.is_completed,
                    isArchived: task.is_archived,
                    isDeleted: task.is_deleted,
                    finalMatch: match
                  })
                }
                return match
              })

              logger.info(LogTags.STORE_TASKS, 'Area staging filter result', {
                viewKey,
                areaId: identifier,
                totalTasks: allTasksArray.value.length,
                filteredCount: filteredTasks.length,
                filteredTaskIds: filteredTasks.map(t => t.id)
              })

              return filteredTasks
            } else {
              // misc::staging - 全部 staging 任务
              logger.debug(LogTags.STORE_TASKS, 'Using global staging tasks', {
                viewKey,
                count: stagingTasks.value.length
              })
              return stagingTasks.value
            }
          } else if (subtype === 'archive') {
            // misc::archive - 归档任务
            logger.debug(LogTags.STORE_TASKS, 'Using archived tasks', {
              viewKey,
              count: archivedTasks.value.length
            })
            return archivedTasks.value
          }
          break

        case 'daily':
          if (subtype && identifier === undefined) {
            // daily::${date} - 指定日期任务
            const date = subtype
            const tasks = getTasksByDate_Mux.value(date)
            logger.debug(LogTags.STORE_TASKS, 'Using daily tasks', {
              viewKey,
              date,
              count: tasks.length
            })
            return tasks
          }
          break

        case 'area':
          if (subtype) {
            // area::${areaId} - 指定 area 的所有任务
            const areaId = subtype
            const tasks = getTasksByArea_Mux.value(areaId)
            logger.debug(LogTags.STORE_TASKS, 'Using area tasks', {
              viewKey,
              areaId,
              count: tasks.length
            })
            return tasks
          }
          break

        default:
          logger.warn(LogTags.STORE_TASKS, 'Unknown viewKey format', { viewKey })
          return []
      }

      logger.warn(LogTags.STORE_TASKS, 'No matching viewKey handler', { viewKey, parts })
      return []
    }
  })

  // ============================================================
  // ACTIONS - 基础状态操作
  // ============================================================

  /**
   * 批量添加或更新任务（单一数据源）
   * 使用 shared 工具进行状态更新
   */
  function addOrUpdateTasks(newTasks: (TaskCard | TaskDetail)[]) {
    for (const task of newTasks) {
      if (!task || !task.id) {
        logger.warn(LogTags.STORE_TASKS, 'Skipping task without ID', { task })
        continue
      }

      // 正确的做法：直接用服务器返回的权威数据进行设置
      // tasks.value 是一个响应式 Map，调用 .set() 会被 Vue 侦测到
      // Vue 会自动将新设置的 task 对象转换为响应式代理
      updateMapItem(tasks, task.id, task)
    }
  }

  /**
   * 添加或更新单个任务
   */
  function addOrUpdateTask(task: TaskCard | TaskDetail) {
    addOrUpdateTasks([task])
  }

  /**
   * 从 state 中移除任务
   */
  function removeTask(id: string) {
    removeMapItem(tasks, id)
  }

  /**
   * 替换指定日期的所有任务（用于刷新场景）
   * 先删除该日期的所有旧任务，再添加新任务
   * @param date 日期字符串 (YYYY-MM-DD)
   * @param newTasks 新的任务列表
   */
  function replaceTasksForDate(date: string, newTasks: (TaskCard | TaskDetail)[]) {
    // 1. 找出该日期的所有旧任务ID
    const oldTaskIds = allTasksArray.value
      .filter((task) => {
        // 检查任务是否属于该日期
        return task.schedules?.some((schedule) => schedule.scheduled_day === date)
      })
      .map((task) => task.id)

    logger.debug(LogTags.STORE_TASKS, 'Replacing tasks for date', {
      date,
      oldTaskCount: oldTaskIds.length,
      newTaskCount: newTasks.length,
      oldTaskIds,
      newTaskIds: newTasks.map((t) => t.id),
    })

    // 2. 创建新的 Map，先删除旧任务
    const newMap = new Map(tasks.value)
    for (const taskId of oldTaskIds) {
      newMap.delete(taskId)
    }

    // 3. 添加新任务
    for (const task of newTasks) {
      if (!task || !task.id) {
        logger.warn(LogTags.STORE_TASKS, 'Skipping task without ID during replace', { task })
        continue
      }
      newMap.set(task.id, task)
    }

    // 4. 更新响应式状态
    tasks.value = newMap

    logger.info(LogTags.STORE_TASKS, 'Successfully replaced tasks for date', {
      date,
      finalTaskCount: newMap.size,
    })
  }

  return {
    // State
    tasks,
    isLoading,
    error,
    withLoading,

    // Getters (导线 - Wires)
    allTasks,
    allTasksArray,
    stagingTasks,
    plannedTasks,
    incompleteTasks,
    completedTasks,
    archivedTasks,
    scheduledTasks,

    // Getters (多路复用器 - Mux)
    getTaskById_Mux,
    getTasksByDate_Mux,
    getTasksByProject_Mux,
    getTasksByArea_Mux,
    getTasksByViewKey_Mux,

    // Actions
    addOrUpdateTasks,
    addOrUpdateTask,
    removeTask,
    replaceTasksForDate,
  }
}
