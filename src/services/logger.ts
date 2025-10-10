/**
 * 统一前端日志系统
 * 核心设计：从"打印"到"分发" - 生成结构化日志事件，根据配置决定如何处理
 */

// 1. 定义日志级别
export const LogLevel = {
  DEBUG: 0,
  INFO: 1,
  WARN: 2,
  ERROR: 3,
  SILENT: 4, // 特殊级别，关闭所有日志
} as const

export type LogLevel = (typeof LogLevel)[keyof typeof LogLevel]

// 2. 定义日志事件的结构
export interface LogEvent {
  level: LogLevel
  tag: string
  message: string
  timestamp: string
  context?: Record<string, any>
  correlationId?: string
  sessionId?: string
}

// 3. 日志处理器的接口 (这是"分发"的核心)
export interface LogHandler {
  handle(event: LogEvent): void
}

// 4. 控制台处理器 - 开发环境友好的彩色输出
export class ConsoleHandler implements LogHandler {
  private levelColors: Record<LogLevel, string> = {
    [LogLevel.DEBUG]: '#6B7280', // 灰色
    [LogLevel.INFO]: '#10B981', // 绿色
    [LogLevel.WARN]: '#F59E0B', // 橙色
    [LogLevel.ERROR]: '#EF4444', // 红色
    [LogLevel.SILENT]: '#000000', // 黑色（不应该显示）
  }

  private levelNames: Record<LogLevel, string> = {
    [LogLevel.DEBUG]: 'DEBUG',
    [LogLevel.INFO]: 'INFO',
    [LogLevel.WARN]: 'WARN',
    [LogLevel.ERROR]: 'ERROR',
    [LogLevel.SILENT]: 'SILENT',
  }

  handle(event: LogEvent): void {
    const color = this.levelColors[event.level] || '#000000'
    const levelName = this.levelNames[event.level] || 'UNKNOWN'
    const time = new Date(event.timestamp).toLocaleTimeString()

    // 格式: [时间] [级别] [标签] 消息
    console.log(
      `%c[${time}] %c[${levelName}] %c[${event.tag}]%c ${event.message}`,
      'color: #9CA3AF; font-size: 11px;', // 时间
      `color: ${color}; font-weight: bold;`, // 级别
      `color: ${color}; font-weight: bold; background: ${color}15; padding: 1px 4px; border-radius: 2px;`, // 标签
      'color: inherit;', // 消息
      event.context || ''
    )
  }
}

// 5. 远程上报处理器 - 生产环境错误收集
export class RemoteHandler implements LogHandler {
  private queue: LogEvent[] = []
  private batchSize = 10
  private flushInterval = 2000 // 2秒
  private maxRetries = 3
  private retryDelay = 1000
  private endpoint: string

  constructor(endpoint: string = '/api/logs') {
    this.endpoint = endpoint
    this.startBatchFlush()
  }

  handle(event: LogEvent): void {
    // 只处理 WARN 和 ERROR 级别
    if (event.level >= LogLevel.WARN) {
      // 脱敏处理
      const sanitizedEvent = this.sanitizeEvent(event)
      this.queue.push(sanitizedEvent)

      // 如果是 ERROR 立即刷新
      if (event.level === LogLevel.ERROR) {
        this.flush()
      }
    }
  }

  private sanitizeEvent(event: LogEvent): LogEvent {
    const sensitiveKeys = ['password', 'token', 'cookie', 'authorization', 'email', 'phone']
    const sanitizedContext = { ...event.context }

    // 移除敏感字段
    sensitiveKeys.forEach((key) => {
      if (sanitizedContext && key in sanitizedContext) {
        delete sanitizedContext[key]
      }
    })

    return {
      ...event,
      context: sanitizedContext,
    }
  }

  private startBatchFlush(): void {
    setInterval(() => {
      if (this.queue.length > 0) {
        this.flush()
      }
    }, this.flushInterval)
  }

