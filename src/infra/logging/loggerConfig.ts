/**
 * 日志系统环境配置
 * 根据不同环境设置不同的日志策略
 */

import { LogLevel, type SamplingConfig } from './logger'

export interface LoggerEnvironmentConfig {
  level: LogLevel
  remoteEndpoint?: string
  sampling: SamplingConfig
  enableConsole: boolean
  enableRemote: boolean
  enableRingBuffer: boolean
}

/**
 * 获取当前环境的日志配置
 */
export function getLoggerConfig(): LoggerEnvironmentConfig {
  const env = import.meta.env.MODE

  switch (env) {
    case 'development':
      return {
        level: LogLevel.DEBUG,
        enableConsole: true,
        enableRemote: false,
        enableRingBuffer: true,
        sampling: {
          debug: 1.0,
          info: 1.0,
          warn: 1.0,
          error: 1.0,
        },
      }

    case 'test':
      return {
        level: LogLevel.INFO,
        enableConsole: true,
        enableRemote: true,
        enableRingBuffer: true,
        remoteEndpoint: '/api/logs',
        sampling: {
          debug: 0.1,
          info: 0.5,
          warn: 1.0,
          error: 1.0,
        },
      }

    case 'production':
      return {
        level: LogLevel.WARN,
        enableConsole: false,
        enableRemote: true,
        enableRingBuffer: true,
        remoteEndpoint: '/api/logs',
        sampling: {
          debug: 0.0,
          info: 0.0,
          warn: 1.0,
          error: 1.0,
        },
      }

    default:
      // 默认配置（开发环境）
      return {
        level: LogLevel.DEBUG,
        enableConsole: true,
        enableRemote: false,
        enableRingBuffer: true,
        sampling: {
          debug: 1.0,
          info: 1.0,
          warn: 1.0,
          error: 1.0,
        },
      }
  }
}

/**
 * 从环境变量覆盖配置（如果存在）
 */
export function applyEnvironmentOverrides(
  config: LoggerEnvironmentConfig
): LoggerEnvironmentConfig {
  const overrides: Partial<LoggerEnvironmentConfig> = {}

  // 日志级别覆盖
  const envLevel = import.meta.env.VITE_LOG_LEVEL
  if (envLevel && envLevel in LogLevel) {
    overrides.level = (LogLevel as any)[envLevel]
  }

  // 远程端点覆盖
  const envEndpoint = import.meta.env.VITE_LOG_REMOTE_ENDPOINT
  if (envEndpoint) {
    overrides.remoteEndpoint = envEndpoint
    overrides.enableRemote = true
  }

  // 采样率覆盖
  const samplingOverrides: Partial<SamplingConfig> = {}

  const debugSampling = import.meta.env.VITE_LOG_SAMPLING_DEBUG
  if (debugSampling !== undefined) {
    samplingOverrides.debug = parseFloat(debugSampling) || 0
  }

  const infoSampling = import.meta.env.VITE_LOG_SAMPLING_INFO
  if (infoSampling !== undefined) {
    samplingOverrides.info = parseFloat(infoSampling) || 0
  }

  const warnSampling = import.meta.env.VITE_LOG_SAMPLING_WARN
  if (warnSampling !== undefined) {
    samplingOverrides.warn = parseFloat(warnSampling) || 0
  }

  const errorSampling = import.meta.env.VITE_LOG_SAMPLING_ERROR
  if (errorSampling !== undefined) {
    samplingOverrides.error = parseFloat(errorSampling) || 0
  }

  if (Object.keys(samplingOverrides).length > 0) {
    overrides.sampling = { ...config.sampling, ...samplingOverrides }
  }

  return { ...config, ...overrides }
}
