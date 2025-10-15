# 🚀 CPU 日志系统使用指南

## 📖 快速开始

### 1. 打开控制台，立即看到指令执行

CPU 日志系统已经自动集成到流水线中，**无需任何配置**即可使用！

```bash
# 启动应用后，打开浏览器控制台（F12）
# 你会看到类似这样的输出：

🎯 [20:30:15.123] task.create 指令创建
  { id: 'instr_xxx', correlationId: 'corr_xxx' }

✅ [20:30:15.248] task.create → 成功 125ms
  流水线阶段:
  IF  ████ 0ms
  SCH ████ 0ms
  EX  ████████████████████ 123ms
  WB  ████ 2ms
  ✓ 乐观更新
```

### 2. 调整控制台输出级别

前往 **CPU 调试器页面**（左侧边栏 → CPU 调试）：

![Console Level Selector](控制台级别选择器)

选择你想要的级别：

- **关闭 (SILENT)**: 完全静音，适合生产环境
- **最小 (MINIMAL)**: 只看成功/失败，适合日常使用
- **正常 (NORMAL)**: 看关键阶段，**推荐用于开发**
- **详细 (VERBOSE)**: 看所有细节，适合深度调试
- **调试 (DEBUG)**: 看 payload 和 result，适合排查问题

### 3. 实际使用场景

#### 场景 1：验证任务创建是否成功

```typescript
// 你在看板创建了一个任务
// 立即在控制台看到：

🎯 [20:30:15.123] task.create 指令创建
✅ [20:30:15.248] task.create → 成功 125ms
  流水线阶段:
  IF  ████ 0ms
  EX  ████████████████████ 123ms

// 如果失败，会自动展开：
❌ [20:30:15.456] task.create → 失败 24ms
  原因: 网络错误
  💡 建议: 检查后端服务是否运行
```

#### 场景 2：调试拖放排期为什么闪烁

```typescript
// 1. 设置控制台级别为 VERBOSE
cpuConsole.setLevel(ConsoleLevel.VERBOSE)

// 2. 拖动任务到新日期
// 3. 在控制台看到完整流程：

🎯 [20:30:15.123] schedule.update 指令创建
  🔄 [20:30:15.124] 乐观更新已应用  // ← 这里应该立即更新 UI
✅ [20:30:15.248] schedule.update → 成功 125ms

// 如果看到回滚：
⚠️ [20:30:15.456] schedule.update 乐观更新已回滚
  原因: HTTP 500: database is locked

// 说明：乐观更新失败 → UI 会闪烁 → 需要修复后端
```

#### 场景 3：分析性能瓶颈

在 CPU 调试器页面：

```typescript
// 1. 点击"打印统计信息"按钮
// 控制台输出：

📊 流水线统计
  总指令数: 150
  成功: 142 (94.7%)
  失败: 8 (5.3%)
  平均延迟: 125ms

// 2. 使用 CPUDebugger 查询最慢的指令
import { cpuDebugger } from '@/cpu/logging'

const slowest = cpuDebugger.getSlowestInstructions(10)
console.table(slowest)

// 输出：
// ┌─────┬────────────────────┬──────────┬──────────┐
// │ Rank│ Type               │ Duration │ ID       │
// ├─────┼────────────────────┼──────────┼──────────┤
// │  1  │ schedule.update    │ 2345ms   │ instr_xx │
// │  2  │ task.create        │ 1234ms   │ instr_yy │
// └─────┴────────────────────┴──────────┴──────────┘

// 3. 诊断为什么慢
const diagnosis = cpuDebugger.diagnoseSlowInstruction('instr_xx')
console.log(diagnosis)

// 输出：
// {
//   bottleneck: { stage: 'SCH→EX', duration: 2000ms, percentage: 85% },
//   suggestions: [
//     '调度器等待时间较长，存在资源冲突',
//     'SCH→EX 占总耗时 85.3%，是主要瓶颈'
//   ]
// }
```

#### 场景 4：查询特定类型指令的性能

