import { ref, computed } from 'vue'
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
      console.log(`🔍 [API Config] Using discovered port: ${discoveredPort}`)
      return
    }

    // 监听端口发现事件
    const unlisten = await listen<number>('sidecar-port-discovered', (event) => {
      const port = event.payload
      sidecarPort.value = port
      isPortDiscovered.value = true
      console.log(`🔍 [API Config] Port discovered via event: ${port}`)
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
        console.log(`🔍 [API Config] Port discovered via polling: ${currentPort}`)
        break
      }

      attempts++
    }

    if (!isPortDiscovered.value) {
      console.warn(`⚠️ [API Config] Port discovery timeout, using default port: ${DEFAULT_PORT}`)
      sidecarPort.value = DEFAULT_PORT
    }
  } catch (error) {
    console.error('❌ [API Config] Failed to initialize API config:', error)
    sidecarPort.value = DEFAULT_PORT
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
