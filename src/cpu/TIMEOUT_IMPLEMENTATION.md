# ⏱️ 超时机制实现原理

## 🎯 设计原则

**超时配置在指令定义中，超时控制在 EX 阶段执行**

- ✅ 每个指令可以自定义超时时间
- ✅ 符合 ISA（指令集架构）设计理念
- ✅ 流水线保持简单，不关心具体策略

---

## 🔍 实现原理

### 1. 在指令元数据中配置超时

```typescript
// src/cpu/isa/task-isa.ts
export const TaskISA: ISADefinition = {
  'task.create': {
    meta: {
      description: '创建任务',
      category: 'task',
      resourceIdentifier: () => ['tasks:create'],
      priority: 6,
      timeout: 10000, // 🔥 10 秒超时
    },
    request: {
      method: 'POST',
      url: '/tasks',
      body: (payload) => payload,
    },
    commit: async (result, payload, context) => {
      // ...
    },
  },

  'schedule.update': {
    meta: {
      description: '更新日程',
      category: 'schedule',
      resourceIdentifier: (payload) => [`task:${payload.task_id}`],
      priority: 6,
      timeout: 5000, // 🔥 5 秒超时（更快的操作）
    },
    // ...
  },
}
```

### 2. EX 阶段应用超时控制

```typescript
// src/cpu/stages/EX.ts
export class ExecuteStage {
  async execute(instruction: QueuedInstruction): Promise<void> {
    const isa = ISA[instruction.type]

    // 🔥 创建执行 Promise
    const executePromise = (async () => {
      if (isa.request) {
        return await executeRequest(isa.request, instruction.payload, instruction.context)
      } else if (isa.execute) {
        return await isa.execute(instruction.payload, instruction.context)
      }
    })()

    // 🔥 如果指令定义了超时，应用超时控制
    if (isa.meta.timeout) {
      const timeoutPromise = new Promise<never>((_, reject) => {
        setTimeout(() => {
          reject(new Error(`指令 ${instruction.type} 执行超时（${isa.meta.timeout}ms）`))
        }, isa.meta.timeout)
      })

      // 竞赛：谁先完成就用谁
      result = await Promise.race([executePromise, timeoutPromise])
    } else {
      // 没有超时，直接执行
      result = await executePromise
    }
  }
}
```

### 3. 流程图

```
组件发射指令
    ↓
IF: 创建指令
    ↓
SCH: 调度
    ↓
EX: 读取指令的 timeout 配置
    ↓
  ┌─────────────────────────────┐
  │ Promise.race([              │
  │   executePromise,    ← 执行  │
  │   timeoutPromise     ← 超时  │
  │ ])                          │
  └─────────────────────────────┘
    │           │
    │           └─────> 超时（5秒后）
    │                     ↓
    │                  reject(Error)
    │                     ↓
    │                  回滚乐观更新
    │                     ↓
    │                  Promise reject
    │
    └──────> 执行完成（3秒）
              ↓
           resolve(result)
              ↓
           commit 数据
              ↓
           Promise resolve
```

---

## 📊 副作用分析

### ✅ 优点

1. **粒度控制**

   ```typescript
   // 不同指令有不同的超时需求
   'task.create': { timeout: 10000 },        // 创建任务：10秒
   'schedule.update': { timeout: 5000 },     // 更新日程：5秒
   'file.upload': { timeout: 60000 },        // 上传文件：60秒
   'debug.quick_success': { timeout: undefined }, // 无超时
   ```

2. **业务语义清晰**

   ```typescript
   // 从指令定义就能看出性能要求
   meta: {
     timeout: 5000,  // "这个操作应该在 5 秒内完成"
   }
   ```

3. **流水线保持简单**
   - 流水线不需要知道超时逻辑
   - 只需要在 EX 阶段应用 ISA 的配置

### ⚠️ 潜在问题

#### 问题 1: 超时后请求仍在执行

```typescript
// ⚠️ 超时不会取消正在执行的网络请求
const executePromise = fetch('/api/tasks', { ... })
const timeoutPromise = timeout(5000)

const result = await Promise.race([executePromise, timeoutPromise])

// 如果超时，timeoutPromise reject
// 但 executePromise（fetch）仍在后台执行！
```

**影响**：

- ❌ 网络请求会继续发送
- ❌ 后端可能会处理这个请求
- ❌ SSE 事件可能会返回（但会被去重）

**解决方案**：使用 AbortController

