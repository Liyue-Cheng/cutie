import { pipeline } from '@/cpu'
import type { ToolResult } from '../shared/cutie-tools'

interface CreateScheduleParams {
  task_id: string
  scheduled_date: string
}

export async function createSchedule(params: CreateScheduleParams): Promise<ToolResult> {
  if (!params.task_id) {
    return {
      success: false,
      message: '创建日程失败：缺少 task_id',
    }
  }
  if (!params.scheduled_date) {
    return {
      success: false,
      message: '创建日程失败：缺少 scheduled_date',
    }
  }

  try {
    await pipeline.dispatch('schedule.create', {
      task_id: params.task_id,
      scheduled_day: params.scheduled_date,
    })

    return {
      success: true,
      message: `已为任务创建日程：${params.scheduled_date}`,
      data: null,
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    return {
      success: false,
      message: `创建日程失败：${errorMessage}`,
    }
  }
}
