/**
 * WB阶段：Write Back（写回）
 * 
 * 职责：
 * 1. 调用 commit 函数（成功时）
 * 2. 回滚乐观更新（失败时）
 * 3. 注册到中断处理器（成功时）
 */

import type { QueuedInstruction } from '../types'
import { InstructionStatus, PipelineStage } from '../types'
import { instructionTracker } from '../tracking/InstructionTracker'
import { ISA } from '../isa'
import { interruptHandler } from '../interrupt/InterruptHandler'
import { logger, LogTags } from '@/infra/logging/logger'

export class WriteBackStage {
  /**
   * 回滚乐观更新
   */
  private rollbackOptimisticUpdate(instruction: QueuedInstruction): void {
    const definition = ISA[instruction.type]
    
    if (instruction.optimisticSnapshot && definition?.optimistic?.rollback) {
      logger.warn(LogTags.SYSTEM_PIPELINE, 'WB: 回滚乐观更新', {
        instructionId: instruction.id,
        type: instruction.type,
      })
      
      try {
        definition.optimistic.rollback(instruction.optimisticSnapshot)
      } catch (rollbackError) {
        logger.error(
          LogTags.SYSTEM_PIPELINE,
          'WB: 回滚失败',
          rollbackError instanceof Error ? rollbackError : new Error(String(rollbackError)),
          {
            instructionId: instruction.id,
            type: instruction.type,
          }
        )
      }
    }
  }

  /**
   * 写回/完成指令
   */
  async writeBack(instruction: QueuedInstruction, success: boolean): Promise<void> {
    // 标记WB阶段
    instruction.timestamps.WB = Date.now()
    instructionTracker.markPhase(instruction.id, PipelineStage.WB)

    const definition = ISA[instruction.type]

    if (success) {
      // ==================== 成功路径 ====================
      
      // 🔥 调用 commit 函数（如果存在）
      if (definition && definition.commit && instruction.result !== undefined) {
        try {
          await definition.commit(instruction.result, instruction.payload, instruction.context)
        } catch (error) {
          logger.error(
            LogTags.SYSTEM_PIPELINE,
            'WB: commit失败',
            error instanceof Error ? error : new Error(String(error)),
            {
              instructionId: instruction.id,
              type: instruction.type,
            }
          )
          
          // commit失败 → 回滚乐观更新
          this.rollbackOptimisticUpdate(instruction)
          
          // 设置为失败状态
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
      
      logger.info(LogTags.SYSTEM_PIPELINE, 'WB: 指令完成', {
        instructionId: instruction.id,
        type: instruction.type,
      })
      
    } else {
      // ==================== 失败路径 ====================
      
      // 🔥 回滚乐观更新
      this.rollbackOptimisticUpdate(instruction)
      
      // 设置失败状态
      instruction.status = InstructionStatus.FAILED
      instructionTracker.failInstruction(instruction.id, instruction.error || new Error('未知错误'))
      
      logger.error(
        LogTags.SYSTEM_PIPELINE,
        'WB: 指令失败',
        instruction.error || new Error('未知错误'),
        {
          instructionId: instruction.id,
          type: instruction.type,
        }
      )
    }
  }
}
