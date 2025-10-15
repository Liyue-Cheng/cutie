import { createApp } from 'vue'
import { createPinia, setActivePinia } from 'pinia'

import App from './App.vue'
import router from './router' // 导入路由
import i18n from './i18n'
import { initializeApiConfig } from '@/composables/useApiConfig'
import { logger, LogLevel, LogTags } from '@/infra/logging/logger'
import {
  setupGlobalErrorHandling,
  createVueErrorHandler,
  createVueWarnHandler,
} from '@/infra/errors/errorHandler'
import { initCommandBus } from '@/commandBus'
import { setupAutoTracking, getTrackingStats } from '@/infra/logging/AutoInstructionTracker'
import { initializeDragStrategies } from '@/infra/drag'
import './style.css'

// 设置全局错误处理
setupGlobalErrorHandling()

const pinia = createPinia()
const app = createApp(App)

// 配置Vue错误处理
app.config.errorHandler = createVueErrorHandler()
app.config.warnHandler = createVueWarnHandler()

app.use(pinia)
// 允许在组件外使用 stores（用于自动追踪器拦截 store mutations）
setActivePinia(pinia)
app.use(i18n)
app.use(router) // 确保已经 use 了 router

// 初始化命令总线（需要在 pinia 初始化之后）
initCommandBus()

// 🚀 启用自动指令追踪（零集成！）
if (import.meta.env.DEV) {
  setupAutoTracking()
}

// 🎯 初始化拖放策略系统
initializeDragStrategies()

// 设置全局日志控制接口（仅开发环境）
if (import.meta.env.DEV) {
  ;(window as any).appLogger = {
    setLevel: (level: 'DEBUG' | 'INFO' | 'WARN' | 'ERROR' | 'SILENT') => {
      logger.setLevel((LogLevel as any)[level])
      console.log(`🔧 Logger level set to ${level}`)
    },
    filterByTag: (tags: string | string[]) => {
      const tagArray = Array.isArray(tags) ? tags : [tags]
      logger.setTagFilters(tagArray)
      console.log(`🔧 Logger filtering by tags:`, tagArray)
    },
    resetFilters: () => {
      logger.setTagFilters([])
      console.log('🔧 Logger tag filters reset.')
    },
    trackingOnly: () => {
      logger.setLevel(LogLevel.INFO)
      logger.setTagFilters([LogTags.INSTRUCTION_TRACKER])
      console.log('🎯 Tracking-only mode enabled! Only instruction tracking logs will be shown.')
    },
    setSampling: (config: { debug?: number; info?: number; warn?: number; error?: number }) => {
      logger.setSampling(config)
      console.log('🔧 Logger sampling updated:', config)
    },
    getStats: () => {
      const stats = logger.getStats()
      console.table(stats)
      return stats
    },
    getTrackingStats: () => {
      const stats = getTrackingStats()
      console.log('🎯 Instruction Tracking Stats:', stats)
      return stats
    },
    applyPreset: (presetName: string) => {
      import('@/infra/logging/loggerSettings').then(({ applyPreset }) => {
        const preset = applyPreset(presetName as any)
        if (preset) {
          logger.setLevel(preset.level)
          logger.setTagFilters(preset.tagWhitelist)
          console.log(`🎯 Applied preset: ${presetName}`, preset)
        }
      })
    },
    help: () => {
      console.log(`
🔧 Logger Control Commands:
  appLogger.setLevel('INFO')           // 设置日志级别: DEBUG, INFO, WARN, ERROR, SILENT
  appLogger.filterByTag('API')         // 按单个标签过滤
  appLogger.filterByTag(['API', 'Drag']) // 按多个标签过滤
  appLogger.resetFilters()             // 显示所有日志
  appLogger.trackingOnly()             // 🎯 只显示指令追踪日志（推荐）
  appLogger.setSampling({debug: 0.1})  // 设置采样率 (0-1)
  appLogger.applyPreset('dragOnly')    // 应用预设配置
  appLogger.getStats()                 // 查看当前配置
  appLogger.getTrackingStats()         // 查看指令追踪统计 🎯
  appLogger.help()                     // 显示此帮助

🎯 可用预设:
  default, errorsOnly, dragOnly, apiOnly, componentsOnly, performance

🏷️  常用标签:
  ${Object.values(LogTags).join(', ')}

💡 快速调试:
  appLogger.trackingOnly()             // 🎯 清洁模式：只看指令追踪（推荐）
  appLogger.applyPreset('errorsOnly')  // 只看错误和警告
  appLogger.applyPreset('apiOnly')     // 只看API相关日志

🎯 指令追踪已自动启用！现在点击任务完成按钮，然后执行：
  appLogger.trackingOnly()
  就能看到干净的四级流水线追踪日志了！
      `)
    },
  }

  // 显示初始化信息
  logger.info('System:Init', 'Logger system initialized', {
    environment: import.meta.env.MODE,
    level: logger.getStats().level,
  })

  // 显示帮助信息
  console.log('🔧 Logger system ready! Type appLogger.help() for commands.')
}

// 初始化API配置
initializeApiConfig()
  .then(async () => {
    logger.info('System:Init', 'API configuration initialized')

    // ✅ 在应用启动时加载所有 areas（解决 N+1 查询问题）
    const { useAreaStore } = await import('@/stores/area')
    const areaStore = useAreaStore()
    await areaStore.fetchAreas()
    logger.info('System:Init', 'All areas loaded')
  })
  .catch((error) => {
    logger.error('System:Init', 'Failed to initialize API configuration', error)
  })

app.mount('#app')
