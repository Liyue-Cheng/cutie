/**
 * 指令追踪器
 */

import { InstructionStatus, PipelineStage } from '../types'
import type { InstructionTrace, PhaseTimestamps } from './types'

export class InstructionTracker {
  private traces = new Map<string, InstructionTrace>()

  /**
   * 开始追踪指令
   */
  startInstruction(instructionId: string, type: string, payload: any, correlationId: string): void {
    const trace: InstructionTrace = {
      instructionId,
      type,
      payload,
      correlationId,
      timestamps: {
        IF: Date.now(),
      },
      status: InstructionStatus.PENDING,
    }

    this.traces.set(instructionId, trace)
  }

  /**
   * 标记阶段时间戳
   */
  markPhase(instructionId: string, phase: PipelineStage): void {
    const trace = this.traces.get(instructionId)
    if (!trace) return

    trace.timestamps[phase] = Date.now()

    // 更新状态
    if (phase === PipelineStage.SCH) {
      trace.status = InstructionStatus.ISSUED
    } else if (phase === PipelineStage.EX) {
      trace.status = InstructionStatus.EXECUTING
    } else if (phase === PipelineStage.RES) {
      trace.status = InstructionStatus.RESPONDED
    } else if (phase === PipelineStage.WB) {
      trace.status = InstructionStatus.COMMITTED
    }
  }

  /**
   * 记录网络请求结果
   */
  recordNetworkResult(instructionId: string, result: any): void {
    const trace = this.traces.get(instructionId)
    if (!trace) return

    trace.networkResult = result
  }

  /**
   * 完成指令追踪
   */
  completeInstruction(instructionId: string): void {
    const trace = this.traces.get(instructionId)
    if (!trace) return

    trace.status = InstructionStatus.COMMITTED
    trace.duration = this.calculateDuration(trace.timestamps)

    console.log(
      `%c🎯 指令完成: ${trace.type}`,
      'color: #4CAF50; font-weight: bold',
      this.formatTraceInfo(trace)
    )
  }

  /**
   * 标记指令失败
   */
  failInstruction(instructionId: string, error: Error): void {
    const trace = this.traces.get(instructionId)
    if (!trace) return

    trace.status = InstructionStatus.FAILED
    trace.error = error
    trace.duration = this.calculateDuration(trace.timestamps)

    console.error(
      `%c❌ 指令失败: ${trace.type}`,
      'color: #F44336; font-weight: bold',
      this.formatTraceInfo(trace),
      error
    )
  }

  /**
   * 获取指令追踪记录
   */
  getTrace(instructionId: string): InstructionTrace | undefined {
    return this.traces.get(instructionId)
  }

  /**
   * 获取所有追踪记录
   */
  getAllTraces(): InstructionTrace[] {
    return Array.from(this.traces.values()).sort((a, b) => b.timestamps.IF - a.timestamps.IF)
  }

  /**
   * 清空追踪记录
   */
  clearTraces(): void {
    this.traces.clear()
  }

  /**
   * 计算总耗时
   */
  private calculateDuration(timestamps: PhaseTimestamps): number {
    const lastTimestamp =
      timestamps.WB || timestamps.RES || timestamps.EX || timestamps.SCH || timestamps.IF
    return lastTimestamp - timestamps.IF
  }

  /**
   * 格式化阶段耗时
   */
  private formatPhaseDurations(timestamps: PhaseTimestamps): string {
    const durations: string[] = []

    if (timestamps.SCH) {
      durations.push(`IF→SCH: ${timestamps.SCH - timestamps.IF}ms`)
    }
    if (timestamps.EX && timestamps.SCH) {
      durations.push(`SCH→EX: ${timestamps.EX - timestamps.SCH}ms`)
    }
    if (timestamps.RES && timestamps.EX) {
      durations.push(`EX→RES: ${timestamps.RES - timestamps.EX}ms`)
    }
    if (timestamps.WB && timestamps.RES) {
      durations.push(`RES→WB: ${timestamps.WB - timestamps.RES}ms`)
    }

    return durations.join(' | ')
  }

  /**
   * 格式化追踪信息（用于日志输出）
   */
  private formatTraceInfo(trace: InstructionTrace): any {
    return {
      instructionId: trace.instructionId,
      correlationId: trace.correlationId,
      duration: `${trace.duration}ms`,
      phaseDurations: this.formatPhaseDurations(trace.timestamps),
      status: trace.status,
      result: trace.networkResult,
    }
  }
}

// 导出全局单例
export const instructionTracker = new InstructionTracker()
