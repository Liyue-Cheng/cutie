/**
 * 事件订阅服务 - SSE客户端
 *
 * 负责建立与后端 SSE 端点的连接，并将领域事件分发到各个 Store
 */

/// 领域事件接口（与后端保持一致）
export interface DomainEvent {
  event_id: string
  event_type: string
  version: number
  aggregate_type: string
  aggregate_id: string
  aggregate_version: number | null
  correlation_id: string | null
  occurred_at: string
  payload: Record<string, any>
}

/// 事件处理器类型
export type EventHandler = (event: DomainEvent) => void | Promise<void>

/// 事件订阅器
export class EventSubscriber {
  private eventSource: EventSource | null = null
  private handlers: Map<string, EventHandler[]> = new Map()
  private reconnectAttempts = 0
  private maxReconnectAttempts = 10
  private reconnectDelay = 1000
  private isManualClose = false
  private apiBaseUrl: string

  constructor(apiBaseUrl: string) {
    this.apiBaseUrl = apiBaseUrl
  }

  // 连接到 SSE 端点
  connect() {
    if (this.eventSource) {
      console.warn('[EventSubscriber] Already connected')
      return
    }

    this.isManualClose = false
    const url = `${this.apiBaseUrl}/events/stream`
    console.log('[EventSubscriber] Connecting to', url)

    this.eventSource = new EventSource(url)

    // 监听所有事件类型（通过 event 字段区分）
    this.eventSource.addEventListener('task.completed', (e: MessageEvent) => {
      this.handleEvent('task.completed', e.data)
    })

    this.eventSource.addEventListener('task.updated', (e: MessageEvent) => {
      this.handleEvent('task.updated', e.data)
    })

    this.eventSource.addEventListener('task.deleted', (e: MessageEvent) => {
      this.handleEvent('task.deleted', e.data)
    })

    this.eventSource.addEventListener('time_blocks.deleted', (e: MessageEvent) => {
      this.handleEvent('time_blocks.deleted', e.data)
    })

    this.eventSource.addEventListener('time_blocks.truncated', (e: MessageEvent) => {
      this.handleEvent('time_blocks.truncated', e.data)
    })

    // 连接成功
    this.eventSource.onopen = () => {
      console.log('[EventSubscriber] Connected to event stream')
      this.reconnectAttempts = 0
    }

    // 连接错误
    this.eventSource.onerror = (error) => {
      console.error('[EventSubscriber] Connection error:', error)
      this.eventSource?.close()
      this.eventSource = null

      // 自动重连
      if (!this.isManualClose && this.reconnectAttempts < this.maxReconnectAttempts) {
        this.reconnectAttempts++
        const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1)
        console.log(
          `[EventSubscriber] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`
        )
        setTimeout(() => this.connect(), delay)
      }
    }
  }

  /// 断开连接
  disconnect(): void {
    this.isManualClose = true
    if (this.eventSource) {
      this.eventSource.close()
      this.eventSource = null
      console.log('[EventSubscriber] Disconnected')
    }
  }

  /// 订阅特定事件类型
  on(eventType: string, handler: EventHandler): void {
    if (!this.handlers.has(eventType)) {
      this.handlers.set(eventType, [])
    }
    this.handlers.get(eventType)!.push(handler)
  }

  /// 取消订阅
  off(eventType: string, handler: EventHandler): void {
    const handlers = this.handlers.get(eventType)
    if (handlers) {
      const index = handlers.indexOf(handler)
      if (index > -1) {
        handlers.splice(index, 1)
      }
    }
  }

  /// 处理接收到的事件
  private handleEvent(eventType: string, data: string): void {
    try {
      const event: DomainEvent = JSON.parse(data)
      console.log(`[EventSubscriber] Received event: ${eventType}`, event)

      const handlers = this.handlers.get(eventType) || []
      for (const handler of handlers) {
        try {
          const result = handler(event)
          if (result instanceof Promise) {
            result.catch((err) => {
              console.error(`[EventSubscriber] Handler error for ${eventType}:`, err)
            })
          }
        } catch (err) {
          console.error(`[EventSubscriber] Handler error for ${eventType}:`, err)
        }
      }
    } catch (err) {
      console.error('[EventSubscriber] Failed to parse event data:', err)
    }
  }
}

/// 全局单例（在 main.ts 中初始化）
let globalSubscriber: EventSubscriber | null = null

export function initEventSubscriber(apiBaseUrl: string): EventSubscriber {
  if (globalSubscriber) {
    console.warn('[EventSubscriber] Already initialized')
    return globalSubscriber
  }

  globalSubscriber = new EventSubscriber(apiBaseUrl)
  globalSubscriber.connect()
  return globalSubscriber
}

export function getEventSubscriber(): EventSubscriber | null {
  return globalSubscriber
}
