# Cutie Frontend-as-a-CPU 架构说明文档

## 🎯 架构理念

Cutie 项目采用了独创的 **"Frontend-as-a-CPU"** 架构设计理念，将整个前端应用类比为一个现代 CPU 处理器，通过模拟 CPU 的各个组件和工作流程来构建高效、可维护的前端架构。

### 核心思想

- **前端 = CPU处理器**：整个前端应用就是一个信息处理器
- **组件间协作 = CPU流水线**：各层协同工作，形成高效的数据处理流水线
- **状态管理 = 寄存器操作**：精确控制数据的读写和传输
- **事件处理 = 中断机制**：响应外部事件和异步操作

---

## 🏗️ 整体架构图

```
┌─────────────────────────────────────────────────────────────┐
│                    Vue Components                           │
│                 (Input/Output Devices)                     │
│                用户界面和交互设备层                            │
└─────────────────────┬───────────────────────────────────────┘
                      │ 用户操作
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                  Command Bus                                │
│               (Instruction Decoder)                         │
│                   指令译码器                                  │
│        commandBus.emit('task.complete', payload)           │
└─────────────────────┬───────────────────────────────────────┘
                      │ 译码后的指令
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                Command Handlers                             │
│                (Execution Units)                            │
│                   执行单元                                    │
│              业务逻辑处理器                                    │
└─────────────────────┬───────────────────────────────────────┘
                      │ API调用 + 事务ID
                      ▼
┌─────────────────────────────────────────────────────────────┐
│              Transaction Processor                          │
│             (Reorder Buffer + Commit Unit)                  │
│               重排序缓冲 + 提交单元                             │
│            统一处理HTTP响应和SSE事件                           │
└─────────────────────┬───────────────────────────────────────┘
                      │ 处理后的数据
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                  Pinia Stores                               │
│                (Register File)                              │
│                   寄存器堆                                    │
│               单一数据源状态管理                               │
└─────────────────────┬───────────────────────────────────────┘
                      │ 响应式更新
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                Vue Components                               │
│                  (Display)                                  │
│                   显示设备                                    │
│                自动响应状态变化                                │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔧 核心组件详解

### 1. 基础设施层 (Infrastructure Layer)

位于 `src/infra/`，提供类似 CPU 硬件的基础能力：

#### 🚌 Command Bus (指令总线)
- **作用**：统一接收和分发用户操作指令
- **CPU类比**：指令总线 (Instruction Bus)
- **位置**：`src/commandBus/`

```typescript
// 使用示例
import { commandBus } from '@/commandBus'

// 完成任务
await commandBus.emit('task.complete', { id: 'task-123' })

// 创建任务
await commandBus.emit('task.create', {
  title: '新任务',
  area_id: 'area-456'
})
```

#### 🔄 Transaction Processor (事务处理器)
- **作用**：统一处理 HTTP 响应和 SSE 事件，防重复，自动应用副作用
- **CPU类比**：重排序缓冲 (Reorder Buffer) + 提交单元 (Commit Unit)
- **位置**：`src/infra/transaction/`

```typescript
// 自动处理事务结果
await transactionProcessor.applyTaskTransaction(result, {
  correlation_id: correlationId,
  source: 'http'
})
```

#### 🔗 Correlation ID (关联追踪)
- **作用**：为每个操作生成唯一标识，追踪请求生命周期
- **CPU类比**：事务ID生成器 (Transaction ID Generator)
- **位置**：`src/infra/correlation/`

#### 📡 Events (事件系统)
- **作用**：处理服务器推送事件，实现实时同步
- **CPU类比**：中断控制器 (Interrupt Controller)
- **位置**：`src/infra/events/`

#### 📊 Logging (日志系统)
- **作用**：提供结构化日志和指令追踪
- **CPU类比**：调试跟踪单元 (Debug Trace Unit)
- **位置**：`src/infra/logging/`

```typescript
// 自动指令追踪
logger.info('System:Command', 'Task completed', {
  taskId: 'task-123',
  correlation: correlationId
})
```

### 2. 命令处理层 (Command Handlers)

位于 `src/commandBus/handlers/`，类似 CPU 的执行单元：

#### 特点
- **职责分离**：只负责业务逻辑编排，不直接操作状态
- **事务管理**：自动生成关联ID，处理事务结果
- **错误处理**：统一的错误捕获和上报

```typescript
// 任务完成处理器示例
const handleCompleteTask: CommandHandlerMap['task.complete'] = async (payload) => {
  // 1. 生成关联ID
  const correlationId = generateCorrelationId()

  // 2. 调用API
  const result = await apiPost(`/tasks/${payload.id}/completion`, {}, {
    headers: { 'X-Correlation-ID': correlationId }
  })

  // 3. 使用事务处理器统一处理结果
  await transactionProcessor.applyTaskTransaction(result, {
    correlation_id: correlationId,
    source: 'http'
  })
}
```

### 3. 状态管理层 (Store Layer)

采用 Pinia，类比为 CPU 的寄存器堆：

#### 🗄️ 核心设计原则

```typescript
/**
 * Task Store V4.0 - 纯状态容器版本
 *
 * RTL 架构原则：
 * - State: 寄存器 (registers)，只存储数据
 * - Mutations: 寄存器写入操作 (_mut 后缀)
 * - Getters: 导线 (wires) 和多路复用器 (_Mux 后缀)
 * - ❌ 不包含 API 调用（由 Command Handler 负责）
 * - ❌ 不包含业务逻辑（由 Command Handler 负责）
 */
