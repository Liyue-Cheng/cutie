# KiloCode → Cutie 精简移植方案

## 需求澄清

**Cutie 只需要的功能**:
- ✅ 添加任务
- ✅ 读取任务
- ✅ 修改任务
- ✅ 创建/修改日程
- ✅ 查询任务状态

**不需要的功能**:
- ❌ 文件读写操作
- ❌ 代码执行/终端命令
- ❌ 浏览器自动化
- ❌ MCP外部工具集成
- ❌ 代码搜索/编辑
- ❌ 复杂的模式切换

---

## 大幅简化的架构

### 原 KiloCode 架构复杂度
```
3514行 Task.ts + 30+工具 + 50+模型提供商 + MCP + 权限系统 + ...
```

### Cutie 精简架构
```
~500行核心代码 + 5-8个工具 + 1个模型 + 简单权限
```

**复杂度降低**: 约 **80%** 🎉

---

## 精简后的移植清单

### ✅ 仍需移植的核心代码 (~15%)

#### 1. **流式解析器** ⭐⭐⭐ (必须，但可简化)
**原文件**: `kilocode/src/core/assistant-message/AssistantMessageParser.ts` (400+行)

**精简版本**: 只需要 **JSON格式解析**，删除 XML 解析

```typescript
// ✅ 只保留这部分
public *processNativeToolCalls(toolCalls: NativeToolCall[]): Generator {
  for (const toolCall of toolCalls) {
    // 处理 OpenAI JSON 格式工具调用
    if (toolCall.function?.name && toolCall.function?.arguments) {
      const toolUse = {
        type: "tool_use",
        name: toolCall.function.name,
        params: JSON.parse(toolCall.function.arguments)
      }
      yield toolUse
    }
  }
}
```

**可删除**:
- ❌ `processChunk()` - XML解析器 (245-350行)
- ❌ 逐字符状态机逻辑
- ❌ 部分工具调用处理

**精简后**: ~100行 (从400行减少75%)

---

#### 2. **工具调用类型定义** ⭐⭐⭐ (必须)
**原文件**: `kilocode/src/shared/tools.ts`

**只需要这些类型**:
```typescript
// ✅ 保留
export interface ToolUse {
  type: "tool_use"
  name: string  // 简化：不需要 ToolName 枚举
  params: Record<string, any>
}

export interface TextContent {
  type: "text"
  content: string
}

export type AssistantMessageContent = TextContent | ToolUse

// ❌ 删除
// - TOOL_GROUPS (不需要分组)
// - 权限检查相关
// - DiffStrategy
// - 浏览器相关
```

**精简后**: ~30行 (从200+行减少85%)

---

#### 3. **系统提示词** ⭐⭐ (需要，但大幅简化)
**原文件**: `kilocode/src/core/prompts/system.ts` (200+行)

**精简版本**:
```typescript
// ✅ 极简版提示词
export function generateSystemPrompt(): string {
  return `你是一个任务管理助手，可以帮助用户管理他们的任务和日程。

## 可用工具

${getTaskToolDescriptions()}

## 工具使用规则

1. 使用 JSON 格式调用工具
2. 每次只调用一个工具
3. 在工具执行完成后，根据结果继续对话

## 你的目标

帮助用户高效地管理任务和日程，提供清晰的反馈。`
}
```

**可删除**:
- ❌ 模式系统
- ❌ MCP描述
- ❌ 自定义指令
- ❌ VS Code特定说明
- ❌ Markdown格式化指南（模型自己知道）
- ❌ 复杂的功能说明

**精简后**: ~50行 (从200+行减少75%)

---

#### 4. **工具描述生成** ⭐⭐ (需要，但超级简单)
**原文件**: `kilocode/src/core/prompts/tools/index.ts` (200+行)

