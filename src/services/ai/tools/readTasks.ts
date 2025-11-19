import { useTaskStore } from '@/stores/task'
import type { ToolResult } from '../shared/cutie-tools'

interface ReadTasksParams {
  view_context: string
}

export async function readTasks(params: ReadTasksParams): Promise<ToolResult> {
  if (!params.view_context) {
    return {
      success: false,
      message: '读取任务失败：缺少视图上下文（view_context）',
    }
  }

  const taskStore = useTaskStore()
  let data: any[] = []

  try {
    if (params.view_context.startsWith('daily::')) {
      const date = params.view_context.slice('daily::'.length)
      const result = await taskStore.fetchDailyTasks_DMA(date)
      data = result || []
    } else if (params.view_context === 'staging') {
      const result = await taskStore.fetchStagingTasks_DMA()
      data = result || []
    } else if (params.view_context === 'planned') {
      const result = await taskStore.fetchPlannedTasks_DMA()
      data = result || []
    } else {
      // 回退：尝试使用不完整的搜索或返回错误
      // 目前暂时返回空数组并提示
      return {
        success: false,
        message: `读取任务失败：不支持的视图上下文 ${params.view_context}（目前支持: daily::YYYY-MM-DD, staging, planned）`,
      }
    }

    return {
      success: true,
      message: `成功获取任务列表（view_context = ${params.view_context}）`,
      data,
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    return {
      success: false,
      message: `读取任务失败：${errorMessage}`,
    }
  }
}
