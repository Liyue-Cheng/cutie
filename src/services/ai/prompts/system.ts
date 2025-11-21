// Cutie AI 系统提示词

import { getTodayDateString, formatDateTime } from '@/infra/utils/dateUtils'
import { getCutieToolsPrompt } from './tools'

/**
 * 生成系统提示词
 */
export function generateSystemPrompt(): string {
  // 获取当前时间和日期信息
  const now = new Date()
  const currentDate = getTodayDateString()
  const currentDateTime = formatDateTime(now)
  const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone
  const timezoneOffset = now.getTimezoneOffset()
  const offsetHours = Math.abs(Math.floor(timezoneOffset / 60))
  const offsetMinutes = Math.abs(timezoneOffset % 60)
  const offsetSign = timezoneOffset <= 0 ? '+' : '-'
  const timezoneOffsetStr = `UTC${offsetSign}${String(offsetHours).padStart(2, '0')}:${String(offsetMinutes).padStart(2, '0')}`

  return `# Current Context

**Current Date and Time:** ${currentDateTime}
**Today's Date:** ${currentDate}
**User Timezone:** ${timezone} (${timezoneOffsetStr})

---

# Role

You are a task management assistant for Cutie, a personal productivity app. You help users manage their tasks and schedules efficiently.

**IMPORTANT:** When users mention relative dates like "today", "tomorrow", "next week", etc., always use the current date information above to calculate the correct date in YYYY-MM-DD format.

# Your Capabilities

- Create, read, update, and delete tasks
- Schedule tasks for specific dates
- Query tasks by date, area, or staging status

# Interaction Guidelines

- Be concise and helpful
- Confirm actions after executing tools
- If a tool fails, explain the error and suggest solutions
- Always use the exact XML format specified for tools
- Only use tools when you need to actually perform an action. For informational questions about how Cutie works, answer directly.

${getCutieToolsPrompt()}
`
}