**精简版本**:
```typescript
// ✅ 只需要这几个工具的描述
const TASK_TOOLS = {
  create_task: {
    description: "创建新任务",
    parameters: {
      title: "任务标题",
      area_id: "所属区域ID（可选）",
      scheduled_date: "计划日期（可选，格式：YYYY-MM-DD）"
    }
  },
  read_tasks: {
    description: "读取任务列表",
    parameters: {
      view_context: "视图上下文（如 daily::2024-01-01, staging, area::uuid）"
    }
  },
  update_task: {
    description: "更新任务",
    parameters: {
      task_id: "任务ID",
      title: "新标题（可选）",
      completed: "是否完成（可选）"
    }
  },
  create_schedule: {
    description: "为任务创建日程",
    parameters: {
      task_id: "任务ID",
      scheduled_date: "日期（YYYY-MM-DD）"
    }
  },
  delete_task: {
    description: "删除任务",
    parameters: {
      task_id: "任务ID"
    }
  }
}

export function getTaskToolDescriptions(): string {
  return Object.entries(TASK_TOOLS)
    .map(([name, tool]) => {
      const params = Object.entries(tool.parameters)
        .map(([key, desc]) => `  - ${key}: ${desc}`)
        .join('\n')
      return `### ${name}\n${tool.description}\n参数:\n${params}`
    })
    .join('\n\n')
}
```

**可删除**:
- ❌ 所有文件操作工具描述
- ❌ 命令执行工具
- ❌ 浏览器工具
- ❌ MCP工具
- ❌ 复杂的动态生成逻辑

**精简后**: ~60行 (从200+行减少70%)

---

#### 5. **工具执行框架** ⭐⭐⭐ (核心，但可大幅简化)
**原文件**: `kilocode/src/core/assistant-message/presentAssistantMessage.ts` (700+行)

**精简版本**:
```typescript
export async function executeToolCall(
  toolUse: ToolUse,
  onApproval?: (tool: string) => Promise<boolean>
): Promise<ToolResult> {
  // 1. 用户批准（可选）
  if (onApproval) {
    const approved = await onApproval(toolUse.name)
    if (!approved) {
      return {
        success: false,
        message: "用户拒绝执行此工具"
      }
    }
  }

  // 2. 路由到具体工具
  try {
    switch (toolUse.name) {
      case "create_task":
        return await createTaskTool(toolUse.params)
      case "read_tasks":
        return await readTasksTool(toolUse.params)
      case "update_task":
        return await updateTaskTool(toolUse.params)
      case "create_schedule":
        return await createScheduleTool(toolUse.params)
      case "delete_task":
        return await deleteTaskTool(toolUse.params)
      default:
        return {
          success: false,
          message: `未知工具: ${toolUse.name}`
        }
    }
  } catch (error) {
    return {
      success: false,
      message: `工具执行失败: ${error.message}`
    }
  }
}
```

**可删除**:
- ❌ 复杂的锁机制
- ❌ 流式更新逻辑
- ❌ 检查点系统
- ❌ 30+个工具的case分支
- ❌ 工具重复检测
- ❌ 部分工具处理
- ❌ YOLO模式守门员

**精简后**: ~100行 (从700+行减少85%)

---

#### 6. **任务执行引擎** ⭐⭐⭐ (核心，但可超级简化)
**原文件**: `kilocode/src/core/task/Task.ts` (3514行！)

**精简版本**: 只需要一个简单的流处理循环

```typescript
export async function processAiStream(
  stream: ReadableStream,
  onToolCall: (toolUse: ToolUse) => Promise<ToolResult>,
  onTextChunk: (text: string) => void
): Promise<void> {
  const parser = new JsonToolCallParser()

  for await (const chunk of parseSSEStream(stream)) {
    switch (chunk.type) {
      case "text":
        onTextChunk(chunk.text)
        break

      case "tool_call":
        const toolUse = parser.processToolCall(chunk.toolCall)
        if (toolUse.complete) {
          const result = await onToolCall(toolUse)
          // 将结果反馈给模型（如需多轮对话）
        }
        break
    }
  }
}
```

**可删除**:
- ❌ 整个类的复杂状态管理
- ❌ 消息历史管理
- ❌ API提供商切换
- ❌ 检查点系统
- ❌ 中断/恢复逻辑
- ❌ 错误恢复机制
- ❌ 重试逻辑
- ❌ 模式管理

**精简后**: ~150行 (从3514行减少95%！)

---

### ❌ 完全不需要移植的代码 (~85%)

#### 1. **文件操作工具** (全部跳过)
- `readFileTool.ts`
- `writeToFileTool.ts`
- `applyDiffTool.ts`
- `editFileTool.ts`
- `searchFilesTool.ts`
- `listFilesTool.ts`

#### 2. **代码相关工具** (全部跳过)
- `listCodeDefinitionNamesTool.ts`
- `codebaseSearchTool.ts`
- 代码索引服务

#### 3. **系统交互工具** (全部跳过)
- `executeCommandTool.ts`
- 终端集成
- Shell API

#### 4. **浏览器工具** (全部跳过)
- `browserActionTool.ts`
- 浏览器会话管理

#### 5. **MCP集成** (全部跳过)
- `McpHub.ts`
- `useMcpToolTool.ts`
- `accessMcpResourceTool.ts`
- MCP服务器管理

#### 6. **复杂功能** (全部跳过)
- 多模型提供商系统（50+个提供商）
- 模式系统（Architect, Coder, Debugger等）
- 权限和分组管理
- 检查点系统
- 工具重复检测
- YOLO模式守门员
- 自定义指令系统
- 本地规则系统
- Diff策略系统

#### 7. **VS Code集成** (全部跳过)
- 编辑器集成
- DiffViewProvider
- VS Code特定API

---

## 精简后的实现方案

### 新的目录结构

```
cutie/src/services/ai/
├── types/
│   └── index.ts                    # 核心类型定义 (~30行)
├── parser/
│   └── JsonToolCallParser.ts      # JSON工具调用解析 (~100行)
├── prompts/
│   ├── system.ts                   # 系统提示词 (~50行)
│   └── tools.ts                    # 工具描述 (~60行)
├── tools/
│   ├── createTask.ts               # 创建任务工具 (~30行)
│   ├── readTasks.ts                # 读取任务工具 (~40行)
│   ├── updateTask.ts               # 更新任务工具 (~30行)
│   ├── createSchedule.ts           # 创建日程工具 (~30行)
│   └── deleteTask.ts               # 删除任务工具 (~20行)
├── executor/
│   ├── ToolExecutor.ts             # 工具执行器 (~100行)
│   └── StreamProcessor.ts          # 流处理器 (~150行)
└── client/
    └── AiClient.ts                 # AI API客户端 (~100行)

