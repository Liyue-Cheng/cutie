/**
 * 循环规则视图操作
 *
 * ⚠️ 已废弃：所有操作应通过CPU指令系统完成
 *
 * 使用示例：
 * ```typescript
 * import { pipeline } from '@/cpu'
 *
 * // 获取所有循环规则
 * await pipeline.dispatch('recurrence.fetch_all', {})
 *
 * // 按模板ID获取循环规则
 * await pipeline.dispatch('recurrence.fetch_by_template', { template_id })
 * ```
 */

import { pipeline } from '@/cpu'

export async function fetchAllRecurrences(): Promise<void> {
  await pipeline.dispatch('recurrence.fetch_all', {})
}

export async function fetchRecurrencesByTemplateId(templateId: string): Promise<void> {
  await pipeline.dispatch('recurrence.fetch_by_template', { template_id: templateId })
}