```

#### 模块化组织

```
src/stores/task/
├── index.ts           # Store 组合器
├── core.ts           # 核心状态 + 计算属性
├── mutations.ts      # 纯数据操作 (_mut 后缀)
├── loaders.ts        # DMA数据加载 (_DMA 后缀)
└── event-handlers.ts # SSE事件处理
```

#### 使用示例

```typescript
const taskStore = useTaskStore()

// ✅ 读取状态 (寄存器读取)
const allTasks = taskStore.allTasks
const stagingTasks = taskStore.stagingTasks

// ✅ 选择器 (多路复用器)
const task = taskStore.getTaskById_Mux('task-123')
const dailyTasks = taskStore.getTasksByDate_Mux('2025-10-15')

// ✅ 状态更新 (寄存器写入)
taskStore.addOrUpdateTask_mut(newTask)

// ❌ 错误用法 - Store不应包含API调用
// taskStore.createTask() // 应该使用 commandBus.emit()
```

### 4. 可组合函数层 (Composables Layer)

位于 `src/composables/`，提供复用的逻辑模块：

#### 🎯 设计特点
- **职责单一**：每个 composable 只解决一个特定问题
- **可组合性**：可以灵活组合使用
- **类型安全**：完整的 TypeScript 支持

#### 主要模块

**拖拽系统** (`src/composables/drag/`)
```typescript
// 跨看板拖拽
const crossViewDrag = useCrossViewDrag()
crossViewDrag.startNormalDrag(task, sourceView)

// 自动滚动
const { handleAutoScroll } = useAutoScroll()
```

**视图操作**
```typescript
// 视图任务查询
const { getViewTasks } = useViewTasks()
const tasks = await getViewTasks(viewContext)

// 循环规则操作
const { createRecurrence } = useRecurrenceOperations()
```

---

## 🌊 数据流与执行流程

### 完整的指令流水线

以"完成任务"操作为例：

```
[IF] Instruction Fetch     │ 用户点击"完成"按钮
        ↓
[ID] Instruction Decode    │ commandBus.emit('task.complete', {id})
        ↓
[EX] Execute              │ Command Handler 调用 API
        ↓
[MEM] Memory Access       │ Transaction Processor 处理响应
        ↓
[WB] Write Back           │ Store 更新状态，组件自动刷新
```

### 1. 指令获取阶段 (IF)
```typescript
// 组件中的用户交互
function handleCompleteTask() {
  // 用户操作触发指令
  commandBus.emit('task.complete', { id: task.id })
}
```

### 2. 指令译码阶段 (ID)
```typescript
// CommandBus 自动分发到对应处理器
commandBus.on('task.complete', handleCompleteTask)
```

### 3. 执行阶段 (EX)
```typescript
// Command Handler 执行业务逻辑
const handleCompleteTask = async (payload) => {
  const correlationId = generateCorrelationId()
  const result = await apiPost(`/tasks/${payload.id}/completion`, {}, {
    headers: { 'X-Correlation-ID': correlationId }
  })
  // ...
}
```

### 4. 内存访问阶段 (MEM)
```typescript
// Transaction Processor 统一处理结果
await transactionProcessor.applyTaskTransaction(result, {
  correlation_id: correlationId,
  source: 'http'
})
```

### 5. 写回阶段 (WB)
```typescript
// Store 更新状态
taskStore.addOrUpdateTask_mut(result.task)

