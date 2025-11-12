/**
 * Area Store - View Operations (DMA)
 *
 * 职责：数据加载操作（绕过指令系统的直接内存访问）
 */

import { pipeline } from '@/cpu'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * 加载所有 Areas
 */
export async function fetchAll() {
  try {
    await pipeline.dispatch('area.fetch_all', {})
    logger.info(LogTags.STORE_AREA, 'Fetched all areas')
  } catch (error) {
    logger.error(
      LogTags.STORE_AREA,
      'Failed to fetch areas',
      error instanceof Error ? error : new Error(String(error))
    )
    throw error
  }
}

/**
 * 加载单个 Area
 */
export async function fetchById(id: string) {
  try {
    await pipeline.dispatch('area.get', { id })
    logger.info(LogTags.STORE_AREA, 'Fetched area', { id })
  } catch (error) {
    logger.error(
      LogTags.STORE_AREA,
      'Failed to fetch area',
      error instanceof Error ? error : new Error(String(error)),
      { id }
    )
    throw error
  }
}
