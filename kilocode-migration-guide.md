# KiloCode → Cutie 代码移植指南

## 项目架构对比

### KiloCode (VS Code 扩展)
- **前端**: React + TypeScript + VS Code Webview
- **后端**: VS Code Extension API (Node.js)
- **通信**: postMessage (Extension ↔ Webview)
- **状态管理**: React Context
- **构建**: ESBuild

### Cutie (Tauri 桌面应用)
- **前端**: Vue 3 + TypeScript + CPU Pipeline
- **后端**: Rust + Tauri (Axum HTTP Server)
- **通信**: HTTP (REST + SSE)
- **状态管理**: Pinia (RTL硬件设计模式)
- **构建**: Vite

---

## 移植分类

### ✅ 可以直接移植的代码 (约30%)

#### 1. 核心算法和工具逻辑
**原因**: 纯TypeScript逻辑，不依赖特定框架

##### 1.1 流式解析器核心算法 ⭐⭐⭐
**源文件**: `kilocode/src/core/assistant-message/AssistantMessageParser.ts`

**可移植部分**:
```typescript
// ✅ XML解析状态机逻辑
public processChunk(chunk: string): AssistantMessageContent[] {
  // 逐字符解析逻辑
  // 工具名称/参数提取
  // 状态管理
}

// ✅ JSON工具调用累加逻辑
private nativeToolCallsAccumulator: Map<string, NativeToolCall> = new Map()
private processedNativeToolCallIds: Set<string> = new Set()
```

**移植到**: 新建 `cutie/src/services/ai/parser/`
```
cutie/src/services/ai/parser/
├── StreamParser.ts           // 流式解析器基类
├── XmlToolCallParser.ts      // XML格式工具调用解析
├── JsonToolCallParser.ts     // JSON格式工具调用解析
└── types.ts                  // 解析器类型定义
```

**改动点**:
- 移除 Anthropic SDK 依赖
- 将 `AssistantMessageContent` 类型改为 Cutie 的消息类型
- 保持核心解析算法不变

---

##### 1.2 工具执行工具函数 ⭐⭐
**源文件**: `kilocode/src/core/tools/*.ts`

**可移植工具实现**:
```typescript
// ✅ 通用工具处理逻辑
const askApproval = async (type: string, message: string) => { ... }
const handleError = async (action: string, error: Error) => { ... }
const pushToolResult = (content: ToolResponse) => { ... }
```

**移植到**: `cutie/src/services/ai/tools/`
```
cutie/src/services/ai/tools/
├── base/
│   ├── ToolExecutor.ts       // 工具执行基类
│   └── ToolContext.ts        // 工具执行上下文
├── file/
│   ├── readFileTool.ts       // ✅ 可直接移植
│   └── writeFileTool.ts      // ✅ 可直接移植
└── task/
    └── updateTodoTool.ts     // ✅ 可直接移植
```

**改动点**:
- 文件操作改用 Tauri API (`@tauri-apps/api/fs`)
- 移除 VS Code API 依赖
- 保持工具逻辑和接口签名

---

##### 1.3 工具描述生成逻辑 ⭐⭐
**源文件**: `kilocode/src/core/prompts/tools/*.ts`

**可移植部分**:
```typescript
// ✅ 工具提示词生成函数
export function getExecuteCommandDescription(args: ToolArgs): string { ... }
export function getReadFileDescription(args: ToolArgs): string { ... }
export function getWriteToFileDescription(args: ToolArgs): string { ... }
```

**移植到**: `cutie/src/services/ai/prompts/tools/`
```
cutie/src/services/ai/prompts/tools/
├── index.ts                  // 工具映射表
├── file-operations.ts        // 文件操作工具描述
├── task-operations.ts        // 任务操作工具描述
└── types.ts                  // 工具参数类型
```

**改动点**:
- 移除 VS Code 特定功能描述
- 调整为 Cutie 的任务管理场景
- 保持提示词生成模式

---

##### 1.4 系统提示词组装逻辑 ⭐⭐
**源文件**: `kilocode/src/core/prompts/system.ts`