```typescript
import { cpuLogger } from '@/cpu/logging'

// 分析 schedule.update 的性能
const perf = cpuLogger.analyzeInstructionPerformance('schedule.update')

console.log(`
  执行次数: ${perf.count}
  成功率: ${(perf.successRate * 100).toFixed(1)}%
  平均延迟: ${perf.avgLatency.toFixed(0)}ms
  P50: ${perf.p50.toFixed(0)}ms
  P95: ${perf.p95.toFixed(0)}ms
  P99: ${perf.p99.toFixed(0)}ms
`)

// 输出：
// 执行次数: 50
// 成功率: 96.0%
// 平均延迟: 125ms
// P50: 120ms
// P95: 180ms
// P99: 250ms
```

#### 场景 5：分析资源冲突

```typescript
import { cpuLogger } from '@/cpu/logging'

// 查看哪些资源冲突最多
const conflicts = cpuLogger.analyzeResourceConflicts()

console.table(conflicts.slice(0, 5)) // 前 5 个热点

// 输出：
// ┌───┬──────────────────────┬──────────────┬─────────────┐
// │ # │ Resource             │ ConflictCount│ AvgWaitTime │
// ├───┼──────────────────────┼──────────────┼─────────────┤
// │ 0 │ task:abc123          │ 23           │ 156ms       │
// │ 1 │ schedule:xyz789      │ 15           │ 89ms        │
// └───┴──────────────────────┴──────────────┴─────────────┘

// 说明：task:abc123 发生了 23 次冲突，平均等待 156ms
// 建议：检查是否有多个操作同时修改这个任务
```

#### 场景 6：分析乐观更新回滚率

```typescript
import { cpuLogger } from '@/cpu/logging'

// 查看乐观更新的回滚情况
const rollbackStats = cpuLogger.analyzeOptimisticRollbackRate()

console.log(`
  总乐观更新: ${rollbackStats.totalOptimistic}
  回滚次数: ${rollbackStats.rollbackCount}
  回滚率: ${(rollbackStats.rollbackRate * 100).toFixed(1)}%
`)

console.table(rollbackStats.byInstructionType)

// 输出：
// ┌────────────────────┬───────┬──────────┬────────┐
// │ Type               │ Total │ Rollbacks│ Rate   │
// ├────────────────────┼───────┼──────────┼────────┤
// │ schedule.update    │ 50    │ 2        │ 4.0%   │
// │ task.update        │ 30    │ 0        │ 0.0%   │
// └────────────────────┴───────┴──────────┴────────┘

// 说明：schedule.update 有 4% 的回滚率
// 建议：如果 > 5%，说明乐观更新不够准确
```

---

## 🎨 高级用法

### 1. 导出数据进行离线分析

```typescript
import { cpuLogger } from '@/cpu/logging'

// 导出最近 1 小时的所有指令数据
const data = cpuLogger.exportData({
  timeRange: {
    start: Date.now() - 3600000,
    end: Date.now(),
  },
})

// 保存为 JSON
const json = JSON.stringify(data, null, 2)
const blob = new Blob([json], { type: 'application/json' })
const url = URL.createObjectURL(blob)
const a = document.createElement('a')
a.href = url
a.download = `cpu-logs-${Date.now()}.json`
a.click()

// 可以用 Excel、Python、R 等工具分析
```

### 2. 复杂查询

```typescript
import { cpuLogger, CPUEventType } from '@/cpu/logging'

// 查询：所有执行超过 100ms 的 schedule.update 指令
const slowUpdates = cpuLogger.query({
  instructionType: 'schedule.update',
  minLatency: 100,
  timeRange: {
    start: Date.now() - 3600000, // 最近 1 小时
    end: Date.now(),
  },
})

console.log(`找到 ${slowUpdates.length} 条慢指令`)

// 查询：所有触发了回滚的指令
const rollbacks = cpuLogger.query({
  eventType: CPUEventType.OPTIMISTIC_ROLLED_BACK,
})

console.log(`找到 ${rollbacks.length} 次回滚`)

// 查询：所有资源冲突事件
const conflicts = cpuLogger.query({
  tags: ['conflict'],
})

console.log(`找到 ${conflicts.length} 次资源冲突`)
```

