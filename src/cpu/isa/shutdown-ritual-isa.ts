import type { ISADefinition } from 'front-cpu'
import type {
  ShutdownRitualProgress,
  ShutdownRitualSettings,
  ShutdownRitualState,
  ShutdownRitualStep,
  UpdateShutdownRitualStepSortResponse,
} from '@/types/dtos'
import { useShutdownRitualStore } from '@/stores/shutdown-ritual'
import type {
  CreateShutdownRitualStepPayload,
  DeleteShutdownRitualStepPayload,
  ReorderShutdownRitualStepPayload,
  ToggleShutdownRitualProgressPayload,
  UpdateShutdownRitualSettingsPayload,
  UpdateShutdownRitualStepPayload,
} from '@/stores/shutdown-ritual'

export const ShutdownRitualISA: ISADefinition = {
  'shutdown_ritual.fetch_state': {
    meta: {
      description: '获取每日收尾小仪式状态（某一天）',
      category: 'shutdown_ritual',
      resourceIdentifier: (payload: { date: string }) => [`shutdown_ritual:state:${payload.date}`],
      priority: 3,
      timeout: 10000,
    },
    request: {
      method: 'GET',
      url: (payload: { date: string }) => `/shutdown-ritual/state?date=${encodeURIComponent(payload.date)}`,
    },
    commit: async (result: ShutdownRitualState) => {
      const store = useShutdownRitualStore()
      store.setState_mut(result)
    },
  },

  'shutdown_ritual.settings.update': {
    meta: {
      description: '更新每日收尾小仪式配置（标题等）',
      category: 'shutdown_ritual',
      resourceIdentifier: () => ['shutdown_ritual:settings'],
      priority: 6,
      timeout: 10000,
    },
    request: {
      method: 'PATCH',
      url: '/shutdown-ritual/settings',
      body: (payload: UpdateShutdownRitualSettingsPayload) => ({
        title: payload.title,
      }),
    },
    commit: async (result: ShutdownRitualSettings) => {
      const store = useShutdownRitualStore()
      store.setTitle_mut(result.title)
    },
  },

  'shutdown_ritual.step.create': {
    meta: {
      description: '创建小仪式步骤',
      category: 'shutdown_ritual',
      resourceIdentifier: () => [],
      priority: 6,
      timeout: 10000,
    },
    validate: async (payload: CreateShutdownRitualStepPayload) => {
      return Boolean(payload.title?.trim())
    },
    request: {
      method: 'POST',
      url: '/shutdown-ritual/steps',
      body: (payload: CreateShutdownRitualStepPayload) => ({ title: payload.title }),
    },
    commit: async (result: ShutdownRitualStep) => {
      const store = useShutdownRitualStore()
      store.addOrUpdateStep_mut(result)
    },
  },

  'shutdown_ritual.step.update': {
    meta: {
      description: '更新小仪式步骤标题',
      category: 'shutdown_ritual',
      resourceIdentifier: (payload: UpdateShutdownRitualStepPayload) => [`shutdown_ritual:step:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },
    validate: async (payload: UpdateShutdownRitualStepPayload) => {
      return Boolean(payload.id) && Boolean(payload.title?.trim())
    },
    request: {
      method: 'PATCH',
      url: (payload: UpdateShutdownRitualStepPayload) => `/shutdown-ritual/steps/${payload.id}`,
      body: (payload: UpdateShutdownRitualStepPayload) => ({ title: payload.title }),
    },
    commit: async (result: ShutdownRitualStep) => {
      const store = useShutdownRitualStore()
      store.addOrUpdateStep_mut(result)
    },
  },

  'shutdown_ritual.step.delete': {
    meta: {
      description: '删除小仪式步骤',
      category: 'shutdown_ritual',
      resourceIdentifier: (payload: DeleteShutdownRitualStepPayload) => [`shutdown_ritual:step:${payload.id}`],
      priority: 6,
      timeout: 10000,
    },
    validate: async (payload: DeleteShutdownRitualStepPayload) => Boolean(payload.id),
    request: {
      method: 'DELETE',
      url: (payload: DeleteShutdownRitualStepPayload) => `/shutdown-ritual/steps/${payload.id}`,
    },
    commit: async (_result: { id: string }, payload: DeleteShutdownRitualStepPayload) => {
      const store = useShutdownRitualStore()
      store.removeStep_mut(payload.id)
    },
  },

  'shutdown_ritual.step.reorder': {
    meta: {
      description: '更新小仪式步骤排序（LexoRank）',
      category: 'shutdown_ritual',
      resourceIdentifier: (payload: ReorderShutdownRitualStepPayload) => [`shutdown_ritual:step:${payload.step_id}`],
      priority: 6,
      timeout: 10000,
    },
    validate: async (payload: ReorderShutdownRitualStepPayload) => Boolean(payload.step_id),
    request: {
      method: 'PATCH',
      url: (payload: ReorderShutdownRitualStepPayload) =>
        `/shutdown-ritual/steps/${payload.step_id}/order-rank`,
      body: (payload: ReorderShutdownRitualStepPayload) => ({
        prev_step_id: payload.prev_step_id ?? null,
        next_step_id: payload.next_step_id ?? null,
      }),
    },
    commit: async (result: UpdateShutdownRitualStepSortResponse) => {
      const store = useShutdownRitualStore()
      store.updateStepRank_mut(result.step_id, result.new_rank)
    },
  },

  'shutdown_ritual.progress.toggle': {
    meta: {
      description: '切换小仪式步骤完成状态（当日）',
      category: 'shutdown_ritual',
      resourceIdentifier: (payload: ToggleShutdownRitualProgressPayload) => [
        `shutdown_ritual:progress:${payload.date}:${payload.step_id}`,
      ],
      priority: 6,
      timeout: 10000,
    },
    validate: async (payload: ToggleShutdownRitualProgressPayload) => Boolean(payload.step_id) && Boolean(payload.date),
    request: {
      method: 'POST',
      url: '/shutdown-ritual/progress/toggle',
      body: (payload: ToggleShutdownRitualProgressPayload) => ({
        step_id: payload.step_id,
        date: payload.date,
      }),
    },
    commit: async (result: ShutdownRitualProgress) => {
      const store = useShutdownRitualStore()
      store.setProgress_mut(result)
    },
  },
}


