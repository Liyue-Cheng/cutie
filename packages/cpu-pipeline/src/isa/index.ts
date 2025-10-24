/**
 * ISA（指令集架构）管理
 *
 * 提供全局ISA注册和访问机制
 */

import type { ISADefinition } from './types'

// 全局ISA存储
let globalISA: ISADefinition = {}

/**
 * 注册ISA（由项目调用）
 */
export function registerISA(isa: ISADefinition): void {
  globalISA = { ...globalISA, ...isa }
}

/**
 * 获取ISA（供Pipeline内部使用）
 */
export function getISA(): ISADefinition {
  return globalISA
}

/**
 * 清空ISA（用于测试）
 */
export function clearISA(): void {
  globalISA = {}
}

// 导出类型
export type { ISADefinition, InstructionDefinition } from './types'
