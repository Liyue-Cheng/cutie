/**
 * 拖拽决策服务
 *
 * 根据业务逻辑文档，明确定义所有拖拽场景的行为
 * 决策流程：
 * 1. 第一层：时间关系判断（过去→今天/未来、今天→未来、今天→过去等）
 * 2. 第二层：任务状态判断（已完成、未完成+PLANNED、未完成+PRESENCE_LOGGED）
 */

import type { TaskCard } from '@/types/dtos'

// ==================== 类型定义 ====================

/**
 * 时间关系类型
 */
export type TimeRelation =
  | 'past-to-today-or-future' // 过去 → 今天/未来
  | 'past-to-past' // 过去 → 过去
  | 'today-to-future' // 今天 → 未来
  | 'today-to-past' // 今天 → 过去（拒绝）
  | 'future-to-today' // 未来 → 今天
  | 'future-to-past' // 未来 → 过去（拒绝）
  | 'future-to-future' // 未来 → 未来
  | 'same-day' // 同一天（重排序）

/**
 * 任务工作状态
 */
export type TaskWorkStatus =
  | 'completed' // 已完成
  | 'worked' // 未完成但有工作记录（PRESENCE_LOGGED 或 COMPLETED_ON_DAY）
  | 'planned' // 未完成且仅计划（PLANNED）
  | 'unknown' // 无法判断

/**
 * 拖拽决策结果
 */
export interface DragDecision {
  /** 是否允许拖拽 */
  allowed: boolean

  /** 是否保留源日程 */
  keepSourceSchedule: boolean

  /** 是否删除源日程 */
  deleteSourceSchedule: boolean

  /** 是否创建新日程 */
  createTargetSchedule: boolean

  /** 是否更新日程日期（改期） */
  updateScheduleDate: boolean

  /** 是否需要重开任务 */
  reopenTask: boolean

  /** 是否保留源看板元素（拖拽时源元素可见） */
  keepSourceElement: boolean

  /** 决策说明（用于调试） */
  reason: string

  /** 场景标识 */
  scenario: string
}

// ==================== 核心决策函数 ====================

/**
 * 做出拖拽决策
 *
 * @param task 被拖拽的任务
 * @param sourceDate 源日期 (YYYY-MM-DD)
 * @param targetDate 目标日期 (YYYY-MM-DD)
 * @param today 今天的日期 (YYYY-MM-DD)
 * @returns 拖拽决策结果
 */
export function makeDragDecision(
  task: TaskCard,
  sourceDate: string,
  targetDate: string,
  today: string
): DragDecision {
  // 第一层：判断时间关系
  const timeRelation = determineTimeRelation(sourceDate, targetDate, today)

  console.log('🎯 [DragDecision] Time relation:', {
    sourceDate,
    targetDate,
    today,
    timeRelation,
  })

  // 第二层：根据时间关系和任务状态做决策
  switch (timeRelation) {
    case 'same-day':
      return handleSameDay(task, sourceDate)

    case 'past-to-today-or-future':
      return handlePastToTodayOrFuture(task, sourceDate, targetDate)

    case 'past-to-past':
      return handlePastToPast(task, sourceDate, targetDate)

    case 'today-to-future':
      return handleTodayToFuture(task, sourceDate, targetDate)

    case 'today-to-past':
      return handleTodayToPast()

    case 'future-to-today':
      return handleFutureToToday(task, sourceDate, targetDate)

    case 'future-to-past':
      return handleFutureToPast()

    case 'future-to-future':
      return handleFutureToFuture(task, sourceDate, targetDate)

    default:
      return createRejectedDecision('未知的时间关系', 'unknown')
  }
}

// ==================== 时间关系判断 ====================

/**
 * 判断时间关系
 */
