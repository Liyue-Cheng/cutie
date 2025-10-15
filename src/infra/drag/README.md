# 拖放策略系统 (Drag Strategy System)

全新的拖放策略系统，完全重写，采用声明式设计。

## 🎯 设计目标

1. **单一入口** - 所有拖放都经过统一的策略站
2. **声明式** - 策略以元数据形式注册，支持条件匹配
3. **可组合** - 策略可以组合和复用
4. **可观测** - 完整的追踪日志
5. **类型安全** - TypeScript 严格类型

## 📁 目录结构

```
src/infra/drag/
├── types.ts                 # 类型定义
├── strategy-registry.ts     # 策略注册中心
├── strategy-matcher.ts      # 策略匹配算法
├── strategy-executor.ts     # 策略执行引擎
├── index.ts                 # 统一导出 + 初始化
├── strategies/              # 策略实现
│   ├── task-scheduling.ts   # 任务调度策略
│   └── index.ts             # 策略导出
└── README.md                # 本文档
```

## 🚀 快速开始

### 1. 在组件中使用

```typescript
import { useDragStrategy } from '@/composables/drag/useDragStrategy'

const dragStrategy = useDragStrategy()

// 在 onDrop 回调中执行策略
onDrop: async (session) => {
  const result = await dragStrategy.executeDrop(session, targetZone)

  if (result.success) {
    console.log('✅', result.message)
  } else {
    console.error('❌', result.error)
  }
}
```

### 2. 查看已注册的策略

在浏览器控制台中：

```javascript
// 查看所有策略
strategyRegistry.debug()

// 获取统计信息
strategyRegistry.getStats()

// 按标签查找
strategyRegistry.findByTag('scheduling')
```

## 📝 创建新策略

### 策略定义

```typescript
import type { Strategy } from '@/infra/drag/types'

export const myCustomStrategy: Strategy = {
  // 唯一标识
  id: 'my-custom-strategy',

  // 策略名称
  name: 'My Custom Strategy',

  // 匹配条件
  conditions: {
    source: {
      viewKey: 'misc::staging', // 精确匹配
      taskStatus: 'staging',
    },
    target: {
      viewKey: /^daily::\d{4}-\d{2}-\d{2}$/, // 正则匹配
    },
    priority: 100, // 优先级（数字越大越优先）
  },

  // 执行动作
  action: {
    name: 'my_action',
    description: '我的自定义动作',

    // 前置检查（可选）
    async canExecute(ctx) {
      // 返回 true/false
      return true
    },

    // 执行逻辑（打印模式）
    async execute(ctx) {
      console.log(`📝 [PRINT MODE] 会执行:`, {
        task: ctx.task.title,
        from: ctx.sourceViewId,
        to: ctx.targetViewId,
      })

      return {
        success: true,
        message: '[PRINT MODE] 操作描述',
        affectedViews: [ctx.sourceViewId, ctx.targetViewId],
      }
    },
  },

  // 标签（可选）
  tags: ['custom', 'scheduling'],

  // 是否启用（可选，默认 true）
  enabled: true,
}
```

### 注册策略

在 `strategies/index.ts` 中导出：

```typescript
export { myCustomStrategy } from './my-custom-strategy'
```

## 🔍 策略匹配规则

策略按以下顺序匹配（所有条件都必须满足）：

1. **源视图条件**
   - `viewType` - 视图类型（支持数组）
   - `viewKey` - 视图键（支持字符串或正则）
   - `taskStatus` - 任务状态（支持数组）
   - `customCheck` - 自定义检查函数

2. **目标视图条件**
   - `viewType` - 视图类型（支持数组）
   - `viewKey` - 视图键（支持字符串或正则）
   - `acceptsStatus` - 接受的任务状态（数组）
   - `customCheck` - 自定义检查函数

3. **拖放模式**
   - `dragMode` - `'normal'` | `'copy'` | `'scheduled'`

4. **优先级**
   - 所有匹配的策略按 `priority` 降序排序
   - 返回第一个匹配的策略

## 📊 调试技巧

### 1. 查看策略执行日志

打开浏览器控制台，执行：

```javascript
// 只显示策略日志
appLogger.filterByTag('Drag:Strategy')

// 显示所有拖放相关日志
appLogger.filterByTag(['Drag:Strategy', 'Drag:CrossView', 'InstructionTracker'])
```

### 2. 查看策略匹配过程

策略执行时会自动打印详细信息：

```
🎯 Drag Strategy: Staging to Daily Schedule (staging-to-daily)
  📋 Strategy Details
  🔍 Matching Conditions
  📦 Context Data
  ⚙️ Strategy Conditions
  🎬 Action to Execute
```

### 3. 手动测试策略

```javascript
// 获取策略执行器
const executor = window.strategyExecutor

// 模拟一个拖放会话
const session = {
  id: 'test-001',
  source: {
    viewId: 'misc::staging',
    viewType: 'status',
    viewKey: 'misc::staging',
  },
  object: {
    type: 'task',
    data: {
      /* task data */
    },
  },
  dragMode: 'normal',
}

// 执行策略
await executor.execute(session, 'daily::2025-01-15')
```

## 🏷️ 内置策略

### 任务调度策略 (task-scheduling.ts)

1. **staging-to-daily** - 暂存区 → 日程
   - Priority: 100
   - Tags: `scheduling`, `staging`, `daily`

2. **daily-to-daily** - 日程 → 日程（重新安排）
   - Priority: 90
   - Tags: `scheduling`, `daily`, `reschedule`

3. **daily-to-staging** - 日程 → 暂存区（退回）
   - Priority: 95
   - Tags: `scheduling`, `staging`, `daily`, `return`

4. **staging-reorder** - 暂存区内部重排序
   - Priority: 80
   - Tags: `scheduling`, `staging`, `reorder`

## 🔧 高级用法

### 动态启用/禁用策略

```javascript
// 禁用某个策略
strategyRegistry.disable('staging-to-daily')

// 重新启用
strategyRegistry.enable('staging-to-daily')

// 查看状态
strategyRegistry.get('staging-to-daily').enabled
```

### 运行时注册策略

```javascript
import { strategyRegistry } from '@/infra/drag'

strategyRegistry.register({
  id: 'runtime-strategy',
  name: 'Runtime Added Strategy',
  conditions: {
    /* ... */
  },
  action: {
    /* ... */
  },
})
```

### 查询策略

```javascript
// 获取所有策略
const all = strategyRegistry.getAll()

// 按标签查找
const scheduling = strategyRegistry.findByTag('scheduling')

// 查找所有匹配的策略（调试用）
const matches = strategyRegistry.findAllMatches(session, targetZone)
```

## 📚 相关文档

- [类型定义](./types.ts)
- [策略示例](./strategies/task-scheduling.ts)
- [组件 API](../../composables/drag/useDragStrategy.ts)

## ⚠️ 注意事项

1. **当前为打印模式** - 策略只打印执行计划，不执行实际业务
2. **策略 ID 必须唯一** - 重复注册会覆盖
3. **优先级很重要** - 多个策略匹配时，只执行优先级最高的
4. **正则匹配** - `viewKey` 支持 `RegExp`，用于匹配一类视图
5. **类型安全** - 充分利用 TypeScript 类型检查

## 🚀 未来计划

- [ ] 添加策略预览（hover 提示）
- [ ] 支持策略组合（pipeline）
- [ ] 添加策略回滚机制
- [ ] 支持异步条件检查
- [ ] 添加策略性能监控
- [ ] 切换到实际执行模式
