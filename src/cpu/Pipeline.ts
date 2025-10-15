/**
 * CPU流水线主控制器
 */

import { InstructionFetchStage } from './stages/IF'
import { SchedulerStage } from './stages/SCH'
import { ExecuteStage } from './stages/EX'
import { ResponseStage } from './stages/RES'
import { WriteBackStage } from './stages/WB'
import { instructionTracker } from './tracking/InstructionTracker'
import type { QueuedInstruction } from './types'
import { ref } from 'vue'

export interface PipelineStatus {
  ifBufferSize: number
  schPendingSize: number
  schActiveSize: number
  totalCompleted: number
  totalFailed: number
}

export class Pipeline {
  private IF: InstructionFetchStage
  private SCH: SchedulerStage
  private EX: ExecuteStage
  private RES: ResponseStage
  private WB: WriteBackStage

  private isRunning = false
  private tickInterval: number | null = null
  private readonly TICK_INTERVAL_MS = 16 // ~60fps

  // 响应式状态（用于Vue组件）
  public status = ref<PipelineStatus>({
    ifBufferSize: 0,
    schPendingSize: 0,
    schActiveSize: 0,
    totalCompleted: 0,
    totalFailed: 0,
  })

  constructor() {
    this.IF = new InstructionFetchStage()
    this.SCH = new SchedulerStage()
    this.EX = new ExecuteStage()
    this.RES = new ResponseStage()
    this.WB = new WriteBackStage()
  }

  /**
   * 发射指令（外部API）
   */
  dispatch<TPayload>(
    type: string,
    payload: TPayload,
    source: 'user' | 'system' | 'test' = 'user'
  ): void {
    // 🔒 检查流水线是否在运行
    if (!this.isRunning) {
      console.warn('%c⚠️ 流水线未启动，指令被拒绝', 'color: #FF9800; font-weight: bold', {
        type,
        payload,
      })
      return
    }

    // IF: 获取指令
    const instruction = this.IF.fetchInstruction(type, payload, source)

    // 加入调度队列
    this.SCH.addInstruction(instruction)

    // 立即尝试调度
    this.SCH.tick()

    // 🔥 立即执行新发射的指令（避免tick延迟）
    this.processActiveInstructions()

    // 更新状态
    this.updateStatus()
  }

  /**
   * 启动流水线
   */
  start(): void {
    if (this.isRunning) return

    this.isRunning = true

    // 启动调度器的tick循环
    this.tickInterval = window.setInterval(() => {
      this.SCH.tick()
      this.processActiveInstructions()
      this.updateStatus()
    }, this.TICK_INTERVAL_MS)

    console.log('%c🚀 CPU流水线已启动', 'color: #2196F3; font-weight: bold')
  }

  /**
   * 停止流水线
   */
  stop(): void {
    if (!this.isRunning) return

    this.isRunning = false

    if (this.tickInterval !== null) {
      clearInterval(this.tickInterval)
      this.tickInterval = null
    }

    console.log('%c⏸️ CPU流水线已停止', 'color: #FF9800; font-weight: bold')
  }

  /**
   * 重置流水线
   */
  reset(): void {
    this.stop()

    // 清空所有阶段
    this.IF.clear()
    this.SCH.clear()

    // 清空追踪记录
    instructionTracker.clearTraces()

    // 重置状态
    this.status.value = {
      ifBufferSize: 0,
      schPendingSize: 0,
      schActiveSize: 0,
      totalCompleted: 0,
      totalFailed: 0,
    }

    console.log('%c🔄 CPU流水线已重置', 'color: #9C27B0; font-weight: bold')
  }

  /**
   * 处理正在执行的指令
   */
  private async processActiveInstructions(): Promise<void> {
    const activeInstructions = this.SCH.getActiveInstructions()

    for (const instruction of activeInstructions) {
      // 已经在执行中，跳过
      if (instruction.timestamps.EX) {
        continue
      }

      // 异步执行指令
      this.executeInstruction(instruction)
    }
  }

  /**
   * 执行单个指令
   */
  private async executeInstruction(instruction: QueuedInstruction): Promise<void> {
    let error: Error | undefined

    try {
      // EX: 执行
      await this.EX.execute(instruction)
    } catch (err) {
      error = err as Error
    }

    // RES: 处理响应
    const { success } = this.RES.processResponse(instruction, error)

    // WB: 写回
    await this.WB.writeBack(instruction, success)

    // 释放资源
    this.SCH.releaseInstruction(instruction.id)

    // 更新状态
    this.updateStatus()
  }

  /**
   * 更新流水线状态
   */
  private updateStatus(): void {
    const traces = instructionTracker.getAllTraces()

    this.status.value = {
      ifBufferSize: this.IF.getBufferSize(),
      schPendingSize: this.SCH.getPendingQueueSize(),
      schActiveSize: this.SCH.getActiveCount(),
      totalCompleted: traces.filter((t) => t.status === 'committed').length,
      totalFailed: traces.filter((t) => t.status === 'failed').length,
    }
  }

  /**
   * 获取流水线状态（供外部使用）
   */
  getStatus(): PipelineStatus {
    return this.status.value
  }

  /**
   * 获取IF缓冲区
   */
  getIFBuffer(): QueuedInstruction[] {
    return this.IF.getBuffer()
  }

  /**
   * 获取SCH pending队列
   */
  getSCHPendingQueue(): QueuedInstruction[] {
    return this.SCH.getPendingQueue()
  }

  /**
   * 获取SCH active指令
   */
  getSCHActiveInstructions(): QueuedInstruction[] {
    return this.SCH.getActiveInstructions()
  }
}
