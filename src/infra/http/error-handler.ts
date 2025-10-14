/**
 * 统一错误处理工具
 *
 * 提供标准化的错误处理和日志记录功能
 */

import { logger, LogTags } from '@/infra/logging/logger'

export interface ErrorContext {
  operation: string
  store: string
  details?: Record<string, any>
}

/**
 * 格式化错误信息
 * @param error 原始错误
 * @param context 错误上下文
 * @returns 格式化后的错误信息
 */
export function formatError(error: unknown, context: ErrorContext): string {
  const { operation, store, details } = context

  let errorMessage = `[${store}] Failed to ${operation}`

  if (error instanceof Error) {
    errorMessage += `: ${error.message}`
  } else {
    errorMessage += `: ${String(error)}`
  }

  return errorMessage
}

/**
 * 记录错误日志
 * @param error 原始错误
 * @param context 错误上下文
 */
export function logError(error: unknown, context: ErrorContext): void {
  const { operation, store, details } = context

  logger.error(
    LogTags.STORE_TASKS,
    `Error in ${operation}`,
    error instanceof Error ? error : new Error(String(error)),
    {
      store,
      operation,
      ...details,
    }
  )
}

/**
 * 处理 API 错误
 * @param error 原始错误
 * @param context 错误上下文
 * @returns 格式化后的错误信息
 */
export function handleApiError(error: unknown, context: ErrorContext): string {
  logError(error, context)
  return formatError(error, context)
}

/**
 * 创建错误处理器
 * @param store Store 名称
 * @returns 错误处理函数
 */
export function createErrorHandler(store: string) {
  return (error: unknown, operation: string, details?: Record<string, any>): string => {
    return handleApiError(error, { operation, store, details })
  }
}
