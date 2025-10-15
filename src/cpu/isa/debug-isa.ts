/**
 * 调试用指令集
 */

import type { ISADefinition } from './types'

export const DebugISA: ISADefinition = {
  'debug.fetch_baidu': {
    meta: {
      description: '向百度发送GET请求',
      category: 'debug',
      resourceIdentifier: () => ['http:baidu'],
      priority: 5,
      timeout: 10000,
    },
    execute: async (payload, context) => {
      const response = await fetch('https://www.baidu.com', {
        method: 'GET',
        mode: 'no-cors', // 避免CORS问题
      })
      return {
        status: response.status || 'opaque',
        type: response.type,
        correlationId: context.correlationId,
        timestamp: Date.now(),
      }
    },
  },

  'debug.fetch_with_delay': {
    meta: {
      description: '带延迟的请求（测试流水线）',
      category: 'debug',
      resourceIdentifier: (payload) => [`delayed:${payload.id || 'default'}`],
      priority: 5,
      timeout: 15000,
    },
    validate: async (payload) => {
      if (payload.delay && payload.delay > 10000) {
        console.warn('延迟时间过长，最大10秒')
        return false
      }
      return true
    },
    execute: async (payload, context) => {
      const delay = payload.delay || 2000
      await new Promise((resolve) => setTimeout(resolve, delay))
      return {
        delayed: delay,
        message: `请求延迟了 ${delay}ms`,
        correlationId: context.correlationId,
        timestamp: Date.now(),
      }
    },
  },

  'debug.fetch_fail': {
    meta: {
      description: '必定失败的请求（测试错误处理）',
      category: 'debug',
      resourceIdentifier: () => ['fail:test'],
      priority: 5,
    },
    execute: async (payload, context) => {
      await new Promise((resolve) => setTimeout(resolve, 500))
      throw new Error(payload.errorMessage || '模拟的网络请求失败')
    },
  },

  'debug.quick_success': {
    meta: {
      description: '立即成功的指令',
      category: 'debug',
      resourceIdentifier: (payload) => [`quick:${payload.id || 'default'}`],
      priority: 8,
    },
    execute: async (payload, context) => {
      return {
        success: true,
        message: '立即成功',
        data: payload.data || null,
        correlationId: context.correlationId,
        timestamp: Date.now(),
      }
    },
  },

  'debug.conflicting_resource': {
    meta: {
      description: '测试资源冲突（操作相同资源ID）',
      category: 'debug',
      resourceIdentifier: () => ['resource:shared'], // 固定使用相同资源ID
      priority: 5,
    },
    execute: async (payload, context) => {
      const delay = payload.delay || 1000
      await new Promise((resolve) => setTimeout(resolve, delay))
      return {
        resource: 'shared',
        message: '访问共享资源成功',
        correlationId: context.correlationId,
        timestamp: Date.now(),
      }
    },
  },

  'debug.test_timeout': {
    meta: {
      description: '测试超时机制（5秒超时，但执行10秒）',
      category: 'debug',
      resourceIdentifier: () => ['timeout:test'],
      priority: 5,
      timeout: 5000, // 🔥 5 秒超时
    },
    execute: async (payload, context) => {
      // 故意执行 10 秒（会触发超时）
      const delay = payload.delay || 10000
      await new Promise((resolve) => setTimeout(resolve, delay))
      return {
        success: true,
        message: '不应该看到这个（因为会超时）',
        correlationId: context.correlationId,
        timestamp: Date.now(),
      }
    },
  },
}