总计: ~740行 (vs KiloCode的4000+行核心代码)
```

---

## 技术简化决策

### 1. **只支持 JSON 格式工具调用**
**原因**:
- OpenAI、Anthropic 都原生支持 JSON
- 无需复杂的 XML 解析
- 代码量减少 70%

**决策**: ✅ 只实现 JSON，删除 XML 支持

---

### 2. **只支持单一模型**
**原因**:
- 不需要 50+ 模型提供商
- 从一个模型开始够用
- 大幅简化代码

**决策**: ✅ 先只支持 OpenAI GPT-4o

---

### 3. **无需用户批准流程**（可选）
**原因**:
- 任务管理工具很安全
- 不涉及文件系统或命令执行
- 可以直接执行

**决策**: ✅ 初期可以跳过，后期可选添加

---

### 4. **工具串行执行，无并发**
**原因**:
- 任务管理工具不需要并发
- 串行执行足够快
- 大幅简化状态管理

**决策**: ✅ 串行执行

---

### 5. **无需检查点系统**
**原因**:
- 不涉及代码编辑
- 任务操作可以直接回滚
- 不需要复杂的版本控制

**决策**: ✅ 删除检查点系统

---

### 6. **简化错误处理**
**原因**:
- 不需要复杂的重试逻辑
- 失败直接返回错误信息
- 让模型决定如何处理

**决策**: ✅ 简单的 try-catch 即可

---

## 精简后的实施计划

### Phase 1: 核心解析和类型 (2天)
**工作量**: ~200行代码

1. **Day 1 上午**: 类型定义
   - `cutie/src/services/ai/types/index.ts`
   - `ToolUse`, `TextContent`, `ToolResult` 等

2. **Day 1 下午**: JSON解析器
   - `cutie/src/services/ai/parser/JsonToolCallParser.ts`
   - 只处理 OpenAI 格式

3. **Day 2 上午**: 提示词系统
   - `cutie/src/services/ai/prompts/system.ts`
   - `cutie/src/services/ai/prompts/tools.ts`

4. **Day 2 下午**: 单元测试
   - 验证解析器正确性
   - 测试提示词生成

---

### Phase 2: 工具实现 (3天)
**工作量**: ~150行代码

1. **Day 3**: 创建和读取工具
   - `createTask.ts` - 调用 Cutie 现有的 `/api/tasks` 端点
   - `readTasks.ts` - 调用 `/api/views/{context}/tasks` 端点

2. **Day 4**: 更新和删除工具
   - `updateTask.ts` - PATCH `/api/tasks/{id}`
   - `deleteTask.ts` - DELETE `/api/tasks/{id}`

3. **Day 5**: 日程工具
   - `createSchedule.ts` - POST `/api/tasks/{id}/schedules`
   - 集成测试

---

### Phase 3: 执行引擎 (3天)
**工作量**: ~250行代码

1. **Day 6**: 工具执行器
   - `ToolExecutor.ts`
   - 工具路由
   - 错误处理

2. **Day 7**: 流处理器
   - `StreamProcessor.ts`
   - SSE流解析
   - 工具调用触发

3. **Day 8**: AI客户端
   - `AiClient.ts`
   - 与后端通信
   - 流式响应处理

---

### Phase 4: 后端支持 (2天)
**工作量**: Rust ~200行

1. **Day 9**: 流式端点
   - `src-tauri/src/features/ai/endpoints/stream.rs`
   - SSE支持
   - 工具调用反馈

2. **Day 10**: 测试和调试
   - 端到端测试
   - 修复bug

---

### Phase 5: UI集成 (2天)
**工作量**: Vue ~200行

1. **Day 11**: 聊天组件
   - `src/components/ai/ChatPanel.vue`
   - 消息展示
   - 工具调用可视化

2. **Day 12**: 完善和优化
   - 样式调整
   - 用户体验优化

---

## 总计工作量估算

| 阶段 | 代码量 | 时间 |
|-----|-------|------|
| Phase 1: 解析和类型 | ~200行 TS | 2天 |
| Phase 2: 工具实现 | ~150行 TS | 3天 |
| Phase 3: 执行引擎 | ~250行 TS | 3天 |
| Phase 4: 后端支持 | ~200行 Rust | 2天 |
| Phase 5: UI集成 | ~200行 Vue | 2天 |
| **总计** | **~1000行** | **12天** |

**对比 KiloCode**: 减少了 **90%** 的代码量和 **70%** 的时间！

---

## 快速启动代码示例

### 1. 极简类型定义 (30行)
```typescript
// cutie/src/services/ai/types/index.ts

