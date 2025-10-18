/**
 * 循环规则CRUD操作
 *
 * ⚠️ 已废弃：所有操作应通过CPU指令系统完成
 *
 * 使用示例：
 * ```typescript
 * import { pipeline } from '@/cpu'
 *
 * // 创建循环规则
 * await pipeline.dispatch('recurrence.create', payload)
 *
 * // 更新循环规则
 * await pipeline.dispatch('recurrence.update', { id, ...updates })
 *
 * // 删除循环规则
 * await pipeline.dispatch('recurrence.delete', { id })
 *
 * // 批量更新模板和实例
 * await pipeline.dispatch('recurrence.update_template_and_instances', { recurrence_id, ...updates })
 * ```
 */

import { pipeline } from '@/cpu'
import type {
  CreateTaskRecurrencePayload,
  TaskRecurrence,
  UpdateTaskRecurrencePayload,
} from '@/types/dtos'

export async function createRecurrence(
  payload: CreateTaskRecurrencePayload
): Promise<TaskRecurrence> {
  return await pipeline.dispatch('recurrence.create', payload)
}

export async function updateRecurrence(
  id: string,
  payload: UpdateTaskRecurrencePayload
): Promise<TaskRecurrence> {
  return await pipeline.dispatch('recurrence.update', { id, ...payload })
}

export async function deleteRecurrence(id: string): Promise<void> {
  await pipeline.dispatch('recurrence.delete', { id })
}

/**
 * 批量更新模板和所有未完成实例
 */
export async function updateTemplateAndInstances(
  recurrenceId: string,
  updates: any
): Promise<any> {
  return await pipeline.dispatch('recurrence.update_template_and_instances', {
    recurrence_id: recurrenceId,
    ...updates,
  })
}
