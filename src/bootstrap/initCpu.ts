/**
 * CPU Pipeline 初始化模块
 *
 * 负责启动 CPU 流水线系统
 * 必须在 Pinia 初始化之后、Store 使用之前调用
 */

import { logger } from '@/infra/logging/logger'

/**
 * 初始化 CPU Pipeline
 */
export async function initCpu(): Promise<void> {
  const { pipeline } = await import('@/cpu')

  pipeline.start()

  logger.info('System:Init', 'CPU Pipeline started')
}