**可移植部分**:
```typescript
// ✅ 提示词组装模式
const basePrompt = `${roleDefinition}
${markdownFormattingSection()}
${getSharedToolUseSection()}
${getToolDescriptionsForMode(...)}
${getToolUseGuidelinesSection(...)}
${getRulesSection(...)}
${getCustomInstructions(...)}
`
```

**移植到**: `cutie/src/services/ai/prompts/system.ts`

**改动点**:
- 移除 VS Code Extension Context 依赖
- 移除 MCP 相关部分（除非需要）
- 调整为 Cutie 的任务管理场景
- 添加 Cutie 特定的系统信息

---

#### 2. 类型定义和接口
**原因**: 纯类型定义，框架无关

##### 2.1 工具调用类型 ⭐⭐⭐
**源文件**: `kilocode/src/shared/tools.ts`

**可移植类型**:
```typescript
// ✅ 工具调用基础类型
export interface ToolUse {
  type: "tool_use"
  name: ToolName
  params: Record<string, string>
  partial?: boolean
  toolUseId?: string // OpenAI 格式专用
}

export interface TextContent {
  type: "text"
  content: string
}

export type AssistantMessageContent = TextContent | ToolUse
```

**移植到**: `cutie/src/types/ai.ts`

**改动点**: 无，直接复制

---

##### 2.2 流式数据块类型 ⭐⭐⭐
**源文件**: `kilocode/src/api/transform/stream.ts`

**可移植类型**:
```typescript
// ✅ 流式响应类型
export type ApiStreamChunk =
  | ApiStreamNativeToolCallsChunk  // JSON工具调用
  | ApiStreamTextChunk             // 文本
  | ApiStreamUsageChunk            // Token统计
  | ApiStreamError                 // 错误

export interface ApiStreamNativeToolCallsChunk {
  type: "native_tool_calls"
  toolCalls: Array<{
    index?: number
    id?: string
    function?: {
      name: string
      arguments: string
    }
  }>
}
```

**移植到**: `cutie/src/types/ai.ts`

**改动点**: 无，直接复制

---

#### 3. 纯工具函数
**原因**: 无副作用的工具函数

##### 3.1 重复检测器 ⭐⭐
**源文件**: `kilocode/src/core/tools/ToolRepetitionDetector.ts`

**可移植逻辑**:
```typescript
// ✅ 工具重复调用检测
export class ToolRepetitionDetector {
  check(toolUse: ToolUse): {
    allowExecution: boolean;
    askUser?: AskInfo
  }
}
```

**移植到**: `cutie/src/services/ai/tools/ToolRepetitionDetector.ts`

**改动点**: 无，直接复制

---

##### 3.2 工具验证逻辑 ⭐⭐
**源文件**: `kilocode/src/core/tools/validateToolUse.ts`

**可移植逻辑**:
```typescript
// ✅ 工具调用验证
export function validateToolUse(
  toolName: ToolName,
  mode: string,
  customModes: ModeConfig[],
  options: { apply_diff: boolean },
  params: Record<string, any>
): void {
  // 验证工具名称
  // 验证权限
  // 验证参数
}
```

**移植到**: `cutie/src/services/ai/tools/validateToolUse.ts`

**改动点**: 移除模式配置，简化为 Cutie 的权限检查

---

### 🔄 需要适配的代码 (约50%)

#### 1. 状态管理和数据流
**原因**: React Context → Pinia Stores

##### 1.1 任务执行引擎 ⭐⭐⭐
**源文件**: `kilocode/src/core/task/Task.ts` (3514行)

**核心逻辑可移植**:
```typescript
// ✅ 流式API调用循环
private async *readApiStreamIterator(): AsyncGenerator<ApiStreamChunk> {
  for await (const chunk of this.apiStream) {
    switch (chunk.type) {
      case "native_tool_calls": { ... }
      case "text": { ... }
    }
  }
}
```

**需要改造**:
```typescript
// ❌ React状态管理
this.assistantMessageContent = [...newContent]

// ✅ 改为 Pinia Store
const aiStore = useAiStore()
aiStore.updateAssistantMessages(newContent)

// ❌ VS Code API
await vscode.workspace.fs.readFile(...)

// ✅ 改为 Tauri API
await readTextFile(path, { dir: BaseDirectory.AppData })
```

