/**
 * API 配置模块
 *
 * 职责：
 * 1. 发现后端 sidecar 端口
 * 2. 初始化 SSE 事件订阅器
 * 3. 提供 API 基础 URL
 *
 * 注意：Store 的事件订阅初始化已移至 bootstrap/initStores.ts
 */

import { ref, computed } from 'vue'
import { logger, LogTags } from '@/infra/logging/logger'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// 默认端口（fallback）
const DEFAULT_PORT = 3030

// 响应式的端口状态
const sidecarPort = ref<number | null>(null)
const isPortDiscovered = ref(false)

// 计算属性：API基础URL
export const apiBaseUrl = computed(() => {
  const port = sidecarPort.value || DEFAULT_PORT
  return `http://127.0.0.1:${port}/api`
})

/**
 * 初始化 API 配置
 *
 * 1. 发现后端端口
 * 2. 初始化 SSE 事件订阅器
 */
export async function initializeApiConfig(): Promise<void> {
  try {
    // 首先尝试从 Tauri 获取已发现的端口
    const discoveredPort = await invoke<number | null>('get_sidecar_port')
    if (discoveredPort) {
      sidecarPort.value = discoveredPort
      isPortDiscovered.value = true
      logger.info(LogTags.SYSTEM_API, 'Using discovered port', { port: discoveredPort })
      await initEventSubscriber(discoveredPort)
      return
    }

    // 监听端口发现事件
    await listen<number>('sidecar-port-discovered', (event) => {
      const port = event.payload
      sidecarPort.value = port
      isPortDiscovered.value = true
      logger.info(LogTags.SYSTEM_API, 'Port discovered via event', { port })

      initEventSubscriber(port).catch((error) => {
        logger.error(
          LogTags.SYSTEM_API,
          'Failed to initialize event subscriber',
          error instanceof Error ? error : new Error(String(error))
        )
      })
    })

    // 等待端口发现（最多10秒）
    let attempts = 0
    const maxAttempts = 100 // 10秒，每100ms检查一次

    while (!isPortDiscovered.value && attempts < maxAttempts) {
      await new Promise((resolve) => setTimeout(resolve, 100))

      // 定期检查端口是否已发现
      const currentPort = await invoke<number | null>('get_sidecar_port')
      if (currentPort) {
        sidecarPort.value = currentPort
        isPortDiscovered.value = true
        logger.info(LogTags.SYSTEM_API, 'Port discovered via polling', { port: currentPort })
        await initEventSubscriber(currentPort)
        break
      }

      attempts++
    }

    if (!isPortDiscovered.value) {
      logger.warn(LogTags.SYSTEM_API, 'Port discovery timeout, using default port', {
        port: DEFAULT_PORT,
      })
      sidecarPort.value = DEFAULT_PORT
      await initEventSubscriber(DEFAULT_PORT)
    }
  } catch (error) {
    logger.error(
      LogTags.SYSTEM_API,
      'Failed to initialize API config',
      error instanceof Error ? error : new Error(String(error))
    )
    sidecarPort.value = DEFAULT_PORT
  }
}

/**
 * 初始化 SSE 事件订阅器
 */
async function initEventSubscriber(port: number): Promise<void> {
  const apiUrl = `http://127.0.0.1:${port}/api`

  const { initEventSubscriber: init } = await import('@/infra/events/events')
  init(apiUrl)

  logger.info(LogTags.SYSTEM_API, 'Event subscriber initialized', { apiUrl })
}

/**
 * 等待 API 准备就绪
 */
export async function waitForApiReady(): Promise<string> {
  if (!isPortDiscovered.value) {
    await initializeApiConfig()
  }
  return apiBaseUrl.value
}

/**
 * API 配置 composable
 */
export function useApiConfig() {
  return {
    apiBaseUrl,
    sidecarPort: computed(() => sidecarPort.value),
    isPortDiscovered: computed(() => isPortDiscovered.value),
    initializeApiConfig,
    waitForApiReady,
  }
}
