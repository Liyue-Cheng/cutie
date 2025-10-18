/**
 * CPU 控制台打印系统
 *
 * 职责：
 * 1. 实时打印指令执行过程
 * 2. 美观的彩色输出
 * 3. 分级别控制详细程度
 * 4. 可折叠的详细信息
 */

import type { QueuedInstruction } from '../types'
import { ConsoleLevel } from './types'
import { formatCallSourceShort } from './stack-parser'

export class CPUConsole {
  private enabled: boolean = true
  private level: ConsoleLevel = ConsoleLevel.NORMAL
  private filter: Set<string> = new Set() // 指令类型过滤

  constructor() {
    this.loadSettings()
  }

  /**
   * 加载设置
   */
  private loadSettings(): void {
    const savedLevel = localStorage.getItem('cpu-console-level')
    if (savedLevel) {
      this.level = parseInt(savedLevel) as ConsoleLevel
    }

    const savedFilter = localStorage.getItem('cpu-console-filter')
    if (savedFilter) {
      try {
        const types = JSON.parse(savedFilter)
        this.filter = new Set(types)
      } catch (e) {
        // 忽略解析错误
      }
    }
  }

  /**
   * 配置方法
   */
  setLevel(level: ConsoleLevel): void {
    this.level = level
    localStorage.setItem('cpu-console-level', level.toString())
  }

  setFilter(types: string[]): void {
    this.filter = new Set(types)
    localStorage.setItem('cpu-console-filter', JSON.stringify(types))
  }

  enable(): void {
    this.enabled = true
  }

  disable(): void {
    this.enabled = false
  }

  getLevel(): ConsoleLevel {
    return this.level
  }

  // ==================== 打印方法 ====================

  /**
   * 指令创建
   */
  onInstructionCreated(instruction: QueuedInstruction): void {
    if (!this.shouldPrint(instruction.type)) return

    if (this.level >= ConsoleLevel.NORMAL) {
      // 🔍 格式化调用源信息
      const callSourceInfo = instruction.context.callSource
        ? ` %c📍 ${formatCallSourceShort(instruction.context.callSource)}`
        : ''

      // 🎯 使用与指令成功一致的分组格式
      console.groupCollapsed(
        `%c[指令创建] %c${this.formatTime()} %c${instruction.type}%c${callSourceInfo}`,
        'color: #3b82f6; font-weight: bold',
        'color: #666; font-size: 11px',
        'color: #3b82f6; font-weight: bold; background: #3b82f615; padding: 2px 6px; border-radius: 3px',
        'color: #3b82f6',
        ...(callSourceInfo ? ['color: #8b5cf6; font-weight: bold'] : [])
      )

      // 🔥 显示指令基本信息
      console.log('%c📋 指令信息:', 'color: #3b82f6; font-weight: bold')
      console.table({
        'Instruction ID': instruction.id,
        'Correlation ID': instruction.context.correlationId,
        'Type': instruction.type,
        'Status': instruction.status,
        'Source': instruction.context.source,
        'Retry Count': instruction.context.retryCount,
      })

      // 🔥 显示指令参数
      if (this.level >= ConsoleLevel.DEBUG) {
        console.log('%c📝 指令参数 (Payload):', 'color: #3b82f6; font-weight: bold')
        console.log(instruction.payload)
      } else {
        console.log('%c📝 指令参数: (use level=DEBUG to see payload)', 'color: #666; font-style: italic')
      }

      // 🔥 显示调用源详情
      if (instruction.context.callSource && this.level >= ConsoleLevel.VERBOSE) {
        console.log('%c📍 调用源详情:', 'color: #8b5cf6; font-weight: bold')
        console.table({
          'File': instruction.context.callSource.file,
          'Line': instruction.context.callSource.line,
          'Column': instruction.context.callSource.column,
          'Function': instruction.context.callSource.function || 'N/A',
        })
      }

      console.groupEnd()
    }
  }