**移植策略**:
1. 创建 `AiTaskExecutor` 类封装核心逻辑
2. 使用 Pinia Store 管理状态
3. 用 Cutie 的 CPU Pipeline 替代直接状态管理
4. 保持流式处理算法不变

**移植到**:
```
cutie/src/services/ai/
├── executor/
│   ├── AiTaskExecutor.ts     // 核心执行引擎（从Task.ts改造）
│   ├── StreamProcessor.ts    // 流式处理器
│   └── ToolOrchestrator.ts   // 工具编排器
└── state/
    └── ExecutionState.ts     // 执行状态管理
```

---

##### 1.2 工具执行编排 ⭐⭐⭐
**源文件**: `kilocode/src/core/assistant-message/presentAssistantMessage.ts`

**可移植逻辑**:
```typescript
// ✅ 工具路由逻辑
switch (block.name) {
  case "write_to_file": await writeToFileTool(...); break
  case "read_file": await readFileTool(...); break
  case "execute_command": await executeCommandTool(...); break
}
```

**需要改造**:
```typescript
// ❌ 直接函数调用
await cline.say("tool", toolMessage)

// ✅ 改为 CPU Pipeline 指令
pipeline.dispatch('ai.tool_status_update', {
  tool: block.name,
  status: 'executing'
})

// ❌ VS Code用户交互
const { response } = await cline.ask("tool", message)

// ✅ 改为 Cutie UI交互
const approval = await showToolApprovalDialog({
  tool: block.name,
  params: block.params
})
```

**移植到**: `cutie/src/services/ai/executor/ToolOrchestrator.ts`

---

#### 2. UI组件和渲染
**原因**: React → Vue 3

##### 2.1 工具调用渲染组件 ⭐⭐
**源文件**: `kilocode/webview-ui/src/components/chat/ChatRow.tsx`

**可移植逻辑**:
- 工具类型识别
- 工具状态展示
- 结果格式化

**需要重写**:
```tsx
// ❌ React组件
const ChatRow = memo((props: ChatRowProps) => {
  const tool = JSON.parse(message.text)
  return (
    <ToolUseBlock>
      <ToolUseBlockHeader>{tool.tool}</ToolUseBlockHeader>
    </ToolUseBlock>
  )
})

// ✅ 改为 Vue 组件
<template>
  <div class="tool-use-block">
    <div class="tool-use-header">{{ tool.tool }}</div>
  </div>
</template>

<script setup lang="ts">
const tool = computed(() => JSON.parse(props.message.text))
</script>
```

**移植到**: `cutie/src/components/ai/ChatMessage.vue`

---

##### 2.2 工具UI组件
**源文件**: `kilocode/webview-ui/src/components/common/ToolUseBlock.tsx`

**移植策略**: 完全重写为 Vue 组件

**移植到**:
```
cutie/src/components/ai/
├── ChatMessage.vue           // 聊天消息主组件
├── ToolUseBlock.vue          // 工具使用块
├── ToolApprovalDialog.vue    // 工具批准对话框
└── ToolResultDisplay.vue     // 工具结果展示
```

---

#### 3. 前后端通信
**原因**: postMessage → HTTP/SSE

##### 3.1 消息通信机制
**源文件**: `kilocode/src/core/webview/ClineProvider.ts`

**需要完全重写**:
```typescript
// ❌ VS Code postMessage
vscode.postMessage({
  type: "newTask",
  text: userMessage
})

// ✅ 改为 HTTP + SSE
// HTTP: 发送任务请求
await fetch('/api/ai/tasks', {
  method: 'POST',
  body: JSON.stringify({ message: userMessage })
})

// SSE: 接收流式更新
const eventSource = new EventSource('/api/ai/tasks/stream')
eventSource.onmessage = (event) => {
  const chunk = JSON.parse(event.data)
  aiStore.handleStreamChunk(chunk)
}
```

**移植到**: `cutie/src/services/ai/client/AiClient.ts`

---

### ❌ 需要完全重写的代码 (约20%)

#### 1. VS Code 特定功能
**原因**: 依赖 VS Code API，无法移植

##### 1.1 终端集成
**源文件**: `kilocode/src/integrations/terminal/Terminal.ts`
- **结论**: 完全重写
- **替代方案**: 使用 Tauri 的 Shell API (`@tauri-apps/api/shell`)

