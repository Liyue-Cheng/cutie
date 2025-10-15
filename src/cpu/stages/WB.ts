/**
 * WB阶段：Write Back（写回）
 */

import type { QueuedInstruction } from '../types'
import { InstructionStatus, PipelineStage } from '../types'
import { instructionTracker } from '../tracking/InstructionTracker'
import { ISA } from '../isa'
import { interruptHandler } from '../interrupt/InterruptHandler'

export class WriteBackStage {
  /**
   * 写回/完成指令
   */
  async writeBack(instruction: QueuedInstruction, success: boolean): Promise<void> {
    // 标记WB阶段
    instruction.timestamps.WB = Date.now()
    instructionTracker.markPhase(instruction.id, PipelineStage.WB)

    if (success) {
      // 🔥 调用 commit 函数（如果存在）
      const definition = ISA[instruction.type]
      if (definition && definition.commit && instruction.result !== undefined) {
        try {
          await definition.commit(instruction.result, instruction.payload, instruction.context)
        } catch (error) {
          console.error(`❌ WB: commit失败 [${instruction.type}]`, error)
          // commit失败也算失败
          instruction.status = InstructionStatus.FAILED
          instruction.error = error instanceof Error ? error : new Error(String(error))
          instructionTracker.failInstruction(instruction.id, instruction.error)
          return
        }
      }

      // 🔥 注册到中断处理器（用于 SSE 去重）
      interruptHandler.register(instruction.context.correlationId, {
        type: instruction.type,
        payload: instruction.payload,
      })

      // 成功场景
      instruction.status = InstructionStatus.COMMITTED
      instructionTracker.completeInstruction(instruction.id)
    } else {
      // 失败场景
      instruction.status = InstructionStatus.FAILED
      instructionTracker.failInstruction(instruction.id, instruction.error || new Error('未知错误'))
    }
  }
}
