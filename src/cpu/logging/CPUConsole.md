# 🎨 CPU Console - 实时控制台打印系统

**设计目标**: 为开发者提供直观、美观、实时的指令执行反馈  
**核心原则**: 快速定位、分级显示、美观易读、零配置可用

---

## 📋 设计理念

### 日志审计 vs 控制台打印

```typescript
// CPULogger (审计系统)
// - 用途：事后分析、性能监控、问题排查
// - 特点：结构化、可查询、批量处理
// - 场景：找出所有执行超过100ms的指令、分析资源冲突

// CPUConsole (控制台系统)
// - 用途：实时查看指令是否正确执行
// - 特点：直观、美观、即时反馈
// - 场景：我拖动了任务，指令发了吗？成功了吗？
```

### 用户期望

开发者希望在控制台中快速看到：

```
✅ [12:34:56.123] task.update → 执行成功 (125ms)
├─ IF  0ms
├─ SCH 0ms
├─ EX  123ms (乐观更新 ✓ | 网络请求 ✓)
└─ WB  2ms

❌ [12:34:56.456] schedule.update → 失败 (24ms)
├─ 原因: database is locked
├─ 已回滚乐观更新 ✓
└─ 建议: 后端并发控制问题
```

---

## 🎨 CPUConsole 设计

### 核心类

