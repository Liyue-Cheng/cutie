/**
 * 模板调度策略
 *
 * 处理模板相关的拖放操作：
 * - 模板 → 日程看板：从模板创建任务并添加日程
 * - 日程看板 → 模板：从任务创建模板
 */

import type { Strategy } from '../types'
import {
  extractTaskIds,
  insertTaskAt,
  moveTaskWithin,
  extractDate,
  createOperationRecord,
  type OperationRecord,
} from './strategy-utils'
import { pipeline } from '@/cpu'
import { isTemplate, isTaskCard } from '@/types/dtos'

/**
 * 策略 1：Template → Daily
 *
 * 操作链：
 * 1. 从模板创建任务 (template.create_task)
 * 2. 为新任务添加日程 (schedule.create)
 * 3. 插入到 Daily 视图 (view.update_sorting)
 */
export const templateToDailyStrategy: Strategy = {
  id: 'template-to-daily',
  name: 'Template to Daily Schedule',

  conditions: {
    source: {
      viewKey: 'misc::template',
      objectType: 'template',
    },
    target: {
      viewKey: /^daily::\d{4}-\d{2}-\d{2}$/,
    },
    priority: 90,
  },

  action: {
    name: 'create_task_from_template_with_schedule',
    description: '从模板创建任务并安排到指定日期（3步操作）',

    async execute(ctx) {
      // 类型守卫
      if (!isTemplate(ctx.draggedObject)) {
        throw new Error('Expected template object')
      }
      const template = ctx.draggedObject

      const targetDate = extractDate(ctx.targetZone)!
      const operations: OperationRecord[] = []

      try {
        // 🎯 步骤 1: 从模板创建任务
        const createTaskPayload = {
          template_id: template.id,
          variables: { date: targetDate }, // 可以传递变量
        }
        const newTask = await pipeline.dispatch('template.create_task', createTaskPayload)
        operations.push(createOperationRecord('create_task', ctx.targetViewId, createTaskPayload))

        // 🎯 步骤 2: 为新任务添加日程
        const schedulePayload = {
          task_id: newTask.id,
          scheduled_day: targetDate,
        }
        await pipeline.dispatch('schedule.create', schedulePayload)
        operations.push(createOperationRecord('create_schedule', ctx.targetViewId, schedulePayload))

        // 🎯 步骤 3: 插入到 Daily 视图（更新排序）
        const targetSorting = extractTaskIds(ctx.targetContext)
        const newTargetSorting = insertTaskAt(targetSorting, newTask.id, ctx.dropIndex)
        const sortPayload = buildTaskLexoPayload(ctx.targetViewId, newTargetSorting, newTask.id)
        if (sortPayload) {
          await pipeline.dispatch('task.update_sort_position', sortPayload)
          operations.push(
            createOperationRecord('update_sort_position', ctx.targetViewId, sortPayload)
          )
        }

        return {
          success: true,
          message: `✅ Created task from template and scheduled to ${targetDate}`,
          operations,
          affectedViews: [ctx.targetViewId],
        }
      } catch (error) {
        return {
          success: false,
          message: `❌ Failed to create from template: ${error instanceof Error ? error.message : String(error)}`,
          operations,
          affectedViews: [ctx.targetViewId],
        }
      }
    },
  },

  tags: ['template', 'daily', 'create', 'multi-step'],
}

/**
 * 策略 2：Daily → Template
 *
 * 操作链：
 * 1. 从任务创建模板 (template.from_task)
 * 2. 插入到模板视图 (view.update_sorting)
 * 注意：保留源任务（不删除、不移除）
 */
