import { pipeline } from '@/cpu'
import type { ToolResult } from '../shared/cutie-tools'

interface UpdateTaskParams {
  task_id: string
  title?: string
  completed?: boolean
}

export async function updateTask(params: UpdateTaskParams): Promise<ToolResult> {
  if (!params.task_id) {
    return {
      success: false,
      message: '更新任务失败：缺少 task_id',
    }
  }

  try {
    // 1. 处理普通字段更新 (title)
    if (params.title) {
      await pipeline.dispatch('task.update', {
        id: params.task_id,
        updates: {
          title: params.title,
        },
      })
    }

    // 2. 处理完成状态
    if (typeof params.completed === 'boolean') {
      if (params.completed) {
        await pipeline.dispatch('task.complete', {
          id: params.task_id,
          view_context: 'ai_tool', // 标记来源
        })
      } else {
        await pipeline.dispatch('task.reopen', {
          id: params.task_id,
        })
      }
    }

    return {
      success: true,
      message: '任务更新成功',
      data: { id: params.task_id },
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    return {
      success: false,
      message: `更新任务失败：${errorMessage}`,
    }
  }
}
