import { pipeline } from '@/cpu'
import type { ToolResult } from '../shared/cutie-tools'

interface CreateTaskParams {
  title: string
  area_id?: string
  scheduled_date?: string
}

export async function createTask(params: CreateTaskParams): Promise<ToolResult> {
  if (!params.title) {
    return {
      success: false,
      message: '创建任务失败：缺少标题（title）',
    }
  }

  try {
    let result

    if (params.scheduled_date) {
      // 如果有日期，使用 task.create_with_schedule
      result = await pipeline.dispatch('task.create_with_schedule', {
        title: params.title,
        scheduled_day: params.scheduled_date,
        // area_id 支持吗？查看 TaskISA 好像不支持，暂忽略
      })
    } else {
      // 否则使用 task.create
      result = await pipeline.dispatch('task.create', {
        title: params.title,
        area_id: params.area_id || undefined,
      })
    }

    return {
      success: true,
      message: `任务「${params.title}」已创建`,
      data: result,
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    return {
      success: false,
      message: `创建任务失败：${errorMessage}`,
    }
  }
}