function determineTimeRelation(
  sourceDate: string,
  targetDate: string,
  today: string
): TimeRelation {
  // 同一天
  if (sourceDate === targetDate) {
    return 'same-day'
  }

  const isSourcePast = sourceDate < today
  const isSourceToday = sourceDate === today
  const isSourceFuture = sourceDate > today

  const isTargetPast = targetDate < today
  const isTargetToday = targetDate === today
  const isTargetFuture = targetDate > today

  // 过去 → 今天/未来
  if (isSourcePast && (isTargetToday || isTargetFuture)) {
    return 'past-to-today-or-future'
  }

  // 过去 → 过去
  if (isSourcePast && isTargetPast) {
    return 'past-to-past'
  }

  // 今天 → 未来
  if (isSourceToday && isTargetFuture) {
    return 'today-to-future'
  }

  // 今天 → 过去（拒绝）
  if (isSourceToday && isTargetPast) {
    return 'today-to-past'
  }

  // 未来 → 今天
  if (isSourceFuture && isTargetToday) {
    return 'future-to-today'
  }

  // 未来 → 过去（拒绝）
  if (isSourceFuture && isTargetPast) {
    return 'future-to-past'
  }

  // 未来 → 未来
  if (isSourceFuture && isTargetFuture) {
    return 'future-to-future'
  }

  // 默认：未知
  return 'same-day'
}

/**
 * 获取任务在指定日期的工作状态
 */
function getTaskWorkStatus(task: TaskCard, date: string): TaskWorkStatus {
  // 判断任务是否已完成
  if (task.is_completed) {
    return 'completed'
  }

  // 查找该日期的 schedule
  const schedule = task.schedules?.find((s) => s.scheduled_day === date)

  if (!schedule) {
    return 'unknown'
  }

  // 判断是否有工作记录
  const outcome = schedule.outcome.toLowerCase()
  if (outcome === 'presence_logged' || outcome === 'completed_on_day') {
    return 'worked'
  }

  if (outcome === 'planned') {
    return 'planned'
  }

  return 'unknown'
}

// ==================== 场景处理函数 ====================

/**
 * 场景：同一天内重排序
 */
function handleSameDay(_task: TaskCard, _date: string): DragDecision {
  return {
    allowed: true,
    keepSourceSchedule: true,
    deleteSourceSchedule: false,
    createTargetSchedule: false,
    updateScheduleDate: false,
    reopenTask: false,
    keepSourceElement: false, // 重排序时不需要保留源元素
    reason: '同一天内重新排序，不修改日程',
    scenario: 'same-day',
  }
}

/**
 * 场景组 1：过去 → 今天/未来
 *
 * 根据业务逻辑：
 * - 情况 1.1：已完成任务 → 重开 + 创建新日程，保留源schedule
 * - 情况 1.2：未完成任务 → 保留源schedule + 创建新日程
 */
function handlePastToTodayOrFuture(
  task: TaskCard,
  sourceDate: string,
  targetDate: string
): DragDecision {
  const workStatus = getTaskWorkStatus(task, sourceDate)

  console.log('🎯 [DragDecision] Past to today/future:', {
    taskId: task.id,
    sourceDate,
    targetDate,
    workStatus,
    isCompleted: task.is_completed,
  })

  switch (workStatus) {
    case 'completed':
      // 情况 1.1：已完成任务
      return {
        allowed: true,
        keepSourceSchedule: true, // 保留源schedule（历史记录）
        deleteSourceSchedule: false,
        createTargetSchedule: true, // 创建新日程
        updateScheduleDate: false,
        reopenTask: true, // 重开任务
        keepSourceElement: true, // 保留源元素显示
        reason: '从过去拖已完成任务：保留历史 + 重开 + 创建新日程',
        scenario: 'past-to-future-completed',
      }

    case 'worked':
    case 'planned':
    case 'unknown':
      // 情况 1.2：未完成任务（不论 PLANNED 还是 PRESENCE_LOGGED）
      return {
        allowed: true,
        keepSourceSchedule: true, // 保留源schedule（历史记录）
        deleteSourceSchedule: false,
        createTargetSchedule: true, // 创建新日程
        updateScheduleDate: false,
        reopenTask: false, // 本来就未完成
        keepSourceElement: workStatus === 'worked', // 有工作记录时保留源元素
        reason: '从过去拖未完成任务：保留历史 + 创建新日程',
        scenario: 'past-to-future-incomplete',
      }

    default:
      return createRejectedDecision('无法判断任务状态', 'past-to-future-unknown')
  }
}

