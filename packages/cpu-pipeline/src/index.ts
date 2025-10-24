/**
 * CPU Pipeline - 前端指令执行系统
 *
 * 解耦版：通过依赖注入实现零耦合架构
 */

// 核心Pipeline
export { Pipeline } from './Pipeline'
export type { PipelineStatus, PipelineConfig } from './Pipeline'

// 核心类型
export type {
  QueuedInstruction,
  InstructionContext,
  InstructionStatus,
  PipelineStage,
  WriteBackExecution,
} from './types'
export {
  PipelineStage as PipelineStageEnum,
  InstructionStatus as InstructionStatusEnum,
} from './types'

// ISA相关
export { registerISA, getISA, clearISA } from './isa'
export type { ISADefinition, InstructionDefinition } from './isa/types'
export type {
  InstructionMeta,
  HttpMethod,
  RequestConfig,
  MultiRequestConfig,
  OptimisticSnapshot,
  OptimisticConfig,
} from './isa/types'

// 依赖注入
export { setHttpClient } from './utils/request'
export { setCorrelationIdGenerator } from './stages/IF'

// 抽象接口
export type {
  IHttpClient,
  ILogger,
  ICorrelationIdGenerator,
  IReactiveState,
  RequestConfig as IRequestConfig,
} from './interfaces'

// 日志系统
export * from './logging'