```typescript
// 🔧 改进版：可取消的请求
async function executeRequest(config, payload, context) {
  const abortController = new AbortController()

  // 传递 signal 给 fetch
  const response = await fetch(url, {
    signal: abortController.signal,
    // ...
  })

  // 超时时取消请求
  if (timeout) {
    setTimeout(() => {
      abortController.abort()
    }, timeout)
  }
}
```

#### 问题 2: 乐观更新的回滚

```typescript
// 场景：
// 1. 应用乐观更新（UI 立即变化）
// 2. 开始网络请求
// 3. 超时（5 秒）
// 4. 回滚乐观更新（UI 变回去）
// 5. 网络请求实际成功（10 秒后）
// 6. SSE 事件返回（任务实际创建了）

// ⚠️ 问题：用户看到"失败"，但后端成功了
```

**影响**：

- ❌ UI 和后端状态不一致
- ❌ 用户体验差（以为失败了，但实际成功）

**解决方案**：

1. **合理设置超时时间**（根据实际后端性能）
2. **后端幂等性**（重复请求不会重复创建）
3. **SSE 最终一致性**（后端成功后，SSE 会同步状态）

#### 问题 3: Promise 内存泄漏

```typescript
// ⚠️ 超时后，executePromise 仍然 pending
const executePromise = executeRequest(...)  // Promise 对象
const timeoutPromise = timeout(5000)

await Promise.race([executePromise, timeoutPromise])  // timeoutPromise 赢了

// executePromise 仍然存在（虽然没人等待它）
// 直到请求真正完成或失败才会被 GC
```

**影响**：

- ⚠️ 短时间内有少量内存占用
- ✅ 浏览器会自动垃圾回收

**解决方案**：可接受，无需特殊处理

---

## 🛡️ 改进方案

### 方案 1: 添加 AbortController 支持

```typescript
// src/cpu/utils/request.ts
import type { RequestConfig } from '../isa/types'
import type { InstructionContext } from '../types'
import { apiGet, apiPost, apiPatch, apiDelete, apiPut } from '@/stores/shared'

export async function executeRequest(
  config: RequestConfig | RequestConfig[],
  payload: any,
  context: InstructionContext,
  abortSignal?: AbortSignal // 🔥 支持取消
): Promise<any> {
  if (Array.isArray(config)) {
    const results = await Promise.all(
      config.map((req) => executeSingleRequest(req, payload, context, abortSignal))
    )
    return results
  } else {
    return await executeSingleRequest(config, payload, context, abortSignal)
  }
}

async function executeSingleRequest(
  config: RequestConfig,
  payload: any,
  context: InstructionContext,
  abortSignal?: AbortSignal
): Promise<any> {
  const url = typeof config.url === 'function' ? config.url(payload) : config.url
  const body = config.body ? config.body(payload) : payload

  const headers = {
    'X-Correlation-ID': context.correlationId,
    ...config.headers,
  }

  // 🔥 传递 abortSignal 给 API 函数
  const options = {
    headers,
    signal: abortSignal, // 支持取消
  }

  switch (config.method) {
    case 'GET':
      return await apiGet(url, context.correlationId, options)
    case 'POST':
      return await apiPost(url, body, options)
    // ...
  }
}
```

然后在 EX 阶段：

```typescript
// src/cpu/stages/EX.ts
async execute(instruction: QueuedInstruction): Promise<void> {
  const isa = ISA[instruction.type]

  // 🔥 创建 AbortController
  const abortController = new AbortController()

  const executePromise = (async () => {
    if (isa.request) {
      return await executeRequest(
        isa.request,
        instruction.payload,
        instruction.context,
        abortController.signal  // 传递 signal
      )
    } else if (isa.execute) {
      return await isa.execute(instruction.payload, instruction.context)
    }
  })()

  if (isa.meta.timeout) {
    const timeoutPromise = new Promise<never>((_, reject) => {
      setTimeout(() => {
        abortController.abort()  // 🔥 取消请求
        reject(new Error(`指令执行超时（${isa.meta.timeout}ms）`))
      }, isa.meta.timeout)
    })

    result = await Promise.race([executePromise, timeoutPromise])
  }
}
```

### 方案 2: 超时后等待清理完成

```typescript
// 超时后，给请求一点时间完成（优雅降级）
if (isa.meta.timeout) {
  try {
    result = await Promise.race([executePromise, timeoutPromise])
  } catch (error) {
    if (error.message.includes('超时')) {
      // 🔥 超时后，等待最多 1 秒让请求完成
      const gracePeriod = Promise.race([
        executePromise.catch(() => null),
        new Promise((resolve) => setTimeout(resolve, 1000)),
      ])

      await gracePeriod
    }
    throw error
  }
}
```

