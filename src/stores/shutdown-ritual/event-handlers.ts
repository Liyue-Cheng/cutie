/**
 * Shutdown Ritual Store event handlers (v4.0 - via INT)
 */

import type {
  ShutdownRitualProgress,
  ShutdownRitualStep,
  UpdateShutdownRitualStepSortResponse,
} from '@/types/dtos'
import { logger, LogTags } from '@/infra/logging/logger'
import * as core from './core'

export function initEventSubscriptions() {
  import('@/cpu/interrupt/InterruptHandler').then(({ interruptHandler }) => {
    interruptHandler.on('shutdown_ritual.step.created', handleEvent)
    interruptHandler.on('shutdown_ritual.step.updated', handleEvent)
    interruptHandler.on('shutdown_ritual.step.deleted', handleEvent)
    interruptHandler.on('shutdown_ritual.step.reordered', handleEvent)
    interruptHandler.on('shutdown_ritual.progress.toggled', handleEvent)
    interruptHandler.on('shutdown_ritual.settings.updated', handleEvent)

    logger.info(LogTags.SYSTEM_PIPELINE, 'Shutdown ritual event subscriptions initialized (via INT)')
  })
}

async function handleEvent(event: any) {
  try {
    const eventType = event.eventType as string
    const payload = event.payload

    switch (eventType) {
      case 'shutdown_ritual.settings.updated': {
        core.setTitle_mut((payload as { title: string | null }).title ?? null)
        break
      }
      case 'shutdown_ritual.step.created':
      case 'shutdown_ritual.step.updated': {
        core.addOrUpdateStep_mut(payload as ShutdownRitualStep)
        break
      }
      case 'shutdown_ritual.step.deleted': {
        core.removeStep_mut((payload as { id: string }).id)
        break
      }
      case 'shutdown_ritual.step.reordered': {
        const p = payload as UpdateShutdownRitualStepSortResponse
        core.updateStepRank_mut(p.step_id, p.new_rank)
        break
      }
      case 'shutdown_ritual.progress.toggled': {
        core.setProgress_mut(payload as ShutdownRitualProgress)
        break
      }
      default:
        logger.warn(LogTags.SYSTEM_PIPELINE, 'Unknown shutdown ritual event type', { eventType })
    }
  } catch (error) {
    logger.error(
      LogTags.SYSTEM_PIPELINE,
      'Failed to process shutdown ritual event',
      error instanceof Error ? error : new Error(String(error)),
      { eventType: event?.eventType }
    )
  }
}