/**
 * 场景：过去 → 过去
 *
 * 根据业务逻辑：
 * - 如果源日期有PRESENCE记录（worked或completed），保留源元素
 * - 如果源日期仅为PLANNED，不保留源元素
 */
function handlePastToPast(task: TaskCard, sourceDate: string, targetDate: string): DragDecision {
  const workStatus = getTaskWorkStatus(task, sourceDate)

  console.log('🎯 [DragDecision] Past to past:', {
    taskId: task.id,
    sourceDate,
    targetDate,
    workStatus,
    isCompleted: task.is_completed,
  })

  switch (workStatus) {
    case 'completed':
      // 已完成任务
      return {
        allowed: true,
        keepSourceSchedule: true, // 保留源schedule（历史记录）
        deleteSourceSchedule: false,
        createTargetSchedule: true, // 创建新日程
        updateScheduleDate: false,
        reopenTask: true, // 重开任务
        keepSourceElement: true, // 保留源元素显示
        reason: '过去日期间拖已完成任务：保留历史 + 重开 + 创建新日程',
        scenario: 'past-to-past-completed',
      }

    case 'worked':
      // 未完成但有工作记录
      return {
        allowed: true,
        keepSourceSchedule: true, // 保留源schedule（工作记录）
        deleteSourceSchedule: false,
        createTargetSchedule: true, // 创建新日程
        updateScheduleDate: false,
        reopenTask: false,
        keepSourceElement: true, // 保留源元素显示（有PRESENCE记录）
        reason: '过去日期间拖有工作记录的任务：保留工作记录 + 创建新日程',
        scenario: 'past-to-past-worked',
      }

    case 'planned':
      // 仅计划
      return {
        allowed: true,
        keepSourceSchedule: false,
        deleteSourceSchedule: true, // 删除源schedule（标准改期）
        createTargetSchedule: true, // 创建新日程
        updateScheduleDate: false,
        reopenTask: false,
        keepSourceElement: false, // 不保留源元素（仅计划）
        reason: '过去日期间拖仅计划的任务：删除源日程 + 创建新日程（标准改期）',
        scenario: 'past-to-past-planned',
      }

    case 'unknown':
    default:
      // 默认：标准改期
      return {
        allowed: true,
        keepSourceSchedule: false,
        deleteSourceSchedule: false,
        createTargetSchedule: false,
        updateScheduleDate: true, // 更新日期
        reopenTask: false,
        keepSourceElement: false,
        reason: '过去日期间拖任务：更新日程日期',
        scenario: 'past-to-past-default',
      }
  }
}

/**
 * 场景组 2：今天 → 未来
 *
 * 根据业务逻辑：
 * - 情况 2.1：未完成 + PLANNED → 删除源schedule（轻量改期）
 * - 情况 2.2：未完成 + PRESENCE_LOGGED → 保留源schedule
 * - 情况 2.3：已完成 → 保留源schedule，重开任务
 */
