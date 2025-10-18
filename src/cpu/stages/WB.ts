/**
 * WB阶段：Write Back（写回）
 *
 * 职责：
 * 1. 调用 commit 函数（成功时）
 * 2. 回滚乐观更新（失败时）
 * 3. 注册到中断处理器（成功时）
 */

import type { QueuedInstruction } from '../types'
import { InstructionStatus } from '../types'
import { ISA } from '../isa'
import { interruptHandler } from '../interrupt/InterruptHandler'
import { cpuEventCollector, cpuConsole } from '../logging'

export class WriteBackStage {
  /**
   * 回滚乐观更新
   */
  private rollbackOptimisticUpdate(instruction: QueuedInstruction): void {
    const definition = ISA[instruction.type]

    if (instruction.optimisticSnapshot && definition?.optimistic?.rollback) {
      // 记录回滚操作
      if (!instruction.writeBackExecution) {
        instruction.writeBackExecution = { hasCommit: false, rollbackExecuted: true }
      }
      instruction.writeBackExecution.rollbackExecuted = true
      instruction.writeBackExecution.rollbackSnapshot = instruction.optimisticSnapshot

      // 🎯 记录乐观更新回滚事件
      cpuEventCollector.onOptimisticRolledBack(
        instruction.id,
        instruction.type,
        instruction.context.correlationId,
        instruction.optimisticSnapshot,
        '指令执行失败',
        instruction.error
      )
      cpuConsole.onOptimisticRolledBack(instruction, '指令执行失败')

      try {
        definition.optimistic.rollback(instruction.optimisticSnapshot)
      } catch (rollbackError) {
        instruction.writeBackExecution.rollbackError = rollbackError instanceof Error ? rollbackError : new Error(String(rollbackError))
        console.error('❌ [CPU] 乐观更新回滚失败:', {
          instructionId: instruction.id,
          type: instruction.type,
          error: rollbackError,
        })
      }
    }
  }

  /**
   * 写回/完成指令
   */
  async writeBack(instruction: QueuedInstruction, success: boolean): Promise<void> {
    // 标记WB阶段
    instruction.timestamps.WB = Date.now()

    const definition = ISA[instruction.type]

    // 初始化WB执行记录
    instruction.writeBackExecution = {
      hasCommit: !!(definition && definition.commit),
      rollbackExecuted: false,
    }

    if (success) {
      // ==================== 成功路径 ====================

      // 🔥 调用 commit 函数（如果存在）
      if (definition && definition.commit) {
        // 记录commit调用参数
        instruction.writeBackExecution.commitArgs = {
          result: instruction.result,
          payload: instruction.payload,
          context: instruction.context,
          optimisticSnapshot: instruction.optimisticSnapshot,
        }

        try {
          await definition.commit(
            instruction.result,
            instruction.payload,
            instruction.context,
            instruction.optimisticSnapshot // 🔥 传递乐观更新快照
          )
          instruction.writeBackExecution.commitSuccess = true
        } catch (error) {
          instruction.writeBackExecution.commitSuccess = false
          instruction.writeBackExecution.commitError = error instanceof Error ? error : new Error(String(error))

          // commit失败 → 回滚乐观更新
          this.rollbackOptimisticUpdate(instruction)

          // 设置为失败状态
          instruction.status = InstructionStatus.FAILED
          instruction.error = instruction.writeBackExecution.commitError
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
    } else {
      // ==================== 失败路径 ====================

      // 🔥 回滚乐观更新
      this.rollbackOptimisticUpdate(instruction)

      // 设置失败状态
      instruction.status = InstructionStatus.FAILED
    }
  }
}
