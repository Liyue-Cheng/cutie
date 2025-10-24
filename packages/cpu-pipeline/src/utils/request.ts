/**
 * CPU Pipeline 统一请求工具（解耦版）
 *
 * 通过依赖注入实现HTTP客户端解耦
 *
 * 支持：
 * 1. 单个请求
 * 2. 多个请求（并发/串行）
 * 3. 自动添加 correlation-id
 */

import type { IHttpClient } from '../interfaces'
import type { InstructionContext } from '../types'
import type { RequestConfig, MultiRequestConfig } from '../isa/types'

// 全局HTTP客户端（通过setHttpClient注入）
let httpClient: IHttpClient | null = null

/**
 * 设置HTTP客户端（必须在使用前调用）
 */
export function setHttpClient(client: IHttpClient): void {
  httpClient = client
}

/**
 * 执行单个 HTTP 请求
 */
async function executeSingleRequest(
  config: RequestConfig,
  payload: any,
  context: InstructionContext
): Promise<any> {
  if (!httpClient) {
    throw new Error('HttpClient未初始化，请先调用setHttpClient()')
  }

  // 解析 URL
  const url = typeof config.url === 'function' ? config.url(payload) : config.url

  // 解析请求体
  const body = config.body ? config.body(payload) : payload

  // 统一添加 correlation-id
  const headers = {
    'X-Correlation-ID': context.correlationId,
    ...config.headers,
  }

  const requestConfig = { headers }

  // 根据方法执行请求
  switch (config.method) {
    case 'GET':
      return await httpClient.get(url, requestConfig)
    case 'POST':
      return await httpClient.post(url, body, requestConfig)
    case 'PUT':
      return await httpClient.put(url, body, requestConfig)
    case 'PATCH':
      return await httpClient.patch(url, body, requestConfig)
    case 'DELETE':
      return await httpClient.delete(url, requestConfig)
    default:
      throw new Error(`Unsupported HTTP method: ${config.method}`)
  }
}

/**
 * 判断是否是多请求配置
 */
function isMultiRequestConfig(
  config: RequestConfig | MultiRequestConfig
): config is MultiRequestConfig {
  return 'requests' in config && Array.isArray(config.requests)
}

/**
 * 执行 HTTP 请求（单个或多个）
 *
 * @param config 请求配置（单个或多个）
 * @param payload 指令负载
 * @param context 指令上下文
 * @returns 请求结果
 */
export async function executeRequest(
  config: RequestConfig | MultiRequestConfig,
  payload: any,
  context: InstructionContext
): Promise<any> {
  // 单个请求
  if (!isMultiRequestConfig(config)) {
    return await executeSingleRequest(config, payload, context)
  }

  // 多个请求
  const { requests, mode, combineResults } = config

  let results: any[]

  if (mode === 'parallel') {
    // 🔥 并发执行所有请求
    results = await Promise.all(requests.map((req) => executeSingleRequest(req, payload, context)))
  } else {
    // 🔥 串行执行所有请求
    results = []
    for (const req of requests) {
      const result = await executeSingleRequest(req, payload, context)
      results.push(result)
    }
  }

  // 合并结果（如果提供了合并函数）
  if (combineResults) {
    return combineResults(results)
  }

  // 默认：返回结果数组
  return results
}
