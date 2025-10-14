import { waitForApiReady } from '@/composables/useApiConfig'

/**
 * 统一的 API 客户端
 *
 * 提供标准化的 API 调用接口，包含：
 * - 自动等待 API 就绪
 * - 统一的错误处理
 * - 标准的响应格式解析
 * - Correlation ID 支持
 */

export interface ApiRequestOptions extends RequestInit {
  correlationId?: string
}

export interface ApiResponse<T = any> {
  data: T
  timestamp?: string
  correlation_id?: string
}

/**
 * 执行 API 调用
 * @param endpoint API 端点（不包含 base URL）
 * @param options 请求选项
 * @returns 解析后的响应数据
 */
export async function apiCall<T = any>(
  endpoint: string,
  options: ApiRequestOptions = {}
): Promise<T> {
  const { correlationId, ...fetchOptions } = options

  const apiBaseUrl = await waitForApiReady()

  // 添加 Correlation ID 到请求头
  const headers = new Headers(fetchOptions.headers)
  if (correlationId) {
    headers.set('X-Correlation-ID', correlationId)
  }

  const response = await fetch(`${apiBaseUrl}${endpoint}`, {
    ...fetchOptions,
    headers,
  })

  if (!response.ok) {
    let errorMessage = `HTTP ${response.status}`
    try {
      const errorData = await response.json()
      errorMessage += `: ${JSON.stringify(errorData)}`
    } catch {
      // 忽略解析错误，使用基本错误信息
    }
    throw new Error(errorMessage)
  }

  // 处理 204 No Content 响应（DELETE 操作）
  if (response.status === 204) {
    return undefined as T
  }

  const result: ApiResponse<T> = await response.json()
  return result.data
}

/**
 * GET 请求的便捷方法
 */
export async function apiGet<T = any>(endpoint: string, correlationId?: string): Promise<T> {
  return apiCall<T>(endpoint, { method: 'GET', correlationId })
}

/**
 * POST 请求的便捷方法
 */
export async function apiPost<T = any>(
  endpoint: string,
  data?: any,
  options?: ApiRequestOptions
): Promise<T> {
  return apiCall<T>(endpoint, {
    ...options,
    method: 'POST',
    headers: { 'Content-Type': 'application/json', ...options?.headers },
    body: data ? JSON.stringify(data) : undefined,
  })
}

/**
 * PUT 请求的便捷方法
 */
export async function apiPut<T = any>(
  endpoint: string,
  data?: any,
  correlationId?: string
): Promise<T> {
  return apiCall<T>(endpoint, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: data ? JSON.stringify(data) : undefined,
    correlationId,
  })
}

/**
 * PATCH 请求的便捷方法
 */
export async function apiPatch<T = any>(
  endpoint: string,
  data?: any,
  options?: ApiRequestOptions
): Promise<T> {
  return apiCall<T>(endpoint, {
    ...options,
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json', ...options?.headers },
    body: data ? JSON.stringify(data) : undefined,
  })
}

/**
 * DELETE 请求的便捷方法
 */
export async function apiDelete<T = any>(
  endpoint: string,
  options?: ApiRequestOptions
): Promise<T> {
  return apiCall<T>(endpoint, { ...options, method: 'DELETE' })
}
