/**
 * 开发工具模块
 *
 * 仅在开发环境下加载，提供：
 * - 日志控制接口 (appLogger)
 * - CPU Pipeline 调试接口 (cpuPipeline)
 */

import { logger, LogLevel, LogTags } from '@/infra/logging/logger'

/**
 * 设置开发工具
 */
export function setupDevTools(): void {
  // 日志控制接口
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
  appLogger.help()                     // 显示此帮助

🎯 可用预设:
  default, errorsOnly, dragOnly, apiOnly, componentsOnly, performance

🏷️  常用标签:
  ${Object.values(LogTags).join(', ')}

💡 快速调试:
  appLogger.applyPreset('errorsOnly')  // 只看错误和警告
  appLogger.applyPreset('apiOnly')     // 只看API相关日志

💡 CPU Pipeline 日志:
  前往 CPU 调试页面调整控制台级别，或使用：
  const { cpuConsole, ConsoleLevel } = await import('@/cpu/logging')
  cpuConsole.setLevel(ConsoleLevel.VERBOSE)
      `)
    },
  }

  // 显示初始化信息
  logger.info('System:Init', 'Dev tools initialized', {
    environment: import.meta.env.MODE,
    level: logger.getStats().level,
  })

  console.log('🔧 Dev tools ready! Type appLogger.help() for commands.')
}
