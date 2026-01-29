/**
 * 应用入口
 *
 * 职责：
 * 1. 创建 Vue 应用实例
 * 2. 调用 bootstrap 初始化
 * 3. 挂载应用
 */

import { createApp } from 'vue'
import App from './App.vue'
import { bootstrap } from '@/bootstrap'
import { logger } from '@/infra/logging/logger'
import './style.css'

async function main() {
  // 开发环境启用 FrontCPU 指令日志/控制台（debug 入口会初始化 logging provider）
  if (import.meta.env.DEV) {
    const { cpuConsole } = await import('front-cpu/debug')
    // 强制中文输出
    cpuConsole.setLocale('zh-CN')
  }

  const app = createApp(App)

  try {
    // 执行所有初始化
    await bootstrap(app)

    // 挂载应用
    app.mount('#app')
    logger.info('System:Init', 'App mounted successfully')
  } catch (error) {
    logger.error(
      'System:Init',
      'Bootstrap failed',
      error instanceof Error ? error : new Error(String(error))
    )

    // 即使初始化失败也要挂载应用，显示错误状态
    app.mount('#app')
  }
}

main()