  /**
   * 指令成功
   */
  onInstructionSuccess(instruction: QueuedInstruction, duration: number): void {
    if (!this.shouldPrint(instruction.type)) return

    // 🔍 格式化调用源信息
    const callSourceInfo = instruction.context.callSource
      ? ` %c📍 ${formatCallSourceShort(instruction.context.callSource)}`
      : ''

    // 🎯 核心：折叠分组，方便查看
    console.groupCollapsed(
      `%c[指令成功] %c${this.formatTime()} %c${instruction.type}%c %c${duration}ms${callSourceInfo}`,
      'color: #10b981; font-weight: bold',
      'color: #666; font-size: 11px',
      'color: #10b981; font-weight: bold; background: #10b98115; padding: 2px 6px; border-radius: 3px',
      'color: #10b981',
      'color: #10b981; font-weight: bold',
      ...(callSourceInfo ? ['color: #8b5cf6; font-weight: bold'] : [])
    )

    // 🔥 显示指令输入参数
    if (this.level >= ConsoleLevel.NORMAL) {
      console.log('%c📝 指令参数 (Payload):', 'color: #3b82f6; font-weight: bold')
      console.log(instruction.payload)
    }

    // 🔥 显示后端返回结果
    if (instruction.result && this.level >= ConsoleLevel.NORMAL) {
      console.log('%c📥 后端返回 (Result):', 'color: #10b981; font-weight: bold')
      console.log(instruction.result)
    }

    // 🔥 显示WB阶段真实执行内容
    if (this.level >= ConsoleLevel.VERBOSE && instruction.writeBackExecution) {
      const wbExec = instruction.writeBackExecution
      console.log('%c💾 WB阶段执行记录:', 'color: #8b5cf6; font-weight: bold')

      if (wbExec.hasCommit) {
        if (wbExec.commitSuccess === true) {
          console.log('  ✅ commit() 函数执行成功')
          console.log('  📝 commit 调用参数:', wbExec.commitArgs)
        } else if (wbExec.commitSuccess === false) {
          console.log('  ❌ commit() 函数执行失败')
          console.log('  📝 commit 调用参数:', wbExec.commitArgs)
          console.log('  🚨 commit 错误:', wbExec.commitError)
        } else {
          console.log('  ⚠️  commit() 状态未知')
        }
      } else {
        console.log('  ⏭️  无 commit() 函数')
      }

      if (wbExec.rollbackExecuted) {
        console.log('  🔄 执行了乐观更新回滚')
        console.log('  📋 回滚快照:', wbExec.rollbackSnapshot)
        if (wbExec.rollbackError) {
          console.log('  🚨 回滚错误:', wbExec.rollbackError)
        }
      }

      // 显示中断处理器注册（成功时）
      if (instruction.status === 'committed') {
        console.log('  🎯 已注册到中断处理器 (SSE去重)')
      }
    }

    // 显示流水线阶段
    if (this.level >= ConsoleLevel.VERBOSE) {
      this.printPipelineStages(instruction)
    }

    // 显示详细信息
    if (this.level >= ConsoleLevel.DEBUG) {
      this.printInstructionDetails(instruction)
    }

    console.groupEnd()
  }

  /**
   * 指令失败
   */
  onInstructionFailure(instruction: QueuedInstruction, error: Error, duration: number): void {
    if (!this.shouldPrint(instruction.type)) return

    // 🔍 格式化调用源信息
    const callSourceInfo = instruction.context.callSource
      ? ` %c📍 ${formatCallSourceShort(instruction.context.callSource)}`
      : ''

    // 🔥 失败时自动展开，方便排查
    console.group(
      `%c[指令失败] %c${this.formatTime()} %c${instruction.type}%c %c${duration}ms${callSourceInfo}`,
      'color: #ef4444; font-weight: bold',
      'color: #666; font-size: 11px',
      'color: #ef4444; font-weight: bold; background: #ef444415; padding: 2px 6px; border-radius: 3px',
      'color: #ef4444',
      'color: #ef4444; font-weight: bold',
      ...(callSourceInfo ? ['color: #8b5cf6; font-weight: bold'] : [])
    )

    // 显示错误信息
    console.error(`%c原因: ${error.message}`, 'color: #ef4444; font-weight: bold')

    // 🔥 显示指令输入参数
    if (this.level >= ConsoleLevel.NORMAL) {
      console.log('%c📝 指令参数 (Payload):', 'color: #3b82f6; font-weight: bold')
      console.log(instruction.payload)
    }

    // 显示是否回滚
    if (instruction.optimisticSnapshot) {
      console.log('%c🔄 已回滚乐观更新', 'color: #f59e0b; font-weight: bold')
    }

    // 显示流水线阶段
    if (this.level >= ConsoleLevel.VERBOSE) {
      this.printPipelineStages(instruction)
    }

    // 显示详细信息
    if (this.level >= ConsoleLevel.VERBOSE) {
      this.printInstructionDetails(instruction)
      console.error('Error Stack:', error.stack)
    }

    // 🔥 智能建议
    this.printSuggestions(instruction, error)

    console.groupEnd()
  }

