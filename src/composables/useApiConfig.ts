import { ref, computed } from 'vue'
import { logger, LogTags } from '@/services/logger'
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

// 初始化端口发现
export async function initializeApiConfig() {
  try {
    // 首先尝试从Tauri获取已发现的端口
    const discoveredPort = await invoke<number | null>('get_sidecar_port')
    if (discoveredPort) {
      sidecarPort.value = discoveredPort
      isPortDiscovered.value = true
      logger.info(LogTags.SYSTEM_API, 'Using discovered port', { port: discoveredPort })

      // ✅ 初始化事件订阅
      await initializeEventSubscriptions(discoveredPort)
      return
    }

    // 监听端口发现事件
    await listen<number>('sidecar-port-discovered', (event) => {
      const port = event.payload
      sidecarPort.value = port
      isPortDiscovered.value = true
      logger.info(LogTags.SYSTEM_API, 'Port discovered via event', { port })

      // ✅ 初始化事件订阅
      initializeEventSubscriptions(port).catch((error) => {
        logger.error(
          LogTags.SYSTEM_API,
          'Failed to initialize event subscriptions',
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

        // ✅ 初始化事件订阅
        await initializeEventSubscriptions(currentPort)
        break
      }

      attempts++
    }

    if (!isPortDiscovered.value) {
      logger.warn(LogTags.SYSTEM_API, 'Port discovery timeout, using default port', {
        port: DEFAULT_PORT,
      })
      sidecarPort.value = DEFAULT_PORT

      // ✅ 初始化事件订阅（使用默认端口）
      await initializeEventSubscriptions(DEFAULT_PORT)
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

// ✅ 初始化事件订阅系统
async function initializeEventSubscriptions(port: number) {
  try {
    const apiUrl = `http://127.0.0.1:${port}/api`

    // 动态导入事件服务
    const { initEventSubscriber } = await import('@/services/events')
    initEventSubscriber(apiUrl)
    logger.info(LogTags.SYSTEM_API, 'Event subscriber initialized')

    // 初始化各个 Store 的事件订阅
    const { useTaskStore } = await import('@/stores/task')
    const { useTimeBlockStore } = await import('@/stores/timeblock')
    const { useTrashStore } = await import('@/stores/trash')
    const { useTemplateStore } = await import('@/stores/template')

    const taskStore = useTaskStore()
    const timeBlockStore = useTimeBlockStore()
    const trashStore = useTrashStore()
    const templateStore = useTemplateStore()

    taskStore.initEventSubscriptions()
    timeBlockStore.initEventSubscriptions()
    trashStore.initEventSubscriptions()
    templateStore.initEventSubscriptions()
    logger.info(LogTags.SYSTEM_API, 'Store event subscriptions initialized')
  } catch (error) {
    logger.error(
      LogTags.SYSTEM_API,
      'Failed to initialize event subscriptions',
      error instanceof Error ? error : new Error(String(error))
    )
  }
}

// 等待API准备就绪
export async function waitForApiReady(): Promise<string> {
  if (!isPortDiscovered.value) {
    await initializeApiConfig()
  }
  return apiBaseUrl.value
}

// 导出状态供其他组件使用
export function useApiConfig() {
  return {
    apiBaseUrl,
    sidecarPort: computed(() => sidecarPort.value),
    isPortDiscovered: computed(() => isPortDiscovered.value),
    initializeApiConfig,
    waitForApiReady,
  }
}
