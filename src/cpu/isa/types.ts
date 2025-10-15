/**
 * ISA 指令集类型定义（混合架构版）
 *
 * 支持：
 * 1. 声明式请求配置（单个或多个请求）
 * 2. 自定义执行逻辑
 * 3. 乐观更新
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
 * HTTP 请求方法
 */
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE'

/**
 * 单个 HTTP 请求配置
 */
export interface RequestConfig<TPayload = any> {
  method: HttpMethod
  url: string | ((payload: TPayload) => string)
  body?: (payload: TPayload) => any
  headers?: Record<string, string>
}

/**
 * 多个 HTTP 请求配置
 */
export interface MultiRequestConfig<TPayload = any> {
  /** 请求列表 */
  requests: Array<RequestConfig<TPayload>>
  /** 执行模式：parallel（并发）或 sequential（串行） */
  mode: 'parallel' | 'sequential'
  /** 合并多个请求的结果（可选） */
  combineResults?: (results: any[]) => any
}

/**
 * 乐观更新快照
 */
export interface OptimisticSnapshot {
  [key: string]: any
}

/**
 * 乐观更新配置
 */
export interface OptimisticConfig<TPayload = any> {
  /** 是否启用乐观更新 */
  enabled: boolean
  /** 应用乐观更新，返回快照用于回滚 */
  apply: (payload: TPayload, context: InstructionContext) => OptimisticSnapshot
  /** 回滚乐观更新 */
  rollback: (snapshot: OptimisticSnapshot) => void
}

/**
 * 指令定义（混合架构）
 *
 * 两种方式（互斥）：
 * 1. 声明式：配置 `request`（单个或多个）+ `commit`
 * 2. 自定义：配置 `execute` + `commit`
 */
export interface InstructionDefinition<TPayload = any, TResult = any> {
  /** 指令元数据 */
  meta: InstructionMeta

  // ==================== 声明式配置（推荐，80% 场景） ====================

  /**
   * HTTP 请求配置
   * - 单个请求：RequestConfig
   * - 多个请求：MultiRequestConfig（支持并发/串行）
   *
   * 与 `execute` 互斥，优先使用 `request`
   */
  request?: RequestConfig<TPayload> | MultiRequestConfig<TPayload>

  /**
   * 乐观更新配置（可选）
   */
  optimistic?: OptimisticConfig<TPayload>

  // ==================== 自定义执行（灵活，20% 场景） ====================

  /**
   * 前置验证（可选）
   */
  validate?: (payload: TPayload, context: InstructionContext) => Promise<boolean>

  /**
   * 自定义执行逻辑（与 `request` 互斥）
   *
   * 适用场景：
   * - 复杂的条件逻辑
   * - 需要在请求间传递数据
   * - 需要与旧系统集成
   */
  execute?: (payload: TPayload, context: InstructionContext) => Promise<TResult>

  /**
   * 提交结果（写回阶段调用）
   *
   * 用于处理执行结果，如更新 Store
   * @param result 执行结果
   * @param payload 指令负载
   * @param context 指令上下文
   * @param optimisticSnapshot 乐观更新快照（如果有）
   */
  commit?: (
    result: TResult,
    payload: TPayload,
    context: InstructionContext,
    optimisticSnapshot?: any
  ) => Promise<void>
}

/**
 * ISA 定义类型
 */
export type ISADefinition = Record<string, InstructionDefinition>