  /**
   * 乐观更新应用
   */
  onOptimisticApplied(instruction: QueuedInstruction): void {
    if (!this.shouldPrint(instruction.type)) return

    if (this.level >= ConsoleLevel.VERBOSE) {
      // 🔍 格式化调用源信息
      const callSourceInfo = instruction.context.callSource
        ? ` %c📍 ${formatCallSourceShort(instruction.context.callSource)}`
        : ''

      console.groupCollapsed(
        `%c[乐观更新] %c${this.formatTime()} %c${instruction.type}%c${callSourceInfo}`,
        'color: #8b5cf6; font-weight: bold',
        'color: #666; font-size: 11px',
        'color: #8b5cf6; font-weight: bold; background: #8b5cf615; padding: 2px 6px; border-radius: 3px',
        'color: #8b5cf6',
        ...(callSourceInfo ? ['color: #8b5cf6; font-weight: bold'] : [])
      )

      // 显示乐观更新的 payload
      if (this.level >= ConsoleLevel.DEBUG) {
        console.log('%c📝 更新内容:', 'color: #8b5cf6; font-weight: bold')
        console.log(instruction.payload)
      }

      // 显示快照信息
      if (instruction.optimisticSnapshot) {
        console.log('%c💾 已保存快照（用于回滚）', 'color: #10b981; font-size: 11px')
      }

      console.groupEnd()
    }
  }

  /**
   * 乐观更新回滚
   */
  onOptimisticRolledBack(instruction: QueuedInstruction, reason: string): void {
    if (!this.shouldPrint(instruction.type)) return

    // 回滚是重要事件，总是显示
    if (this.level >= ConsoleLevel.MINIMAL) {
      // 🔍 格式化调用源信息
      const callSourceInfo = instruction.context.callSource
        ? ` %c📍 ${formatCallSourceShort(instruction.context.callSource)}`
        : ''

      // 🔥 回滚重要，使用展开分组便于立即查看
      console.group(
        `%c[乐观回滚] %c${this.formatTime()} %c${instruction.type}%c${callSourceInfo}`,
        'color: #f59e0b; font-weight: bold',
        'color: #666; font-size: 11px',
        'color: #f59e0b; font-weight: bold; background: #f59e0b15; padding: 2px 6px; border-radius: 3px',
        'color: #f59e0b',
        ...(callSourceInfo ? ['color: #8b5cf6; font-weight: bold'] : [])
      )

      // 显示回滚原因
      console.log('%c⚠️ 回滚原因:', 'color: #f59e0b; font-weight: bold')
      console.log(reason)

      // 显示指令信息
      console.log('%c📋 指令信息:', 'color: #f59e0b; font-weight: bold')
      console.table({
        'Instruction ID': instruction.id,
        'Correlation ID': instruction.context.correlationId,
        'Type': instruction.type,
      })

      console.groupEnd()
    }
  }

  /**
   * 资源冲突
   */
  onSchedulerConflict(
    instruction: QueuedInstruction,
    conflictingWith: string[],
    waitTime: number
  ): void {
    if (!this.shouldPrint(instruction.type)) return

    if (this.level >= ConsoleLevel.VERBOSE) {
      console.log(`%c  ⏳ ${this.formatTime()} 资源冲突，等待 ${waitTime}ms`, 'color: #f59e0b', {
        instructionId: instruction.id,
        conflictingWith,
      })
    }
  }

  /**
   * 网络请求
   */
  onNetworkRequest(instruction: QueuedInstruction, method: string, url: string): void {
    if (!this.shouldPrint(instruction.type)) return

    if (this.level >= ConsoleLevel.DEBUG) {
      console.log(`%c  🌐 ${this.formatTime()} ${method} ${url}`, 'color: #06b6d4', {
        instructionId: instruction.id,
        correlationId: instruction.context.correlationId,
      })
    }
  }

  /**
   * 网络响应
   */
  onNetworkResponse(instruction: QueuedInstruction, status: number, latency: number): void {
    if (!this.shouldPrint(instruction.type)) return

    if (this.level >= ConsoleLevel.DEBUG) {
      const statusColor = status >= 200 && status < 300 ? '#10b981' : '#ef4444'
      console.log(
        `%c  ← ${this.formatTime()} HTTP ${status} (${latency}ms)`,
        `color: ${statusColor}`,
        {
          instructionId: instruction.id,
        }
      )
    }
  }

  // ==================== 辅助方法 ====================

  /**
   * 打印流水线阶段
   */
  private printPipelineStages(instruction: QueuedInstruction): void {
    const timestamps = instruction.timestamps

    console.log('%c流水线阶段:', 'color: #666; font-weight: bold')

    // 打印各阶段之间的耗时
    const transitions = []

    if (timestamps.IF && timestamps.SCH) {
      transitions.push({ label: 'IF→SCH', duration: timestamps.SCH - timestamps.IF })
    }
    if (timestamps.SCH && timestamps.EX) {
      transitions.push({ label: 'SCH→EX', duration: timestamps.EX - timestamps.SCH })
    }
    if (timestamps.EX && timestamps.WB) {
      transitions.push({ label: 'EX→WB', duration: timestamps.WB - timestamps.EX })
    }

    for (const transition of transitions) {
      const bar = this.createDurationBar(transition.duration)
      console.log(
        `  %c${transition.label}%c ${bar} %c${transition.duration}ms`,
        'color: #3b82f6; font-weight: bold',
        'color: #666',
        'color: #666; font-weight: bold'
      )
    }

    // 打印总耗时
    if (timestamps.IF && timestamps.WB) {
      const total = timestamps.WB - timestamps.IF
      console.log(
        `  %c总耗时: %c${total}ms`,
        'color: #666; font-weight: bold',
        'color: #10b981; font-weight: bold; font-size: 14px'
      )
    }

    // 特殊标记
    if (instruction.optimisticSnapshot) {
      console.log('  %c✓ 乐观更新', 'color: #8b5cf6')
    }
  }

