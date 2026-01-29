/**
 * Area 指令集（声明式架构版）
 *
 * 特点：
 * 1. 使用声明式 request 配置
 * 2. 统一的 commit 逻辑
 * 3. 支持 AI 自动染色功能
 * 4. 完整的 CRUD 操作
 */

import type { ISADefinition } from 'front-cpu'
import type { Area } from '@/stores/area'
import { useAreaStore } from '@/stores/area'

export const AreaISA: ISADefinition = {
  'area.fetch_all': {
    meta: {
      description: '获取所有 Areas',
      category: 'area',
      resourceIdentifier: () => ['areas:all'],
      priority: 5,
      timeout: 5000,
    },

    validate: async () => true,

    request: {
      method: 'GET',
      url: '/areas',
    },

    commit: async (result: Area[]) => {
      const areaStore = useAreaStore()
      areaStore.replaceAll_mut(result)
    },
  },

  'area.get': {
    meta: {
      description: '获取单个 Area',
      category: 'area',
      resourceIdentifier: (payload) => [`area:${payload.id}`],
      priority: 5,
      timeout: 5000,
    },

    validate: async (payload) => {
      if (!payload.id?.trim()) {
        console.warn('❌ Area ID 不能为空')
        return false
      }
      return true
    },

    request: {
      method: 'GET',
      url: (payload) => `/areas/${payload.id}`,
    },

    commit: async (result: Area) => {
      const areaStore = useAreaStore()
      areaStore.addOrUpdate_mut(result)
    },
  },

  'area.create': {
    meta: {
      description: '创建 Area',
      category: 'area',
      resourceIdentifier: () => [],
      priority: 6,
      timeout: 5000,
    },

    validate: async (payload) => {
      if (!payload.name?.trim()) {
        console.warn('❌ Area 名称不能为空')
        return false
      }
      if (!payload.color?.trim()) {
        console.warn('❌ Area 颜色不能为空')
        return false
      }
      return true
    },

    request: {
      method: 'POST',
      url: '/areas',
      body: (payload) => payload,
    },

    commit: async (result: Area) => {
      const areaStore = useAreaStore()
      areaStore.addOrUpdate_mut(result)
    },
  },

  'area.update': {
    meta: {
      description: '更新 Area',
      category: 'area',
      resourceIdentifier: (payload) => [`area:${payload.id}`],
      priority: 6,
      timeout: 5000,
    },

    validate: async (payload) => {
      if (!payload.id?.trim()) {
        console.warn('❌ Area ID 不能为空')
        return false
      }
      return true
    },

    request: {
      method: 'PATCH',
      url: (payload) => `/areas/${payload.id}`,
      body: (payload) => {
        const body: Record<string, any> = {}
        if (payload.name !== undefined) body.name = payload.name
        if (payload.color !== undefined) body.color = payload.color
        if (payload.parent_area_id !== undefined) body.parent_area_id = payload.parent_area_id
        return body
      },
    },

    commit: async (result: Area) => {
      const areaStore = useAreaStore()
      areaStore.addOrUpdate_mut(result)
    },
  },

  'area.delete': {
    meta: {
      description: '删除 Area',
      category: 'area',
      resourceIdentifier: (payload) => [`area:${payload.id}`],
      priority: 6,
      timeout: 5000,
    },

    validate: async (payload) => {
      if (!payload.id?.trim()) {
        console.warn('❌ Area ID 不能为空')
        return false
      }
      return true
    },

    request: {
      method: 'DELETE',
      url: (payload) => `/areas/${payload.id}`,
    },

    commit: async (_result: any, payload: any) => {
      const areaStore = useAreaStore()
      areaStore.remove_mut(payload.id)
    },
  },

  'area.suggest_color': {
    meta: {
      description: 'AI 根据 Area 名称推荐颜色',
      category: 'area',
      resourceIdentifier: (payload) => [`area:suggest_color:${payload.area_name}`],
      priority: 5,
      timeout: 15000, // AI 调用可能较慢
    },

    validate: async (payload) => {
      if (!payload.area_name?.trim()) {
        console.warn('❌ Area 名称不能为空')
        return false
      }
      return true
    },

    request: {
      method: 'POST',
      url: '/areas/suggest-color',
      body: (payload) => ({
        area_name: payload.area_name,
      }),
    },

    commit: async (result: { suggested_color: string }) => {
      // AI 染色不直接修改 store，只返回建议的颜色值
      // 由调用方决定是否应用
      console.log('✅ AI 建议颜色:', result.suggested_color)
    },
  },
}
