// Cutie 任务管理工具定义
// 基于 Kilocode 的工具系统，精简为只支持任务相关操作

export interface TextContent {
  type: 'text'
  content: string
  partial: boolean
}

export interface ToolUse {
  type: 'tool_use'
  name: ToolName
  params: Record<string, string>
  partial: boolean
  toolUseId?: string
}

export type AssistantMessageContent = TextContent | ToolUse

// Cutie 支持的工具名称
export const toolNames = [
  'create_task',
  'read_tasks',
  'update_task',
  'delete_task',
  'create_schedule',
] as const

export type ToolName = (typeof toolNames)[number]

// Cutie 工具的参数名称
export const toolParamNames = [
  'title',
  'area_id',
  'scheduled_date',
  'view_context',
  'task_id',
  'completed',
] as const

export type ToolParamName = (typeof toolParamNames)[number]

export interface ToolResult {
  success: boolean
  message: string
  data?: any
}
