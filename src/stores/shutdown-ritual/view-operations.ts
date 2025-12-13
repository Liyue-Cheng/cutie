import { apiGet } from '@/stores/shared'
import { logger, LogTags } from '@/infra/logging/logger'
import type { ShutdownRitualState } from '@/types/dtos'
import * as core from './core'

/**
 * DMA: fetch shutdown ritual state for a date
 * API: GET /shutdown-ritual/state?date=YYYY-MM-DD
 */
export async function fetchState_DMA(date: string): Promise<ShutdownRitualState> {
  const encoded = encodeURIComponent(date)
  const state: ShutdownRitualState = await apiGet(`/shutdown-ritual/state?date=${encoded}`)
  core.setState_mut(state)
  logger.info(LogTags.STORE_TASKS, 'DMA: Loaded shutdown ritual state', {
    date: state.date,
    steps: state.steps.length,
    progress: state.progress.length,
  })
  return state
}


