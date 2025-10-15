/**
 * EX阶段：Execute（执行）
 *
 * 支持：
 * 1. 声明式请求（单个或多个）
 * 2. 自定义执行逻辑
 * 3. 乐观更新 + 自动回滚
 */

import type { QueuedInstruction } from '../types'
import { InstructionStatus, PipelineStage } from '../types'
import { ISA } from '../isa'
import { instructionTracker } from '../tracking/InstructionTracker'
import { executeRequest } from '../utils/request'

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

      // 步骤2: 执行乐观更新（可选）
      if (isa.optimistic?.enabled) {
        instruction.optimisticSnapshot = isa.optimistic.apply(
          instruction.payload,
          instruction.context
        )
      }

      // 步骤3: 标记 EX 阶段开始
      instruction.status = InstructionStatus.EXECUTING
      instruction.timestamps.EX = Date.now()
      instructionTracker.markPhase(instruction.id, PipelineStage.EX)

      // 步骤4: 执行网络请求/操作
      let result: any

      if (isa.request) {
        // 🔥 声明式请求（单个或多个，全部完成后再继续）
        result = await executeRequest(isa.request, instruction.payload, instruction.context)
      } else if (isa.execute) {
        // 🔥 自定义执行逻辑
        result = await isa.execute(instruction.payload, instruction.context)
      } else {
        throw new Error(`指令 ${instruction.type} 既没有 request 也没有 execute`)
      }

      // 保存结果
      instruction.result = result
      instructionTracker.recordNetworkResult(instruction.id, result)
    } catch (error) {
      // 失败时回滚乐观更新
      if (instruction.optimisticSnapshot && isa.optimistic?.rollback) {
        isa.optimistic.rollback(instruction.optimisticSnapshot)
      }

      // 保存错误信息
      instruction.error = error as Error
      throw error
    }
  }
}
