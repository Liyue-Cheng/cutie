/**
 * 应用启动引导模块
 *
 * 集中管理所有初始化逻辑，确保正确的执行顺序：
 * 1. 基础设置（错误处理、插件）
 * 2. API 配置（等待后端就绪）
 * 3. CPU Pipeline 启动
 * 4. Store 初始化（事件订阅 + 数据加载）
 * 5. 主题初始化
 * 6. 开发工具（仅 DEV 环境）
 */

import type { App } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import router from '@/router'
import i18n from '@/i18n'
import { logger } from '@/infra/logging/logger'
import {
  setupGlobalErrorHandling,
  createVueErrorHandler,
  createVueWarnHandler,
} from '@/infra/errors/errorHandler'
import { initializeDragStrategies } from '@/infra/drag'
import { initializeApiConfig } from '@/composables/useApiConfig'
import { initCpu } from './initCpu'
import { initStores } from './initStores'
import { initTheme } from './initTheme'
import { setupDevTools } from './devTools'

/**
 * 应用启动引导
 * @param app Vue 应用实例
 */
export async function bootstrap(app: App): Promise<void> {
  // ============================================================
  // Phase 1: 基础设置（同步）
  // ============================================================
  logger.info('System:Init', 'Bootstrap started')

  // 全局错误处理
  setupGlobalErrorHandling()
  app.config.errorHandler = createVueErrorHandler()
  app.config.warnHandler = createVueWarnHandler()

  // Vue 插件
  const pinia = createPinia()
  app.use(pinia)
  setActivePinia(pinia) // 允许在组件外使用 stores
  app.use(i18n)
  app.use(router)

  // 拖放策略系统
  initializeDragStrategies()

  logger.info('System:Init', 'Phase 1 complete: Basic setup')

  // ============================================================
  // Phase 2: API 配置（等待后端就绪）
  // ============================================================
  await initializeApiConfig()
  logger.info('System:Init', 'Phase 2 complete: API config')

  // ============================================================
  // Phase 3: CPU Pipeline 启动
  // ============================================================
  await initCpu()
  logger.info('System:Init', 'Phase 3 complete: CPU Pipeline')

  // ============================================================
  // Phase 4: Store 初始化
  // ============================================================
  await initStores()
  logger.info('System:Init', 'Phase 4 complete: Stores')

  // ============================================================
  // Phase 5: 主题初始化
  // ============================================================
  initTheme()
  logger.info('System:Init', 'Phase 5 complete: Theme')

  // ============================================================
  // Phase 6: 开发工具（仅 DEV）
  // ============================================================
  if (import.meta.env.DEV) {
    setupDevTools()
    logger.info('System:Init', 'Phase 6 complete: Dev tools')
  }

  logger.info('System:Init', 'Bootstrap complete')
}
