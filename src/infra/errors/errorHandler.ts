/**
 * 全局错误处理器
 * 捕获未处理的异常和Promise拒绝，并通过日志系统记录
 */

import { logger, LogTags } from '../logging/logger'

/**
 * 设置全局错误捕获
 */
export function setupGlobalErrorHandling(): void {
  // 捕获未处理的JavaScript错误
  window.addEventListener('error', (event) => {
    logger.error(LogTags.CRITICAL, 'Uncaught JavaScript error', event.error, {
      message: event.message,
      filename: event.filename,
      lineno: event.lineno,
      colno: event.colno,
      stack: event.error?.stack,
    })
  })

  // 捕获未处理的Promise拒绝
  window.addEventListener('unhandledrejection', (event) => {
    logger.error(LogTags.CRITICAL, 'Unhandled Promise rejection', event.reason, {
      reason: event.reason,
      stack: event.reason?.stack,
    })
  })

  // 捕获资源加载错误（可选）
  window.addEventListener(
    'error',
    (event) => {
      if (event.target && event.target !== window) {
        const target = event.target as HTMLElement
        logger.error(LogTags.CRITICAL, 'Resource loading error', undefined, {
          tagName: target.tagName,
          src: (target as any).src || (target as any).href,
          type: event.type,
        })
      }
    },
    true
  ) // 使用捕获阶段
}

/**
 * Vue错误处理器
 * 需要在Vue应用中手动调用
 */
export function createVueErrorHandler() {
  return (error: unknown, instance: any, info: string) => {
    logger.error(
      LogTags.CRITICAL,
      'Vue error',
      error instanceof Error ? error : new Error(String(error)),
      {
        componentName: instance?.$options?.name || 'Unknown',
        errorInfo: info,
        stack: error instanceof Error ? error.stack : undefined,
      }
    )
  }
}

/**
 * Vue警告处理器
 * 需要在Vue应用中手动调用
 */
export function createVueWarnHandler() {
  return (message: string, instance: any, trace: string) => {
    logger.warn(LogTags.CRITICAL, 'Vue warning', {
      message,
      componentName: instance?.$options?.name || 'Unknown',
      trace,
    })
  }
}
