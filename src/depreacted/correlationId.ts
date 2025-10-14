/**
 * Correlation ID 生成器
 *
 * 用于关联 HTTP 请求和 SSE 事件，实现去重和请求追踪
 */

import { nanoid } from 'nanoid'

/**
 * 生成唯一的 correlation ID
 */
export function generateCorrelationId(): string {
  return `corr_${Date.now()}_${nanoid(10)}`
}

/**
 * Correlation ID 存储（用于调试和追踪）
 */
class CorrelationStore {
  private activeIds = new Set<string>()
  private maxSize = 1000

  add(id: string): void {
    this.activeIds.add(id)

    // 防止内存泄漏，限制大小
    if (this.activeIds.size > this.maxSize) {
      const firstId = this.activeIds.values().next().value
      if (firstId !== undefined) {
        this.activeIds.delete(firstId)
      }
    }
  }

  has(id: string): boolean {
    return this.activeIds.has(id)
  }

  remove(id: string): void {
    this.activeIds.delete(id)
  }

  clear(): void {
    this.activeIds.clear()
  }

  getAll(): string[] {
    return Array.from(this.activeIds)
  }
}

export const correlationStore = new CorrelationStore()
