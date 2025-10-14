/**
 * Infra 基础设施层统一导出
 *
 * 架构理念：
 * - 基础设施层与业务逻辑完全分离
 * - 可复用于其他项目
 * - 类比 CPU 的硬件层
 */

// Transaction Processor（事务处理器 - Reorder Buffer）
export * from './transaction'

// Correlation ID（关联追踪 - Transaction ID）
export * from './correlation'

// SSE Events（中断控制器）
export * from './events'

// HTTP（网络传输层）
export * from './http'

// Logging（调试跟踪单元）
export * from './logging'

// Error Handling（异常处理单元）
export * from './errors'
