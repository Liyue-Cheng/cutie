/**
 * 项目拖放策略
 *
 * 支持场景：
 * 1. Daily → Project: 设置任务的 project_id
 * 2. Daily → Section: 设置任务的 project_id 和 section_id
 * 3. Project → Project: 项目内重排
 * 4. Section → Section: 跨章节移动
 */

import type { Strategy, StrategyContext, StrategyResult } from '../types'
import { pipeline } from '@/cpu'
import { extractTaskIds, insertTaskAt, removeTaskFrom } from './strategy-utils'

/**
 * 策略1: Daily → Project
 * 从日历拖任务到项目视图 = 设置项目归属（保留日程）
 */
export const dailyToProjectStrategy: Strategy = {
  id: 'daily-to-project',
  name: 'Daily → Project (设置项目归属)',

  conditions: {
    source: {
      viewKey: /^daily::/,
      objectType: 'task',
    },
    target: {
      viewKey: /^project::[^:]+::section::all$/,
    },
    priority: 85,
  },

  action: {
    name: 'set_project',
    description: '设置任务的项目归属（保留日程安排）',
    async execute(ctx: StrategyContext): Promise<StrategyResult> {
      const taskId = ctx.session.object.data.id

      try {
        // 解析目标 project_id
        const parts = ctx.targetViewId.split('::')
        const projectId = parts[1]

        if (!projectId) {
          return { success: false, message: '❌ 无效的项目ID' }
        }

        // 步骤 1: 更新任务的 project_id（保留 section_id = null）
        await pipeline.dispatch('task.update', {
          id: taskId,
          updates: {
            project_id: projectId,
            section_id: null,
          },
        })

        // 步骤 2: 更新目标视图排序
        const targetTaskIds = extractTaskIds(ctx.targetContext)
        const newTargetOrder = insertTaskAt(targetTaskIds, taskId, ctx.dropIndex)

        await pipeline.dispatch('viewpreference.update_sorting', {
          context_key: ctx.targetViewId,
          sorted_task_ids: newTargetOrder,
        })

        return {
          success: true,
          message: '✅ 任务已添加到项目',
          affectedViews: [ctx.sourceViewId, ctx.targetViewId],
        }
      } catch (error) {
        return {
          success: false,
          message: `❌ ${error instanceof Error ? error.message : '操作失败'}`,
        }
      }
    },
  },

  tags: ['project', 'scheduling'],
}

/**
 * 策略2: Daily → Section
 * 从日历拖任务到项目章节 = 设置项目和章节归属（保留日程）
 */
export const dailyToSectionStrategy: Strategy = {
  id: 'daily-to-section',
  name: 'Daily → Section (设置项目和章节归属)',

  conditions: {
    source: {
      viewKey: /^daily::/,
      objectType: 'task',
    },
    target: {
      viewKey: /^project::[^:]+::section::[^:]+$/,
    },
    priority: 90,
  },

  action: {
    name: 'set_project_and_section',
    description: '设置任务的项目和章节归属（保留日程安排）',
    async execute(ctx: StrategyContext): Promise<StrategyResult> {
      const taskId = ctx.session.object.data.id

      try {
        // 解析目标 project_id 和 section_id
        const parts = ctx.targetViewId.split('::')
        const projectId = parts[1]
        const sectionId = parts[3]

        if (!projectId || !sectionId || sectionId === 'all') {
          return { success: false, message: '❌ 无效的章节ID' }
        }

        // 步骤 1: 更新任务的 project_id 和 section_id
        await pipeline.dispatch('task.update', {
          id: taskId,
          updates: {
            project_id: projectId,
            section_id: sectionId,
          },
        })

        // 步骤 2: 更新目标视图排序
        const targetTaskIds = extractTaskIds(ctx.targetContext)
        const newTargetOrder = insertTaskAt(targetTaskIds, taskId, ctx.dropIndex)

        await pipeline.dispatch('viewpreference.update_sorting', {
          context_key: ctx.targetViewId,
          sorted_task_ids: newTargetOrder,
        })

        return {
          success: true,
          message: '✅ 任务已添加到章节',
          affectedViews: [ctx.sourceViewId, ctx.targetViewId],
        }
      } catch (error) {
        return {
          success: false,
          message: `❌ ${error instanceof Error ? error.message : '操作失败'}`,
        }
      }
    },
  },

  tags: ['project', 'scheduling'],
}

