/**
 * CPU流水线系统导出
 */

export { Pipeline } from './Pipeline'
export { ISA } from './isa'
export type { QueuedInstruction, InstructionContext, InstructionStatus, PipelineStage } from './types'

// 创建全局单例
import { Pipeline } from './Pipeline'

export const pipeline = new Pipeline()

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
  cpuPipeline.getTraces()      - 获取所有追踪记录

示例：
  cpuPipeline.start()
  cpuPipeline.dispatch('debug.fetch_baidu', {})
  cpuPipeline.dispatch('debug.quick_success', { data: 'test' })
      `)
    },
  }
}