  private async flush(): Promise<void> {
    if (this.queue.length === 0) return

    const batch = this.queue.splice(0, this.batchSize)
    await this.sendBatch(batch, 0)
  }

  private async sendBatch(batch: LogEvent[], retryCount: number): Promise<void> {
    try {
      const response = await fetch(this.endpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          logs: batch,
          appVersion: import.meta.env.VITE_APP_VERSION || 'unknown',
          environment: import.meta.env.MODE,
        }),
      })

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`)
      }
    } catch (error) {
      if (retryCount < this.maxRetries) {
        // 指数退避重试
        setTimeout(
          () => {
            this.sendBatch(batch, retryCount + 1)
          },
          this.retryDelay * Math.pow(2, retryCount)
        )
      } else {
        console.error('Failed to send logs after retries:', error)
        // 可以考虑存储到 localStorage 作为离线缓存
        this.storeOffline(batch)
      }
    }
  }

  private storeOffline(batch: LogEvent[]): void {
    try {
      const offline = JSON.parse(localStorage.getItem('logger_offline') || '[]')
      offline.push(...batch)
      // 限制离线存储大小
      if (offline.length > 100) {
        offline.splice(0, offline.length - 100)
      }
      localStorage.setItem('logger_offline', JSON.stringify(offline))
    } catch (error) {
      console.error('Failed to store logs offline:', error)
    }
  }
}

// 6. 环形缓冲区处理器 - 用户操作轨迹记录
export class RingBufferHandler implements LogHandler {
  private buffer: LogEvent[] = []
  private maxSize: number

  constructor(maxSize: number = 50) {
    this.maxSize = maxSize
  }

  handle(event: LogEvent): void {
    // 只记录 INFO 和 DEBUG 级别的用户操作轨迹
    if (event.level <= LogLevel.INFO) {
      this.buffer.push(event)
      if (this.buffer.length > this.maxSize) {
        this.buffer.shift()
      }
    }
  }

  getRecentEvents(): LogEvent[] {
    return [...this.buffer]
  }

  clear(): void {
    this.buffer = []
  }
}

// 7. 采样配置
export interface SamplingConfig {
  debug: number
  info: number
  warn: number
  error: number
}

// 8. 核心 Logger 类
export class Logger {
  private handlers: LogHandler[] = []
  private level: LogLevel = LogLevel.DEBUG
  private tagFilters: string[] = []
  private sampling: SamplingConfig = {
    debug: 1.0,
    info: 1.0,
    warn: 1.0,
    error: 1.0,
  }
  private sessionId: string
  private ringBuffer?: RingBufferHandler

  constructor() {
    this.sessionId = this.generateSessionId()
    this.loadPersistedConfig()
    this.applySettings()
    this.setupDefaultHandlers()
  }

  private applySettings(): void {
    // 动态导入设置以避免循环依赖
    import('./loggerSettings')
      .then((settings) => {
        // 应用基础设置
        this.setLevel(settings.GLOBAL_LOG_LEVEL)

        // 应用采样率
        this.setSampling({
          debug: settings.SAMPLING_RATES[LogLevel.DEBUG],
          info: settings.SAMPLING_RATES[LogLevel.INFO],
          warn: settings.SAMPLING_RATES[LogLevel.WARN],
          error: settings.SAMPLING_RATES[LogLevel.ERROR],
        })

        // 根据设置配置处理器
        this.setupHandlers(settings)
      })
      .catch((error) => {
        console.warn('Failed to load logger settings:', error)
      })
  }

  private setupHandlers(settings: any): void {
    // 清除现有处理器
    this.handlers = []

    // 根据设置添加处理器
    if (settings.ENABLE_CONSOLE) {
      this.addHandler(new ConsoleHandler())
    }

    // 总是添加环形缓冲区处理器
    this.ringBuffer = new RingBufferHandler(settings.RING_BUFFER_SIZE || 50)
    this.addHandler(this.ringBuffer)

    if (settings.ENABLE_REMOTE && settings.REMOTE_ENDPOINT) {
      this.addHandler(new RemoteHandler(settings.REMOTE_ENDPOINT))
    }
  }

  private generateSessionId(): string {
    return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  private loadPersistedConfig(): void {
    try {
      const savedLevel = localStorage.getItem('logger.level')
      if (savedLevel && savedLevel in LogLevel) {
        this.level = (LogLevel as any)[savedLevel]
      }

      const savedTags = localStorage.getItem('logger.tags')
      if (savedTags) {
        this.tagFilters = JSON.parse(savedTags)
      }

      const savedSampling = localStorage.getItem('logger.sampling')
      if (savedSampling) {
        this.sampling = { ...this.sampling, ...JSON.parse(savedSampling) }
      }
    } catch (error) {
      console.warn('Failed to load persisted logger config:', error)
    }
  }

  private setupDefaultHandlers(): void {
    // 环境配置会异步设置处理器，这里提供默认的控制台处理器作为后备
    if (this.handlers.length === 0) {
      this.addHandler(new ConsoleHandler())
    }
  }

  // --- 配置方法 ---
  setLevel(level: LogLevel): void {
    this.level = level
    // 找到对应的级别名称
    const levelName =
      Object.keys(LogLevel).find((key) => (LogLevel as any)[key] === level) || 'UNKNOWN'
    localStorage.setItem('logger.level', levelName)
  }

  setTagFilters(tags: string[]): void {
    this.tagFilters = tags
    localStorage.setItem('logger.tags', JSON.stringify(tags))
  }

  setSampling(config: Partial<SamplingConfig>): void {
    this.sampling = { ...this.sampling, ...config }
    localStorage.setItem('logger.sampling', JSON.stringify(this.sampling))
  }

  addHandler(handler: LogHandler): void {
    this.handlers.push(handler)
  }

  // --- 黑白名单过滤方法 ---
  private settingsCache: any = null

  private shouldFilterBySettings(tag: string, level: LogLevel): boolean {
    // 使用缓存的设置进行同步检查
    if (this.settingsCache) {
      return this.settingsCache.shouldFilterTag(tag) || this.settingsCache.shouldFilterLevel(level)
    }

    // 如果没有缓存，异步加载设置
    if (!this.settingsCache) {
      import('./loggerSettings')
        .then((settings) => {
          this.settingsCache = settings
        })
        .catch(() => {
          // 加载失败时设置空缓存
          this.settingsCache = {
            shouldFilterTag: () => false,
            shouldFilterLevel: () => false,
          }
        })
    }

    // 首次加载时不过滤
    return false
  }

  // --- 核心日志记录方法 ---
  private log(
    level: LogLevel,
    tag: string,
    message: string,
    context?: Record<string, any>,
    correlationId?: string
  ): void {
    // 级别过滤
    if (level < this.level) {
      return
    }

    // 黑白名单过滤（同步检查）
    if (this.shouldFilterBySettings(tag, level)) {
      return
    }

    // 标签过滤（保留原有的运行时过滤功能）
    if (this.tagFilters.length > 0 && !this.tagFilters.some((filter) => tag.includes(filter))) {
      return
    }

    // 采样过滤
    const samplingRate = this.getSamplingRate(level)
    if (Math.random() > samplingRate) {
      return
    }

    // 增强上下文信息
    const enrichedContext = this.enrichContext(context)

    const event: LogEvent = {
      level,
      tag,
      message,
      timestamp: new Date().toISOString(),
      context: enrichedContext,
      correlationId,
      sessionId: this.sessionId,
    }

    // 如果是 ERROR 级别，附加用户操作轨迹
    if (level === LogLevel.ERROR && this.ringBuffer) {
      event.context = {
        ...event.context,
        userTrace: this.ringBuffer.getRecentEvents().slice(-10), // 最近10条操作
      }
    }

    // 分发给所有处理器
    this.handlers.forEach((handler) => {
      try {
        handler.handle(event)
      } catch (error) {
        console.error('Logger handler failed:', error)
      }
    })
  }

  private getSamplingRate(level: LogLevel): number {
    switch (level) {
      case LogLevel.DEBUG:
        return this.sampling.debug
      case LogLevel.INFO:
        return this.sampling.info
      case LogLevel.WARN:
        return this.sampling.warn
      case LogLevel.ERROR:
        return this.sampling.error
      default:
        return 1.0
    }
  }

  private enrichContext(context?: Record<string, any>): Record<string, any> {
    const baseContext = {
      appVersion: import.meta.env.VITE_APP_VERSION || 'unknown',
      environment: import.meta.env.MODE,
      userAgent: navigator.userAgent,
      url: window.location.href,
      timestamp: Date.now(),
    }

    return context ? { ...baseContext, ...context } : baseContext
  }

  // --- 公开的 API ---
  debug(tag: string, message: string, context?: Record<string, any>, correlationId?: string): void {
    this.log(LogLevel.DEBUG, tag, message, context, correlationId)
  }

  info(tag: string, message: string, context?: Record<string, any>, correlationId?: string): void {
    this.log(LogLevel.INFO, tag, message, context, correlationId)
  }

  warn(tag: string, message: string, context?: Record<string, any>, correlationId?: string): void {
    this.log(LogLevel.WARN, tag, message, context, correlationId)
  }

  error(
    tag: string,
    message: string,
    error?: Error,
    context?: Record<string, any>,
    correlationId?: string
  ): void {
    const fullContext = {
      ...context,
      errorName: error?.name,
      errorMessage: error?.message,
      stack: error?.stack,
    }
    this.log(LogLevel.ERROR, tag, message, fullContext, correlationId)
  }

  // --- 工具方法 ---
  createCorrelationId(): string {
    return `corr_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
  }

  getSessionId(): string {
    return this.sessionId
  }

  // --- 调试方法 ---
  getStats(): {
    level: string
    tagFilters: string[]
    sampling: SamplingConfig
    sessionId: string
    handlersCount: number
  } {
    return {
      level:
        Object.keys(LogLevel).find((key) => (LogLevel as any)[key] === this.level) || 'UNKNOWN',
      tagFilters: this.tagFilters,
      sampling: this.sampling,
      sessionId: this.sessionId,
      handlersCount: this.handlers.length,
    }
  }
}