```typescript
/**
 * CPU 控制台打印系统
 * 
 * 职责：
 * 1. 实时打印指令执行过程
 * 2. 美观的彩色输出
 * 3. 分级别控制详细程度
 * 4. 可折叠的详细信息
 */
export class CPUConsole {
  private enabled: boolean = true
  private level: ConsoleLevel = ConsoleLevel.NORMAL
  private filter: Set<string> = new Set() // 指令类型过滤
  
  /**
   * 控制台级别
   */
  enum ConsoleLevel {
    SILENT = 0,    // 不输出任何内容
    MINIMAL = 1,   // 只输出成功/失败
    NORMAL = 2,    // 输出关键阶段
    VERBOSE = 3,   // 输出所有细节
    DEBUG = 4,     // 输出调试信息（包括 payload）
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
  
  // ==================== 打印方法 ====================
  
  /**
   * 指令创建
   */
  onInstructionCreated(instruction: QueuedInstruction): void {
    if (!this.shouldPrint(instruction.type)) return
    
    if (this.level >= ConsoleLevel.NORMAL) {
      console.log(
        `%c🎯 ${this.formatTime()} %c${instruction.type}%c 指令创建`,
        'color: #666; font-size: 11px',
        'color: #3b82f6; font-weight: bold; background: #3b82f615; padding: 2px 6px; border-radius: 3px',
        'color: #666',
        {
          id: instruction.id,
          correlationId: instruction.context.correlationId,
          payload: this.level >= ConsoleLevel.DEBUG ? instruction.payload : '(use level=DEBUG to see)',
        }
      )
    }
  }
  
  /**
   * 指令成功
   */
  onInstructionSuccess(instruction: QueuedInstruction, duration: number): void {
    if (!this.shouldPrint(instruction.type)) return
    
    // 🎯 核心：折叠分组，方便查看
    console.groupCollapsed(
      `%c✅ ${this.formatTime()} %c${instruction.type}%c → 成功 %c${duration}ms`,
      'color: #666; font-size: 11px',
      'color: #10b981; font-weight: bold; background: #10b98115; padding: 2px 6px; border-radius: 3px',
      'color: #10b981',
      'color: #10b981; font-weight: bold'
    )
    
    // 显示流水线阶段
    if (this.level >= ConsoleLevel.NORMAL) {
      this.printPipelineStages(instruction)
    }
    
    // 显示详细信息
    if (this.level >= ConsoleLevel.VERBOSE) {
      this.printInstructionDetails(instruction)
    }
    
    console.groupEnd()
  }
  
  /**
   * 指令失败
   */
  onInstructionFailure(
    instruction: QueuedInstruction,
    error: Error,
    duration: number
  ): void {
    if (!this.shouldPrint(instruction.type)) return
    
    // 🔥 失败时自动展开，方便排查
    console.group(
      `%c❌ ${this.formatTime()} %c${instruction.type}%c → 失败 %c${duration}ms`,
      'color: #666; font-size: 11px',
      'color: #ef4444; font-weight: bold; background: #ef444415; padding: 2px 6px; border-radius: 3px',
      'color: #ef4444',
      'color: #ef4444; font-weight: bold'
    )
    
    // 显示错误信息
    console.error(`%c原因: ${error.message}`, 'color: #ef4444; font-weight: bold')
    
    // 显示流水线阶段
    this.printPipelineStages(instruction)
    
    // 显示是否回滚
    if (instruction.optimisticSnapshot) {
      console.log('%c✓ 已回滚乐观更新', 'color: #f59e0b; font-weight: bold')
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
      console.log(
        `%c  🔄 ${this.formatTime()} 乐观更新已应用`,
        'color: #8b5cf6',
        {
          instructionId: instruction.id,
          hasSnapshot: !!instruction.optimisticSnapshot,
        }
      )
    }
  }
  
  /**
   * 乐观更新回滚
   */
  onOptimisticRolledBack(instruction: QueuedInstruction, reason: string): void {
    if (!this.shouldPrint(instruction.type)) return
    
    // 回滚是重要事件，总是显示
    if (this.level >= ConsoleLevel.MINIMAL) {
      console.warn(
        `%c⚠️  ${this.formatTime()} %c${instruction.type}%c 乐观更新已回滚`,
        'color: #666; font-size: 11px',
        'color: #f59e0b; font-weight: bold; background: #f59e0b15; padding: 2px 6px; border-radius: 3px',
        'color: #f59e0b',
        {
          instructionId: instruction.id,
          reason,
        }
      )
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
      console.log(
        `%c  ⏳ ${this.formatTime()} 资源冲突，等待 ${waitTime}ms`,
        'color: #f59e0b',
        {
          instructionId: instruction.id,
          conflictingWith,
        }
      )
    }
  }
  
  /**
   * 网络请求
   */
  onNetworkRequest(
    instruction: QueuedInstruction,
    method: string,
    url: string
  ): void {
    if (!this.shouldPrint(instruction.type)) return
    
    if (this.level >= ConsoleLevel.DEBUG) {
      console.log(
        `%c  🌐 ${this.formatTime()} ${method} ${url}`,
        'color: #06b6d4',
        {
          instructionId: instruction.id,
          correlationId: instruction.context.correlationId,
        }
      )
    }
  }
  
  /**
   * 网络响应
   */
  onNetworkResponse(
    instruction: QueuedInstruction,
    status: number,
    latency: number
  ): void {
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
    
    // 计算各阶段耗时
    const stages = [
      { name: 'IF', time: timestamps.IF },
      { name: 'SCH', time: timestamps.SCH },
      { name: 'EX', time: timestamps.EX },
      { name: 'WB', time: timestamps.WB },
    ]
    
    let lastTime = timestamps.IF
    
    console.log('%c流水线阶段:', 'color: #666; font-weight: bold')
    
    for (const stage of stages) {
      if (stage.time) {
        const duration = stage.time - lastTime
        const bar = this.createDurationBar(duration)
        
        console.log(
          `  %c${stage.name}%c ${bar} %c${duration}ms`,
          'color: #3b82f6; font-weight: bold; min-width: 30px',
          'color: #666',
          'color: #666; font-weight: bold'
        )
        
        lastTime = stage.time
      }
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
      'Type': instruction.type,
      'Status': instruction.status,
      'Created At': new Date(instruction.timestamps.IF).toISOString(),
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
    const duration = instruction.timestamps.WB
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
    
    // 根据耗时着色
    if (duration < 50) {
      return `${bar}` // 绿色
    } else if (duration < 200) {
      return `${bar}` // 黄色
    } else {
      return `${bar}` // 红色
    }
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
    })
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
  printStats(stats: {
    total: number
    success: number
    failed: number
    avgLatency: number
  }): void {
    if (!this.enabled) return
    
    console.group('%c📊 流水线统计', 'color: #3b82f6; font-weight: bold; font-size: 14px')
    
    console.log(
      `  总指令数: %c${stats.total}`,
      'color: #3b82f6; font-weight: bold'
    )
    
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
    
    console.log(
      `  平均延迟: %c${stats.avgLatency.toFixed(0)}ms`,
      'color: #666; font-weight: bold'
    )
    
    console.groupEnd()
  }
}

// 导出全局单例
export const cpuConsole = new CPUConsole()

// 导出枚举
export { ConsoleLevel }
```

---

## 🎨 使用示例

### 1. 在流水线中集成