##### 1.2 编辑器集成
**源文件**: `kilocode/src/integrations/editor/DiffViewProvider.ts`
- **结论**: 不需要（Cutie 无代码编辑功能）

##### 1.3 MCP服务器管理
**源文件**: `kilocode/src/services/mcp/McpHub.ts`
- **结论**: 暂时跳过，除非 Cutie 需要外部工具集成

---

#### 2. 多模型API处理
**源文件**: `kilocode/src/api/providers/*.ts`

**可参考但需重写**:
- KiloCode: 50+个模型提供商，复杂的流处理
- Cutie: 从单一模型开始，逐步扩展

**重写方案**:
```
cutie/src/services/ai/providers/
├── BaseProvider.ts           // 基础提供商接口
├── OpenAiProvider.ts         // OpenAI实现
└── AnthropicProvider.ts      // Anthropic实现
```

---

#### 3. Rust后端实现
**Cutie需要新增Rust后端**:

```
cutie/src-tauri/src/features/ai/
├── endpoints/
│   ├── chat.rs               // AI聊天端点（已存在）
│   ├── tools.rs              // 工具调用端点（新增）
│   └── stream.rs             // SSE流式端点（新增）
├── services/
│   ├── model_client.rs       // 模型API客户端
│   ├── tool_executor.rs      // 工具执行器
│   └── stream_handler.rs     // 流式响应处理
└── shared/
    ├── types.rs              // AI相关类型
    └── prompts.rs            // 提示词模板
```

---

## 移植优先级建议

### 第一阶段：基础架构 (1-2周)
**目标**: 建立AI工具调用的基本框架

1. **类型定义** ✅ (1天)
   - 复制 `ToolUse`、`AssistantMessageContent` 等类型
   - 定义 Cutie 的 AI 消息类型

2. **流式解析器** ⭐⭐⭐ (3天)
   - 移植 `AssistantMessageParser.ts` 的核心逻辑
   - 实现 XML 和 JSON 两种格式支持
   - 单元测试验证

3. **系统提示词** ⭐⭐ (2天)
   - 移植提示词组装逻辑
   - 适配 Cutie 的任务管理场景
   - 定义工具描述模板

4. **Pinia Store** ⭐⭐ (3天)
   - 创建 `ai` store
   - 状态管理：消息历史、工具状态、执行状态
   - 与 CPU Pipeline 集成

---

### 第二阶段：工具系统 (2-3周)
**目标**: 实现基础工具调用

1. **工具执行框架** ⭐⭐⭐ (5天)
   - 移植工具编排逻辑
   - 实现工具路由器
   - 用户批准机制

2. **基础工具实现** ⭐⭐ (5天)
   - `read_task` - 读取任务信息
   - `create_task` - 创建任务
   - `update_task` - 更新任务
   - `create_schedule` - 创建日程

3. **工具验证和安全** ⭐⭐ (2天)
   - 移植 `ToolRepetitionDetector`
   - 移植 `validateToolUse`
   - 权限检查

---

### 第三阶段：执行引擎 (2-3周)
**目标**: 完整的AI任务执行循环

1. **任务执行器** ⭐⭐⭐ (7天)
   - 移植 `Task.ts` 的核心逻辑
   - 实现流式处理循环
   - 工具调用编排
   - 错误处理和重试

2. **Rust后端支持** ⭐⭐⭐ (5天)
   - 实现流式SSE端点
   - 工具调用API
   - 模型API客户端

3. **状态同步** ⭐⭐ (2天)
   - 前后端状态同步
   - SSE事件处理
   - CPU Pipeline指令集成

---

### 第四阶段：UI和交互 (1-2周)
**目标**: 用户友好的AI交互界面

1. **聊天UI** ⭐⭐ (4天)
   - 创建聊天消息组件
   - 工具调用可视化
   - 流式内容展示

2. **工具批准UI** ⭐⭐ (3天)
   - 工具批准对话框
   - 参数预览
   - 批准/拒绝操作

3. **结果展示** ⭐ (2天)
   - 工具执行结果格式化
   - 错误信息展示
   - Token使用统计

---

