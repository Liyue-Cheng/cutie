/**
 * EX阶段：Execute（执行）
 *
 * 支持：
 * 1. 声明式请求（单个或多个）
 * 2. 自定义执行逻辑
 * 3. 乐观更新 + 自动回滚
 * 4. 超时控制（基于指令配置）
 */

import type { QueuedInstruction } from '../types'
import { InstructionStatus } from '../types'
import { getISA } from '../isa'
import { executeRequest } from '../utils/request'
import { cpuEventCollector, cpuConsole } from '../logging'

export class ExecuteStage {
  /**
   * 执行指令
   */
  async execute(instruction: QueuedInstruction): Promise<void> {
    const ISA = getISA()
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

        // 🎯 记录乐观更新应用事件
        cpuEventCollector.onOptimisticApplied(
          instruction.id,
          instruction.type,
          instruction.context.correlationId,
          instruction.optimisticSnapshot,
          {}
        )
        cpuConsole.onOptimisticApplied(instruction)
      }

      // 步骤3: 标记 EX 阶段开始
      instruction.status = InstructionStatus.EXECUTING
      instruction.timestamps.EX = Date.now()

      // 步骤4: 执行网络请求/操作（带超时控制）
      let result: any

      // 🔥 根据指令配置的超时时间创建执行 Promise
      const executePromise = (async () => {
        if (isa.request) {
          // 声明式请求
          return await executeRequest(isa.request, instruction.payload, instruction.context)
        } else if (isa.execute) {
          // 自定义执行逻辑
          return await isa.execute(instruction.payload, instruction.context)
        } else {
          throw new Error(`指令 ${instruction.type} 既没有 request 也没有 execute`)
        }
      })()

      // 🔥 如果指令定义了超时时间，应用超时控制
      if (isa.meta.timeout) {
        const timeoutPromise = new Promise<never>((_, reject) => {
          setTimeout(() => {
            reject(new Error(`指令 ${instruction.type} 执行超时（${isa.meta.timeout}ms）`))
          }, isa.meta.timeout)
        })

        result = await Promise.race([executePromise, timeoutPromise])
      } else {
        // 没有配置超时，直接执行
        result = await executePromise
      }

      // 保存结果
      instruction.result = result
    } catch (error) {
      // 保存错误信息（回滚由 WB 阶段统一处理）
      instruction.error = error as Error
      throw error
    }
  }
}
