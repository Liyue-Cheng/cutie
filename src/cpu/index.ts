/**
 * CPU流水线系统（项目集成层）
 *
 * 集成解耦的CPU Pipeline核心包与项目特定实现
 */

import {
  Pipeline,
  setHttpClient,
  setCorrelationIdGenerator,
  registerISA,
} from 'front-cpu'
import { httpAdapter } from '@/cpu-adapters/httpAdapter'
import { correlationIdAdapter } from '@/cpu-adapters/correlationIdAdapter'
import { createVueReactiveState } from '@/cpu-adapters/vueAdapter'

// 导入业务ISA
import { ISA } from './isa'

// 🔧 初始化依赖注入
setHttpClient(httpAdapter)
setCorrelationIdGenerator(correlationIdAdapter)

// 🔧 注册业务ISA
registerISA(ISA)

// 🔧 创建流水线实例（使用Vue响应式）
export const pipeline = new Pipeline({
  tickInterval: 16,
  maxConcurrency: 10,
  reactiveStateFactory: createVueReactiveState,
})

// 导出ISA供外部使用
export { ISA }

// 导出类型
export type {
  QueuedInstruction,
  InstructionContext,
  InstructionStatus,
  PipelineStage,
} from 'front-cpu'

// 开发环境：暴露到window用于调试
if (import.meta.env.DEV) {
  ;(window as any).cpuPipeline = {
    pipeline,
    dispatch: (type: string, payload: any) => pipeline.dispatch(type, payload),
    start: () => pipeline.start(),
    stop: () => pipeline.stop(),
    reset: () => pipeline.reset(),
    getStatus: () => pipeline.getStatus(),
    help: () => {
      console.log(`
🎯 CPU流水线调试指南

全局实例：
  window.cpuPipeline

方法：
  cpuPipeline.start()          - 启动流水线
  cpuPipeline.stop()           - 停止流水线
  cpuPipeline.reset()          - 重置流水线
  cpuPipeline.dispatch(type, payload) - 发射指令
  cpuPipeline.getStatus()      - 获取流水线状态

示例：
  cpuPipeline.start()
  cpuPipeline.dispatch('debug.fetch_baidu', {})
  cpuPipeline.dispatch('debug.quick_success', { data: 'test' })
      `)
    },
  }
}
