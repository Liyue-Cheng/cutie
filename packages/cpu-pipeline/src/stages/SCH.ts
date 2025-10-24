/**
 * SCH阶段：Scheduler（指令调度器）
 */

import type { QueuedInstruction } from '../types'
import { InstructionStatus } from '../types'
import { getISA } from '../isa'

export class SchedulerStage {
  private pendingQueue: QueuedInstruction[] = []
  private activeInstructions: Map<string, QueuedInstruction> = new Map()
  private activeResources: Set<string> = new Set()
  private maxConcurrency: number

  constructor(maxConcurrency: number = 10) {
    this.maxConcurrency = maxConcurrency
  }

  /**
   * 调度循环（tick）
   */
  tick(): void {
    // 🔥 边检查边发射，避免批量检查导致的竞态条件
    // 每发射一个指令，资源状态立即更新，下一个指令检查时会看到最新状态

    let issued = true
    while (issued) {
      issued = false

      for (const instruction of this.pendingQueue) {
        if (this.canIssue(instruction)) {
          this.issue(instruction)
          issued = true
          break // 跳出for循环，重新检查pending队列
        }
      }

      // 如果这轮没有发射任何指令，说明所有指令都被阻塞了，退出
    }
  }

  /**
   * 添加指令到调度队列
   */
  addInstruction(instruction: QueuedInstruction): void {
    this.pendingQueue.push(instruction)
  }

  /**
   * 判断指令是否可以发射
   */
  private canIssue(instruction: QueuedInstruction): boolean {
    // 检查并发数限制
    if (this.activeInstructions.size >= this.maxConcurrency) {
      return false
    }

    // 检查资源冲突
    if (this.hasResourceConflict(instruction)) {
      return false
    }

    return true
  }

  /**
   * 发射指令
   */
  private issue(instruction: QueuedInstruction): void {
    // 从pending队列移除
    const index = this.pendingQueue.indexOf(instruction)
    if (index !== -1) {
      this.pendingQueue.splice(index, 1)
    }

    // 标记为issued
    instruction.status = InstructionStatus.ISSUED
    instruction.timestamps.SCH = Date.now()

    // 加入active列表
    this.activeInstructions.set(instruction.id, instruction)

    // 占用资源
    const resourceIds = this.getResourceIds(instruction)
    for (const resourceId of resourceIds) {
      this.activeResources.add(resourceId)
    }
  }

  /**
   * 检测资源冲突
   */
  private hasResourceConflict(instruction: QueuedInstruction): boolean {
    const resourceIds = this.getResourceIds(instruction)

    for (const resourceId of resourceIds) {
      if (this.activeResources.has(resourceId)) {
        return true
      }
    }

    return false
  }

  /**
   * 从payload中提取资源ID
   */
  private getResourceIds(instruction: QueuedInstruction): string[] {
    const ISA = getISA()
    const isa = ISA[instruction.type]
    if (!isa) {
      return []
    }

    return isa.meta.resourceIdentifier(instruction.payload)
  }

  /**
   * 释放指令占用的资源
   */
  releaseInstruction(instructionId: string): void {
    const instruction = this.activeInstructions.get(instructionId)
    if (!instruction) return

    // 释放资源
    const resourceIds = this.getResourceIds(instruction)
    for (const resourceId of resourceIds) {
      this.activeResources.delete(resourceId)
    }

    // 从active列表移除
    this.activeInstructions.delete(instructionId)
  }

  /**
   * 获取pending队列（用于调试）
   */
  getPendingQueue(): QueuedInstruction[] {
    return [...this.pendingQueue]
  }

  /**
   * 获取正在执行的指令列表（用于调试）
   */
  getActiveInstructions(): QueuedInstruction[] {
    return Array.from(this.activeInstructions.values())
  }

  /**
   * 获取pending队列大小
   */
  getPendingQueueSize(): number {
    return this.pendingQueue.length
  }

  /**
   * 获取活跃指令数量
   */
  getActiveCount(): number {
    return this.activeInstructions.size
  }

  /**
   * 清空调度器
   */
  clear(): void {
    this.pendingQueue = []
    this.activeInstructions.clear()
    this.activeResources.clear()
  }
}