// 自动应用副作用
if (result.side_effects?.deleted_time_blocks) {
  for (const block of result.side_effects.deleted_time_blocks) {
    timeBlockStore.removeTimeBlock_mut(block.id)
  }
}
```

---

## 🎯 指令追踪系统

### 自动四级流水线追踪

项目实现了完全自动化的指令追踪系统，可以在开发环境中清晰看到每个操作的完整流水线：

```typescript
// 在 main.ts 中一键启用
if (import.meta.env.DEV) {
  enableAutoTracking().then(() => {
    logger.info('System:Init', '🎯 Automatic instruction tracking enabled!')
  })
}
```

### 追踪输出示例

```
🎯 [IF] Instruction: command.task.complete
🔧 [EX] Execute: task.complete {"id":"task-123"}
📡 [RES] HTTP Response: POST /tasks/task-123/completion (200)
💾 [WB] WriteBack: TaskStore, transaction, updateUI
✅ Pipeline Complete: command.task.complete (duration: 342ms)
```

### 开发调试工具

```javascript
// 浏览器控制台中可用的调试命令
appLogger.trackingOnly()        // 只显示指令追踪
appLogger.getTrackingStats()    // 查看追踪统计
appLogger.help()                // 显示所有可用命令
```

---

## 🏆 架构优势

### 1. **清晰的职责分离**
- 每个层次都有明确的职责边界
- 避免了传统架构中的循环依赖
- 便于维护和测试

### 2. **高度可预测性**
- 数据流向清晰，类似CPU流水线
- 状态变化可追踪
- 错误处理统一

### 3. **性能优化**
- 事务处理器自动去重
- 寄存器式的状态管理
- 响应式更新最小化

### 4. **开发体验**
- 自动指令追踪
- 结构化日志
- 完整的TypeScript支持

### 5. **可扩展性**
- 模块化设计
- 策略模式支持
- 插件式架构

---

## 🛠️ 开发指南

### 添加新功能的标准流程

1. **定义命令类型** (`src/commandBus/types.ts`)
```typescript
export type NewFeatureCommand = {
  type: 'feature.action'
  payload: { id: string; data: any }
}
```

2. **实现命令处理器** (`src/commandBus/handlers/`)
```typescript
const handleFeatureAction: CommandHandlerMap['feature.action'] = async (payload) => {
  const correlationId = generateCorrelationId()
  const result = await apiPost('/feature/action', payload, {
    headers: { 'X-Correlation-ID': correlationId }
  })
  await transactionProcessor.applyFeatureTransaction(result, {
    correlation_id: correlationId,
    source: 'http'
  })
}
```

3. **扩展Store状态** (`src/stores/feature/`)
```typescript
// 添加对应的状态管理
const mutations = {
  addOrUpdateFeature_mut: (item) => { /* ... */ },
  removeFeature_mut: (id) => { /* ... */ }
}
```

4. **在组件中使用**
```typescript
// 直接调用命令总线
await commandBus.emit('feature.action', { id, data })
```

### 最佳实践

#### ✅ 推荐做法
```typescript
// 使用命令总线处理用户操作
await commandBus.emit('task.complete', { id })

// 使用选择器获取数据
const task = taskStore.getTaskById_Mux(id)

// 使用 _mut 后缀的纯状态操作
taskStore.addOrUpdateTask_mut(task)
```

#### ❌ 避免做法
```typescript
// 直接在组件中调用API
await api.completeTask(id) // ❌

// 在Store中调用API
store.completeTask(id) // ❌

// 绕过命令总线
store.addOrUpdateTask_mut(await api.getTask(id)) // ❌
```

---

## 🔍 类比总结

| CPU组件 | Frontend组件 | 具体实现 | 职责 |
|---------|-------------|----------|------|
| 指令总线 | Command Bus | `commandBus` | 指令分发 |
| 指令译码器 | Command Handlers | `handlers/` | 业务逻辑 |
| 执行单元 | API Client | `api calls` | 外部交互 |
| 重排序缓冲 | Transaction Processor | `transactionProcessor` | 结果处理 |
| 寄存器堆 | Pinia Stores | `stores/` | 状态存储 |
| 多路复用器 | Getters/Selectors | `_Mux` 后缀 | 数据选择 |
| 导线 | Computed | `computed()` | 数据传输 |
| 中断控制器 | SSE Events | `events/` | 异步事件 |
| 调试单元 | Logger System | `logging/` | 追踪调试 |

---

## 🚀 总结

Cutie 的 Frontend-as-a-CPU 架构是一个创新的前端架构模式，它通过：

1. **将复杂的前端应用类比为CPU处理器**
2. **建立清晰的数据流水线**
3. **实现精确的状态管理**
4. **提供完整的开发工具链**

为现代前端应用提供了一个高效、可维护、可扩展的架构解决方案。

这种架构特别适合：
- 复杂的业务应用
- 需要实时同步的系统
- 对性能要求较高的应用
- 需要精确状态管理的场景

通过这种"硬件思维"来设计软件架构，我们获得了更好的可预测性、可维护性和开发体验。

---

**版本**: 1.0
**最后更新**: 2025-10-15
**作者**: Cutie Architecture Team

---

**🎮 Ready to build your CPU-like frontend? Happy coding!**