### 3. 实时监控

```typescript
import { cpuDebugger } from '@/cpu/logging'

// 每 5 秒打印一次实时统计
setInterval(() => {
  const stats = cpuDebugger.getRealtimeStats(5) // 最近 5 秒

  console.log(`
    📊 实时统计（最近 5 秒）
    指令吞吐量: ${stats.instructionsPerSecond.toFixed(2)} IPS
    平均延迟: ${stats.avgLatency.toFixed(0)}ms
    错误率: ${(stats.errorRate * 100).toFixed(1)}%
    热门指令: ${stats.topInstructionTypes.map((t) => t.type).join(', ')}
  `)
}, 5000)
```

### 4. 指令重放（时间旅行调试）

```typescript
import { cpuDebugger } from '@/cpu/logging'

// 重放某个指令的完整执行过程
const replay = cpuDebugger.replayInstruction('instr_abc123')

console.log('指令重放：')
console.table(replay.timeline)

// 输出：
// ┌───┬──────────────┬───────┬──────────────────────┐
// │ # │ Time         │ Stage │ Event                │
// ├───┼──────────────┼───────┼──────────────────────┤
// │ 0 │ 1634567890123│ IF    │ instruction.created  │
// │ 1 │ 1634567890124│ SCH   │ instruction.issued   │
// │ 2 │ 1634567890125│ EX    │ optimistic.applied   │
// │ 3 │ 1634567890126│ EX    │ network.request_sent │
// │ 4 │ 1634567890250│ EX    │ network.response_... │
// │ 5 │ 1634567890251│ WB    │ instruction.committed│
// └───┴──────────────┴───────┴──────────────────────┘

console.log(`执行成功: ${replay.success}`)
```

---

## 🎯 最佳实践

### 开发环境配置

```typescript
// src/main.ts 或其他入口文件

import { cpuConsole, ConsoleLevel } from '@/cpu/logging'

if (import.meta.env.DEV) {
  // 开发环境：详细模式
  cpuConsole.setLevel(ConsoleLevel.VERBOSE)
} else {
  // 生产环境：只看失败
  cpuConsole.setLevel(ConsoleLevel.MINIMAL)
}
```

### 调试特定功能时

```typescript
import { cpuConsole } from '@/cpu/logging'

// 只看 schedule 相关指令
cpuConsole.setFilter(['schedule.update', 'schedule.create', 'schedule.delete'])

// 开发完成后，清除过滤器
cpuConsole.setFilter([])
```

### 定期检查性能

```typescript
// 每天早上查看昨天的性能
import { cpuLogger } from '@/cpu/logging'

const yesterday = Date.now() - 86400000
const events = cpuLogger.getEventsByTimeRange(yesterday, Date.now())

console.log(`昨天执行了 ${events.length} 条指令`)

const instructions = new Set(events.map((e) => e.instructionId))
console.log(`共 ${instructions.size} 个唯一指令`)

// 找出最慢的 10 个
import { cpuDebugger } from '@/cpu/logging'
const slowest = cpuDebugger.getSlowestInstructions(10)
console.table(slowest)
```

---

## 🔧 故障排查流程

### 问题 1: UI 操作后没有反应

```typescript
// 1. 检查控制台是否有指令
// 如果没有 → 指令没有发射
// 如果有失败 → 看错误信息

// 2. 如果指令成功但 UI 没更新
// 检查是否有 commit 函数
import { ISA } from '@/cpu/isa'
console.log(ISA['task.update'].commit) // 应该有函数

// 3. 检查是否被中断处理器去重了
import { interruptHandler } from '@/cpu/interrupt/InterruptHandler'
console.log(interruptHandler.getStats())
```

### 问题 2: UI 闪烁

