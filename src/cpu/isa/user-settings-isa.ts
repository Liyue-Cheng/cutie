/**
 * 用户设置指令集（声明式架构版）
 *
 * 特点：
 * 1. 使用声明式 request 配置
 * 2. 统一的 commit 逻辑
 * 3. 支持批量操作
 */

import type { ISADefinition } from '@cutie/cpu-pipeline'
import type {
  UserSettingDto,
  UpdateSettingRequest,
  UpdateBatchSettingsRequest,
  BatchUpdateResponse,
  ResetResponse,
} from '@/types/user-settings'
import { useUserSettingsStore } from '@/stores/user-settings'

export const UserSettingsISA: ISADefinition = {
  'user_settings.fetch_all': {
    meta: {
      description: '获取所有用户设置',
      category: 'user_settings',
      resourceIdentifier: () => ['user_settings:all'],
      priority: 5,
      timeout: 5000,
    },

    validate: async () => true,

    request: {
      method: 'GET',
      url: '/user-settings',
    },

    commit: async (result: UserSettingDto[]) => {
      const store = useUserSettingsStore()
      store.replaceAll_mut(result)
    },
  },

  'user_settings.get': {
    meta: {
      description: '获取单个设置',
      category: 'user_settings',
      resourceIdentifier: (payload) => [`user_setting:${payload.key}`],
      priority: 5,
      timeout: 5000,
    },

    validate: async (payload) => {
      if (!payload.key?.trim()) {
        console.warn('❌ 设置键不能为空')
        return false
      }
      return true
    },

    request: {
      method: 'GET',
      url: (payload) => `/user-settings/${payload.key}`,
    },

    commit: async (result: UserSettingDto) => {
      const store = useUserSettingsStore()
      store.addOrUpdateSetting_mut(result)
    },
  },

  'user_settings.update': {
    meta: {
      description: '更新单个设置',
      category: 'user_settings',
      resourceIdentifier: (payload) => [`user_setting:${payload.key}`],
      priority: 6,
      timeout: 5000,
    },

    validate: async (payload) => {
      if (!payload.key?.trim()) {
        console.warn('❌ 设置键不能为空')
        return false
      }
      if (payload.value === undefined) {
        console.warn('❌ 设置值不能为 undefined')
        return false
      }
      if (!payload.value_type) {
        console.warn('❌ 值类型不能为空')
        return false
      }
      return true
    },

    request: {
      method: 'PUT',
      url: (payload) => `/user-settings/${payload.key}`,
      body: (payload) => ({
        value: payload.value,
        value_type: payload.value_type,
      } as UpdateSettingRequest),
    },

    commit: async (result: UserSettingDto) => {
      const store = useUserSettingsStore()
      store.addOrUpdateSetting_mut(result)
    },
  },

  'user_settings.update_batch': {
    meta: {
      description: '批量更新设置',
      category: 'user_settings',
      resourceIdentifier: () => ['user_settings:batch'],
      priority: 6,
      timeout: 10000,
    },

    validate: async (payload) => {
      if (!payload.settings || !Array.isArray(payload.settings)) {
        console.warn('❌ settings 必须是数组')
        return false
      }
      if (payload.settings.length === 0) {
        console.warn('❌ settings 不能为空')
        return false
      }
      for (const setting of payload.settings) {
        if (!setting.key?.trim()) {
          console.warn('❌ 设置键不能为空')
          return false
        }
        if (setting.value === undefined) {
          console.warn('❌ 设置值不能为 undefined')
          return false
        }
      }
      return true
    },

    request: {
      method: 'PUT',
      url: '/user-settings',
      body: (payload) => payload as UpdateBatchSettingsRequest,
    },

    commit: async (result: BatchUpdateResponse) => {
      const store = useUserSettingsStore()
      store.addOrUpdateBatch_mut(result.settings)
    },
  },

  'user_settings.reset': {
    meta: {
      description: '重置所有设置为默认值',
      category: 'user_settings',
      resourceIdentifier: () => ['user_settings:all'],
      priority: 7,
      timeout: 5000,
    },

    validate: async () => true,

    request: {
      method: 'POST',
      url: '/user-settings/reset',
      body: () => ({}),
    },

    commit: async (result: ResetResponse) => {
      const store = useUserSettingsStore()
      store.replaceAll_mut(result.settings)
    },
  },
}

