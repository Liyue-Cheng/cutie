// Cutie AI 系统提示词
// 复用 Kilocode 的 XML 工具调用格式

import { getSystemPromptWithTools } from './tools'

export function generateSystemPrompt(): string {
  return getSystemPromptWithTools()
}