## 技术决策建议

### 1. 是否支持多模型？
**KiloCode**: 50+模型
**Cutie建议**: 先支持1-2个核心模型

- **阶段1**: OpenAI GPT-4o
- **阶段2**: 添加 Anthropic Claude
- **阶段3**: 按需扩展

---

### 2. 是否支持MCP？
**KiloCode**: 完整MCP集成
**Cutie建议**: 暂不支持，优先内置工具

**原因**:
- MCP复杂度高
- Cutie场景聚焦任务管理
- 可以后期扩展

---

### 3. 工具调用格式？
**KiloCode**: XML + JSON双格式
**Cutie建议**: 优先JSON，可选XML

**原因**:
- JSON更简洁，易于解析
- OpenAI原生支持JSON
- XML可作为备选方案

---

### 4. 流式处理方案？
**KiloCode**: 完整流式解析
**Cutie建议**: 保持流式处理

**原因**:
- 更好的用户体验
- 实时工具调用反馈
- 移植成本可控

---

## 代码复用率估算

| 类别 | 可复用程度 | 说明 |
|-----|-----------|------|
| **类型定义** | 95% | 几乎可直接复制 |
| **解析算法** | 90% | 核心算法可复用，接口需调整 |
| **工具逻辑** | 70% | API调用需改造 |
| **提示词系统** | 80% | 模板可复用，内容需调整 |
| **执行引擎** | 60% | 核心逻辑可复用，状态管理需重构 |
| **UI组件** | 10% | React→Vue需完全重写 |
| **通信机制** | 5% | postMessage→HTTP需完全重写 |
| **总体** | **~60%** | 核心逻辑可复用，接口需适配 |

---

## 实施路线图

### 阶段目标

```
Phase 1: 基础架构 (Week 1-2)
  ✓ 类型定义
  ✓ 流式解析器
  ✓ 提示词系统
  ✓ Pinia Store

Phase 2: 工具系统 (Week 3-5)
  ✓ 工具执行框架
  ✓ 5-10个基础工具
  ✓ 验证和安全

Phase 3: 执行引擎 (Week 6-8)
  ✓ 任务执行器
  ✓ Rust后端
  ✓ 状态同步

Phase 4: UI和交互 (Week 9-10)
  ✓ 聊天UI
  ✓ 工具批准UI
  ✓ 结果展示
```

---

## 快速开始检查清单

### 开始移植前准备
- [ ] 阅读完整的 `kilocode-reading-guide.md`
- [ ] 重点理解 Task.ts、AssistantMessageParser.ts、presentAssistantMessage.ts
- [ ] 在本地运行 KiloCode，观察工具调用流程
- [ ] 设计 Cutie 的 AI 功能需求文档

### 移植第一步
- [ ] 创建 `cutie/src/types/ai.ts`，复制基础类型
- [ ] 创建 `cutie/src/services/ai/` 目录结构
- [ ] 实现最简单的工具解析器（仅支持JSON格式）
- [ ] 编写单元测试验证解析器

### 集成到 Cutie
- [ ] 创建 `useAiStore()` Pinia Store
- [ ] 定义 AI 相关的 CPU Pipeline 指令集
- [ ] 实现 Rust 后端的 AI 聊天端点
- [ ] 创建简单的聊天UI测试

---

## 总结

**可直接移植** (30%):
- ✅ 类型定义
- ✅ 流式解析算法
- ✅ 工具描述生成
- ✅ 提示词组装逻辑
- ✅ 重复检测、验证等工具函数

**需要适配** (50%):
- 🔄 任务执行引擎 (React状态 → Pinia Store)
- 🔄 工具编排器 (函数调用 → CPU Pipeline)
- 🔄 工具实现 (VS Code API → Tauri API)
- 🔄 UI组件 (React → Vue)

**需要重写** (20%):
- ❌ 前后端通信 (postMessage → HTTP/SSE)
- ❌ Rust后端实现 (全新)
- ❌ VS Code特定功能
- ❌ 多模型提供商（简化版）

**建议策略**:
1. 从类型定义和解析器开始
2. 逐步实现工具系统
3. 最后完成UI和交互
4. 采用迭代式开发，先实现MVP，后优化

祝移植顺利！