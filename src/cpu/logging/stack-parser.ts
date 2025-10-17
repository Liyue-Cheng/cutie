/**
 * 调用栈解析工具
 *
 * 用于从 Error.stack 中提取调用源信息（文件路径、行号、列号）
 */

import type { CallSource } from './types'

/**
 * 捕获并解析调用栈，获取调用源信息
 *
 * @param skipFrames 跳过的栈帧数（用于跳过内部函数）
 * @returns 调用源信息，解析失败返回 undefined
 */
export function captureCallSource(skipFrames: number = 0): CallSource | undefined {
  try {
    const error = new Error()
    const stack = error.stack

    if (!stack) {
      return undefined
    }

    // 解析调用栈
    // 典型格式：
    // Chrome: "    at functionName (file:///path/to/file.ts:123:45)"
    // Firefox: "functionName@file:///path/to/file.ts:123:45"
    const lines = stack.split('\n')

    // 跳过第一行（Error message）和 captureCallSource 本身
    // 从第 2 行开始（索引 2），再加上额外跳过的帧数
    const startIndex = 2 + skipFrames

    // 🔥 改进：查找第一个可以成功解析的行
    // 这样可以跳过 Promise constructor、async wrapper 等无法解析的行
    for (let i = startIndex; i < lines.length; i++) {
      const line = lines[i]
      if (!line) continue

      // 跳过不包含有用信息的行
      if (line.includes('<anonymous>') || line.includes('new Promise')) {
        continue
      }

      // 跳过 CPU 内部文件（Pipeline.ts, IF.ts 等）
      if (
        line.includes('/cpu/Pipeline.ts') ||
        line.includes('/cpu/stages/') ||
        line.includes('/cpu/logging/')
      ) {
        continue
      }

      const result = parseStackLine(line)

      if (result) {
        return result
      }
    }

    return undefined
  } catch (error) {
    console.warn('Failed to capture call source:', error)
    return undefined
  }
}

/**
 * 解析单行调用栈
 */
function parseStackLine(line: string): CallSource | undefined {
  try {
    // Chrome/V8 格式: "    at functionName (file:///path/to/file.ts:123:45)"
    // 或: "    at file:///path/to/file.ts:123:45"
    const chromeMatch = line.match(/at\s+(?:(.+?)\s+\()?(.+?):(\d+):(\d+)\)?/)
    if (chromeMatch) {
      const [, functionName, filePath, lineStr, columnStr] = chromeMatch
      if (!filePath || !lineStr || !columnStr) {
        return undefined
      }
      return {
        file: cleanFilePath(filePath),
        line: parseInt(lineStr, 10),
        column: parseInt(columnStr, 10),
        function: functionName?.trim() || undefined,
        raw: line.trim(),
      }
    }

    // Firefox 格式: "functionName@file:///path/to/file.ts:123:45"
    const firefoxMatch = line.match(/(.+?)@(.+?):(\d+):(\d+)/)
    if (firefoxMatch) {
      const [, functionName, filePath, lineStr, columnStr] = firefoxMatch
      if (!filePath || !lineStr || !columnStr) {
        return undefined
      }
      return {
        file: cleanFilePath(filePath),
        line: parseInt(lineStr, 10),
        column: parseInt(columnStr, 10),
        function: functionName?.trim() || undefined,
        raw: line.trim(),
      }
    }

    return undefined
  } catch (error) {
    console.warn('Failed to parse stack line:', line, error)
    return undefined
  }
}

/**
 * 清理文件路径
 * - 移除 file:// 协议
 * - 移除 webpack:// 前缀
 * - 移除 Vite 时间戳参数 (?t=...)
 * - 转换为相对于项目根目录的路径
 */
function cleanFilePath(filePath: string): string {
  let cleaned = filePath

  // 移除 file:// 协议
  cleaned = cleaned.replace(/^file:\/\/\//, '')

  // 移除 webpack:// 前缀
  cleaned = cleaned.replace(/^webpack:\/\/\//, '')

  // 移除 http:// 或 https://（开发服务器）
  cleaned = cleaned.replace(/^https?:\/\/[^/]+\//, '')

  // 🔥 移除 Vite 时间戳参数 (?t=1760628451326)
  cleaned = cleaned.replace(/\?t=\d+/, '')

  // 移除其他查询参数
  const withoutQuery = cleaned.split('?')[0]
  cleaned = withoutQuery || cleaned

  // 尝试提取相对路径（从 src/ 或 @/ 开始）
  const srcMatch = cleaned.match(/(src\/.+)/)
  if (srcMatch && srcMatch[1]) {
    return srcMatch[1]
  }

  // 如果包含完整路径，尝试提取文件名和上层目录
  const segments = cleaned.split('/')
  if (segments.length >= 3) {
    // 返回最后3个段（例如：components/parts/TaskCard.vue）
    return segments.slice(-3).join('/')
  }

  return cleaned
}

/**
 * 格式化调用源为可读字符串
 */
export function formatCallSource(callSource: CallSource): string {
  const funcPrefix = callSource.function ? `${callSource.function} @ ` : ''
  return `${funcPrefix}${callSource.file}:${callSource.line}:${callSource.column}`
}

/**
 * 格式化调用源为简短字符串（仅文件和行号）
 */
export function formatCallSourceShort(callSource: CallSource): string {
  return `${callSource.file}:${callSource.line}`
}