export const dailyToTemplateStrategy: Strategy = {
  id: 'daily-to-template',
  name: 'Daily Task to Template',

  conditions: {
    source: {
      viewKey: /^daily::\d{4}-\d{2}-\d{2}$/,
      objectType: 'task',
    },
    target: {
      viewKey: 'misc::template',
    },
    priority: 90,
  },

  action: {
    name: 'save_task_as_template',
    description: '从任务创建模板（2步操作，保留原任务）',

    async execute(ctx) {
      // 类型守卫
      if (!isTaskCard(ctx.draggedObject)) {
        throw new Error('Expected task object')
      }
      const task = ctx.draggedObject

      const operations: OperationRecord[] = []

      try {
        // 🎯 步骤 1: 从任务创建模板
        const createTemplatePayload = {
          task_id: task.id,
          title: task.title, // 使用任务原始标题，不添加额外后缀
          category: 'GENERAL' as const,
        }
        const newTemplate = await pipeline.dispatch('template.from_task', createTemplatePayload)
        operations.push(
          createOperationRecord('create_template', ctx.targetViewId, createTemplatePayload)
        )

        // 🎯 步骤 2: 插入到模板视图（更新排序）
        const targetSorting = extractTaskIds(ctx.targetContext)
        const newTargetSorting = insertTaskAt(targetSorting, newTemplate.id, ctx.dropIndex)
        const templatePayload = buildTemplateSortPayload(newTargetSorting, newTemplate.id)
        if (templatePayload) {
          await pipeline.dispatch('template.update_sort_rank', templatePayload)
          operations.push(
            createOperationRecord('update_sort_position', ctx.targetViewId, templatePayload)
          )
        }

        // 注意：不更新源视图排序，保留原任务在原位置

        return {
          success: true,
          message: `✅ Saved task as template`,
          operations,
          affectedViews: [ctx.targetViewId],
        }
      } catch (error) {
        return {
          success: false,
          message: `❌ Failed to save as template: ${error instanceof Error ? error.message : String(error)}`,
          operations,
          affectedViews: [ctx.targetViewId],
        }
      }
    },
  },

  tags: ['template', 'daily', 'save', 'multi-step'],
}

/**
 * 策略 3：Template 内部重排序
 *
 * 操作链：
 * 1. 更新模板排序 (template.update_sort_rank)
 */
export const templateReorderStrategy: Strategy = {
  id: 'template-reorder',
  name: 'Template Internal Reorder',

  conditions: {
    source: {
      viewKey: 'misc::template',
      objectType: 'template',
    },
    target: {
      viewKey: 'misc::template',
    },
    priority: 85,
  },

  action: {
    name: 'reorder_in_template',
    description: '在模板列表内重新排序（1步操作）',

    async execute(ctx) {
      // 类型守卫
      if (!isTemplate(ctx.draggedObject)) {
        throw new Error('Expected template object')
      }
      const template = ctx.draggedObject

      const operations: OperationRecord[] = []

      try {
        const sorting = extractTaskIds(ctx.targetContext)
        const newSorting = moveTaskWithin(sorting, template.id, ctx.dropIndex ?? sorting.length)
        const templatePayload = buildTemplateSortPayload(newSorting, template.id)
        if (templatePayload) {
          await pipeline.dispatch('template.update_sort_rank', templatePayload)
          operations.push(
            createOperationRecord('update_sort_position', ctx.targetZone, templatePayload)
          )
        }

        return {
          success: true,
          message: `✅ Reordered in template list`,
          reorderOnly: true,
          operations,
          affectedViews: [ctx.sourceViewId],
        }
      } catch (error) {
        return {
          success: false,
          message: `❌ Failed to reorder: ${error instanceof Error ? error.message : String(error)}`,
          operations,
          affectedViews: [ctx.sourceViewId],
        }
      }
    },
  },

  tags: ['template', 'reorder'],
}

function buildTaskLexoPayload(viewKey: string, order: string[], taskId: string) {
  const index = order.indexOf(taskId)
  if (index === -1) return null

  const prev = index > 0 ? order[index - 1] : null
  const next = index < order.length - 1 ? order[index + 1] : null

  return {
    view_context: viewKey,
    task_id: taskId,
    prev_task_id: prev,
    next_task_id: next,
  }
}

function buildTemplateSortPayload(order: string[], templateId: string) {
  const index = order.indexOf(templateId)
  if (index === -1) return null

  const prev = index > 0 ? order[index - 1] : null
  const next = index < order.length - 1 ? order[index + 1] : null

  return {
    template_id: templateId,
    prev_template_id: prev,
    next_template_id: next,
  }
}
