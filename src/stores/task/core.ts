import { ref, computed } from 'vue'
import type { TaskCard, TaskDetail } from '@/types/dtos'
import { updateMapItem, removeMapItem, createLoadingState } from '@/stores/shared'
import { logger, LogTags } from '@/infra/logging/logger'
import { getTodayDateString } from '@/infra/utils/dateUtils'

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
  // UTILITY FUNCTIONS - 工具函数
  // ============================================================

  /**
   * 对循环任务进行去重，每个循环规则只保留最近的未完成任务
   * @param tasks - 待处理的任务列表
   * @returns 去重后的任务列表
   */
  function deduplicateRecurringTasks(tasks: TaskCard[]): TaskCard[] {
    // 按 recurrence_id 分组
    const recurrenceGroups = new Map<string, TaskCard[]>()
    const nonRecurringTasks: TaskCard[] = []

    for (const task of tasks) {
      if (task.recurrence_id) {
        const group = recurrenceGroups.get(task.recurrence_id) || []
        group.push(task)
        recurrenceGroups.set(task.recurrence_id, group)
      } else {
        nonRecurringTasks.push(task)
      }
    }

    // 对每个循环规则，只保留最近的未完成任务
    const filteredRecurringTasks: TaskCard[] = []
    for (const [_recurrenceId, groupTasks] of recurrenceGroups) {
      // 过滤出未完成的任务
      const incompleteTasks = groupTasks.filter((t) => !t.is_completed)

      if (incompleteTasks.length > 0) {
        // 按 recurrence_original_date 降序排序，取最新的一个
        incompleteTasks.sort((a, b) => {
          const dateA = a.recurrence_original_date || ''
          const dateB = b.recurrence_original_date || ''
          return dateB.localeCompare(dateA)
        })
        filteredRecurringTasks.push(incompleteTasks[0]!)
      }
    }

    return [...nonRecurringTasks, ...filteredRecurringTasks]
  }

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
   * ✅ 实时计算：根据 schedules 数组判断，不依赖 schedule_status
   * ✅ 过滤规则：
   *    - !is_completed（未完成）
   *    - !is_archived（未归档）
   *    - !is_deleted（未删除）
   *    - 无当前或未来日程（实时计算）
   *    - 排除 EXPIRE 类型且已过期的循环任务
   */
  const stagingTasks = computed(() => {
    // ⚠️ 使用 getTodayDateString() 获取本地日期，符合 TIME_CONVENTION.md
    const today = getTodayDateString()

    return allTasksArray.value.filter((task) => {
      // 基础状态检查
      if (task.is_completed || task.is_archived || task.is_deleted) {
        return false
      }

      // 🔥 实时计算：没有当前或未来的日程 = staging
      const hasFutureOrTodaySchedule =
        task.schedules?.some((schedule) => schedule.scheduled_day >= today) ?? false

      if (hasFutureOrTodaySchedule) {
        return false
      }

      // 🔥 排除 EXPIRE 类型且已过期的循环任务
      if (
        task.recurrence_id &&
        task.recurrence_original_date &&
        task.recurrence_expiry_behavior === 'EXPIRE'
      ) {
        // 判断是否过期：原始日期 < 今天
        if (task.recurrence_original_date < today) {
          return false
        }
      }

      return true
    })
  })

  /**
   * Planned 任务（已安排且未完成）
   * ✅ 动态过滤：任务完成后自动消失
   * ✅ 性能优化：复用 allTasksArray
   * ✅ 排除已删除的任务：删除后立即消失
   * ✅ 实时计算：根据 schedules 数组判断，不依赖 schedule_status
   */
  const plannedTasks = computed(() => {
    // ⚠️ 使用 getTodayDateString() 获取本地日期，符合 TIME_CONVENTION.md
    const today = getTodayDateString()

    return allTasksArray.value.filter((task) => {
      // 基础过滤：未完成 + 未删除
      if (task.is_completed || task.is_deleted) {
        return false
      }

      // 🔥 实时计算：有当前或未来的日程 = scheduled
      const hasFutureOrTodaySchedule =
        task.schedules?.some((schedule) => schedule.scheduled_day >= today) ?? false

      return hasFutureOrTodaySchedule
    })
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
   * 截止日期任务（有截止日期的未完成任务，按截止日期排序）
   * ✅ 动态过滤：任务完成后自动消失
   * ✅ 性能优化：复用 allTasksArray
   * ✅ 排除已删除和已归档的任务
   * ✅ 循环任务去重：每个循环规则只保留最近的未完成任务
   * ✅ 按截止日期排序（最近的在前）
   *
   * 对应 viewKey: misc::deadline
   */
  const deadlineTasks = computed(() => {
    const tasksWithDueDate = allTasksArray.value.filter(
      (task) => task.due_date && !task.is_archived && !task.is_completed && !task.is_deleted
    )

    // 对循环任务去重：每个循环规则只保留最近的未完成任务
    const deduplicated = deduplicateRecurringTasks(tasksWithDueDate)

    // 按截止日期排序（最近的在前）
    return deduplicated.sort((a, b) => {
      const dateA = new Date(a.due_date!.date).getTime()
      const dateB = new Date(b.due_date!.date).getTime()
      return dateA - dateB
    })
  })

  /**
   * 已安排的任务（包括已完成和未完成）
   * ✅ 排除已删除的任务：删除后立即消失
   * ✅ 实时计算：根据 schedules 数组判断，不依赖 schedule_status
   * @deprecated 使用 plannedTasks（只含未完成）
   */
  const scheduledTasks = computed(() => {
    // ⚠️ 使用 getTodayDateString() 获取本地日期，符合 TIME_CONVENTION.md
    const today = getTodayDateString()

    return allTasksArray.value.filter((task) => {
      if (task.is_deleted) return false

      // 🔥 实时计算：有当前或未来的日程 = scheduled
      const hasFutureOrTodaySchedule =
        task.schedules?.some((schedule) => schedule.scheduled_day >= today) ?? false

      return hasFutureOrTodaySchedule
    })
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
   * ✅ 循环任务去重：每个循环规则只保留最近的未完成任务
   * ✅ 纯函数，不调用 API
   */
  const getTasksByProject_Mux = computed(() => {
    return (projectId: string) => {
      const projectTasks = allTasksArray.value.filter(
        (task) => task.project_id === projectId && !task.is_deleted
      )
      return deduplicateRecurringTasks(projectTasks)
    }
  })

  /**
   * Mux: 根据区域 ID 获取任务列表（多路复用器）
   * ✅ 性能优化：复用 allTasksArray
   * ✅ 排除已删除的任务：删除后立即消失
   * ✅ 循环任务去重：每个循环规则只保留最近的未完成任务
   * ✅ 纯函数，不调用 API
   */
  const getTasksByArea_Mux = computed(() => {
    return (areaId: string) => {
      const areaTasks = allTasksArray.value.filter(
        (task) => task.area_id === areaId && !task.is_deleted
      )
      return deduplicateRecurringTasks(areaTasks)
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
      const [type, subtype, identifier, extraIdentifier] = parts

      logger.debug(LogTags.STORE_TASKS, 'getTasksByViewKey_Mux called', {
        viewKey,
        parts,
        totalTasks: allTasksArray.value.length,
      })

      switch (type) {
        case 'misc':
          if (subtype === 'staging') {
            if (identifier === 'no-project') {
              // misc::staging::no-project - 无项目的 staging 任务（不管有没有区域）
              const today = getTodayDateString()
              const filteredTasks = allTasksArray.value.filter((task) => {
                // 基础检查：必须没有 project_id（不检查 area_id）
                if (
                  task.project_id ||
                  task.is_completed ||
                  task.is_archived ||
                  task.is_deleted
                ) {
                  return false
                }

                // 🔥 实时计算：没有当前或未来的日程 = staging
                const hasFutureOrTodaySchedule =
                  task.schedules?.some((schedule) => schedule.scheduled_day >= today) ?? false
                if (hasFutureOrTodaySchedule) {
                  return false
                }

                // 🔥 排除 EXPIRE 类型且已过期的循环任务
                if (
                  task.recurrence_id &&
                  task.recurrence_original_date &&
                  task.recurrence_expiry_behavior === 'EXPIRE'
                ) {
                  if (task.recurrence_original_date < today) {
                    return false
                  }
                }

                return true
              })

              logger.debug(LogTags.STORE_TASKS, 'No-project staging filter result', {
                viewKey,
                totalTasks: allTasksArray.value.length,
                filteredCount: filteredTasks.length,
              })

              return filteredTasks
            } else if (identifier === 'recent-carryover') {
              // misc::staging::recent-carryover - 最近结转的 staging 任务
              // 定义：过去5天内有排期，但目前属于 staging 的任务
              const today = getTodayDateString()
              const todayDate = new Date(today)
              const fiveDaysAgo = new Date(todayDate)
              fiveDaysAgo.setDate(todayDate.getDate() - 5)
              const fiveDaysAgoStr = fiveDaysAgo.toISOString().split('T')[0]!

              const filteredTasks = allTasksArray.value.filter((task) => {
                // 基础状态检查
                if (task.is_completed || task.is_archived || task.is_deleted) {
                  return false
                }

                // 🔥 必须是 staging 状态（没有当前或未来的日程）
                const hasFutureOrTodaySchedule =
                  task.schedules?.some((schedule) => schedule.scheduled_day >= today) ?? false
                if (hasFutureOrTodaySchedule) {
                  return false
                }

                // 🔥 排除 EXPIRE 类型且已过期的循环任务
                if (
                  task.recurrence_id &&
                  task.recurrence_original_date &&
                  task.recurrence_expiry_behavior === 'EXPIRE'
                ) {
                  if (task.recurrence_original_date < today) {
                    return false
                  }
                }

                // 🔥 必须在过去5天内有排期（不含今天）
                const hasRecentPastSchedule =
                  task.schedules?.some((schedule) => {
                    const scheduleDay = schedule.scheduled_day
                    return scheduleDay >= fiveDaysAgoStr && scheduleDay < today
                  }) ?? false

                return hasRecentPastSchedule
              })

              logger.debug(LogTags.STORE_TASKS, 'Recent carryover staging filter result', {
                viewKey,
                today,
                fiveDaysAgoStr,
                totalTasks: allTasksArray.value.length,
                filteredCount: filteredTasks.length,
              })

              return filteredTasks
            } else if (identifier === 'project' && extraIdentifier) {
              // misc::staging::project::{projectId} - 无区域的指定项目的 staging 任务
              const projectId = extraIdentifier
              const today = getTodayDateString()
              const filteredTasks = allTasksArray.value.filter((task) => {
                // 基础检查：必须属于指定项目且没有 area_id
                if (
                  task.project_id !== projectId ||
                  task.area_id ||
                  task.is_completed ||
                  task.is_archived ||
                  task.is_deleted
                ) {
                  return false
                }

                // 🔥 实时计算：没有当前或未来的日程 = staging
                const hasFutureOrTodaySchedule =
                  task.schedules?.some((schedule) => schedule.scheduled_day >= today) ?? false
                if (hasFutureOrTodaySchedule) {
                  return false
                }

                // 🔥 排除 EXPIRE 类型且已过期的循环任务
                if (
                  task.recurrence_id &&
                  task.recurrence_original_date &&
                  task.recurrence_expiry_behavior === 'EXPIRE'
                ) {
                  if (task.recurrence_original_date < today) {
                    return false
                  }
                }

                return true
              })

              logger.debug(LogTags.STORE_TASKS, 'Project staging filter result', {
                viewKey,
                projectId,
                totalTasks: allTasksArray.value.length,
                filteredCount: filteredTasks.length,
              })

              return filteredTasks
            } else if (identifier === 'no-area') {
              // 检查是否是 misc::staging::no-area::no-project
              if (extraIdentifier === 'no-project') {
                // misc::staging::no-area::no-project - 无区域且无项目的 staging 任务
                const today = getTodayDateString()
                const filteredTasks = allTasksArray.value.filter((task) => {
                  // 基础检查：必须没有 area_id 且没有 project_id
                  if (
                    task.area_id ||
                    task.project_id ||
                    task.is_completed ||
                    task.is_archived ||
                    task.is_deleted
                  ) {
                    return false
                  }

                  // 🔥 实时计算：没有当前或未来的日程 = staging
                  const hasFutureOrTodaySchedule =
                    task.schedules?.some((schedule) => schedule.scheduled_day >= today) ?? false
                  if (hasFutureOrTodaySchedule) {
                    return false
                  }

                  // 🔥 排除 EXPIRE 类型且已过期的循环任务
                  if (
                    task.recurrence_id &&
                    task.recurrence_original_date &&
                    task.recurrence_expiry_behavior === 'EXPIRE'
                  ) {
                    if (task.recurrence_original_date < today) {
                      return false
                    }
                  }

                  return true
                })

                logger.debug(LogTags.STORE_TASKS, 'No-area-no-project staging filter result', {
                  viewKey,
                  totalTasks: allTasksArray.value.length,
                  filteredCount: filteredTasks.length,
                })

                return filteredTasks
              }

              // misc::staging::no-area - 无区域的 staging 任务（不管有没有项目）
              // ⚠️ 使用 getTodayDateString() 获取本地日期，符合 TIME_CONVENTION.md
              const today = getTodayDateString()
              const filteredTasks = allTasksArray.value.filter((task) => {
                // 基础检查：必须没有 area_id（不检查 project_id）
                if (task.area_id || task.is_completed || task.is_archived || task.is_deleted) {
                  return false
                }

                // 🔥 实时计算：没有当前或未来的日程 = staging
                const hasFutureOrTodaySchedule =
                  task.schedules?.some((schedule) => schedule.scheduled_day >= today) ?? false
                if (hasFutureOrTodaySchedule) {
                  return false
                }

                // 🔥 排除 EXPIRE 类型且已过期的循环任务
                if (
                  task.recurrence_id &&
                  task.recurrence_original_date &&
                  task.recurrence_expiry_behavior === 'EXPIRE'
                ) {
                  if (task.recurrence_original_date < today) {
                    return false
                  }
                }

                return true
              })

              logger.debug(LogTags.STORE_TASKS, 'No-area staging filter result', {
                viewKey,
                totalTasks: allTasksArray.value.length,
                filteredCount: filteredTasks.length,
              })

              return filteredTasks
            } else if (identifier) {
              // 检查是否有更多部分（区域+项目筛选）
              const fifthPart = parts[4]

              if (extraIdentifier === 'no-project') {
                // misc::staging::${areaId}::no-project - 指定区域的无项目 staging 任务
                const areaId = identifier
                const today = getTodayDateString()
                const filteredTasks = allTasksArray.value.filter((task) => {
                  // 基础检查：必须属于指定区域且没有项目
                  if (
                    task.area_id !== areaId ||
                    task.project_id ||
                    task.is_completed ||
                    task.is_archived ||
                    task.is_deleted
                  ) {
                    return false
                  }

                  // 🔥 实时计算：没有当前或未来的日程 = staging
                  const hasFutureOrTodaySchedule =
                    task.schedules?.some((schedule) => schedule.scheduled_day >= today) ?? false
                  if (hasFutureOrTodaySchedule) {
                    return false
                  }

                  // 🔥 排除 EXPIRE 类型且已过期的循环任务
                  if (
                    task.recurrence_id &&
                    task.recurrence_original_date &&
                    task.recurrence_expiry_behavior === 'EXPIRE'
                  ) {
                    if (task.recurrence_original_date < today) {
                      return false
                    }
                  }

                  return true
                })

                logger.debug(LogTags.STORE_TASKS, 'Area no-project staging filter result', {
                  viewKey,
                  areaId,
                  totalTasks: allTasksArray.value.length,
                  filteredCount: filteredTasks.length,
                })

                return filteredTasks
              } else if (extraIdentifier === 'project' && fifthPart) {
                // misc::staging::${areaId}::project::${projectId} - 指定区域的指定项目 staging 任务
                const areaId = identifier
                const projectId = fifthPart
                const today = getTodayDateString()
                const filteredTasks = allTasksArray.value.filter((task) => {
                  // 基础检查：必须属于指定区域和指定项目
                  if (
                    task.area_id !== areaId ||
                    task.project_id !== projectId ||
                    task.is_completed ||
                    task.is_archived ||
                    task.is_deleted
                  ) {
                    return false
                  }

                  // 🔥 实时计算：没有当前或未来的日程 = staging
                  const hasFutureOrTodaySchedule =
                    task.schedules?.some((schedule) => schedule.scheduled_day >= today) ?? false
                  if (hasFutureOrTodaySchedule) {
                    return false
                  }

                  // 🔥 排除 EXPIRE 类型且已过期的循环任务
                  if (
                    task.recurrence_id &&
                    task.recurrence_original_date &&
                    task.recurrence_expiry_behavior === 'EXPIRE'
                  ) {
                    if (task.recurrence_original_date < today) {
                      return false
                    }
                  }

                  return true
                })

                logger.debug(LogTags.STORE_TASKS, 'Area project staging filter result', {
                  viewKey,
                  areaId,
                  projectId,
                  totalTasks: allTasksArray.value.length,
                  filteredCount: filteredTasks.length,
                })

                return filteredTasks
              }

              // misc::staging::${areaId} - 指定 area 的 staging 任务
              // ⚠️ 使用 getTodayDateString() 获取本地日期，符合 TIME_CONVENTION.md
              const today = getTodayDateString()
              const filteredTasks = allTasksArray.value.filter((task) => {
                // 基础检查
                if (
                  task.area_id !== identifier ||
                  task.is_completed ||
                  task.is_archived ||
                  task.is_deleted
                ) {
                  return false
                }

                // 🔥 实时计算：没有当前或未来的日程 = staging
                const hasFutureOrTodaySchedule =
                  task.schedules?.some((schedule) => schedule.scheduled_day >= today) ?? false
                const isStaging = !hasFutureOrTodaySchedule

                if (!isStaging) {
                  return false
                }

                // 🔥 排除 EXPIRE 类型且已过期的循环任务
                if (
                  task.recurrence_id &&
                  task.recurrence_original_date &&
                  task.recurrence_expiry_behavior === 'EXPIRE'
                ) {
                  // 判断是否过期：原始日期 < 今天
                  if (task.recurrence_original_date < today) {
                    return false
                  }
                }

                if (task.area_id === identifier) {
                  logger.debug(LogTags.STORE_TASKS, 'Task area match check', {
                    taskId: task.id,
                    taskTitle: task.title,
                    taskAreaId: task.area_id,
                    targetAreaId: identifier,
                    isStaging,
                    isCompleted: task.is_completed,
                    isArchived: task.is_archived,
                    isDeleted: task.is_deleted,
                    finalMatch: true,
                  })
                }
                return true
              })

              logger.info(LogTags.STORE_TASKS, 'Area staging filter result', {
                viewKey,
                areaId: identifier,
                totalTasks: allTasksArray.value.length,
                filteredCount: filteredTasks.length,
                filteredTaskIds: filteredTasks.map((t) => t.id),
              })

              return filteredTasks
            } else {
              // misc::staging - 全部 staging 任务
              logger.debug(LogTags.STORE_TASKS, 'Using global staging tasks', {
                viewKey,
                count: stagingTasks.value.length,
              })
              return stagingTasks.value
            }
          } else if (subtype === 'archive') {
            // misc::archive - 归档任务
            logger.debug(LogTags.STORE_TASKS, 'Using archived tasks', {
              viewKey,
              count: archivedTasks.value.length,
            })
            return archivedTasks.value
          } else if (subtype === 'completed') {
            if (identifier) {
              // misc::completed::${date} - 指定日期的已完成任务
              const date = identifier
              const tasks = completedTasks.value.filter((task) =>
                task.schedules?.some((schedule) => schedule.scheduled_day === date)
              )

              logger.debug(LogTags.STORE_TASKS, 'Using completed tasks for date', {
                viewKey,
                date,
                count: tasks.length,
              })

              return tasks
            }

            // misc::completed - 所有已完成任务
            logger.debug(LogTags.STORE_TASKS, 'Using completed tasks', {
              viewKey,
              count: completedTasks.value.length,
            })
            return completedTasks.value
          } else if (subtype === 'incomplete') {
            if (identifier) {
              // misc::incomplete::${date} - 指定日期的未完成任务
              const date = identifier
              const tasks = incompleteTasks.value.filter((task) =>
                task.schedules?.some((schedule) => schedule.scheduled_day === date)
              )

              logger.debug(LogTags.STORE_TASKS, 'Using incomplete tasks for date', {
                viewKey,
                date,
                count: tasks.length,
              })

              return tasks
            }

            // misc::incomplete - 所有未完成任务
            logger.debug(LogTags.STORE_TASKS, 'Using incomplete tasks', {
              viewKey,
              count: incompleteTasks.value.length,
            })
            return incompleteTasks.value
          } else if (subtype === 'planned') {
            // misc::planned - 已安排任务
            logger.debug(LogTags.STORE_TASKS, 'Using planned tasks', {
              viewKey,
              count: plannedTasks.value.length,
            })
            return plannedTasks.value
          } else if (subtype === 'all') {
            // misc::all - 所有任务
            logger.debug(LogTags.STORE_TASKS, 'Using all tasks', {
              viewKey,
              count: allTasks.value.length,
            })
            return allTasks.value
          } else if (subtype === 'no-project') {
            // misc::no-project - 无项目任务
            // 🔥 过滤已完成和已归档的任务
            // 🔥 对于循环任务，只显示每个循环规则的最近未完成任务
            const noProjectTasks = allTasksArray.value.filter(
              (task) =>
                !task.project_id && !task.is_deleted && !task.is_completed && !task.is_archived
            )
            const tasks = deduplicateRecurringTasks(noProjectTasks)

            logger.debug(LogTags.STORE_TASKS, 'Using no-project tasks', {
              viewKey,
              total: noProjectTasks.length,
              filtered: tasks.length,
            })
            return tasks
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
              count: tasks.length,
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
              count: tasks.length,
            })
            return tasks
          }
          break

        case 'project':
          if (identifier === 'section' && extraIdentifier) {
            // project::${projectId}::section::{sectionId|all}
            const projectId = subtype
            const sectionId = extraIdentifier

            if (sectionId === 'all') {
              // project::${projectId}::section::all - 项目无section任务
              const noSectionTasks = allTasksArray.value.filter(
                (task) => task.project_id === projectId && !task.section_id && !task.is_deleted
              )
              const tasks = deduplicateRecurringTasks(noSectionTasks)
              logger.debug(LogTags.STORE_TASKS, 'Using project no-section tasks', {
                viewKey,
                projectId,
                total: noSectionTasks.length,
                filtered: tasks.length,
              })
              return tasks
            } else {
              // project::${projectId}::section::${sectionId} - 特定section任务
              const sectionTasks = allTasksArray.value.filter(
                (task) => task.section_id === sectionId && !task.is_deleted
              )
              const tasks = deduplicateRecurringTasks(sectionTasks)
              logger.debug(LogTags.STORE_TASKS, 'Using project section tasks', {
                viewKey,
                projectId,
                sectionId,
                total: sectionTasks.length,
                filtered: tasks.length,
              })
              return tasks
            }
          } else if (subtype && !identifier) {
            // project::${projectId} - 项目所有任务
            const projectId = subtype
            const tasks = getTasksByProject_Mux.value(projectId)
            logger.debug(LogTags.STORE_TASKS, 'Using project tasks', {
              viewKey,
              projectId,
              count: tasks.length,
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
    deadlineTasks,
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
