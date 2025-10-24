/**
 * RES阶段：Response（响应处理）
 */

import type { QueuedInstruction } from '../types'
import { InstructionStatus } from '../types'

export class ResponseStage {
  /**
   * 处理网络响应
   */
  processResponse(
    instruction: QueuedInstruction,
    error?: Error
  ): { success: boolean; shouldRetry: boolean } {
    // 标记RES阶段
    instruction.status = InstructionStatus.RESPONDED
    instruction.timestamps.RES = Date.now()

    if (error) {
      // 失败处理（简化版：不实现重试）
      instruction.error = error
      return { success: false, shouldRetry: false }
    }

    // 成功
    return { success: true, shouldRetry: false }
  }
}