```typescript
// src/cpu/Pipeline.ts
import { cpuConsole } from './logging/CPUConsole'

export class Pipeline {
  async dispatch(type: string, payload: any): Promise<void> {
    const instruction = this.createInstruction(type, payload)
    
    // 🎯 打印指令创建
    cpuConsole.onInstructionCreated(instruction)
    
    try {
      await this.executeInstruction(instruction)
      
      const duration = instruction.timestamps.WB - instruction.timestamps.IF
      
      // 🎯 打印成功
      cpuConsole.onInstructionSuccess(instruction, duration)
    } catch (error) {
      const duration = Date.now() - instruction.timestamps.IF
      
      // 🎯 打印失败
      cpuConsole.onInstructionFailure(instruction, error as Error, duration)
    }
  }
}
```

### 2. 在 EX 阶段集成

```typescript
// src/cpu/stages/EX.ts
import { cpuConsole } from '../logging/CPUConsole'

export class ExecuteStage {
  async execute(instruction: QueuedInstruction): Promise<void> {
    // 应用乐观更新
    if (isa.optimistic?.enabled) {
      instruction.optimisticSnapshot = isa.optimistic.apply(
        instruction.payload,
        instruction.context
      )
      
      // 🎯 打印乐观更新
      cpuConsole.onOptimisticApplied(instruction)
    }
    
    // 网络请求
    if (isa.request) {
      const url = typeof isa.request.url === 'function' 
        ? isa.request.url(instruction.payload) 
        : isa.request.url
      
      // 🎯 打印网络请求
      cpuConsole.onNetworkRequest(instruction, isa.request.method, url)
      
      const startTime = Date.now()
      const result = await executeRequest(isa.request, instruction.payload, instruction.context)
      const latency = Date.now() - startTime
      
      // 🎯 打印网络响应
      cpuConsole.onNetworkResponse(instruction, 200, latency)
      
      instruction.result = result
    }
  }
}
```

### 3. 在 WB 阶段集成

```typescript
// src/cpu/stages/WB.ts
import { cpuConsole } from '../logging/CPUConsole'

export class WriteBackStage {
  private rollbackOptimisticUpdate(instruction: QueuedInstruction): void {
    const definition = ISA[instruction.type]
    
    if (instruction.optimisticSnapshot && definition?.optimistic?.rollback) {
      // 🎯 打印回滚
      cpuConsole.onOptimisticRolledBack(
        instruction,
        '指令执行失败'
      )
      
      definition.optimistic.rollback(instruction.optimisticSnapshot)
    }
  }
}
```

### 4. 在 SCH 阶段集成

```typescript
// src/cpu/stages/SCH.ts
import { cpuConsole } from '../logging/CPUConsole'

export class SchedulerStage {
  private detectConflict(instruction: QueuedInstruction): boolean {
    const conflicts = this.findConflictingInstructions(instruction)
    
    if (conflicts.length > 0) {
      // 🎯 打印资源冲突
      cpuConsole.onSchedulerConflict(
        instruction,
        conflicts.map(i => i.id),
        this.TICK_INTERVAL_MS
      )
      
      return true
    }
    
    return false
  }
}
```

### 5. 控制台配置

```typescript
// 开发环境：详细模式
if (import.meta.env.DEV) {
  cpuConsole.setLevel(ConsoleLevel.VERBOSE)
}

// 生产环境：只看失败
if (import.meta.env.PROD) {
  cpuConsole.setLevel(ConsoleLevel.MINIMAL)
}

// 只看特定指令
cpuConsole.setFilter(['schedule.update', 'task.create'])

// 完全关闭
cpuConsole.disable()
```

### 6. 在调试器中使用

```typescript
// src/views/CPUDebugView.vue
<script setup lang="ts">
import { cpuConsole, ConsoleLevel } from '@/cpu/logging/CPUConsole'

// 控制台级别选择器
const consoleLevel = ref(ConsoleLevel.NORMAL)

watch(consoleLevel, (level) => {
  cpuConsole.setLevel(level)
})

// 打印统计
function printStats() {
  const stats = pipeline.getStats()
  cpuConsole.printStats(stats)
}
</script>

<template>
  <div>
    <select v-model="consoleLevel">
      <option :value="ConsoleLevel.SILENT">关闭</option>
      <option :value="ConsoleLevel.MINIMAL">最小</option>
      <option :value="ConsoleLevel.NORMAL">正常</option>
      <option :value="ConsoleLevel.VERBOSE">详细</option>
      <option :value="ConsoleLevel.DEBUG">调试</option>
    </select>
    
    <button @click="printStats">打印统计</button>
  </div>
</template>
```

---

## 📊 输出效果示例

### 正常模式（NORMAL）