  /**
   * 打印指令详情
   */
  private printInstructionDetails(instruction: QueuedInstruction): void {
    console.log('%c详细信息:', 'color: #666; font-weight: bold')
    console.table({
      'Instruction ID': instruction.id,
      'Correlation ID': instruction.context.correlationId,
      Type: instruction.type,
      Status: instruction.status,
      'Created At': instruction.timestamps.IF
        ? new Date(instruction.timestamps.IF).toISOString()
        : 'N/A',
    })

    if (this.level >= ConsoleLevel.DEBUG) {
      console.log('%cPayload:', 'color: #666; font-weight: bold', instruction.payload)

      if (instruction.result) {
        console.log('%cResult:', 'color: #666; font-weight: bold', instruction.result)
      }
    }
  }

  /**
   * 打印智能建议
   */
  private printSuggestions(instruction: QueuedInstruction, error: Error): void {
    const suggestions: string[] = []

    // 根据错误类型给出建议
    if (error.message.includes('database is locked')) {
      suggestions.push('后端数据库锁定，检查写入许可是否正确获取')
    }

    if (error.message.includes('Network')) {
      suggestions.push('网络错误，检查后端服务是否运行')
    }

    if (error.message.includes('timeout')) {
      suggestions.push('请求超时，考虑增加超时时间或优化后端性能')
    }

    // 根据指令类型给出建议
    const duration =
      instruction.timestamps.WB && instruction.timestamps.IF
        ? instruction.timestamps.WB - instruction.timestamps.IF
        : 0

    if (duration > 1000) {
      suggestions.push(`执行耗时 ${duration}ms，超过 1 秒，检查是否存在性能问题`)
    }

    if (suggestions.length > 0) {
      console.log('%c💡 建议:', 'color: #f59e0b; font-weight: bold')
      suggestions.forEach((s) => {
        console.log(`  • ${s}`)
      })
    }
  }

  /**
   * 创建耗时条形图
   */
  private createDurationBar(duration: number): string {
    const maxWidth = 20
    const width = Math.min(Math.round(duration / 50), maxWidth)
    const bar = '█'.repeat(width)

    return bar
  }

  /**
   * 格式化时间
   */
  private formatTime(): string {
    const now = new Date()
    return now.toLocaleTimeString('zh-CN', {
      hour12: false,
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      fractionalSecondDigits: 3,
    } as any)
  }

  /**
   * 判断是否应该打印
   */
  private shouldPrint(instructionType: string): boolean {
    if (!this.enabled) return false
    if (this.level === ConsoleLevel.SILENT) return false
    if (this.filter.size > 0 && !this.filter.has(instructionType)) return false
    return true
  }

  // ==================== 便捷方法 ====================

  /**
   * 打印分隔线
   */
  printSeparator(title?: string): void {
    if (!this.enabled) return

    if (title) {
      console.log(
        `%c━━━━━━━━━━━━━━━━━━━━━━━━━━ ${title} ━━━━━━━━━━━━━━━━━━━━━━━━━━`,
        'color: #666; font-weight: bold'
      )
    } else {
      console.log('%c━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━', 'color: #666')
    }
  }

  /**
   * 打印统计信息
   */
  printStats(stats: { total: number; success: number; failed: number; avgLatency: number }): void {
    if (!this.enabled) return

    console.group('%c📊 流水线统计', 'color: #3b82f6; font-weight: bold; font-size: 14px')

    console.log(`  总指令数: %c${stats.total}`, 'color: #3b82f6; font-weight: bold')

    console.log(
      `  成功: %c${stats.success} %c(${((stats.success / stats.total) * 100).toFixed(1)}%)`,
      'color: #10b981; font-weight: bold',
      'color: #666'
    )

    console.log(
      `  失败: %c${stats.failed} %c(${((stats.failed / stats.total) * 100).toFixed(1)}%)`,
      'color: #ef4444; font-weight: bold',
      'color: #666'
    )

    console.log(`  平均延迟: %c${stats.avgLatency.toFixed(0)}ms`, 'color: #666; font-weight: bold')

    console.groupEnd()
  }
}

// 导出全局单例
export const cpuConsole = new CPUConsole()

// 导出枚举
export { ConsoleLevel }
