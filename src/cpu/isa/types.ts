/**
 * ISA指令集类型定义（简化调试版）
 */

import type { InstructionContext } from '../types'

/**
 * 指令元数据
 */
export interface InstructionMeta {
  /** 指令描述 */
  description: string
  /** 指令分类 */
  category: 'debug' | 'task' | 'schedule' | 'system'
  /** 资源标识符提取函数 */
  resourceIdentifier: (payload: any) => string[]
  /** 优先级 (0-10) */
  priority: number
  /** 超时时间 (ms) */
  timeout?: number
}

/**
 * 指令定义
 */
export interface InstructionDefinition<TPayload = any, TResult = any> {
  /** 指令元数据 */
  meta: InstructionMeta

  /**
   * 前置验证
   */
  validate?: (payload: TPayload, context: InstructionContext) => Promise<boolean>

  /**
   * 执行逻辑
   */
  execute: (payload: TPayload, context: InstructionContext) => Promise<TResult>

  /**
   * 提交结果（写回阶段调用）
   * - 用于处理执行结果，如更新 Store
   * - 可选：如果不需要处理结果，可以省略
   */
  commit?: (result: TResult, payload: TPayload, context: InstructionContext) => Promise<void>
}

/**
 * ISA定义类型
 */
export type ISADefinition = Record<string, InstructionDefinition>
