/**
 * 策略工具函数
 *
 * 提供策略执行时需要的通用功能
 */

// Note: useTaskStore 和 useViewStore 已移除
// 策略工具函数现在都是纯函数，不依赖全局状态

/**
 * ⚠️ 已删除 getSortedTaskIds()
 *
 * 原因：策略不应该主动查询 Store，所有数据应由调用者（组件）传入
 *
 * V2 迁移指南：
 * - 组件通过 sourceContext 和 targetContext 传入任意数据
 * - 策略使用 extractTaskIds() 辅助函数解包数据
 * - 策略自行保证类型安全
 */

/**
 * 🔥 V2: 从上下文中提取对象ID列表（泛型版本）
 *
 * 灵活性：支持多种数据格式
 * - itemIds: string[]
 * - taskIds: string[] (向后兼容)
 * - displayItems: any[]
 * - displayTasks: any[] (向后兼容)
 * - 自动回退到空数组
 *
 * 注意：会自动过滤掉预览元素（ID 以 "preview-" 开头的）
 */
export function extractObjectIds(context: Record<string, any>): string[] {
  let ids: string[] = []

  // 优先使用 itemIds (新格式)
  if (Array.isArray(context.itemIds)) {
    ids = context.itemIds
  }
  // 向后兼容：taskIds
  else if (Array.isArray(context.taskIds)) {
    ids = context.taskIds
  }
  // 回退：从 displayItems 提取
  else if (Array.isArray(context.displayItems)) {
    ids = context.displayItems.map((item: any) => item.id)
  }
  // 向后兼容：从 displayTasks 提取
  else if (Array.isArray(context.displayTasks)) {
    ids = context.displayTasks.map((t: any) => t.id)
  } else {
    // 最后回退：空数组
    console.warn('[strategy-utils] No object IDs found in context', context)
    return []
  }

  // 🔥 过滤掉预览元素（ID 以 "preview-" 开头的）
  return ids.filter((id) => !id.startsWith('preview-'))
}

/**
 * 向后兼容的别名：extractTaskIds
 */
export function extractTaskIds(context: Record<string, any>): string[] {
  return extractObjectIds(context)
}

/**
 * 从列表中移除指定任务
 */
export function removeTaskFrom(taskIds: string[], taskId: string): string[] {
  return taskIds.filter((id) => id !== taskId)
}

/**
 * 在指定位置插入任务
 * 🔥 如果任务已存在，先移除再插入（避免重复）
 */
export function insertTaskAt(taskIds: string[], taskId: string, index?: number): string[] {
  // 先移除任务（如果已存在）
  const withoutTask = taskIds.filter((id) => id !== taskId)
  const insertIndex = index ?? withoutTask.length
  const safeIndex = Math.max(0, Math.min(insertIndex, withoutTask.length))
  withoutTask.splice(safeIndex, 0, taskId)
  return withoutTask
}

/**
 * 移动任务到新位置（同一列表内）
 */
export function moveTaskWithin(taskIds: string[], taskId: string, newIndex: number): string[] {
  const withoutTask = removeTaskFrom(taskIds, taskId)
  return insertTaskAt(withoutTask, taskId, newIndex)
}

/**
 * 获取任务在列表中的当前索引
 */
export function getTaskIndex(taskIds: string[], taskId: string): number {
  return taskIds.indexOf(taskId)
}

/**
 * 解析日期字符串（从 viewKey 中提取）
 */
export function extractDate(viewKey: string): string | null {
  const match = viewKey.match(/^daily::(\d{4}-\d{2}-\d{2})$/)
  return match ? (match[1] ?? null) : null
}

/**
 * 检查两个 viewKey 是否指向同一天
 */
export function isSameDay(viewKey1: string, viewKey2: string): boolean {
  const date1 = extractDate(viewKey1)
  const date2 = extractDate(viewKey2)
  return date1 !== null && date1 === date2
}

/**
 * 操作记录（用于日志和回滚）
 */
export interface OperationRecord {
  type:
    | 'create_schedule'
    | 'update_schedule'
    | 'delete_schedule'
    | 'update_sorting'
    | 'update_sort_position'
    | 'return_to_staging'
  target: string
  payload?: any
  timestamp: number
}

/**
 * 创建操作记录
 */
export function createOperationRecord(
  type: OperationRecord['type'],
  target: string,
  payload?: any
): OperationRecord {
  return {
    type,
    target,
    payload,
    timestamp: Date.now(),
  }
}