/**
 * 策略3: No Project → Project (从无项目列表分配到项目)
 */
export const noProjectToProjectStrategy: Strategy = {
  id: 'no-project-to-project',
  name: 'No Project → Project (分配项目)',

  conditions: {
    source: {
      viewKey: /^misc::no-project$/,
      objectType: 'task',
    },
    target: {
      viewKey: /^project::[^:]+::section::all$/,
    },
    priority: 88,
  },

  action: {
    name: 'assign_project_from_no_project',
    description: '将无项目任务分配到项目（未分类列表）',
    async execute(ctx: StrategyContext): Promise<StrategyResult> {
      const taskId = ctx.session.object.data.id

      try {
        const parts = ctx.targetViewId.split('::')
        const projectId = parts[1]

        if (!projectId) {
          return { success: false, message: '❌ 无效的项目ID' }
        }

        await pipeline.dispatch('task.update', {
          id: taskId,
          updates: {
            project_id: projectId,
            section_id: null,
          },
        })

        const sourceTaskIds = extractTaskIds(ctx.sourceContext)
        const newSourceOrder = removeTaskFrom(sourceTaskIds, taskId)
        await pipeline.dispatch('viewpreference.update_sorting', {
          context_key: ctx.sourceViewId,
          sorted_task_ids: newSourceOrder,
        })

        const targetTaskIds = extractTaskIds(ctx.targetContext)
        const newTargetOrder = insertTaskAt(targetTaskIds, taskId, ctx.dropIndex)

        await pipeline.dispatch('viewpreference.update_sorting', {
          context_key: ctx.targetViewId,
          sorted_task_ids: newTargetOrder,
        })

        return {
          success: true,
          message: '✅ 任务已分配到项目',
          affectedViews: [ctx.sourceViewId, ctx.targetViewId],
        }
      } catch (error) {
        return {
          success: false,
          message: `❌ ${error instanceof Error ? error.message : '操作失败'}`,
        }
      }
    },
  },

  tags: ['project', 'assignment'],
}

/**
 * 策略4: Project → Project (同项目内重排)
 */
export const projectReorderStrategy: Strategy = {
  id: 'project-reorder',
  name: 'Project → Project (项目内重排)',

  conditions: {
    source: {
      viewKey: /^project::/,
      objectType: 'task',
    },
    target: {
      viewKey: /^project::/,
    },
    priority: 80,
  },

  action: {
    name: 'reorder_in_project',
    description: '在项目内重新排序任务',
    async execute(ctx: StrategyContext): Promise<StrategyResult> {
      // 只有在同一个视图内才执行重排
      if (ctx.sourceViewId !== ctx.targetViewId) {
        return { success: false, message: '❌ 只能在同一视图内重排' }
      }

      try {
        const taskId = ctx.session.object.data.id
        const taskIds = extractTaskIds(ctx.targetContext)
        const oldIndex = taskIds.indexOf(taskId)

        // 计算新顺序
        let newOrder = [...taskIds]
        if (oldIndex !== -1) {
          newOrder = removeTaskFrom(newOrder, taskId)
        }
        newOrder = insertTaskAt(newOrder, taskId, ctx.dropIndex)

        // 更新排序
        await pipeline.dispatch('viewpreference.update_sorting', {
          context_key: ctx.targetViewId,
          sorted_task_ids: newOrder,
        })

        return {
          success: true,
          message: '✅ 任务已重新排序',
          affectedViews: [ctx.targetViewId],
        }
      } catch (error) {
        return {
          success: false,
          message: `❌ ${error instanceof Error ? error.message : '操作失败'}`,
        }
      }
    },
  },

  tags: ['project', 'reorder'],
}

/**
 * 策略5: Section → Section (跨章节移动)
 */