export interface ToolUse {
  type: "tool_use"
  name: string
  params: Record<string, any>
}

export interface TextContent {
  type: "text"
  content: string
}

export type MessageContent = TextContent | ToolUse

export interface ToolResult {
  success: boolean
  message: string
  data?: any
}

export interface ChatMessage {
  role: "user" | "assistant"
  content: string | MessageContent[]
}
```

---

### 2. 极简JSON解析器 (50行)
```typescript
// cutie/src/services/ai/parser/JsonToolCallParser.ts

export class JsonToolCallParser {
  private toolCallBuffer = new Map<string, any>()

  processToolCall(chunk: any): ToolUse | null {
    const id = chunk.id || chunk.index

    // 累加工具调用数据
    if (!this.toolCallBuffer.has(id)) {
      this.toolCallBuffer.set(id, {
        name: '',
        arguments: ''
      })
    }

    const buffer = this.toolCallBuffer.get(id)

    if (chunk.function?.name) {
      buffer.name = chunk.function.name
    }

    if (chunk.function?.arguments) {
      buffer.arguments += chunk.function.arguments
    }

    // 检查是否完整
    if (buffer.name && this.isCompleteJson(buffer.arguments)) {
      this.toolCallBuffer.delete(id)
      return {
        type: "tool_use",
        name: buffer.name,
        params: JSON.parse(buffer.arguments)
      }
    }

    return null
  }

  private isCompleteJson(str: string): boolean {
    try {
      JSON.parse(str)
      return true
    } catch {
      return false
    }
  }
}
```

---

### 3. 极简工具执行器 (60行)
```typescript
// cutie/src/services/ai/executor/ToolExecutor.ts

import { createTask, readTasks, updateTask, deleteTask, createSchedule } from '../tools'

export async function executeToolCall(toolUse: ToolUse): Promise<ToolResult> {
  try {
    switch (toolUse.name) {
      case "create_task":
        return await createTask(toolUse.params)

      case "read_tasks":
        return await readTasks(toolUse.params)

      case "update_task":
        return await updateTask(toolUse.params)

      case "create_schedule":
        return await createSchedule(toolUse.params)

      case "delete_task":
        return await deleteTask(toolUse.params)

      default:
        return {
          success: false,
          message: `未知工具: ${toolUse.name}`
        }
    }
  } catch (error) {
    return {
      success: false,
      message: `执行失败: ${error.message}`
    }
  }
}
```

---

### 4. 极简工具实现示例 (30行)
```typescript
// cutie/src/services/ai/tools/createTask.ts