```
🎯 [12:34:56.123] task.update 指令创建

✅ [12:34:56.248] task.update → 成功 125ms
  流水线阶段:
  IF  ████ 0ms
  SCH ████ 0ms
  EX  ████████████████████ 123ms
  WB  ████ 2ms
  ✓ 乐观更新
```

### 详细模式（VERBOSE）

```
🎯 [12:34:56.123] task.update 指令创建
  { id: 'instr_xxx', correlationId: 'corr_xxx' }

  🔄 [12:34:56.124] 乐观更新已应用

  ⏳ [12:34:56.124] 资源冲突，等待 10ms
  { conflictingWith: ['instr_yyy'] }

✅ [12:34:56.248] task.update → 成功 125ms
  流水线阶段:
  IF  ████ 0ms
  SCH ████ 10ms
  EX  ████████████████████ 113ms
  WB  ████ 2ms
  ✓ 乐观更新
  
  详细信息:
  ┌────────────────┬─────────────────────┐
  │ Instruction ID │ instr_xxx           │
  │ Correlation ID │ corr_xxx            │
  │ Type           │ task.update         │
  │ Status         │ COMMITTED           │
  └────────────────┴─────────────────────┘
```

### 失败模式（自动展开）

```
❌ [12:34:56.456] schedule.update → 失败 24ms
  原因: HTTP 500: database is locked
  
  流水线阶段:
  IF  ████ 0ms
  SCH ████ 0ms
  EX  ████████████████ 22ms
  WB  ████ 2ms
  
  ✓ 已回滚乐观更新
  
  💡 建议:
  • 后端数据库锁定，检查写入许可是否正确获取
```

### 调试模式（DEBUG）

```
🎯 [12:34:56.123] task.update 指令创建
  { id: 'instr_xxx', correlationId: 'corr_xxx', payload: { ... } }

  🔄 [12:34:56.124] 乐观更新已应用

  🌐 [12:34:56.125] PATCH /api/tasks/123
  { correlationId: 'corr_xxx' }

  ← [12:34:56.246] HTTP 200 (121ms)

✅ [12:34:56.248] task.update → 成功 125ms
  流水线阶段: ...
  详细信息: ...
  Payload: { task_id: 'xxx', updates: { ... } }
  Result: { task: { ... }, side_effects: { ... } }
```

---

## 🎛️ 配置选项

### 级别说明

| 级别 | 用途 | 输出内容 |
|------|------|---------|
| SILENT | 完全关闭 | 无输出 |
| MINIMAL | 生产环境 | 只输出成功/失败 |
| NORMAL | 日常开发 | 输出关键阶段 |
| VERBOSE | 深度调试 | 输出所有细节 |
| DEBUG | 问题排查 | 输出 payload/result |

### 过滤器

```typescript
// 只看特定指令
cpuConsole.setFilter(['schedule.update', 'task.create'])

// 看所有指令
cpuConsole.setFilter([])
```

---

## 🎨 最佳实践

1. **开发环境用 VERBOSE**
   - 能看到完整的执行流程
   - 快速定位问题

2. **生产环境用 MINIMAL**
   - 减少控制台噪音
   - 只在出错时查看

3. **调试问题用 DEBUG**
   - 看到完整的 payload 和 result
   - 分析数据流动

4. **使用过滤器聚焦**
   - 开发某个功能时只看相关指令
   - 减少干扰

5. **配合 CPULogger 使用**
   - Console：实时看执行
   - Logger：事后深度分析

---

## 📦 完整架构

```
┌─────────────────────────────────────────┐
│           CPU Pipeline                   │
│  IF → SCH → EX → WB → INT               │
└──────────┬─────────────┬─────────────────┘
           │             │
           ↓             ↓
┌──────────────────┐  ┌──────────────────┐
│   CPUConsole     │  │   CPULogger      │
│  (实时打印)       │  │  (审计分析)       │
│                  │  │                  │
│ • 美观输出       │  │ • 结构化存储      │
│ • 分级控制       │  │ • 强大查询        │
│ • 即时反馈       │  │ • 性能分析        │
│ • 智能建议       │  │ • 离线审计        │
└──────────────────┘  └──────────────────┘
         │                    │
         ↓                    ↓
    开发调试              事后分析
    实时查看              性能监控
    快速定位              问题排查
```

---

**总结**：
- **CPUConsole**：开发者的"眼睛"，实时看到指令执行
- **CPULogger**：系统的"黑匣子"，记录所有细节用于分析
- 两者配合使用，覆盖"实时调试"和"事后分析"两个场景