// 9. 导出单例
export const logger = new Logger()

// 10. 便捷的标签常量 (基于你的项目结构)
export const LogTags = {
  // 组件相关
  COMPONENT_KANBAN: 'Component:InfiniteDailyKanban',
  COMPONENT_KANBAN_COLUMN: 'Component:Kanban:Column',
  COMPONENT_CALENDAR: 'Component:CuteCalendar',
  COMPONENT_BUTTON: 'Component:CuteButton',

  // 拖拽相关
  DRAG_CROSS_VIEW: 'Drag:CrossView',
  DRAG_CONTEXT: 'Drag:Context',
  DRAG_STRATEGY: 'Drag:Strategy',

  // API相关
  API_VIEW_ADAPTER: 'API:ViewAdapter',
  API_TASKS: 'API:Tasks',
  API_TIMEBLOCK: 'API:Timeblock',

  // Store相关
  STORE_TASKS: 'Store:Tasks',
  STORE_TIMEBLOCK: 'Store:Timeblock',
  STORE_VIEW: 'Store:View',
  STORE_AREA: 'Store:Area',
  STORE_TRASH: 'Store:Trash',
  STORE_TEMPLATE: 'Store:Template',

  // 路由相关
  ROUTER: 'Router',

  // 视图相关
  VIEW_HOME: 'View:Home',
  VIEW_TRASH: 'View:Trash',

  // 系统相关
  SYSTEM_INIT: 'System:Init',
  SYSTEM_SSE: 'System:SSE',
  SYSTEM_API: 'System:API',

  // 性能相关
  PERF: 'Perf',

  // 时区相关
  TIMEZONE: 'Timezone',

  // 关键业务
  CRITICAL: 'Critical',
} as const