import { apiBaseUrl } from '@/composables/useApiConfig'

export async function createTask(params: {
  title: string
  area_id?: string
  scheduled_date?: string
}): Promise<ToolResult> {
  const response = await fetch(`${apiBaseUrl.value}/tasks`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      title: params.title,
      area_id: params.area_id,
      scheduled_date: params.scheduled_date
    })
  })

  if (!response.ok) {
    return {
      success: false,
      message: `创建失败: ${await response.text()}`
    }
  }

  const data = await response.json()
  return {
    success: true,
    message: `成功创建任务: ${params.title}`,
    data: data.data
  }
}
```

---

## 与 Cutie 现有架构的集成

### 1. CPU Pipeline 集成
```typescript
// cutie/src/cpu/isa/ai-isa.ts

export const AI_ISA: InstructionSet = {
  'ai.send_message': async (ctx) => {
    const result = await sendAiMessage(ctx.payload.message)
    return result
  },

  'ai.tool_executed': async (ctx) => {
    // 通知前端工具执行完成
    const aiStore = useAiStore()
    aiStore.handleToolResult(ctx.payload)
  }
}
```

### 2. Pinia Store 集成
```typescript
// cutie/src/stores/ai/index.ts

export const useAiStore = defineStore('ai', () => {
  const messages = ref<ChatMessage[]>([])
  const isProcessing = ref(false)

  async function sendMessage(text: string) {
    messages.value.push({ role: 'user', content: text })
    isProcessing.value = true

    // 调用流式处理
    await processAiStream(
      text,
      (toolUse) => executeToolCall(toolUse),
      (chunk) => {
        // 更新UI
        const lastMsg = messages.value[messages.value.length - 1]
        if (lastMsg.role === 'assistant') {
          lastMsg.content += chunk
        } else {
          messages.value.push({ role: 'assistant', content: chunk })
        }
      }
    )

    isProcessing.value = false
  }

  return { messages, isProcessing, sendMessage }
})
```

---

## 总结：为什么可以这么简化？

### 原因分析

| 功能 | KiloCode需要 | Cutie需要 | 简化理由 |
|-----|-------------|----------|---------|
| **文件操作** | ✅ 核心功能 | ❌ 不需要 | 任务管理无需文件系统 |
| **代码编辑** | ✅ 核心功能 | ❌ 不需要 | 不涉及代码 |
| **终端执行** | ✅ 核心功能 | ❌ 不需要 | 任务管理无需Shell |
| **多模型** | ✅ 50+模型 | ✅ 1-2个 | 减少90%复杂度 |
| **XML解析** | ✅ 需要 | ❌ 不需要 | JSON足够 |
| **MCP集成** | ✅ 需要 | ❌ 不需要 | 无外部工具需求 |
| **权限系统** | ✅ 需要 | ⚠️ 简化 | 工具数量少 |
| **检查点** | ✅ 需要 | ❌ 不需要 | 无代码编辑 |

### 最终对比

```
KiloCode 完整实现:
  - 4000+ 行核心代码
  - 30+ 个工具
  - 支持文件/代码/终端/浏览器
  - 复杂的状态管理
  - 多模型切换
  - MCP集成

Cutie 精简实现:
  - ~1000 行核心代码 (75% 减少)
  - 5-8 个工具 (75% 减少)
  - 只支持任务管理
  - 简单的状态管理
  - 单一模型
  - 无外部集成

开发时间: 12 天 vs 2-3 个月
维护成本: 极低 vs 高
```

---

## 行动建议

### 立即开始 (Day 1)
1. ✅ 创建 `cutie/src/services/ai/` 目录
2. ✅ 复制上面的类型定义代码
3. ✅ 实现 `JsonToolCallParser`
4. ✅ 写单元测试验证解析

### 本周完成 (Week 1)
1. ✅ 完成 Phase 1 + Phase 2
2. ✅ 实现 5 个工具
3. ✅ 集成测试

### 下周完成 (Week 2)
1. ✅ 完成 Phase 3 + Phase 4 + Phase 5
2. ✅ 端到端测试
3. ✅ 发布第一个版本

**两周内完成整个AI功能！** 🚀