import { defineStore } from 'pinia'
import * as core from './core'
import * as view from './view-operations'
import * as events from './event-handlers'

export const useShutdownRitualStore = defineStore('shutdown-ritual', () => {
  return {
    // STATE
    currentDate: core.currentDate,
    steps: core.steps,
    progressByStepId: core.progressByStepId,

    // GETTERS
    allStepsOrdered: core.allStepsOrdered,
    getStepById: core.getStepById,
    getProgressByStepId_Mux: core.getProgressByStepId_Mux,
    completedCount: core.completedCount,
    totalCount: core.totalCount,

    // MUTATIONS
    setState_mut: core.setState_mut,
    addOrUpdateStep_mut: core.addOrUpdateStep_mut,
    removeStep_mut: core.removeStep_mut,
    updateStepRank_mut: core.updateStepRank_mut,
    setProgress_mut: core.setProgress_mut,

    // DMA
    fetchState_DMA: view.fetchState_DMA,

    // EVENTS
    initEventSubscriptions: events.initEventSubscriptions,
  }
})

export type {
  CreateShutdownRitualStepPayload,
  DeleteShutdownRitualStepPayload,
  ReorderShutdownRitualStepPayload,
  ToggleShutdownRitualProgressPayload,
  UpdateShutdownRitualStepPayload,
} from './types'


