/**
 * EX阶段：Execute（执行）
 */

import type { QueuedInstruction } from '../types'
import { InstructionStatus, PipelineStage } from '../types'
import { ISA } from '../isa'
import { instructionTracker } from '../tracking/InstructionTracker'

export class ExecuteStage {
  /**
   * 执行指令
   */
  async execute(instruction: QueuedInstruction): Promise<void> {
    const isa = ISA[instruction.type]
    if (!isa) {
      throw new Error(`未找到指令定义: ${instruction.type}`)
    }

    try {
      // 步骤1: 前置验证
      if (isa.validate) {
        const isValid = await isa.validate(instruction.payload, instruction.context)
        if (!isValid) {
          throw new Error(`指令验证失败: ${instruction.type}`)
        }
      }

      // 标记EX阶段开始
      instruction.status = InstructionStatus.EXECUTING
      instruction.timestamps.EX = Date.now()
      instructionTracker.markPhase(instruction.id, PipelineStage.EX)

      // 步骤2: 执行网络请求/操作
      const result = await isa.execute(instruction.payload, instruction.context)

      // 保存结果
      instruction.result = result
      instructionTracker.recordNetworkResult(instruction.id, result)
    } catch (error) {
      // 保存错误信息
      instruction.error = error as Error
      throw error
    }
  }
}