export const sectionToSectionStrategy: Strategy = {
  id: 'section-to-section',
  name: 'Section → Section (跨章节移动)',

  conditions: {
    source: {
      viewKey: /^project::[^:]+::section::(?:[^:]+|all)$/,
      objectType: 'task',
    },
    target: {
      viewKey: /^project::[^:]+::section::(?:[^:]+|all)$/,
    },
    priority: 85,
  },

  action: {
    name: 'move_between_sections',
    description: '在不同章节之间移动任务',
    async execute(ctx: StrategyContext): Promise<StrategyResult> {
      const taskId = ctx.session.object.data.id

      try {
        // 解析源和目标
        const sourceParts = ctx.sourceViewId.split('::')
        const targetParts = ctx.targetViewId.split('::')

        const sourceProjectId = sourceParts[1]
        const targetProjectId = targetParts[1]
        const targetSectionRaw = targetParts[3] ?? 'all'
        const targetSectionId = targetSectionRaw === 'all' ? null : targetSectionRaw

        // 如果是同一个section内重排
        if (ctx.sourceViewId === ctx.targetViewId) {
          const taskIds = extractTaskIds(ctx.targetContext)
          const oldIndex = taskIds.indexOf(taskId)

          let newOrder = [...taskIds]
          if (oldIndex !== -1) {
            newOrder = removeTaskFrom(newOrder, taskId)
          }
          newOrder = insertTaskAt(newOrder, taskId, ctx.dropIndex)

          await pipeline.dispatch('viewpreference.update_sorting', {
            context_key: ctx.targetViewId,
            sorted_task_ids: newOrder,
          })

          return {
            success: true,
            message: '✅ 任务已重新排序',
            affectedViews: [ctx.targetViewId],
          }
        }

        // 跨section移动
        // 步骤 1: 更新任务的 section_id（如果跨项目，也更新 project_id）
        const updates: { section_id: string | null; project_id?: string } = {
          section_id: targetSectionId,
        }

        if (sourceProjectId !== targetProjectId) {
          updates.project_id = targetProjectId
        }

        await pipeline.dispatch('task.update', {
          id: taskId,
          updates,
        })

        // 步骤 2: 更新源视图排序（移除任务）
        const sourceTaskIds = extractTaskIds(ctx.sourceContext)
        const newSourceOrder = removeTaskFrom(sourceTaskIds, taskId)

        await pipeline.dispatch('viewpreference.update_sorting', {
          context_key: ctx.sourceViewId,
          sorted_task_ids: newSourceOrder,
        })

        // 步骤 3: 更新目标视图排序（添加任务）
        const targetTaskIds = extractTaskIds(ctx.targetContext)
        const newTargetOrder = insertTaskAt(targetTaskIds, taskId, ctx.dropIndex)

        await pipeline.dispatch('viewpreference.update_sorting', {
          context_key: ctx.targetViewId,
          sorted_task_ids: newTargetOrder,
        })

        return {
          success: true,
          message: '✅ 任务已移动到新章节',
          affectedViews: [ctx.sourceViewId, ctx.targetViewId],
        }
      } catch (error) {
        return {
          success: false,
          message: `❌ ${error instanceof Error ? error.message : '操作失败'}`,
        }
      }
    },
  },

  tags: ['project', 'section'],
}

/**
 * 策略6: No Project 列表内部重排
 */
export const noProjectReorderStrategy: Strategy = {
  id: 'no-project-reorder',
  name: 'No Project Internal Reorder',

  conditions: {
    source: {
      viewKey: 'misc::no-project',
      objectType: 'task',
    },
    target: {
      viewKey: 'misc::no-project',
    },
    priority: 80,
  },

  action: {
    name: 'reorder_in_no_project',
    description: '在无项目任务列表中重新排序',
    async execute(ctx: StrategyContext): Promise<StrategyResult> {
      if (ctx.sourceViewId !== ctx.targetViewId) {
        return { success: false, message: '❌ 必须在同一视图内排序' }
      }

      try {
        const taskId = ctx.session.object.data.id
        const taskIds = extractTaskIds(ctx.targetContext)
        const oldIndex = taskIds.indexOf(taskId)

        let newOrder = [...taskIds]
        if (oldIndex !== -1) {
          newOrder = removeTaskFrom(newOrder, taskId)
        }
        newOrder = insertTaskAt(newOrder, taskId, ctx.dropIndex)

        await pipeline.dispatch('viewpreference.update_sorting', {
          context_key: 'misc::no-project',
          sorted_task_ids: newOrder,
        })

        return {
          success: true,
          message: '✅ 无项目任务已重新排序',
          affectedViews: ['misc::no-project'],
          reorderOnly: true,
        }
      } catch (error) {
        return {
          success: false,
          message: `❌ ${error instanceof Error ? error.message : '操作失败'}`,
        }
      }
    },
  },

  tags: ['project', 'reorder'],
}
