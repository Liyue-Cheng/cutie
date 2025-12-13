import { computed, ref } from 'vue'
import type { ShutdownRitualProgress, ShutdownRitualState, ShutdownRitualStep } from '@/types/dtos'

/**
 * Shutdown Ritual Store Core
 *
 * - steps: persistent templates
 * - progress: per-day completion state (keyed by step_id)
 */

export const currentDate = ref<string | null>(null)
export const steps = ref(new Map<string, ShutdownRitualStep>())
export const progressByStepId = ref(new Map<string, ShutdownRitualProgress>())

export const allStepsOrdered = computed(() => {
  return Array.from(steps.value.values()).sort((a, b) => a.order_rank.localeCompare(b.order_rank))
})

export const getStepById = computed(() => (id: string) => {
  return steps.value.get(id)
})

export const getProgressByStepId_Mux = computed(() => (stepId: string) => {
  return progressByStepId.value.get(stepId) || null
})

export const completedCount = computed(() => {
  let n = 0
  for (const p of progressByStepId.value.values()) {
    if (p.completed_at) n++
  }
  return n
})

export const totalCount = computed(() => steps.value.size)

export function setState_mut(state: ShutdownRitualState) {
  currentDate.value = state.date

  const stepMap = new Map<string, ShutdownRitualStep>()
  for (const s of state.steps) stepMap.set(s.id, s)
  steps.value = stepMap

  const progressMap = new Map<string, ShutdownRitualProgress>()
  for (const p of state.progress) progressMap.set(p.step_id, p)
  progressByStepId.value = progressMap
}

export function addOrUpdateStep_mut(step: ShutdownRitualStep) {
  const next = new Map(steps.value)
  next.set(step.id, step)
  steps.value = next
}

export function removeStep_mut(id: string) {
  const nextSteps = new Map(steps.value)
  nextSteps.delete(id)
  steps.value = nextSteps

  const nextProgress = new Map(progressByStepId.value)
  nextProgress.delete(id)
  progressByStepId.value = nextProgress
}

export function updateStepRank_mut(stepId: string, newRank: string) {
  const existing = steps.value.get(stepId)
  if (!existing) return
  addOrUpdateStep_mut({ ...existing, order_rank: newRank })
}

export function setProgress_mut(progress: ShutdownRitualProgress) {
  // If store hasn't loaded a date yet, accept it.
  // If date mismatch, ignore to avoid cross-day pollution.
  if (currentDate.value && progress.date !== currentDate.value) return

  const next = new Map(progressByStepId.value)
  next.set(progress.step_id, progress)
  progressByStepId.value = next
}


