/**
 * HTTP客户端适配器
 *
 * 桥接项目的HTTP客户端实现
 */

import { apiGet, apiPost, apiPatch, apiDelete, apiPut } from '@/stores/shared'
import type { IHttpClient, IRequestConfig } from 'front-cpu'

export const httpAdapter: IHttpClient = {
  async get<T>(url: string, config?: IRequestConfig): Promise<T> {
    const correlationId = config?.headers?.['X-Correlation-ID']
    return await apiGet(url, correlationId)
  },

  async post<T>(url: string, data?: any, config?: IRequestConfig): Promise<T> {
    return await apiPost(url, data, config)
  },

  async patch<T>(url: string, data?: any, config?: IRequestConfig): Promise<T> {
    return await apiPatch(url, data, config)
  },

  async put<T>(url: string, data?: any, config?: IRequestConfig): Promise<T> {
    const correlationId = config?.headers?.['X-Correlation-ID']
    return await apiPut(url, data, correlationId)
  },

  async delete<T>(url: string, config?: IRequestConfig): Promise<T> {
    return await apiDelete(url, config)
  },
}
