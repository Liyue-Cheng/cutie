# CPU 五级流水线架构 - 完整文档

> **Frontend as a CPU** - 前端指令执行系统，采用类似现代CPU的五级流水线架构设计

## 📚 目录

- [概述](#概述)
- [核心概念](#核心概念)
- [架构设计](#架构设计)
- [指令生命周期详解](#指令生命周期详解) ⭐️
- [快速开始](#快速开始)
- [详细文档](#详细文档)
  - [指令集 (ISA)](#指令集-isa)
  - [流水线阶段](#流水线阶段)
  - [指令追踪系统](#指令追踪系统)
- [使用指南](#使用指南)
- [可视化调试](#可视化调试)
- [扩展开发](#扩展开发)
- [指令迁移](#指令迁移) 🔄
- [技术细节](#技术细节)
- [常见问题](#常见问题)

---

## 概述

这是一个实验性的前端架构系统，将现代CPU的流水线设计理念应用到前端指令执行中。通过模拟CPU的五级流水线（IF-SCH-EX-RES-WB），实现了：

- ✅ **乱序执行**：不同资源的指令可以并行执行
- ✅ **资源冲突检测**：自动管理指令间的依赖关系
- ✅ **指令调度优化**：最大化并发执行
- ✅ **完整的可观测性**：每条指令的执行过程都可追踪
- ✅ **类型安全**：完整的TypeScript支持

### 为什么这样设计？

传统前端架构按顺序执行操作：

```
操作A → 等待完成 → 操作B → 等待完成 → 操作C
```

CPU流水线架构允许并行执行：

```
操作A、操作B、操作C 同时进行（如果没有资源冲突）
```

**实际场景：**

```typescript
// 传统方式：顺序执行，总耗时约3秒
await completeTask(task1) // 1秒
await completeTask(task2) // 1秒
await updateTask(task3) // 1秒

// CPU流水线：并行执行，总耗时约1秒
pipeline.dispatch('task.complete', { id: task1.id }) // 并行
pipeline.dispatch('task.complete', { id: task2.id }) // 并行
pipeline.dispatch('task.update', { id: task3.id }) // 并行
```

---

## 核心概念

### 1. 指令 (Instruction)

指令是系统的基本执行单元，类似于CPU指令。

**结构：**

```typescript
{
  id: 'instr-1234567890-abc',           // 指令唯一ID
  type: 'debug.fetch_baidu',            // 指令类型
  payload: { /* 参数 */ },              // 指令负载
  status: 'executing',                  // 当前状态
  timestamps: {                         // 各阶段时间戳
    IF: 1697123456789,
    SCH: 1697123456800,
    EX: 1697123456816,
    // ...
  }
}
```

### 2. 资源 (Resource)

资源标识符用于检测指令冲突。同一资源的指令必须顺序执行。

**示例：**

```typescript
// 操作同一个任务 → 相同资源ID → 顺序执行
task.complete(task1) // resourceId: ['task:task1']
task.update(task1) // resourceId: ['task:task1']  ← 必须等待上面完成

// 操作不同任务 → 不同资源ID → 并行执行
task.complete(task1) // resourceId: ['task:task1']
task.complete(task2) // resourceId: ['task:task2']  ← 可以并行
```

### 3. 流水线阶段 (Pipeline Stage)

```
IF (Instruction Fetch)    → 指令获取
SCH (Scheduler)          → 指令调度
EX (Execute)             → 执行
RES (Response)           → 响应处理
WB (Write Back)          → 写回
```

每条指令按顺序经过这些阶段，但不同指令可以同时在不同阶段执行。

---

## 架构设计

```
┌─────────────────────────────────────────────────────────────────┐
│                         组件层                                   │
│  (Vue Component)  →  pipeline.dispatch('type', payload)         │
└──────────────────────────────┬──────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│  IF阶段: Instruction Fetch                                       │
│  • 生成 instructionId 和 correlationId                          │
│  • 开始追踪                                                      │
│  • 放入IF缓冲区                                                  │
└──────────────────────────────┬──────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│  SCH阶段: Scheduler (核心！)                                     │
│  • 检查并发数限制 (max: 10)                                      │
│  • 检查资源冲突                                                  │
│  • 发射可执行的指令                                              │
│  • 维护 pendingQueue 和 activeInstructions                      │
└──────────────────────────────┬──────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│  EX阶段: Execute                                                 │
│  • 前置验证 (validate)                                           │
│  • 执行网络请求/操作 (execute)                                   │
│  • 异步执行，不阻塞流水线                                        │
└──────────────────────────────┬──────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│  RES阶段: Response                                               │
│  • 处理执行结果                                                  │
│  • 成功 → 传递到WB                                               │
│  • 失败 → 标记失败                                               │
└──────────────────────────────┬──────────────────────────────────┘
                               ↓
┌─────────────────────────────────────────────────────────────────┐
│  WB阶段: Write Back                                              │
│  • 成功：标记 committed                                          │
│  • 失败：标记 failed                                             │
│  • 释放资源占用                                                  │
│  • 完成追踪记录                                                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  追踪系统 (InstructionTracker)                                   │
│  • 记录各阶段时间戳                                              │
│  • 记录网络请求结果                                              │
│  • 计算各阶段耗时                                                │
│  • 控制台日志输出                                                │
└─────────────────────────────────────────────────────────────────┘
```

---

## 指令生命周期详解

> 从组件调用到完成执行，一条指令的完整旅程

### 完整执行流程

让我们跟踪一条指令从发射到完成的全过程：

```typescript
// 组件中的调用
pipeline.dispatch('debug.quick_success', { data: 'hello' })
```

#### 时间线视图

```
T0      组件调用 dispatch()
  ↓
T0+0ms  IF阶段: 生成ID、放入缓冲区
  ↓
T0+2ms  SCH阶段: 检查冲突、发射指令
  ↓
T0+5ms  EX阶段: 开始执行
  ↓
T0+150ms RES阶段: 收到响应
  ↓
T0+154ms WB阶段: 完成写回、释放资源
  ↓
T0+156ms 追踪系统: 记录完成
```

---

### 阶段1: 组件发射指令

**文件：** 任意Vue组件或JavaScript代码

```typescript
// 示例：在Vue组件中
import { pipeline } from '@/cpu'

function handleAction() {
  pipeline.dispatch('debug.quick_success', {
    data: 'hello',
    timestamp: Date.now(),
  })
}
```

**调用栈：**

```
Component.handleAction()
  → pipeline.dispatch(type, payload, source)
```

---

### 阶段2: Pipeline.dispatch() - 流水线入口

**文件：** `src/cpu/Pipeline.ts`

**函数：** `dispatch<TPayload>(type: string, payload: TPayload, source)`

```typescript
// Pipeline.ts Line 53-69
dispatch<TPayload>(
  type: string,
  payload: TPayload,
  source: 'user' | 'system' | 'test' = 'user'
): void {
  // 步骤1: IF阶段 - 获取指令
  const instruction = this.IF.fetchInstruction(type, payload, source)

  // 步骤2: 加入调度队列
  this.SCH.addInstruction(instruction)

  // 步骤3: 立即尝试调度（快速响应）
  this.SCH.tick()

  // 步骤4: 更新流水线状态
  this.updateStatus()
}
```

**做了什么：**

1. 调用IF阶段的 `fetchInstruction()` 创建指令对象
2. 将指令加入SCH调度队列
3. 立即触发一次调度（不等待定时tick）
4. 更新响应式状态供UI显示

**数据流：**

```
{ type, payload, source }
  → instruction 对象
  → SCH.pendingQueue
```

---

### 阶段3: IF.fetchInstruction() - 指令获取

**文件：** `src/cpu/stages/IF.ts`

**函数：** `fetchInstruction<TPayload>(type, payload, source)`

```typescript
// IF.ts Line 15-43
fetchInstruction<TPayload>(
  type: string,
  payload: TPayload,
  source: 'user' | 'system' | 'test' = 'user'
): QueuedInstruction<TPayload> {
  // 1️⃣ 生成唯一指令ID
  const instructionId = `instr-${Date.now()}-${++this.idCounter}`

  // 2️⃣ 生成关联ID（用于SSE去重）
  const correlationId = generateCorrelationId()

  // 3️⃣ 创建指令对象
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
    },
    status: InstructionStatus.PENDING,
    timestamps: {
      IF: Date.now(),
    },
  }

  // 4️⃣ 开始追踪
  instructionTracker.startInstruction(instructionId, type, payload, correlationId)

  // 5️⃣ 放入IF缓冲区
  this.enqueue(instruction)

  return instruction
}
```

**做了什么：**

1. 生成唯一的 `instructionId`（格式：`instr-时间戳-计数器`）
2. 生成 `correlationId`（用于追踪和SSE去重）
3. 创建完整的指令对象（包含payload、context、status等）
4. 调用追踪系统开始记录
5. 放入IF缓冲区（虽然目前立即传递，但保留了缓冲机制）

**创建的对象结构：**

```typescript
{
  id: 'instr-1697123456789-1',
  type: 'debug.quick_success',
  payload: { data: 'hello', timestamp: 1697123456789 },
  context: {
    instructionId: 'instr-1697123456789-1',
    correlationId: 'req-1697123456789-abc',
    timestamp: 1697123456789,
    source: 'user',
    retryCount: 0
  },
  status: 'pending',
  timestamps: {
    IF: 1697123456789
  }
}
```

**涉及的函数调用：**

```
IF.fetchInstruction()
  → generateCorrelationId()          // 来自 @/infra/correlation/correlationId
  → instructionTracker.startInstruction()  // 来自追踪系统
  → IF.enqueue()
```

---

### 阶段4: SCH.addInstruction() - 加入调度队列

**文件：** `src/cpu/stages/SCH.ts`

**函数：** `addInstruction(instruction: QueuedInstruction)`

```typescript
// SCH.ts Line 38-40
addInstruction(instruction: QueuedInstruction): void {
  this.pendingQueue.push(instruction)
}
```

**做了什么：**

- 将指令加入待调度队列（`pendingQueue`）
- 等待调度器的 `tick()` 处理

**数据结构变化：**

```
SCH.pendingQueue: []
  → ['instr-1697123456789-1']
```

---

### 阶段5: SCH.tick() - 指令调度（核心）

**文件：** `src/cpu/stages/SCH.ts`

**函数：** `tick()`

```typescript
// SCH.ts Line 19-33
tick(): void {
  // 步骤1: 扫描pending队列，找出可以发射的指令
  const toIssue: QueuedInstruction[] = []

  for (const instruction of this.pendingQueue) {
    if (this.canIssue(instruction)) {
      toIssue.push(instruction)
    }
  }

  // 步骤2: 发射这些指令
  for (const instruction of toIssue) {
    this.issue(instruction)
  }
}
```

**关键判断：canIssue()**

```typescript
// SCH.ts Line 45-57
private canIssue(instruction: QueuedInstruction): boolean {
  // 检查1: 并发数限制
  if (this.activeInstructions.size >= this.maxConcurrency) {
    return false  // 流水线满了
  }

  // 检查2: 资源冲突
  if (this.hasResourceConflict(instruction)) {
    return false  // 资源被占用
  }

  return true  // 可以发射！
}
```

**资源冲突检测：hasResourceConflict()**

```typescript
// SCH.ts Line 89-99
private hasResourceConflict(instruction: QueuedInstruction): boolean {
  // 步骤1: 从ISA中提取资源ID
  const resourceIds = this.getResourceIds(instruction)

  // 步骤2: 检查每个资源是否被占用
  for (const resourceId of resourceIds) {
    if (this.activeResources.has(resourceId)) {
      return true  // 冲突！
    }
  }

  return false  // 无冲突
}

// SCH.ts Line 104-111
private getResourceIds(instruction: QueuedInstruction): string[] {
  const isa = ISA[instruction.type]
  if (!isa) {
    return []
  }

  // 调用ISA定义的resourceIdentifier函数
  return isa.meta.resourceIdentifier(instruction.payload)
}
```

**示例：资源ID提取**

```typescript
// debug-isa.ts
'debug.quick_success': {
  meta: {
    resourceIdentifier: (payload) => [`quick:${payload.id || 'default'}`]
  }
}

// 实际调用
getResourceIds({ type: 'debug.quick_success', payload: { id: 1 } })
  → ['quick:1']
```

**发射指令：issue()**

```typescript
// SCH.ts Line 62-84
private issue(instruction: QueuedInstruction): void {
  // 1️⃣ 从pending队列移除
  const index = this.pendingQueue.indexOf(instruction)
  if (index !== -1) {
    this.pendingQueue.splice(index, 1)
  }

  // 2️⃣ 更新指令状态
  instruction.status = InstructionStatus.ISSUED
  instruction.timestamps.SCH = Date.now()

  // 3️⃣ 记录追踪
  instructionTracker.markPhase(instruction.id, PipelineStage.SCH)

  // 4️⃣ 加入active列表
  this.activeInstructions.set(instruction.id, instruction)

  // 5️⃣ 占用资源（关键！）
  const resourceIds = this.getResourceIds(instruction)
  for (const resourceId of resourceIds) {
    this.activeResources.add(resourceId)
  }
}
```

**做了什么：**

1. 从 `pendingQueue` 移除指令
2. 更新状态为 `issued`，记录SCH时间戳
3. 通知追踪系统
4. 加入 `activeInstructions`（表示正在执行）
5. **占用资源**（添加到 `activeResources`，防止冲突的指令发射）

**数据结构变化：**

```
pendingQueue: ['instr-123'] → []
activeInstructions: {} → { 'instr-123': instruction }
activeResources: [] → ['quick:default']
```

---

### 阶段6: Pipeline.processActiveInstructions() - 处理发射的指令

**文件：** `src/cpu/Pipeline.ts`

**函数：** `processActiveInstructions()`

这个函数在定时tick中被调用（每16ms一次）：

```typescript
// Pipeline.ts Line 80-84
this.tickInterval = window.setInterval(() => {
  this.SCH.tick()
  this.processActiveInstructions() // ← 处理active指令
  this.updateStatus()
}, this.TICK_INTERVAL_MS)
```

```typescript
// Pipeline.ts Line 133-145
private async processActiveInstructions(): Promise<void> {
  const activeInstructions = this.SCH.getActiveInstructions()

  for (const instruction of activeInstructions) {
    // 已经在执行中，跳过
    if (instruction.timestamps.EX) {
      continue
    }

    // 异步执行指令（不阻塞）
    this.executeInstruction(instruction)
  }
}
```

**做了什么：**

- 获取所有active指令
- 过滤掉已经开始执行的指令（通过检查 `timestamps.EX`）
- 对每个新发射的指令调用 `executeInstruction()`
- **注意：这是异步调用，不会阻塞流水线**

---

### 阶段7: Pipeline.executeInstruction() - 执行指令

**文件：** `src/cpu/Pipeline.ts`

**函数：** `executeInstruction(instruction: QueuedInstruction)`

```typescript
// Pipeline.ts Line 150-171
private async executeInstruction(instruction: QueuedInstruction): Promise<void> {
  let error: Error | undefined

  try {
    // 1️⃣ EX阶段: 执行
    await this.EX.execute(instruction)
  } catch (err) {
    error = err as Error
  }

  // 2️⃣ RES阶段: 处理响应
  const { success } = this.RES.processResponse(instruction, error)

  // 3️⃣ WB阶段: 写回
  this.WB.writeBack(instruction, success)

  // 4️⃣ 释放资源
  this.SCH.releaseInstruction(instruction.id)

  // 5️⃣ 更新状态
  this.updateStatus()
}
```

**执行流程：**

1. 调用EX阶段执行指令
2. 捕获可能的异常
3. 调用RES阶段处理响应
4. 调用WB阶段写回结果
5. 释放SCH占用的资源
6. 更新流水线状态

---

### 阶段8: EX.execute() - 执行阶段

**文件：** `src/cpu/stages/EX.ts`

**函数：** `execute(instruction: QueuedInstruction)`

```typescript
// EX.ts Line 13-44
async execute(instruction: QueuedInstruction): Promise<void> {
  // 1️⃣ 获取ISA定义
  const isa = ISA[instruction.type]
  if (!isa) {
    throw new Error(`未找到指令定义: ${instruction.type}`)
  }

  try {
    // 2️⃣ 前置验证（如果有）
    if (isa.validate) {
      const isValid = await isa.validate(instruction.payload, instruction.context)
      if (!isValid) {
        throw new Error(`指令验证失败: ${instruction.type}`)
      }
    }

    // 3️⃣ 标记EX阶段开始
    instruction.status = InstructionStatus.EXECUTING
    instruction.timestamps.EX = Date.now()
    instructionTracker.markPhase(instruction.id, PipelineStage.EX)

    // 4️⃣ 执行指令的核心逻辑
    const result = await isa.execute(instruction.payload, instruction.context)

    // 5️⃣ 保存结果
    instruction.result = result
    instructionTracker.recordNetworkResult(instruction.id, result)
  } catch (error) {
    // 6️⃣ 保存错误
    instruction.error = error as Error
    throw error
  }
}
```

**ISA.execute() 调用示例：**

```typescript
// debug-isa.ts Line 45-52
'debug.quick_success': {
  execute: async (payload, context) => {
    return {
      success: true,
      message: '立即成功',
      data: payload.data || null,
      correlationId: context.correlationId,
      timestamp: Date.now(),
    }
  }
}
```

**做了什么：**

1. 从ISA获取指令定义
2. 执行前置验证（可选）
3. 更新状态和时间戳
4. **调用ISA定义的 `execute()` 方法** - 这是实际业务逻辑
5. 保存执行结果
6. 如果出错，保存错误信息并抛出

**数据变化：**

```
instruction.status: 'issued' → 'executing'
instruction.timestamps.EX: undefined → 1697123456805
instruction.result: undefined → { success: true, ... }
```

---

### 阶段9: RES.processResponse() - 响应处理

**文件：** `src/cpu/stages/RES.ts`

**函数：** `processResponse(instruction, error?)`

```typescript
// RES.ts Line 13-27
processResponse(
  instruction: QueuedInstruction,
  error?: Error
): { success: boolean; shouldRetry: boolean } {
  // 1️⃣ 标记RES阶段
  instruction.status = InstructionStatus.RESPONDED
  instruction.timestamps.RES = Date.now()
  instructionTracker.markPhase(instruction.id, PipelineStage.RES)

  // 2️⃣ 检查是否有错误
  if (error) {
    instruction.error = error
    return { success: false, shouldRetry: false }
  }

  // 3️⃣ 成功
  return { success: true, shouldRetry: false }
}
```

**做了什么：**

1. 更新状态为 `responded`
2. 记录RES时间戳
3. 判断成功或失败
4. 返回处理结果（未来可扩展重试逻辑）

**数据变化：**

```
instruction.status: 'executing' → 'responded'
instruction.timestamps.RES: undefined → 1697123456950
```

---

### 阶段10: WB.writeBack() - 写回阶段

**文件：** `src/cpu/stages/WB.ts`

**函数：** `writeBack(instruction, success)`

```typescript
// WB.ts Line 13-27
writeBack(instruction: QueuedInstruction, success: boolean): void {
  // 1️⃣ 标记WB阶段
  instruction.timestamps.WB = Date.now()
  instructionTracker.markPhase(instruction.id, PipelineStage.WB)

  // 2️⃣ 根据成功/失败更新最终状态
  if (success) {
    instruction.status = InstructionStatus.COMMITTED
    instructionTracker.completeInstruction(instruction.id)
  } else {
    instruction.status = InstructionStatus.FAILED
    instructionTracker.failInstruction(instruction.id, instruction.error || new Error('未知错误'))
  }
}
```

**做了什么：**

1. 记录WB时间戳
2. 成功：标记为 `committed`，调用追踪系统的 `completeInstruction()`
3. 失败：标记为 `failed`，调用追踪系统的 `failInstruction()`

**数据变化：**

```
instruction.status: 'responded' → 'committed'
instruction.timestamps.WB: undefined → 1697123456954
```

**追踪系统输出：**

```typescript
// InstructionTracker.ts Line 66-75
completeInstruction(instructionId: string): void {
  const trace = this.traces.get(instructionId)
  if (!trace) return

  trace.status = InstructionStatus.COMMITTED
  trace.duration = this.calculateDuration(trace.timestamps)

  console.log(
    `%c🎯 指令完成: ${trace.type}`,
    'color: #4CAF50; font-weight: bold',
    this.formatTraceInfo(trace)
  )
}
```

**控制台输出：**

```
🎯 指令完成: debug.quick_success
{
  instructionId: 'instr-1697123456789-1',
  correlationId: 'req-1697123456789-abc',
  duration: '165ms',
  phaseDurations: 'IF→SCH: 2ms | SCH→EX: 3ms | EX→RES: 145ms | RES→WB: 4ms',
  status: 'committed',
  result: { success: true, message: '立即成功', ... }
}
```

---

### 阶段11: SCH.releaseInstruction() - 释放资源

**文件：** `src/cpu/stages/SCH.ts`

**函数：** `releaseInstruction(instructionId)`

```typescript
// SCH.ts Line 116-128
releaseInstruction(instructionId: string): void {
  const instruction = this.activeInstructions.get(instructionId)
  if (!instruction) return

  // 1️⃣ 释放所有占用的资源
  const resourceIds = this.getResourceIds(instruction)
  for (const resourceId of resourceIds) {
    this.activeResources.delete(resourceId)
  }

  // 2️⃣ 从active列表移除
  this.activeInstructions.delete(instructionId)
}
```

**做了什么：**

1. 从 `activeResources` 中移除该指令占用的所有资源ID
2. 从 `activeInstructions` 中移除该指令
3. **关键：此时资源被释放，pending队列中冲突的指令可以在下次tick时发射**

**数据结构变化：**

```
activeInstructions: { 'instr-123': instruction } → {}
activeResources: ['quick:default'] → []
```

---

### 完整调用链总结

```
1. Component.handleAction()
     ↓
2. Pipeline.dispatch(type, payload, source)
     ↓
3. IF.fetchInstruction(type, payload, source)
     ├→ generateCorrelationId()
     ├→ InstructionTracker.startInstruction()
     └→ IF.enqueue(instruction)
     ↓
4. SCH.addInstruction(instruction)
     → pendingQueue.push(instruction)
     ↓
5. SCH.tick()
     ├→ SCH.canIssue(instruction)
     │   ├→ 检查并发数
     │   └→ SCH.hasResourceConflict(instruction)
     │       └→ SCH.getResourceIds(instruction)
     │           └→ ISA[type].meta.resourceIdentifier(payload)
     └→ SCH.issue(instruction)
         ├→ pendingQueue.splice()
         ├→ InstructionTracker.markPhase(id, 'SCH')
         ├→ activeInstructions.set()
         └→ activeResources.add()
     ↓
6. Pipeline.processActiveInstructions()
     └→ Pipeline.executeInstruction(instruction)
         ↓
7. EX.execute(instruction)
     ├→ ISA[type].validate(payload, context)  [可选]
     ├→ InstructionTracker.markPhase(id, 'EX')
     ├→ ISA[type].execute(payload, context)  [核心业务逻辑]
     └→ InstructionTracker.recordNetworkResult()
     ↓
8. RES.processResponse(instruction, error?)
     ├→ InstructionTracker.markPhase(id, 'RES')
     └→ 返回 { success, shouldRetry }
     ↓
9. WB.writeBack(instruction, success)
     ├→ InstructionTracker.markPhase(id, 'WB')
     └→ InstructionTracker.completeInstruction()  [成功]
         或 InstructionTracker.failInstruction()  [失败]
     ↓
10. SCH.releaseInstruction(instructionId)
     ├→ activeResources.delete()
     └→ activeInstructions.delete()
     ↓
11. Pipeline.updateStatus()
     → 更新响应式状态，UI自动更新
```

---

### 文件路径快速参考

| 阶段     | 文件路径                                 | 主要函数                                                     |
| -------- | ---------------------------------------- | ------------------------------------------------------------ |
| **入口** | `src/cpu/Pipeline.ts`                    | `dispatch()`                                                 |
| **IF**   | `src/cpu/stages/IF.ts`                   | `fetchInstruction()`                                         |
| **SCH**  | `src/cpu/stages/SCH.ts`                  | `tick()`, `canIssue()`, `issue()`, `releaseInstruction()`    |
| **EX**   | `src/cpu/stages/EX.ts`                   | `execute()`                                                  |
| **RES**  | `src/cpu/stages/RES.ts`                  | `processResponse()`                                          |
| **WB**   | `src/cpu/stages/WB.ts`                   | `writeBack()`                                                |
| **追踪** | `src/cpu/tracking/InstructionTracker.ts` | `startInstruction()`, `markPhase()`, `completeInstruction()` |
| **ISA**  | `src/cpu/isa/debug-isa.ts`               | 指令定义对象                                                 |
| **类型** | `src/cpu/types.ts`                       | 类型定义                                                     |

---

### 关键时间点

以一个典型的 `debug.quick_success` 指令为例：

| 时间点   | 阶段     | 耗时  | 说明             |
| -------- | -------- | ----- | ---------------- |
| T0       | 组件调用 | -     | 用户触发         |
| T0+0ms   | IF       | 0ms   | 生成ID，创建对象 |
| T0+0ms   | SCH      | 2ms   | 检查冲突，发射   |
| T0+2ms   | EX开始   | -     | 开始执行         |
| T0+5ms   | 执行中   | 145ms | 等待异步操作     |
| T0+150ms | RES      | 0ms   | 收到响应         |
| T0+150ms | WB       | 4ms   | 写回，释放资源   |
| T0+154ms | 完成     | -     | 总耗时154ms      |

**关键洞察：**

- IF + SCH 阶段非常快（< 5ms）
- 大部分时间花在EX阶段的异步操作上
- WB阶段释放资源后，下一条冲突的指令可以立即发射

---

## 快速开始

### 1. 基本使用

```typescript
import { pipeline } from '@/cpu'

// 启动流水线
pipeline.start()

// 发射指令
pipeline.dispatch('debug.quick_success', { data: 'hello' })

// 批量发射
pipeline.dispatch('debug.fetch_baidu', {})
pipeline.dispatch('debug.fetch_with_delay', { delay: 2000 })
pipeline.dispatch('debug.quick_success', { id: 1 })

// 查看状态
console.log(pipeline.getStatus())
// {
//   ifBufferSize: 0,
//   schPendingSize: 1,
//   schActiveSize: 2,
//   totalCompleted: 15,
//   totalFailed: 3
// }
```

### 2. 控制台调试

开发环境下可以直接在浏览器控制台使用：

```javascript
// 查看帮助
cpuPipeline.help()

// 启动流水线
cpuPipeline.start()

// 发射测试指令
cpuPipeline.dispatch('debug.fetch_baidu', {})
cpuPipeline.dispatch('debug.quick_success', { data: 'test' })

// 批量测试冲突检测
cpuPipeline.dispatch('debug.conflicting_resource', { delay: 1000 })
cpuPipeline.dispatch('debug.conflicting_resource', { delay: 1000 })
// ↑ 第二条会等待第一条完成

// 查看追踪记录
cpuPipeline.getTraces()

// 停止流水线
cpuPipeline.stop()

// 重置所有状态
cpuPipeline.reset()
```

### 3. 可视化调试页面

访问应用左边栏的 **"CPU Pipeline"** 入口，打开可视化调试页面。

---

## 详细文档

### 指令集 (ISA)

指令集定义在 `src/cpu/isa/` 目录下。

#### 指令定义结构

```typescript
// src/cpu/isa/types.ts
export interface InstructionDefinition<TPayload, TResult> {
  meta: {
    description: string                              // 指令描述
    category: 'debug' | 'task' | 'schedule' | ...   // 分类
    resourceIdentifier: (payload) => string[]        // 资源ID提取函数
    priority: number                                 // 优先级 (0-10)
    timeout?: number                                 // 超时时间 (ms)
  }

  validate?: (payload, context) => Promise<boolean>  // 前置验证
  execute: (payload, context) => Promise<TResult>    // 执行逻辑
}
```

#### 内置调试指令

| 指令类型                     | 描述              | 用途             |
| ---------------------------- | ----------------- | ---------------- |
| `debug.fetch_baidu`          | 向百度发送GET请求 | 测试真实网络请求 |
| `debug.quick_success`        | 立即成功          | 测试快速执行路径 |
| `debug.fetch_with_delay`     | 带延迟的请求      | 测试流水线并发   |
| `debug.fetch_fail`           | 必定失败的请求    | 测试错误处理     |
| `debug.conflicting_resource` | 固定资源ID        | 测试资源冲突检测 |

#### 示例：定义新指令

```typescript
// src/cpu/isa/debug-isa.ts
export const DebugISA: ISADefinition = {
  'my.custom_instruction': {
    meta: {
      description: '我的自定义指令',
      category: 'debug',

      // 资源标识：从payload中提取资源ID
      resourceIdentifier: (payload) => [`custom:${payload.id}`],

      priority: 5,
      timeout: 5000,
    },

    // 可选：前置验证
    validate: async (payload, context) => {
      if (!payload.id) {
        console.error('缺少必需的id参数')
        return false
      }
      return true
    },

    // 必需：执行逻辑
    execute: async (payload, context) => {
      // 执行实际操作
      const response = await fetch(`/api/something/${payload.id}`)
      const data = await response.json()

      return {
        success: true,
        data,
        correlationId: context.correlationId,
      }
    },
  },
}
```

---

### 流水线阶段

#### IF 阶段 (Instruction Fetch)

**职责：** 接收指令，生成ID，放入缓冲区

**文件：** `src/cpu/stages/IF.ts`

**核心方法：**

```typescript
fetchInstruction(type: string, payload: any): QueuedInstruction {
  const instructionId = `instr-${Date.now()}-${++this.idCounter}`
  const correlationId = generateCorrelationId()

  const instruction = {
    id: instructionId,
    type,
    payload,
    context: { instructionId, correlationId, timestamp, ... },
    status: 'pending',
    timestamps: { IF: Date.now() }
  }

  this.buffer.push(instruction)
  return instruction
}
```

**关键点：**

- 每条指令获得唯一的 `instructionId`
- 生成 `correlationId` 用于SSE事件去重
- 指令初始状态为 `pending`

---

#### SCH 阶段 (Scheduler) ⭐️

**职责：** 调度指令，检测冲突，发射可执行的指令

**文件：** `src/cpu/stages/SCH.ts`

这是流水线的核心组件，实现了类似现代CPU的乱序执行。

**核心数据结构：**

```typescript
private pendingQueue: QueuedInstruction[] = []              // 待发射指令
private activeInstructions: Map<string, QueuedInstruction>  // 执行中指令
private activeResources: Set<string> = new Set()            // 占用的资源
private maxConcurrency = 10                                 // 并发上限
```

**调度循环（每16ms一次）：**

```typescript
tick(): void {
  const toIssue: QueuedInstruction[] = []

  // 扫描pending队列，找出可以发射的指令
  for (const instruction of this.pendingQueue) {
    if (this.canIssue(instruction)) {
      toIssue.push(instruction)
    }
  }

  // 发射指令
  for (const instruction of toIssue) {
    this.issue(instruction)
  }
}
```

**发射条件判断：**

```typescript
canIssue(instruction): boolean {
  // 条件1：检查并发数限制
  if (this.activeInstructions.size >= this.maxConcurrency) {
    return false
  }

  // 条件2：检查资源冲突
  if (this.hasResourceConflict(instruction)) {
    return false
  }

  return true
}

hasResourceConflict(instruction): boolean {
  const resourceIds = this.getResourceIds(instruction)

  for (const resourceId of resourceIds) {
    if (this.activeResources.has(resourceId)) {
      return true  // 资源被占用
    }
  }

  return false
}
```

**发射操作：**

```typescript
issue(instruction): void {
  // 1. 从pending队列移除
  this.pendingQueue.splice(index, 1)

  // 2. 更新状态
  instruction.status = 'issued'
  instruction.timestamps.SCH = Date.now()

  // 3. 加入active列表
  this.activeInstructions.set(instruction.id, instruction)

  // 4. 占用资源（关键！）
  const resourceIds = this.getResourceIds(instruction)
  for (const resourceId of resourceIds) {
    this.activeResources.add(resourceId)
  }
}
```

**资源释放：**

```typescript
releaseInstruction(instructionId): void {
  const instruction = this.activeInstructions.get(instructionId)

  // 释放所有占用的资源
  const resourceIds = this.getResourceIds(instruction)
  for (const resourceId of resourceIds) {
    this.activeResources.delete(resourceId)
  }

  // 从active列表移除
  this.activeInstructions.delete(instructionId)
}
```

**调度器行为示例：**

```javascript
// 场景1：不同资源，并行执行
pipeline.dispatch('debug.quick_success', { id: 1 }) // resourceId: ['quick:1']
pipeline.dispatch('debug.quick_success', { id: 2 }) // resourceId: ['quick:2']
// ✅ 立即并行执行

// 场景2：相同资源，顺序执行
pipeline.dispatch('debug.conflicting_resource', { delay: 2000 }) // resourceId: ['resource:shared']
pipeline.dispatch('debug.conflicting_resource', { delay: 1000 }) // resourceId: ['resource:shared']
// ❌ 第二条必须等待第一条完成

// 场景3：混合场景
pipeline.dispatch('debug.quick_success', { id: 1 }) // 并行
pipeline.dispatch('debug.conflicting_resource', { delay: 1000 }) // 并行
pipeline.dispatch('debug.conflicting_resource', { delay: 500 }) // 等待上一条
pipeline.dispatch('debug.quick_success', { id: 2 }) // 并行
```

**⚠️ 重要修复：防止批量发射冲突指令**

早期版本存在严重的竞态条件bug，已修复：

**问题：**

```typescript
// 旧实现（有BUG）
tick(): void {
  const toIssue = []

  // 步骤1：批量检查
  for (const instruction of this.pendingQueue) {
    if (this.canIssue(instruction)) {  // ← 所有指令都在资源空闲时检查
      toIssue.push(instruction)
    }
  }

  // 步骤2：批量发射
  for (const instruction of toIssue) {
    this.issue(instruction)  // ← 这里才占用资源，太晚了！
  }
}

// 场景：3个冲突指令在队列
// 1. 检查A：资源空闲 ✅
// 2. 检查B：资源空闲 ✅ (还没发射A)
// 3. 检查C：资源空闲 ✅ (还没发射任何)
// 4. 发射A、B、C → 全部同时执行 ❌ 冲突！
```

**修复：**

```typescript
// 新实现（边检查边发射）
tick(): void {
  let issued = true
  while (issued) {
    issued = false

    for (const instruction of this.pendingQueue) {
      if (this.canIssue(instruction)) {
        this.issue(instruction)  // ← 立即发射，立即占用资源
        issued = true
        break  // ← 重新检查队列（资源状态已更新）
      }
    }
  }
}

// 场景：3个冲突指令在队列
// 1. 检查A：资源空闲 ✅ → 发射A → 资源被占用
// 2. 检查B：资源被占用 ❌ → 跳过
// 3. 检查C：资源被占用 ❌ → 跳过
// A完成后，下次tick才会发射B ✅ 正确！
```

---

#### EX 阶段 (Execute)

**职责：** 执行指令的实际操作

**文件：** `src/cpu/stages/EX.ts`

```typescript
async execute(instruction: QueuedInstruction): Promise<void> {
  const isa = ISA[instruction.type]

  // 步骤1: 前置验证
  if (isa.validate) {
    const isValid = await isa.validate(instruction.payload, instruction.context)
    if (!isValid) {
      throw new Error(`指令验证失败`)
    }
  }

  // 步骤2: 标记执行开始
  instruction.status = 'executing'
  instruction.timestamps.EX = Date.now()

  // 步骤3: 执行操作
  const result = await isa.execute(instruction.payload, instruction.context)

  // 步骤4: 保存结果
  instruction.result = result
}
```

**关键点：**

- 异步执行，不阻塞流水线
- 支持前置验证
- 自动捕获异常

---

#### RES 阶段 (Response)

**职责：** 处理执行结果

**文件：** `src/cpu/stages/RES.ts`

```typescript
processResponse(instruction: QueuedInstruction, error?: Error): { success: boolean } {
  instruction.status = 'responded'
  instruction.timestamps.RES = Date.now()

  if (error) {
    instruction.error = error
    return { success: false }
  }

  return { success: true }
}
```

**未来扩展：**

- 重试策略（指数退避）
- 错误分类处理
- 回滚机制

---

#### WB 阶段 (Write Back)

**职责：** 完成指令，释放资源

**文件：** `src/cpu/stages/WB.ts`

```typescript
writeBack(instruction: QueuedInstruction, success: boolean): void {
  instruction.timestamps.WB = Date.now()

  if (success) {
    instruction.status = 'committed'
    instructionTracker.completeInstruction(instruction.id)
  } else {
    instruction.status = 'failed'
    instructionTracker.failInstruction(instruction.id, instruction.error)
  }
}
```

**关键点：**

- 标记最终状态（committed / failed）
- 触发追踪系统记录
- 在Pipeline中会调用 `SCH.releaseInstruction()` 释放资源

---

### 指令追踪系统

**文件：** `src/cpu/tracking/InstructionTracker.ts`

追踪系统记录每条指令的完整生命周期。

#### 追踪记录结构

```typescript
interface InstructionTrace {
  instructionId: string
  type: string
  payload: any
  correlationId: string

  timestamps: {
    IF: number // 指令获取时间
    SCH?: number // 调度时间
    EX?: number // 执行开始时间
    RES?: number // 响应到达时间
    WB?: number // 写回时间
  }

  networkResult?: any
  status: InstructionStatus
  duration?: number // 总耗时
  error?: Error
}
```

#### 主要方法

```typescript
// 开始追踪
tracker.startInstruction(instructionId, type, payload, correlationId)

// 标记阶段
tracker.markPhase(instructionId, PipelineStage.SCH)
tracker.markPhase(instructionId, PipelineStage.EX)
// ...

// 记录结果
tracker.recordNetworkResult(instructionId, result)

// 完成追踪
tracker.completeInstruction(instructionId)

// 失败追踪
tracker.failInstruction(instructionId, error)

// 获取所有记录
const traces = tracker.getAllTraces()
```

#### 控制台输出

成功的指令：

```
🎯 指令完成: debug.quick_success
{
  instructionId: 'instr-1697123456789-abc',
  correlationId: 'req-1697123456789-def',
  duration: '156ms',
  phaseDurations: 'IF→SCH: 2ms | SCH→EX: 5ms | EX→RES: 145ms | RES→WB: 4ms',
  status: 'committed',
  result: { success: true, ... }
}
```

失败的指令：

```
❌ 指令失败: debug.fetch_fail
{
  instructionId: 'instr-1697123456790-xyz',
  correlationId: 'req-1697123456790-ghi',
  duration: '523ms',
  phaseDurations: 'IF→SCH: 1ms | SCH→EX: 3ms | EX→RES: 515ms | RES→WB: 4ms',
  status: 'failed',
}
Error: 模拟的网络请求失败
```

---

## 使用指南

### 在Vue组件中使用

```vue
<script setup lang="ts">
import { pipeline } from '@/cpu'
import { onMounted } from 'vue'

onMounted(() => {
  // 确保流水线已启动
  pipeline.start()
})

function handleQuickAction() {
  pipeline.dispatch('debug.quick_success', {
    data: 'user action',
    timestamp: Date.now(),
  })
}

function handleBatchActions() {
  // 批量发射，自动并行执行
  for (let i = 0; i < 10; i++) {
    pipeline.dispatch('debug.quick_success', { id: i })
  }
}
</script>
```

### 监听流水线状态

```typescript
import { pipeline } from '@/cpu'
import { watch } from 'vue'

// pipeline.status 是响应式的
watch(
  () => pipeline.status.value,
  (status) => {
    console.log('流水线状态:', status)
    // {
    //   ifBufferSize: 0,
    //   schPendingSize: 2,
    //   schActiveSize: 8,
    //   totalCompleted: 150,
    //   totalFailed: 5
    // }
  },
  { deep: true }
)
```

### 获取追踪记录

```typescript
import { instructionTracker } from '@/cpu'

// 获取所有追踪记录
const allTraces = instructionTracker.getAllTraces()

// 过滤成功的指令
const successful = allTraces.filter((t) => t.status === 'committed')

// 过滤失败的指令
const failed = allTraces.filter((t) => t.status === 'failed')

// 过滤执行中的指令
const running = allTraces.filter((t) =>
  ['pending', 'issued', 'executing', 'responded'].includes(t.status)
)

// 计算平均耗时
const avgDuration = successful.reduce((sum, t) => sum + (t.duration || 0), 0) / successful.length
```

---

## 可视化调试

访问应用左边栏的 **"CPU Pipeline"** 入口，打开可视化调试页面。

### 页面功能

#### 1. 控制面板

- **启动/停止/重置**流水线
- 实时显示流水线运行状态

#### 2. 流水线状态可视化

五个阶段的实时状态卡片：

- **IF**: 缓冲区大小
- **SCH**: Pending队列大小、Active指令数量
- **EX**: 执行中指令数量
- **RES**: 响应中指令数量
- **WB**: 已完成/失败指令数量

#### 3. 快速测试按钮

- **请求百度**: 测试真实网络请求
- **立即成功**: 测试快速执行
- **延迟2秒**: 测试流水线并发
- **必定失败**: 测试错误处理
- **资源冲突**: 测试冲突检测
- **批量测试**: 连续发射10个指令

#### 4. 指令追踪表格

实时显示所有指令的执行情况：

- 指令ID、类型、状态
- 各阶段耗时（IF→SCH、SCH→EX、EX→RES、RES→WB）
- 总耗时
- 执行结果或错误信息

**过滤器：**

- 全部
- 成功
- 失败
- 执行中

---

## 扩展开发

### 添加新的指令集

创建新的ISA文件：

```typescript
// src/cpu/isa/task-isa.ts
import type { ISADefinition } from './types'
import type { TaskCard } from '@/types/dtos'
import { apiPost } from '@/stores/shared'
import { useTaskStore } from '@/stores/task'

export const TaskISA: ISADefinition = {
  'task.complete': {
    meta: {
      description: '完成任务',
      category: 'task',
      resourceIdentifier: (payload) => [`task:${payload.id}`],
      priority: 7,
      timeout: 10000,
    },

    validate: async (payload) => {
      const taskStore = useTaskStore()
      const task = taskStore.getTaskById_Mux(payload.id)

      if (!task) {
        console.error('任务不存在:', payload.id)
        return false
      }

      if (task.is_completed) {
        console.warn('任务已完成:', payload.id)
        return false
      }

      return true
    },

    execute: async (payload, context) => {
      const result = await apiPost(
        `/tasks/${payload.id}/completion`,
        {},
        {
          headers: { 'X-Correlation-ID': context.correlationId },
        }
      )

      return result
    },
  },

  'task.create': {
    meta: {
      description: '创建任务',
      category: 'task',
      resourceIdentifier: () => [], // 创建操作没有固定资源
      priority: 5,
    },

    execute: async (payload, context) => {
      const task: TaskCard = await apiPost('/tasks', payload, {
        headers: { 'X-Correlation-ID': context.correlationId },
      })

      const taskStore = useTaskStore()
      taskStore.addOrUpdateTask_mut(task)

      return task
    },
  },
}
```

在 `src/cpu/isa/index.ts` 中导入：

```typescript
import { DebugISA } from './debug-isa'
import { TaskISA } from './task-isa'

export const ISA: ISADefinition = {
  ...DebugISA,
  ...TaskISA,
}
```

### 扩展流水线功能

#### 1. 添加乐观更新支持

```typescript
// 在ISA类型定义中添加
interface InstructionDefinition<TPayload, TResult> {
  meta: { /* ... */ }

  // 新增
  optimistic?: (payload, context) => Promise<OptimisticSnapshot>
  rollback?: (payload, snapshot, context, error) => Promise<void>

  validate?: /* ... */
  execute: /* ... */
}
```

#### 2. 添加重试策略

在 `RES.ts` 中实现：

```typescript
processResponse(instruction, error): { success: boolean; shouldRetry: boolean } {
  if (error) {
    const isa = ISA[instruction.type]
    const retryPolicy = isa.meta.retryPolicy

    if (retryPolicy.enabled && instruction.context.retryCount < retryPolicy.maxRetries) {
      return { success: false, shouldRetry: true }
    }
  }

  return { success: !error, shouldRetry: false }
}
```

#### 3. 添加优先级调度

在 `SCH.ts` 的 `tick()` 方法中：

```typescript
tick(): void {
  // 按优先级排序
  const sorted = this.pendingQueue.sort((a, b) => {
    const priorityA = ISA[a.type]?.meta.priority || 0
    const priorityB = ISA[b.type]?.meta.priority || 0
    return priorityB - priorityA
  })

  // 发射高优先级指令
  for (const instruction of sorted) {
    if (this.canIssue(instruction)) {
      this.issue(instruction)
    }
  }
}
```

---

## 指令迁移

### 从 CommandBus 迁移到 CPU Pipeline

如果你有现有的 CommandBus 指令需要迁移到 CPU Pipeline 系统，请查看完整的迁移指南：

**📘 [指令迁移指南](./MIGRATION_GUIDE.md)**

### 快速迁移示例

**现有 CommandBus Handler:**

```typescript
const handleCompleteTask = async (payload) => {
  const correlationId = generateCorrelationId()
  const result = await apiPost(
    `/tasks/${payload.id}/completion`,
    {},
    {
      headers: { 'X-Correlation-ID': correlationId },
    }
  )
  await transactionProcessor.applyTaskTransaction(result, {
    correlation_id: correlationId,
    source: 'http',
  })
}
```

**迁移到 CPU ISA:**

```typescript
// src/cpu/isa/task-isa.ts
export const TaskISA: ISADefinition = {
  'task.complete': {
    meta: {
      description: '完成任务',
      category: 'task',
      resourceIdentifier: (payload) => [`task:${payload.id}`],
      priority: 7,
      timeout: 10000,
    },

    validate: async (payload) => {
      const task = useTaskStore().getTaskById_Mux(payload.id)
      return task && !task.is_completed
    },

    execute: async (payload, context) => {
      return await apiPost(
        `/tasks/${payload.id}/completion`,
        {},
        {
          headers: { 'X-Correlation-ID': context.correlationId },
        }
      )
    },

    commit: async (result, payload, context) => {
      await transactionProcessor.applyTaskTransaction(result, {
        correlation_id: context.correlationId,
        source: 'http',
      })
    },
  },
}
```

**更新组件调用:**

```typescript
// 旧代码
import { commandBus } from '@/commandBus'
await commandBus.emit('task.complete', { id: taskId })

// 新代码
import { pipeline } from '@/cpu'
pipeline.dispatch('task.complete', { id: taskId })
```

### 迁移优势

✅ **并发执行** - 不同任务的操作并行执行，3倍性能提升  
✅ **资源冲突检测** - 自动管理同一资源的操作顺序  
✅ **完整追踪** - 每个指令的执行过程可视化  
✅ **前置验证** - 避免无效的网络请求  
✅ **类型安全** - 完整的 TypeScript 支持

### 迁移计划

1. ✅ **阶段0**: 调试指令集（已完成）
2. 📋 **阶段1**: 迁移核心任务指令（task.complete, task.create, task.update）
3. 📋 **阶段2**: 迁移日程和时间块指令
4. 📋 **阶段3**: 迁移模板和循环规则指令
5. 📋 **阶段4**: 逐步更新组件调用
6. 📋 **阶段5**: 移除旧 CommandBus（可选）

**详细步骤请参考：[MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md)**

---

## 技术细节

### 并发控制

**最大并发数：** 10（可在 `SCH.ts` 中调整）

```typescript
private maxConcurrency = 10
```

**调度频率：** 16ms一次 tick（约60fps）

```typescript
private readonly TICK_INTERVAL_MS = 16
```

### 性能特性

**并行执行示例：**

```typescript
// 顺序执行耗时
task1: 1000ms
task2: 1000ms
task3: 1000ms
总计: 3000ms

// 并行执行耗时（无资源冲突）
task1: 1000ms \
task2: 1000ms  } 并行
task3: 1000ms /
总计: ~1000ms  (3倍加速！)
```

**调度延迟：**

⚠️ **已优化**：早期版本存在tick延迟问题，现已修复。

**修复前的问题：**

```typescript
// 旧实现：dispatch()只调用SCH.tick()，不执行指令
dispatch(type, payload) {
  this.IF.fetchInstruction(type, payload)
  this.SCH.addInstruction(instruction)
  this.SCH.tick()  // 只发射，不执行
  // ❌ processActiveInstructions()只在定时tick中调用
}

// 结果：指令需要等待下一次tick才开始执行
// 延迟：平均8ms，最坏16ms
```

**修复后：**

```typescript
// 新实现：dispatch()立即执行
dispatch(type, payload) {
  this.IF.fetchInstruction(type, payload)
  this.SCH.addInstruction(instruction)
  this.SCH.tick()  // 发射指令
  this.processActiveInstructions()  // ✅ 立即执行
}

// 结果：指令立即开始执行
// 延迟：< 1ms
```

**关键洞察：**

- 基于tick的系统本质上会引入延迟（最多一个tick周期）
- 通过在 `dispatch()` 中立即调用 `processActiveInstructions()`，消除了这个延迟
- 定时tick仍然保留，用于处理异步完成的指令和pending队列中的指令

### 资源冲突策略

**检测机制：** 基于资源ID的精确匹配

```typescript
// 相同资源ID → 冲突
'task:task-123' === 'task:task-123' // true

// 不同资源ID → 无冲突
'task:task-123' === 'task:task-456' // false
```

**处理策略：**

1. 第一条指令立即发射
2. 第二条指令进入pending队列
3. 第一条完成后，释放资源
4. 下一次tick时发射第二条

### 类型安全

完整的TypeScript支持：

```typescript
// 指令类型自动推导
pipeline.dispatch('debug.quick_success', {
  data: 'test', // ✅ 类型正确
})

pipeline.dispatch('debug.quick_success', {
  wrongField: 'test', // ❌ 类型错误（如果定义了payload类型）
})

// ISA定义类型安全
const isa: InstructionDefinition<MyPayload, MyResult> = {
  meta: {
    /* ... */
  },
  execute: async (payload: MyPayload): Promise<MyResult> => {
    // payload 和 返回值都有类型检查
  },
}
```

---

## 常见问题

### Q: 为什么指令没有立即执行？

**A:** 可能的原因：

1. **流水线未启动** - 确保调用了 `pipeline.start()`
2. **并发数达到上限** - 最多同时执行10条指令
3. **资源冲突** - 同一资源的指令必须顺序执行
4. **调度延迟** - 最多延迟16ms（一个tick周期）

### Q: 如何调试指令执行？

**A:** 三种方式：

1. **控制台日志** - 每条指令完成/失败都会输出日志
2. **可视化页面** - 实时查看流水线状态和追踪记录
3. **追踪API** - `instructionTracker.getAllTraces()` 获取详细信息

### Q: 重置后流水线卡住了？

**A:** 这是已知问题，已修复。确保使用最新代码：

```typescript
function handleReset() {
  pipeline.reset()
  isRunning.value = false // 重要！同步状态
}
```

### Q: 重置后还能发射指令？

**A:** 这是另一个已知问题，已修复。

**问题：** `dispatch()` 没有检查流水线运行状态

```typescript
// 有BUG的代码
dispatch(type, payload) {
  // ❌ 没有检查 isRunning
  this.IF.fetchInstruction(type, payload)
  this.SCH.tick()
  this.processActiveInstructions()  // ← 即使停止也会执行
}

// 场景：
// 1. 用户点击"重置" → isRunning = false
// 2. 用户点击"立即成功" → dispatch()执行
// 3. 指令立即完成 ✅ (不应该！)
```

**修复：** 添加运行状态检查

```typescript
// 修复后的代码
dispatch(type, payload) {
  // ✅ 检查流水线是否运行
  if (!this.isRunning) {
    console.warn('流水线未启动，指令被拒绝')
    return
  }

  // 正常执行...
}

// 修复后：
// 1. 用户点击"重置" → isRunning = false
// 2. 用户点击"立即成功" → dispatch()返回
// 3. 控制台警告：⚠️ 流水线未启动，指令被拒绝
```

**为什么瞬间执行的指令都通过了？**

因为 `dispatch()` 中有 `processActiveInstructions()`，它会立即执行指令：

- `debug.quick_success` 是同步的，立即完成
- 即使定时tick停止了，`dispatch()` 仍然直接执行指令
- 所以看起来"通过了"，但这是bug，不应该允许

### Q: 如何限制某类指令的并发数？

**A:** 可以通过资源ID实现：

```typescript
'my.limited_instruction': {
  meta: {
    resourceIdentifier: () => ['limited:shared'],  // 所有实例共享同一资源
  }
}
```

这样该指令只能顺序执行。

### Q: 支持取消正在执行的指令吗？

**A:** 当前版本不支持，但可以扩展：

```typescript
// 未来功能
pipeline.cancel(instructionId)
```

### Q: 如何实现指令的依赖关系？

**A:** 通过资源ID或手动控制：

```typescript
// 方法1: 资源依赖
'operation.step1': { resourceIdentifier: () => ['workflow:1'] }
'operation.step2': { resourceIdentifier: () => ['workflow:1'] }

// 方法2: 手动控制
await pipeline.dispatch('operation.step1', {})
// 等待完成后
await pipeline.dispatch('operation.step2', {})
```

### Q: 如何监控流水线性能？

**A:** 使用追踪数据：

```typescript
const traces = instructionTracker.getAllTraces()

// 平均耗时
const avgDuration = traces.reduce((sum, t) => sum + (t.duration || 0), 0) / traces.length

// 成功率
const successRate = traces.filter((t) => t.status === 'committed').length / traces.length

// 各阶段耗时分布
const exDurations = traces.map((t) => (t.timestamps.RES || 0) - (t.timestamps.EX || 0))
```

### Q: 为什么要保留tick系统？直接执行不就好了吗？

**A:** tick系统虽然引入了复杂性，但提供了关键优势：

**优势1：批量处理**

```typescript
// 用户快速连续操作
dispatch('task.complete', { id: 1 })
dispatch('task.complete', { id: 2 })
dispatch('task.complete', { id: 3 })

// tick可以批量检查冲突和调度
tick() {
  // 一次性处理所有pending指令
  // 可以进行全局优化（如按优先级排序）
}
```

**优势2：解耦调度和执行**

```typescript
// 指令可以异步执行，不阻塞调度
EX阶段: 执行网络请求（异步）
  ↓
SCH阶段: 继续调度其他指令（不等待）
```

**优势3：更好的可观测性**

- 明确的调度时间点（每16ms一次）
- 可以暂停/恢复流水线
- 可以统计调度器的工作负载

**当前最佳实践：**

- `dispatch()` 立即执行（0延迟）
- tick仍然保留，处理pending队列和清理工作
- 两者结合，兼顾响应性和系统稳定性

### Q: 为什么多个冲突指令会被同时发射？

**A:** 这是早期版本的一个**严重竞态条件bug**，已修复。

**问题原因：** 批量检查和批量发射分离导致

```typescript
// 有BUG的代码
tick() {
  const toIssue = []

  // 第1步：批量检查（此时资源空闲）
  for (const instr of pending) {
    if (canIssue(instr)) {
      toIssue.push(instr)  // ← 3个冲突指令都通过检查
    }
  }

  // 第2步：批量发射（此时才占用资源）
  for (const instr of toIssue) {
    issue(instr)  // ← 全部同时发射，冲突！
  }
}

// 实际场景：
// 指令A完成，释放 'resource:shared'
// tick检查指令B: 资源空闲 ✅
// tick检查指令C: 资源空闲 ✅ (还没发射B，资源还是空的)
// 发射B和C → 冲突！❌
```

**修复方案：** 边检查边发射

```typescript
// 修复后的代码
tick() {
  let issued = true
  while (issued) {
    issued = false
    for (const instr of pending) {
      if (canIssue(instr)) {
        issue(instr)  // ← 立即发射，立即占用资源
        issued = true
        break  // ← 重新检查队列（资源状态已更新）
      }
    }
  }
}

// 修复后的场景：
// 指令A完成，释放 'resource:shared'
// tick检查指令B: 资源空闲 ✅ → 发射B → 占用资源
// tick检查指令C: 资源被占用 ❌ → 跳过
// B完成后，下次tick才会发射C ✅ 正确！
```

**关键点：**

- 每发射一个指令，资源状态立即更新
- 下一个指令检查时会看到最新状态
- 使用while循环重复检查，直到无法发射为止
- 避免了批量处理导致的竞态条件

---

## 未来路线图

当前版本是简化的调试版本，计划扩展：

- [ ] **乐观更新支持** - optimistic update + rollback
- [ ] **重试机制** - 指数退避重试策略
- [ ] **指令取消** - 取消正在执行的指令
- [ ] **优先级调度** - 按优先级顺序发射指令
- [ ] **依赖关系检测** - 自动检测指令间的复杂依赖
- [ ] **Store操作追踪** - 记录每个指令的状态变更
- [ ] **性能监控面板** - 更详细的性能分析工具
- [ ] **时序图可视化** - 图形化展示流水线执行
- [ ] **完整任务指令集** - task/schedule/timeblock指令
- [ ] **测试工具** - 单元测试和集成测试支持

---

## 参考资料

**相关文档：**

- [ISA类型定义](./isa/types.ts)
- [调度器实现](./stages/SCH.ts)
- [追踪系统](./tracking/InstructionTracker.ts)
- [主流水线](./Pipeline.ts)

**设计灵感：**

- 现代CPU超标量架构
- Tomasulo算法（动态调度）
- 乱序执行（Out-of-Order Execution）
- 寄存器重命名（Register Renaming）

---

## 贡献

欢迎贡献代码和想法！这是一个实验性项目，目标是探索前端架构的新可能性。

**改进建议：**

- 性能优化
- 新的调度策略
- 更多指令集
- 调试工具改进
- 文档完善

---

**Made with 🚀 by Frontend-as-a-CPU Architecture**
