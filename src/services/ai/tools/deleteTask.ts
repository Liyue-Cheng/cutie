import { pipeline } from '@/cpu'
import type { ToolResult } from '../shared/cutie-tools'

interface DeleteTaskParams {
  task_id: string
}

export async function deleteTask(params: DeleteTaskParams): Promise<ToolResult> {
  if (!params.task_id) {
    return {
      success: false,
      message: '删除任务失败：缺少 task_id',
    }
  }

  try {
    await pipeline.dispatch('task.delete', {
      id: params.task_id,
    })

    return {
      success: true,
      message: '任务已删除',
      data: null,
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    return {
      success: false,
      message: `删除任务失败：${errorMessage}`,
    }
  }
}
