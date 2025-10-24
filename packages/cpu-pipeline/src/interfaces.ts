/**
 * CPU Pipeline 抽象接口定义
 *
 * 这些接口用于解耦CPU核心系统与具体实现，
 * 通过依赖注入实现零耦合架构。
 */

/**
 * HTTP客户端接口
 */
export interface IHttpClient {
  get<T>(url: string, config?: RequestConfig): Promise<T>
  post<T>(url: string, data?: any, config?: RequestConfig): Promise<T>
  patch<T>(url: string, data?: any, config?: RequestConfig): Promise<T>
  put<T>(url: string, data?: any, config?: RequestConfig): Promise<T>
  delete<T>(url: string, config?: RequestConfig): Promise<T>
}

/**
 * HTTP请求配置
 */
export interface RequestConfig {
  headers?: Record<string, string>
  [key: string]: any
}

/**
 * 日志接口
 */
export interface ILogger {
  debug(tag: string, message: string, data?: any): void
  info(tag: string, message: string, data?: any): void
  warn(tag: string, message: string, data?: any): void
  error(tag: string, message: string, data?: any): void
}

/**
 * CorrelationId生成器接口
 */
export interface ICorrelationIdGenerator {
  generate(): string
}

/**
 * 响应式状态接口
 *
 * 用于适配不同框架的响应式系统（Vue、React、Svelte等）
 */
export interface IReactiveState<T> {
  value: T
  setValue(newValue: T): void
  subscribe(callback: (value: T) => void): () => void
}