---

## 📝 实际副作用总结

| 副作用           | 严重性    | 是否需要处理 | 解决方案                    |
| ---------------- | --------- | ------------ | --------------------------- |
| 超时后请求仍执行 | ⚠️ 中等   | 建议处理     | 使用 AbortController        |
| UI 和后端不一致  | ⚠️ 中等   | 已有保护     | SSE 最终一致性 + 后端幂等性 |
| Promise 内存占用 | ✅ 低     | 无需处理     | 自动垃圾回收                |
| Map 内存泄漏     | ✅ 已解决 | 已处理       | reset() 清理                |

---

## 💡 最佳实践

### 1. 合理设置超时时间

```typescript
// ✅ 根据操作复杂度设置
'task.complete': {
  meta: {
    timeout: 5000,  // 简单操作：5秒
  }
}

'task.create_with_schedule': {
  meta: {
    timeout: 15000,  // 复杂操作：15秒
  }
}

'file.upload': {
  meta: {
    timeout: 60000,  // 文件上传：60秒
  }
}

'debug.quick_success': {
  meta: {
    timeout: undefined,  // 无超时限制
  }
}
```

### 2. 后端确保幂等性

```rust
// 后端：使用 correlationId 实现幂等
pub async fn create_task(payload: CreateTaskPayload, correlation_id: String) {
  // 检查是否已经创建过
  if let Some(existing) = find_by_correlation_id(&correlation_id).await {
    return Ok(existing)  // 返回已创建的任务
  }

  // 创建新任务
  let task = insert_task(payload).await?;
  Ok(task)
}
```

### 3. 处理超时错误

```typescript
// 组件中正确处理超时
async function createTask() {
  try {
    const result = await pipeline.dispatch('task.create', { title: '任务' })
    console.log('✅ 创建成功')
  } catch (error) {
    if (error.message.includes('超时')) {
      // 🔥 超时：提示用户，但任务可能仍会创建
      console.warn('操作超时，请稍后刷新查看结果')

      // 可以选择：
      // 1. 重试
      // 2. 等待 SSE 事件
      // 3. 轮询查询任务是否创建
    } else {
      console.error('创建失败', error)
    }
  }
}
```

---

## 🚀 使用示例

### 测试超时机制

```typescript
import { pipeline } from '@/cpu'

// 1. 测试超时（会失败）
try {
  await pipeline.dispatch('debug.test_timeout', {})
} catch (error) {
  console.log('❌ 如预期，触发超时:', error.message)
  // 输出: "指令 debug.test_timeout 执行超时（5000ms）"
}

// 2. 测试正常执行（会成功）
try {
  await pipeline.dispatch('task.create', { title: '任务' })
  console.log('✅ 在超时前完成')
} catch (error) {
  console.log('❌ 超时或失败:', error.message)
}
```

### 在 CPU 调试器中测试

1. 打开 **CPU 调试页面**
2. 点击 **"测试超时（5秒）"** 按钮
3. 观察控制台：

```
🎯 [21:48:23.123] debug.test_timeout 指令创建

❌ [21:48:28.125] debug.test_timeout → 失败 5002ms
  原因: 指令 debug.test_timeout 执行超时（5000ms）

  流水线阶段:
  IF→SCH  0ms
  SCH→EX  0ms
  EX→WB   5002ms
  总耗时: 5002ms

  💡 建议:
  • 执行耗时 5002ms，超过 1 秒，检查是否存在性能问题
```

---

## 🎯 总结

### 实现原理

- ✅ **配置在指令**：每个指令在 `meta.timeout` 配置超时时间
- ✅ **控制在 EX**：EX 阶段使用 `Promise.race()` 实现超时
- ✅ **自动清理**：超时后自动触发回滚和 Promise reject

### 副作用

- ⚠️ **请求仍执行**：超时后网络请求不会被取消（可用 AbortController 改进）
- ✅ **最终一致性**：SSE 会同步后端的实际状态
- ✅ **内存安全**：Promise 会被自动垃圾回收

### 最佳实践

1. 根据操作复杂度设置合理的超时时间
2. 后端实现幂等性（基于 correlationId）
3. 组件正确处理超时错误
4. 可选：使用 AbortController 取消请求

这个设计**完全符合 ISA 架构理念**，超时是指令的固有属性！🎉