```typescript
// 1. 检查是否有乐观更新
cpuConsole.setLevel(ConsoleLevel.VERBOSE)

// 2. 看是否有回滚
// 如果有 ⚠️ 乐观更新已回滚 → 说明请求失败

// 3. 查看回滚率
import { cpuLogger } from '@/cpu/logging'
const stats = cpuLogger.analyzeOptimisticRollbackRate()
console.log(stats)

// 如果 > 5% → 乐观更新逻辑有问题或后端经常失败
```

### 问题 3: 操作很慢

```typescript
// 1. 找出最慢的指令
import { cpuDebugger } from '@/cpu/logging'
const slowest = cpuDebugger.getSlowestInstructions(10)
console.table(slowest)

// 2. 诊断瓶颈
const diagnosis = cpuDebugger.diagnoseSlowInstruction(slowest[0].instructionId)
console.log(diagnosis)

// 3. 如果瓶颈在 SCH → 资源冲突
// 查看冲突详情
const conflicts = cpuLogger.analyzeResourceConflicts()
console.table(conflicts)

// 4. 如果瓶颈在 EX → 网络慢
// 检查后端性能
```

---

## 📚 API 速查

### CPUConsole（控制台）

```typescript
import { cpuConsole, ConsoleLevel } from '@/cpu/logging'

// 设置级别
cpuConsole.setLevel(ConsoleLevel.VERBOSE)

// 设置过滤器
cpuConsole.setFilter(['task.create', 'task.update'])

// 打印统计
cpuConsole.printStats({ total: 100, success: 95, failed: 5, avgLatency: 125 })

// 打印分隔线
cpuConsole.printSeparator('我的调试会话')

// 启用/禁用
cpuConsole.enable()
cpuConsole.disable()
```

### CPULogger（日志记录器）

```typescript
import { cpuLogger, CPUEventType } from '@/cpu/logging'

// 查询
cpuLogger.getInstructionTrace('instr_xxx')
cpuLogger.getCorrelationTrace('corr_xxx')
cpuLogger.getEventsByType(CPUEventType.NETWORK_ERROR)
cpuLogger.getEventsByInstructionType('task.update')
cpuLogger.getEventsByTimeRange(start, end)
cpuLogger.query({
  /* 复杂条件 */
})

// 分析
cpuLogger.analyzeInstructionPerformance('task.update')
cpuLogger.analyzeResourceConflicts()
cpuLogger.analyzeOptimisticRollbackRate()
cpuLogger.analyzeThroughput(60000)

// 导出
cpuLogger.exportData()
cpuLogger.getStats()
cpuLogger.clear()
```

### CPUDebugger（调试器）

```typescript
import { cpuDebugger } from '@/cpu/logging'

// 查询
cpuDebugger.getSlowestInstructions(10)
cpuDebugger.getFailedInstructions()
cpuDebugger.getRolledBackInstructions()
cpuDebugger.getResourceConflictChain('instr_xxx')

// 诊断
cpuDebugger.diagnoseSlowInstruction('instr_xxx')
cpuDebugger.replayInstruction('instr_xxx')
cpuDebugger.getRealtimeStats(5)
```

---

## 💡 小贴士

1. **日常开发用 NORMAL 级别**，够用了
2. **调试问题用 VERBOSE 或 DEBUG**，能看到所有细节
3. **生产环境用 MINIMAL**，减少噪音
4. **定期导出数据**，离线分析性能趋势
5. **设置过滤器**，专注于当前开发的功能
6. **善用打印统计**，快速了解整体情况
7. **遇到慢指令**，用 `diagnoseSlowInstruction` 分析瓶颈
8. **看到回滚**，检查乐观更新逻辑或后端问题

---

## 🎉 总结

**CPUConsole**: 你的"眼睛"，实时看到指令执行  
**CPULogger**: 系统的"黑匣子"，记录所有细节用于分析  
**CPUDebugger**: 你的"助手"，帮你找问题、分析性能

两者配合使用，覆盖"实时调试"和"事后分析"两个场景，让你的开发效率翻倍！🚀
