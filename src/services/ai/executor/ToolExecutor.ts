import type { ToolUse, ToolResult } from '../shared/cutie-tools'
import { createTask } from '../tools/createTask'
import { readTasks } from '../tools/readTasks'
import { updateTask } from '../tools/updateTask'
import { createSchedule } from '../tools/createSchedule'
import { deleteTask } from '../tools/deleteTask'
import { logger, LogTags } from '@/infra/logging'

function convertParams(xmlParams: Record<string, string>): Record<string, any> {
  const converted: Record<string, any> = {}

  for (const [key, raw] of Object.entries(xmlParams)) {
    const value = raw.trim()

    if (value === 'true') {
      converted[key] = true
    } else if (value === 'false') {
      converted[key] = false
    } else if (/^-?\d+$/.test(value)) {
      converted[key] = parseInt(value, 10)
    } else {
      converted[key] = value
    }
  }

  return converted
}

export async function executeToolCall(toolUse: ToolUse): Promise<ToolResult> {
  logger.debug(LogTags.AI_EXECUTOR, '开始执行工具', {
    toolName: toolUse.name,
    rawParams: toolUse.params,
  })

  try {
    const params = convertParams(toolUse.params)

    logger.debug(LogTags.AI_EXECUTOR, '参数转换完成', {
      toolName: toolUse.name,
      convertedParams: params,
    })

    switch (toolUse.name) {
      case 'create_task':
        return await createTask({
          title: params.title,
          area_id: params.area_id,
          scheduled_date: params.scheduled_date,
        })

      case 'read_tasks':
        return await readTasks({
          view_context: params.view_context,
        })

      case 'update_task':
        return await updateTask({
          task_id: params.task_id,
          title: params.title,
          completed: typeof params.completed === 'boolean' ? params.completed : undefined,
        })

      case 'create_schedule':
        return await createSchedule({
          task_id: params.task_id,
          scheduled_date: params.scheduled_date,
        })

      case 'delete_task':
        return await deleteTask({
          task_id: params.task_id,
        })

      default:
        logger.warn(LogTags.AI_EXECUTOR, '未知工具', { toolName: toolUse.name })
        return {
          success: false,
          message: `未知工具: ${toolUse.name}`,
        }
    }
  } catch (e: any) {
    const error = e instanceof Error ? e : new Error(String(e))
    logger.error(LogTags.AI_EXECUTOR, '工具执行失败', {
      toolName: toolUse.name,
      error: error.message,
    })
    return {
      success: false,
      message: `执行失败: ${error.message}`,
    }
  }
}
