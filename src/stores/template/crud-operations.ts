/**
 * 模板CRUD操作
 *
 * ⚠️ 已废弃：所有操作应通过CPU指令系统完成
 *
 * 使用示例：
 * ```typescript
 * import { pipeline } from '@/cpu'
 *
 * // 创建模板
 * await pipeline.execute('template.create', payload)
 *
 * // 更新模板
 * await pipeline.execute('template.update', { id, ...updates })
 *
 * // 删除模板
 * await pipeline.execute('template.delete', { id })
 *
 * // 从模板创建任务
 * await pipeline.execute('template.create_task', { template_id, variables })
 *
 * // 从任务创建模板
 * await pipeline.execute('template.from_task', { task_id, title, category })
 * ```
 */

// 导出类型定义（从types.ts重新导出）
export type {
  CreateTemplatePayload,
  UpdateTemplatePayload,
  CreateTaskFromTemplatePayload,
  CreateTemplateFromTaskPayload,
} from './types'
