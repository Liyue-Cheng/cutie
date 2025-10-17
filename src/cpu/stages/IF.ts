/**
 * IF阶段：Instruction Fetch（指令获取）
 */

import type { QueuedInstruction } from '../types'
import { InstructionStatus } from '../types'
import { generateCorrelationId } from '@/infra/correlation/correlationId'
import type { CallSource } from '../logging/types'

export class InstructionFetchStage {
  private buffer: QueuedInstruction[] = []
  private idCounter = 0

  /**
   * 获取指令（从组件发射的指令）
   */
  fetchInstruction<TPayload>(
    type: string,
    payload: TPayload,
    source: 'user' | 'system' | 'test' = 'user',
    callSource?: CallSource
  ): QueuedInstruction<TPayload> {
    const instructionId = `instr-${Date.now()}-${++this.idCounter}`
    const correlationId = generateCorrelationId()

    const instruction: QueuedInstruction<TPayload> = {
      id: instructionId,
      type,
      payload,
      context: {
        instructionId,
        correlationId,
        timestamp: Date.now(),
        source,
        retryCount: 0,
        callSource, // 🔍 存储调用源信息
      },
      status: InstructionStatus.PENDING,
      timestamps: {
        IF: Date.now(),
      },
    }

    // 放入缓冲区
    this.enqueue(instruction)

    return instruction
  }

  /**
   * 将指令放入IF缓冲区
   */
  enqueue(instruction: QueuedInstruction): void {
    this.buffer.push(instruction)
  }

  /**
   * 从缓冲区取出指令（供调度器使用）
   */
  dequeue(): QueuedInstruction | undefined {
    return this.buffer.shift()
  }

  /**
   * 获取缓冲区内容（用于调试）
   */
  getBuffer(): QueuedInstruction[] {
    return [...this.buffer]
  }

  /**
   * 获取缓冲区大小
   */
  getBufferSize(): number {
    return this.buffer.length
  }

  /**
   * 清空缓冲区
   */
  clear(): void {
    this.buffer = []
  }
}
