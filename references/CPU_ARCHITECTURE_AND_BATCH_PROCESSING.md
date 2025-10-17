# CPU架构与批处理系统设计文档

## 概述

本文档总结了Cutie项目CPU架构的核心设计、指令批量处理能力分析、重排序缓冲区设计以及完整的调度策略框架。

## 目录

- [CPU核心代码统计](#cpu核心代码统计)
- [指令批量提交能力分析](#指令批量提交能力分析)
- [重排序缓冲区设计方案](#重排序缓冲区设计方案)
- [流水线阶段职责重新设计](#流水线阶段职责重新设计)
- [调度策略完整分析](#调度策略完整分析)

---

## CPU核心代码统计

### 代码规模统计

**核心代码（排除业务ISA）:**
- **总行数**: 3,367行
- **文件大小**: 91.2 KB (93,361 字节)
- **Gzip压缩后**: 23.6 KB (24,140 字节)
- **压缩比**: 74.1% (压缩率 3.87:1)

### 模块分布

| 模块 | 行数 | 占比 | 主要文件 |
|------|------|------|----------|
| **日志系统** | 1,768行 | 52.5% | CPUConsole, CPULogger, CPUEventCollector |
| **流水线阶段** | 472行 | 14.0% | IF, SCH, EX, RES, WB |
| **核心架构** | 452行 | 13.4% | Pipeline, index |
| **指令集架构** | 346行 | 10.3% | types, debug-isa, index |
| **中断处理** | 260行 | 7.7% | InterruptHandler |
| **工具模块** | 195行 | 5.8% | request, types |

### 文件详细分布

```
src/cpu/
├── Pipeline.ts (280行) - CPU流水线核心实现
├── stages/
│   ├── SCH.ts (166行) - 调度阶段
│   ├── WB.ts (97行) - 写回阶段
│   ├── EX.ts (96行) - 执行阶段
│   ├── IF.ts (84行) - 指令获取阶段
│   └── RES.ts (29行) - 响应阶段
├── logging/
│   ├── CPUConsole.ts (495行) - CPU控制台
│   ├── CPULogger.ts (432行) - CPU日志器
│   ├── CPUEventCollector.ts (332行) - 事件收集器
│   ├── CPUDebugger.ts (234行) - CPU调试器
│   └── stack-parser.ts (168行) - 堆栈解析器
├── isa/
│   ├── types.ts (138行) - ISA类型定义
│   ├── debug-isa.ts (182行) - 调试指令集
│   └── index.ts (26行) - 指令集聚合器
└── utils/
    └── request.ts (115行) - 请求工具
```

---

## 指令批量提交能力分析

### 现有支持能力 ✅

#### 1. 并发执行基础设施
- **调度器**: 支持最大10个指令并发执行 (`maxConcurrency = 10`)
- **资源管理**: 完整的资源冲突检测和释放机制
- **流水线**: 5阶段流水线支持乱序执行和并发处理

#### 2. 批量请求支持
- **MultiRequestConfig**: 单个指令可执行多个HTTP请求
- **执行模式**: 支持并发(`parallel`)和串行(`sequential`)两种模式
- **结果合并**: 提供`combineResults`函数自定义结果处理

#### 3. 异步Promise机制
- **Promise管理**: 每个指令都有独立的Promise resolver
- **异步返回**: 支持`await pipeline.dispatch()`等待结果
- **错误处理**: 完整的异常捕获和Promise rejection机制

### 性能提升评估

#### 当前单指令处理流程
```
dispatch() → IF → SCH → EX → RES → WB → Promise.resolve()
│         16ms   tick间隔   │
└─────── 每个指令独立处理 ─────┘
```

#### 批量处理流程
```
dispatchBatch() → IF(批量) → SCH(批量调度) → EX(并发执行) → Promise.all()
│                                          │
└──────── 批量提交，并发执行，统一返回 ──────┘
```

**预期性能提升**:
- **调度效率**: 减少tick循环次数，批量资源冲突检测
- **网络吞吐**: 利用HTTP/2多路复用，减少连接开销
- **Promise开销**: 使用`Promise.all()`替代多个独立Promise
- **整体延迟**: 从串行变并行，总时间接近最慢的单个请求

---

## 重排序缓冲区设计方案

### 设计目标

实现"**乱序执行，顺序（允许某个失败）/原子（一旦失败全部失败）写回**"的功能。

### 核心类型定义

```typescript
/**
 * 批处理执行模式
 */
export const BatchMode = {
  ORDERED: 'ordered',     // 顺序写回（允许部分失败）
  ATOMIC: 'atomic',       // 原子写回（一旦失败全部回滚）
} as const

/**
 * 批处理上下文
 */
export interface BatchContext {
  batchId: string
  mode: BatchMode
  totalInstructions: number
  sequenceNumber: number
  startTime: number
}

/**
 * 重排序缓冲区条目
 */
export interface ROBEntry {
  instruction: QueuedInstruction
  batchContext?: BatchContext
  completed: boolean
  success: boolean
  completionTime?: number
}

/**
 * 批处理状态
 */
export interface BatchStatus {
  batchId: string
  mode: BatchMode
  totalCount: number
  completedCount: number
  successCount: number
  failedCount: number
  canWriteBack: boolean
  allCompleted: boolean
}
```

### RES阶段重排序缓冲区实现

```typescript
export class ResponseStage {
  private reorderBuffer: Map<string, ROBEntry> = new Map()
  private batchStatuses: Map<string, BatchStatus> = new Map()

  processResponse(
    instruction: QueuedInstruction,
    error?: Error
  ): { success: boolean; shouldRetry: boolean; readyForWriteBack: ROBEntry[] } {
    // 标记RES阶段
    instruction.status = InstructionStatus.RESPONDED
    instruction.timestamps.RES = Date.now()

    const success = !error
    if (error) {
      instruction.error = error
    }

    // 创建ROB条目
    const robEntry: ROBEntry = {
      instruction,
      batchContext: instruction.batchContext,
      completed: true,
      success,
      completionTime: Date.now()
    }

    this.reorderBuffer.set(instruction.id, robEntry)

    // 处理批处理逻辑
    const readyForWriteBack = this.processBatchLogic(robEntry)

    return { success, shouldRetry: false, readyForWriteBack }
  }

  private processBatchLogic(completedEntry: ROBEntry): ROBEntry[] {
    if (!completedEntry.batchContext) {
      // 单个指令，直接写回
      return [completedEntry]
    }

    const batchId = completedEntry.batchContext.batchId

    // 更新批处理状态
    this.updateBatchStatus(batchId, completedEntry)

    // 检查是否可以写回
    return this.checkWriteBackReadiness(batchId)
  }

  private checkWriteBackReadiness(batchId: string): ROBEntry[] {
    const batchStatus = this.batchStatuses.get(batchId)!
    const batchEntries = this.getBatchEntries(batchId)

    // 原子模式：一旦有失败立即决定
    if (batchStatus.mode === BatchMode.ATOMIC && batchStatus.failedCount > 0) {
      batchStatus.canWriteBack = true
      return batchEntries // 全部回滚
    }

    // 顺序模式：等待所有指令完成，然后按顺序写回
    if (batchStatus.mode === BatchMode.ORDERED && batchStatus.allCompleted) {
      batchStatus.canWriteBack = true
      // 按序列号排序
      return batchEntries.sort((a, b) =>
        a.batchContext!.sequenceNumber - b.batchContext!.sequenceNumber
      )
    }

    return []
  }
}
```

### 使用示例

```typescript
// 顺序模式：允许部分失败
const results = await pipeline.dispatchBatch([
  { type: 'task.complete', payload: { id: 'task1' } },
  { type: 'task.complete', payload: { id: 'task2' } },  // 可能失败
  { type: 'task.complete', payload: { id: 'task3' } }
], BatchMode.ORDERED)
// results: [success_result, error, success_result]

// 原子模式：一旦失败全部回滚
try {
  const results = await pipeline.dispatchBatch([
    { type: 'task.create', payload: { title: '任务1' } },
    { type: 'task.create', payload: { title: '任务2' } },
    { type: 'schedule.create', payload: { date: '2025-10-18' } }
  ], BatchMode.ATOMIC)
} catch (error) {
  // 任何一个失败，所有乐观更新都被回滚
}
```

---

## 流水线阶段职责重新设计

### 职责分离原则

#### IF阶段：指令上下文构建器
- **核心职责**：根据调用方式构建完整的指令上下文
- **不负责**：调度逻辑、资源冲突检测、优化决策

#### SCH阶段：指令调度器
- **核心职责**：资源冲突检测、并发控制、发射时机决策
- **不负责**：指令上下文构建、结果排序

#### RES阶段：重排序缓冲区
- **核心职责**：根据IF提供的上下文信息进行正确的排序和写回控制

### IF阶段重新设计

```typescript
/**
 * IF阶段：Instruction Fetch & Context Building
 *
 * 职责：
 * 1. 识别指令调用模式（单个 vs 批处理）
 * 2. 为指令补充完整的执行上下文
 * 3. 为批处理指令分配序列号和批处理标识
 * 4. 构建标准化的指令数据结构
 */
export class InstructionFetchStage {
  private buffer: QueuedInstruction[] = []
  private idCounter = 0
  private batchCounter = 0

  /**
   * 单个指令上下文构建
   */
  buildSingleInstruction<TPayload>(
    type: string,
    payload: TPayload,
    source: 'user' | 'system' | 'test' = 'user',
    callSource?: CallSource
  ): QueuedInstruction<TPayload> {
    const instructionId = this.generateInstructionId()
    const correlationId = generateCorrelationId()

    const instruction: QueuedInstruction<TPayload> = {
      id: instructionId,
      type,
      payload,
      context: {
        instructionId,
        correlationId,
        timestamp: Date.now(),
        source,
        retryCount: 0,
        callSource,
      },
      status: InstructionStatus.PENDING,
      timestamps: { IF: Date.now() },
      batchContext: undefined // 单个指令没有批处理上下文
    }

    this.enqueue(instruction)
    return instruction
  }

  /**
   * 批处理指令上下文构建
   */
  buildBatchInstructions<TPayload>(
    instructions: Array<{ type: string; payload: TPayload }>,
    mode: BatchMode,
    source: 'user' | 'system' | 'test' = 'user',
    callSource?: CallSource
  ): QueuedInstruction<TPayload>[] {
    const batchId = `batch-${Date.now()}-${++this.batchCounter}`
    const batchStartTime = Date.now()

    return instructions.map((inst, index) => {
      const instructionId = this.generateInstructionId()
      const correlationId = generateCorrelationId()

      // 构建批处理上下文
      const batchContext: BatchContext = {
        batchId,
        mode,
        totalInstructions: instructions.length,
        sequenceNumber: index,  // 关键：序列号
        startTime: batchStartTime
      }

      const instruction: QueuedInstruction<TPayload> = {
        id: instructionId,
        type: inst.type,
        payload: inst.payload,
        context: {
          instructionId,
          correlationId,
          timestamp: Date.now(),
          source,
          retryCount: 0,
          callSource,
        },
        status: InstructionStatus.PENDING,
        timestamps: { IF: Date.now() },
        batchContext // 包含完整的批处理上下文
      }

      this.enqueue(instruction)
      return instruction
    })
  }
}
```

### Pipeline简化调用

```typescript
export class Pipeline {
  /**
   * 批量发射（使用IF的批处理上下文构建）
   */
  dispatchBatch<TPayload, TResult = any>(
    instructions: Array<{ type: string; payload: TPayload }>,
    mode: BatchMode = BatchMode.ORDERED,
    source: 'user' | 'system' | 'test' = 'user'
  ): Promise<TResult[]> {
    return new Promise((resolve, reject) => {
      const callSource = captureCallSource(1)

      // IF阶段构建完整的批处理上下文
      const queuedInstructions = this.IF.buildBatchInstructions(
        instructions, mode, source, callSource
      )

      const batchId = queuedInstructions[0].batchContext!.batchId
      this.batchPromiseResolvers.set(batchId, {
        resolve, reject, mode,
        instructions: queuedInstructions,
        results: new Array(instructions.length)
      })

      // SCH阶段只管调度，不需要知道批处理逻辑
      queuedInstructions.forEach(instruction => {
        this.SCH.addInstruction(instruction)
        cpuEventCollector.onInstructionCreated(instruction)
        cpuConsole.onInstructionCreated(instruction)
      })

      this.SCH.tick()
      this.processActiveInstructions()
      this.updateStatus()
    })
  }
}
```

---

## 调度策略完整分析

### 调度策略维度分析

#### 维度1：资源冲突处理策略

##### 1.1 阻塞型策略 (Blocking Strategies)
- **严格阻塞**: 同一资源只能有一个指令执行，后续指令完全阻塞
- **读写分离阻塞**: 读指令可以并发，写指令互斥
- **优先级阻塞**: 高优先级指令可以抢占资源，低优先级被阻塞
- **时间片阻塞**: 每个指令最多占用资源X秒，超时自动释放

##### 1.2 并发型策略 (Concurrent Strategies)
- **乐观并发+顺序写回**: 全部执行，按发射顺序写回
- **乐观并发+时间戳写回**: 全部执行，按完成时间写回
- **乐观并发+版本控制**: 全部执行，检测版本冲突后决定写回
- **乐观并发+优先级写回**: 全部执行，高优先级优先写回

##### 1.3 丢弃型策略 (Discard Strategies)
- **后发先至丢弃**: 执行所有，晚完成的被丢弃
- **先发先至丢弃**: 执行所有，早完成的被丢弃
- **单胜者模式**: 只有最快完成的指令被保留
- **多胜者模式**: 前N个完成的指令被保留

#### 维度2：时间窗口限制策略

##### 2.1 发射频率控制
- **固定时间窗口**: X秒内最多发射N条指令
- **滑动时间窗口**: 任意X秒窗口内最多N条指令
- **指数退避**: 频率过高时指数增加间隔时间
- **令牌桶**: 基于令牌桶算法的流量控制

##### 2.2 冷却期策略
- **全局冷却**: 任何指令发射后X秒内不能发射相同类型
- **资源冷却**: 特定资源操作后X秒内不能再次操作
- **用户冷却**: 特定用户X秒内不能发射相同指令
- **渐进冷却**: 冷却时间随频率动态调整

##### 2.3 超时处理
- **硬超时阻塞**: 超时指令直接阻塞，不进入队列
- **软超时降级**: 超时指令降低优先级
- **超时转异步**: 超时指令转为后台异步执行
- **超时合并**: 超时的相同指令自动合并

#### 维度3：相同指令处理策略

##### 3.1 去重策略
- **完全去重**: 完全相同的指令只保留一个
- **参数去重**: 相同类型+关键参数的指令去重
- **用户去重**: 同一用户的相同指令去重
- **会话去重**: 同一会话的相同指令去重

##### 3.2 合并策略
- **参数合并**: 相同类型指令的参数合并为一个批量操作
- **结果共享**: 一个指令执行，结果共享给所有相同指令
- **管道合并**: 相同指令序列合并为管道操作
- **时间合并**: X秒内的相同指令自动合并

##### 3.3 竞争策略
- **先到先得**: 第一个相同指令执行，后续阻塞/丢弃
- **后到优先**: 最新的相同指令执行，之前的取消
- **优先级竞争**: 高优先级的相同指令胜出
- **随机竞争**: 相同指令随机选择一个执行

#### 维度4：执行调度策略

##### 4.1 队列调度
- **FIFO**: 严格按发射顺序执行
- **LIFO**: 后发射先执行（栈模式）
- **优先级队列**: 按指令优先级执行
- **多级反馈队列**: 动态调整优先级的多级队列

##### 4.2 资源感知调度
- **负载均衡**: 平均分配到不同资源
- **亲和性调度**: 相关指令调度到同一资源
- **反亲和性调度**: 冲突指令分散到不同资源
- **热点避免**: 避免所有指令集中在热点资源

##### 4.3 智能调度
- **预测调度**: 基于历史数据预测最优调度时机
- **自适应调度**: 根据系统负载动态调整策略
- **机器学习调度**: 使用ML算法优化调度决策
- **反馈调度**: 根据执行结果反馈调整调度策略

#### 维度5：失败恢复策略

##### 5.1 重试策略
- **立即重试**: 失败后立即重新发射
- **延迟重试**: 等待X秒后重试
- **指数退避重试**: 重试间隔指数增长
- **智能重试**: 根据失败类型选择重试策略

##### 5.2 降级策略
- **功能降级**: 失败后使用简化版本的指令
- **性能降级**: 失败后降低指令优先级
- **服务降级**: 失败后使用备用服务
- **用户降级**: 失败用户使用限制版功能

### 复合调度策略组合

#### 组合策略1：高频交互场景
```
资源策略: 乐观并发+顺序写回
时间策略: 滑动窗口限流
相同指令: 结果共享
调度策略: 优先级队列
失败策略: 智能重试
```

#### 组合策略2：数据一致性关键场景
```
资源策略: 严格阻塞
时间策略: 固定冷却期
相同指令: 完全去重
调度策略: FIFO
失败策略: 立即失败
```

#### 组合策略3：实时性关键场景
```
资源策略: 后发先至丢弃
时间策略: 令牌桶限流
相同指令: 后到优先
调度策略: LIFO
失败策略: 功能降级
```

#### 组合策略4：批量处理场景
```
资源策略: 并发+版本控制
时间策略: 时间合并
相同指令: 参数合并
调度策略: 负载均衡
失败策略: 部分重试
```

### 特殊调度场景

#### 场景1：防抖动调度
- 短时间内多次相同操作只执行最后一次
- 适用于：搜索框输入、窗口resize事件

#### 场景2：节流调度
- 固定时间间隔内最多执行一次
- 适用于：滚动事件、API调用限制

#### 场景3：事务调度
- 相关指令必须全部成功或全部失败
- 适用于：订单处理、账户转账

#### 场景4：流水线调度
- 指令按特定顺序依次执行
- 适用于：数据处理管道、工作流

#### 场景5：竞态调度
- 多个指令竞争，只要有一个成功即可
- 适用于：多路径数据获取、冗余请求

#### 场景6：广播调度
- 一个指令触发多个相关操作
- 适用于：状态同步、通知分发

#### 场景7：条件调度
- 根据当前状态决定是否执行指令
- 适用于：状态机、条件操作

#### 场景8：优雅降级调度
- 系统负载高时自动降低服务质量
- 适用于：高峰期处理、紧急情况

---

## 总结

Cutie项目的CPU架构具备了强大的指令处理能力：

1. **成熟的基础设施** - 91.2KB的核心代码实现了完整的5阶段流水线
2. **批处理能力** - 支持乱序执行、顺序/原子写回的重排序缓冲区
3. **清晰的职责分离** - IF负责上下文构建，SCH负责调度，RES负责排序
4. **丰富的调度策略** - 涵盖5个维度、数十种策略的完整调度框架

这个架构为前端应用提供了类似现代CPU的强大并行处理能力，能够显著提升用户交互的响应性和系统的整体性能。

---

*文档生成时间: 2025-10-17*
*版本: v1.0*