function handleTodayToFuture(task: TaskCard, sourceDate: string, targetDate: string): DragDecision {
  const workStatus = getTaskWorkStatus(task, sourceDate)

  console.log('🎯 [DragDecision] Today to future:', {
    taskId: task.id,
    sourceDate,
    targetDate,
    workStatus,
    isCompleted: task.is_completed,
  })

  switch (workStatus) {
    case 'completed':
      // 情况 2.3：已完成任务
      return {
        allowed: true,
        keepSourceSchedule: true, // 保留源schedule（历史记录）
        deleteSourceSchedule: false,
        createTargetSchedule: true, // 创建新日程
        updateScheduleDate: false,
        reopenTask: true, // 重开任务
        keepSourceElement: true, // 保留源元素显示
        reason: '今天拖已完成任务到未来：保留历史 + 重开 + 创建新日程',
        scenario: 'today-to-future-completed',
      }

    case 'worked':
      // 情况 2.2：未完成 + 有工作记录（PRESENCE_LOGGED）
      return {
        allowed: true,
        keepSourceSchedule: true, // 保留源schedule（工作记录）
        deleteSourceSchedule: false,
        createTargetSchedule: true, // 创建新日程
        updateScheduleDate: false,
        reopenTask: false,
        keepSourceElement: true, // 保留源元素显示
        reason: '今天拖有工作记录的任务到未来：保留工作记录 + 创建新日程',
        scenario: 'today-to-future-worked',
      }

    case 'planned':
      // 情况 2.1：未完成 + 仅计划（PLANNED）
      return {
        allowed: true,
        keepSourceSchedule: false,
        deleteSourceSchedule: true, // 删除源schedule（轻量改期）
        createTargetSchedule: true, // 创建新日程
        updateScheduleDate: false,
        reopenTask: false,
        keepSourceElement: false, // 不保留源元素
        reason: '今天拖仅计划的任务到未来：删除源日程 + 创建新日程（轻量改期）',
        scenario: 'today-to-future-planned',
      }

    case 'unknown':
    default:
      // 默认：更新日期（改期）
      return {
        allowed: true,
        keepSourceSchedule: false,
        deleteSourceSchedule: false,
        createTargetSchedule: false,
        updateScheduleDate: true, // 更新日期
        reopenTask: false,
        keepSourceElement: false,
        reason: '今天拖任务到未来：更新日程日期',
        scenario: 'today-to-future-default',
      }
  }
}

/**
 * 场景组 3：今天 → 过去（拒绝）
 */
function handleTodayToPast(): DragDecision {
  return createRejectedDecision('不允许从今天拖到过去', 'today-to-past-rejected')
}

/**
 * 场景组 4：未来 → 今天
 *
 * 根据业务逻辑：
 * - 情况 4.1：任意状态 → 删除源schedule + 创建新日程
 */
function handleFutureToToday(
  _task: TaskCard,
  _sourceDate: string,
  _targetDate: string
): DragDecision {
  return {
    allowed: true,
    keepSourceSchedule: false,
    deleteSourceSchedule: true, // 删除未来的计划
    createTargetSchedule: true, // 创建今天的日程
    updateScheduleDate: false,
    reopenTask: false,
    keepSourceElement: false,
    reason: '从未来提前到今天：删除未来日程 + 创建今天日程',
    scenario: 'future-to-today',
  }
}

/**
 * 场景组 5：未来 → 过去（拒绝）
 */
function handleFutureToPast(): DragDecision {
  return createRejectedDecision('不允许从未来拖到过去', 'future-to-past-rejected')
}

/**
 * 场景组 6：未来 → 未来
 *
 * 根据业务逻辑：
 * - 情况 6.1：任意状态 → 删除源schedule + 创建新日程
 */
function handleFutureToFuture(
  _task: TaskCard,
  _sourceDate: string,
  _targetDate: string
): DragDecision {
  return {
    allowed: true,
    keepSourceSchedule: false,
    deleteSourceSchedule: true, // 删除源日程
    createTargetSchedule: true, // 创建新日程
    updateScheduleDate: false,
    reopenTask: false,
    keepSourceElement: false,
    reason: '未来日期之间调整：删除源日程 + 创建新日程',
    scenario: 'future-to-future',
  }
}

// ==================== 辅助函数 ====================

/**
 * 创建拒绝决策
 */
function createRejectedDecision(reason: string, scenario: string): DragDecision {
  return {
    allowed: false,
    keepSourceSchedule: true,
    deleteSourceSchedule: false,
    createTargetSchedule: false,
    updateScheduleDate: false,
    reopenTask: false,
    keepSourceElement: true, // 拒绝时保留源元素
    reason,
    scenario,
  }
}
