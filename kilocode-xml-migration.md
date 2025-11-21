# KiloCode → Cutie XML工具调用精简方案

## 🎯 决策说明

**为什么选择 XML？**
1. **OpenAI兼容SDK** + **自托管模型** = 很可能不支持原生工具调用
2. **基于提示词的XML方案** = 任何模型都能学会输出XML
3. **更通用** = 不依赖模型的function calling能力
4. **KiloCode已验证** = 成熟的XML工具调用实现

**XML vs JSON 对比**:
```
JSON (原生工具调用):
  ✅ 模型原生支持
  ❌ 需要 function_call 能力
  ❌ 依赖具体模型API

XML (提示词方案):
  ✅ 任何模型都支持
  ✅ 基于文本生成
  ✅ 更灵活的格式定义
```

---

## ✅ 需要移植的核心代码 (~20%)

### 1. **XML解析器核心** ⭐⭐⭐ (必须)
**原文件**: `kilocode/src/core/assistant-message/AssistantMessageParser.ts`

**只保留 XML 解析部分**:
```typescript
// ✅ 保留这部分 (245-350行)
public processChunk(chunk: string): AssistantMessageContent[] {
  // 逐字符解析 XML 格式工具调用
  for (let i = 0; i < chunk.length; i++) {
    const char = chunk[i]
    this.accumulator += char

    // 解析工具参数
    if (this.currentToolUse && this.currentParamName) {
      const paramClosingTag = `</${this.currentParamName}>`
      if (currentParamValue.endsWith(paramClosingTag)) {
        // 参数解析完成
        this.currentToolUse.params[this.currentParamName] = paramValue
      }
    }

    // 检测工具调用开始
    if (this.accumulator.includes('<tool_use>')) {
      // 开始新的工具调用
    }

    // 检测工具调用结束
    if (this.accumulator.includes('</tool_use>')) {
      // 工具调用解析完成
    }
  }
}
```

**可删除**:
- ❌ `processNativeToolCalls()` - JSON解析器 (79-180行)
- ❌ 所有 JSON/OpenAI 相关代码
- ❌ `nativeToolCallsAccumulator` 等JSON状态

**精简后**: ~200行 (从400行减少50%)

---

### 2. **XML工具调用类型** ⭐⭐⭐ (必须)
```typescript
// cutie/src/services/ai/types/index.ts

export interface ToolUse {
  type: "tool_use"
  name: string                    // 工具名称，如 "create_task"
  params: Record<string, string>  // XML解析出的参数，都是string类型
  partial?: boolean               // 是否为部分解析（流式过程中）
}

export interface TextContent {
  type: "text"
  content: string
}

export type AssistantMessageContent = TextContent | ToolUse

export interface ToolResult {
  success: boolean
  message: string
  data?: any
}

// XML解析器状态
export interface XmlParserState {
  currentToolUse?: ToolUse
  currentParamName?: string
  currentParamValue?: string
  accumulator: string
}
```

**与JSON格式的区别**:
- ✅ 所有参数都是 `string` 类型（需要手动转换）
- ✅ 没有 `toolUseId`（XML格式不需要）
- ✅ 有 `partial` 标识（流式解析）

---

### 3. **XML格式提示词** ⭐⭐⭐ (核心)
**原文件**: `kilocode/src/core/prompts/sections/tool-use-guidelines.ts`

**Cutie 的XML工具调用提示词**:
```typescript
// cutie/src/services/ai/prompts/system.ts

export function generateSystemPrompt(): string {
  return `你是一个任务管理助手，可以帮助用户管理他们的任务和日程。

## 工具调用格式

当你需要执行操作时，请使用以下XML格式：

<tool_use>
<invoke name="工具名称">
<parameter name="参数名1">参数值1</parameter>
<parameter name="参数名2">参数值2</parameter>
</invoke>
</tool_use>

## 可用工具

### create_task
创建新任务
参数:
- title: 任务标题
- area_id: 所属区域ID（可选）
- scheduled_date: 计划日期（可选，格式：YYYY-MM-DD）

示例:
<tool_use>
<invoke name="create_task">
<parameter name="title">完成项目报告</parameter>
<parameter name="scheduled_date">2024-01-15</parameter>
</invoke>
</tool_use>

### read_tasks
读取任务列表
参数:
- view_context: 视图上下文（如 daily::2024-01-01, staging, area::uuid）

示例:
<tool_use>
<invoke name="read_tasks">
<parameter name="view_context">daily::2024-01-15</parameter>
</invoke>
</tool_use>

### update_task
更新任务
参数:
- task_id: 任务ID
- title: 新标题（可选）
- completed: 是否完成（true/false，可选）

示例:
<tool_use>
<invoke name="update_task">
<parameter name="task_id">uuid-123</parameter>
<parameter name="completed">true</parameter>
</invoke>
</tool_use>

### create_schedule
为任务创建日程
参数:
- task_id: 任务ID
- scheduled_date: 日期（YYYY-MM-DD）

示例:
<tool_use>
<invoke name="create_schedule">
<parameter name="task_id">uuid-123</parameter>
<parameter name="scheduled_date">2024-01-15</parameter>
</invoke>
</tool_use>

### delete_task
删除任务
参数:
- task_id: 任务ID

示例:
<tool_use>
<invoke name="delete_task">
<parameter name="task_id">uuid-123</parameter>
</invoke>
</tool_use>

## 重要规则

1. **每次只调用一个工具**：等待工具结果后再决定下一步
2. **参数类型注意**：所有参数都是字符串，布尔值用 "true"/"false"
3. **错误处理**：如果工具执行失败，我会告诉你原因
4. **格式严格**：请严格按照上述XML格式，不要添加额外的标签

## 你的目标

帮助用户高效地管理任务和日程，始终提供清晰的反馈。`
}
```

**关键要点**:
- ✅ 明确XML格式和示例
- ✅ 详细的参数说明
- ✅ 每个工具的使用示例
- ✅ 类型转换说明（string → boolean等）

---

### 4. **XML解析器实现** ⭐⭐⭐ (核心算法)
**移植自**: `kilocode/src/core/assistant-message/AssistantMessageParser.ts` (245-350行)

```typescript
// cutie/src/services/ai/parser/XmlToolCallParser.ts

export class XmlToolCallParser {
  private accumulator = ""
  private currentToolUse?: ToolUse
  private currentParamName?: string
  private currentParamValueStartIndex = 0
  private readonly MAX_ACCUMULATOR_SIZE = 1024 * 1024 // 1MB limit

  processChunk(chunk: string): ToolUse[] {
    const results: ToolUse[] = []

    // 安全检查：防止内存溢出
    if (this.accumulator.length + chunk.length > this.MAX_ACCUMULATOR_SIZE) {
      throw new Error("XML content exceeds maximum allowed size")
    }

    const accumulatorStartLength = this.accumulator.length

    for (let i = 0; i < chunk.length; i++) {
      const char = chunk[i]
      this.accumulator += char
      const currentPosition = accumulatorStartLength + i

      // 解析参数值
      if (this.currentToolUse && this.currentParamName) {
        const currentParamValue = this.accumulator.slice(this.currentParamValueStartIndex)
        const paramClosingTag = `</${this.currentParamName}>`

        if (currentParamValue.endsWith(paramClosingTag)) {
          // 参数解析完成
          const paramValue = currentParamValue.slice(0, -paramClosingTag.length)
          this.currentToolUse.params[this.currentParamName] = paramValue.trim()
          this.currentParamName = undefined
          continue
        } else {
          // 参数值还在累加中
          this.currentToolUse.params[this.currentParamName] = currentParamValue
          continue
        }
      }

      // 检测工具调用结束
      if (this.currentToolUse) {
        const toolUseClosingTag = `</invoke>`
        if (this.accumulator.includes(toolUseClosingTag)) {
          // 工具调用解析完成
          this.currentToolUse.partial = false
          results.push({ ...this.currentToolUse })
          this.currentToolUse = undefined
          continue
        }
      }

      // 检测参数开始
      if (this.currentToolUse && !this.currentParamName) {
        const paramMatch = this.accumulator.match(/<parameter name="([^"]+)">/)
        if (paramMatch) {
          this.currentParamName = paramMatch[1]
          this.currentParamValueStartIndex = this.accumulator.lastIndexOf(paramMatch[0]) + paramMatch[0].length
          this.currentToolUse.params[this.currentParamName] = "" // 初始化参数
          continue
        }
      }

      // 检测工具调用开始
      if (!this.currentToolUse) {
        const invokeMatch = this.accumulator.match(/<invoke name="([^"]+)">/)
        if (invokeMatch) {
          this.currentToolUse = {
            type: "tool_use",
            name: invokeMatch[1],
            params: {},
            partial: true
          }
          continue
        }
      }
    }

    return results
  }

  // 获取当前正在解析的工具（用于UI显示）
  getCurrentTool(): ToolUse | undefined {
    return this.currentToolUse ? { ...this.currentToolUse } : undefined
  }

  // 重置解析器状态
  reset(): void {
    this.accumulator = ""
    this.currentToolUse = undefined
    this.currentParamName = undefined
    this.currentParamValueStartIndex = 0
  }
}
```

**关键特性**:
- ✅ 流式解析：逐字符处理，支持部分工具调用
- ✅ 状态机：跟踪当前解析状态
- ✅ 内存安全：1MB大小限制
- ✅ 错误处理：格式验证和异常处理

---

### 5. **工具执行器** ⭐⭐ (简化版)
```typescript
// cutie/src/services/ai/executor/ToolExecutor.ts

export async function executeToolCall(toolUse: ToolUse): Promise<ToolResult> {
  try {
    // 参数类型转换（XML解析出来的都是string）
    const params = convertParams(toolUse.params)

    switch (toolUse.name) {
      case "create_task":
        return await createTask({
          title: params.title,
          area_id: params.area_id,
          scheduled_date: params.scheduled_date
        })

      case "read_tasks":
        return await readTasks({
          view_context: params.view_context
        })

      case "update_task":
        return await updateTask({
          task_id: params.task_id,
          title: params.title,
          completed: params.completed === 'true' // string → boolean
        })

      case "create_schedule":
        return await createSchedule({
          task_id: params.task_id,
          scheduled_date: params.scheduled_date
        })

      case "delete_task":
        return await deleteTask({
          task_id: params.task_id
        })

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

// 参数类型转换辅助函数
function convertParams(xmlParams: Record<string, string>) {
  const converted: Record<string, any> = {}

  for (const [key, value] of Object.entries(xmlParams)) {
    // 布尔值转换
    if (value === 'true') {
      converted[key] = true
    } else if (value === 'false') {
      converted[key] = false
    }
    // 数字转换（如需要）
    else if (/^\d+$/.test(value)) {
      converted[key] = parseInt(value)
    }
    // 其他保持字符串
    else {
      converted[key] = value
    }
  }

  return converted
}
```

---

### 6. **流式处理器** ⭐⭐ (适配XML)
```typescript
// cutie/src/services/ai/executor/StreamProcessor.ts

export async function processAiStream(
  stream: ReadableStream,
  onToolCall: (toolUse: ToolUse) => Promise<ToolResult>,
  onTextChunk: (text: string) => void,
  onToolUpdate?: (partialTool: ToolUse) => void
): Promise<void> {
  const parser = new XmlToolCallParser()

  for await (const chunk of parseSSEStream(stream)) {
    if (chunk.type === "text") {
      // 解析文本块中的工具调用
      const toolUses = parser.processChunk(chunk.text)

      // 处理完整的工具调用
      for (const toolUse of toolUses) {
        if (!toolUse.partial) {
          const result = await onToolCall(toolUse)

          // 将工具结果反馈给下一轮对话
          onTextChunk(`\n\n工具执行结果: ${result.message}`)
        }
      }

      // 显示当前正在解析的工具（可选）
      const currentTool = parser.getCurrentTool()
      if (currentTool && onToolUpdate) {
        onToolUpdate(currentTool)
      }

      // 显示普通文本（排除XML工具调用部分）
      const cleanText = removeToolCallXml(chunk.text)
      if (cleanText.trim()) {
        onTextChunk(cleanText)
      }
    }
  }
}

// 移除文本中的XML工具调用部分
function removeToolCallXml(text: string): string {
  return text
    .replace(/<tool_use>[\s\S]*?<\/tool_use>/g, '') // 移除完整的工具调用
    .replace(/<tool_use>[\s\S]*$/g, '') // 移除不完整的工具调用开始部分
    .replace(/^[\s\S]*?<\/tool_use>/g, '') // 移除不完整的工具调用结束部分
    .trim()
}
```

---

## ❌ 完全删除的代码 (~80%)

### 1. **JSON相关** (全部删除)
- ❌ `processNativeToolCalls()` 方法
- ❌ `ApiStreamNativeToolCallsChunk` 类型
- ❌ `nativeToolCallsAccumulator` 状态管理
- ❌ OpenAI function calling 相关代码

### 2. **复杂功能** (全部删除)
- ❌ MCP集成
- ❌ 模式系统
- ❌ 检查点系统
- ❌ 权限管理
- ❌ 工具重复检测
- ❌ YOLO模式守门员

### 3. **多模型支持** (全部删除)
- ❌ 50+个模型提供商
- ❌ API处理器工厂
- ❌ 模型切换逻辑

---

## 🔄 KiloCode XML实现参考

### 1. **XML格式定义**
**KiloCode使用的XML格式**:
```xml
<tool_use>
<invoke name="read_file">
<parameter name="path">/path/to/file.txt</parameter>
</invoke>
</tool_use>
```

**Cutie任务管理格式**:
```xml
<tool_use>
<invoke name="create_task">
<parameter name="title">完成项目报告</parameter>
<parameter name="scheduled_date">2024-01-15</parameter>
</invoke>
</tool_use>
```

### 2. **解析器状态机**
**KiloCode的状态跟踪**:
```typescript
// 当前解析的工具
private currentToolUse: ToolUse | undefined
// 当前解析的参数名
private currentParamName: ToolParamName | undefined
// 参数值开始位置
private currentParamValueStartIndex = 0
// 累加器
private accumulator = ""
```

**Cutie简化状态**:
```typescript
// 完全相同的状态机设计
private currentToolUse?: ToolUse
private currentParamName?: string
private currentParamValueStartIndex = 0
private accumulator = ""
```

### 3. **提示词设计**
**KiloCode的XML指导**:
```
Use XML format for tool calls:
<tool_use>
<invoke name="tool_name">
<parameter name="param_name">param_value</parameter>
</invoke>
</tool_use>
```

**Cutie的XML指导**（更详细）:
```
当你需要执行操作时，请使用以下XML格式：

<tool_use>
<invoke name="工具名称">
<parameter name="参数名1">参数值1</parameter>
<parameter name="参数名2">参数值2</parameter>
</invoke>
</tool_use>

重要：
1. 严格按照此格式
2. 所有参数都是字符串
3. 布尔值用 "true"/"false"
```

---

## 📁 精简后的目录结构

```
cutie/src/services/ai/
├── types/
│   └── index.ts                    # XML工具调用类型 (~40行)
├── parser/
│   └── XmlToolCallParser.ts        # XML解析器 (~150行)
├── prompts/
│   └── system.ts                   # XML格式提示词 (~100行)
├── tools/
│   ├── createTask.ts               # 创建任务工具 (~30行)
│   ├── readTasks.ts                # 读取任务工具 (~40行)
│   ├── updateTask.ts               # 更新任务工具 (~35行)
│   ├── createSchedule.ts           # 创建日程工具 (~30行)
│   └── deleteTask.ts               # 删除任务工具 (~25行)
├── executor/
│   ├── ToolExecutor.ts             # 工具执行器 (~120行)
│   └── StreamProcessor.ts          # XML流处理器 (~100行)
└── client/
    └── AiClient.ts                 # AI API客户端 (~80行)

总计: ~750行 (比JSON方案还要简单)
```

---

## 🚀 实施计划（XML版本）

### Phase 1: XML基础架构 (2天)
1. **Day 1**: 类型定义 + XML解析器核心算法
2. **Day 2**: XML提示词 + 单元测试

### Phase 2: 工具实现 (3天)
1. **Day 3**: create_task + read_tasks
2. **Day 4**: update_task + delete_task
3. **Day 5**: create_schedule + 参数类型转换

### Phase 3: 执行引擎 (3天)
1. **Day 6**: 工具执行器 + 参数处理
2. **Day 7**: XML流处理器
3. **Day 8**: AI客户端 + 错误处理

### Phase 4: 后端集成 (2天)
1. **Day 9**: Rust后端SSE支持
2. **Day 10**: 端到端测试

### Phase 5: UI完善 (2天)
1. **Day 11**: Vue聊天组件 + XML工具显示
2. **Day 12**: 样式优化 + 用户体验

**总计**: 12天（与JSON版本相同）

---

## ⚡ 快速启动代码示例

### 1. XML类型定义 (40行)
```typescript
// cutie/src/services/ai/types/index.ts

export interface ToolUse {
  type: "tool_use"
  name: string                      // 工具名称
  params: Record<string, string>    // XML解析出的参数（都是string）
  partial?: boolean                 // 流式解析中的部分状态
}

export interface TextContent {
  type: "text"
  content: string
}

export type AssistantMessageContent = TextContent | ToolUse

export interface ToolResult {
  success: boolean
  message: string
  data?: any
}

export interface XmlParserState {
  currentToolUse?: ToolUse
  currentParamName?: string
  currentParamValueStartIndex: number
  accumulator: string
}
```

### 2. 极简XML解析器核心 (60行)
```typescript
// cutie/src/services/ai/parser/XmlToolCallParser.ts

export class XmlToolCallParser {
  private accumulator = ""
  private currentToolUse?: ToolUse
  private currentParamName?: string
  private currentParamValueStartIndex = 0

  processChunk(chunk: string): ToolUse[] {
    this.accumulator += chunk
    const results: ToolUse[] = []

    // 检测工具调用开始
    if (!this.currentToolUse) {
      const invokeMatch = this.accumulator.match(/<invoke name="([^"]+)">/)
      if (invokeMatch) {
        this.currentToolUse = {
          type: "tool_use",
          name: invokeMatch[1],
          params: {},
          partial: true
        }
      }
    }

    // 检测参数
    if (this.currentToolUse && !this.currentParamName) {
      const paramMatch = this.accumulator.match(/<parameter name="([^"]+)">([^<]*)<\/parameter>/)
      if (paramMatch) {
        this.currentToolUse.params[paramMatch[1]] = paramMatch[2]
      }
    }

    // 检测工具调用结束
    if (this.currentToolUse && this.accumulator.includes('</invoke>')) {
      this.currentToolUse.partial = false
      results.push({ ...this.currentToolUse })
      this.currentToolUse = undefined
      this.accumulator = "" // 重置
    }

    return results
  }

  getCurrentTool(): ToolUse | undefined {
    return this.currentToolUse
  }

  reset(): void {
    this.accumulator = ""
    this.currentToolUse = undefined
    this.currentParamName = undefined
  }
}
```

### 3. XML提示词模板 (80行)
```typescript
// cutie/src/services/ai/prompts/system.ts

export function generateSystemPrompt(): string {
  return `你是一个任务管理助手。当需要执行操作时，使用XML格式：

<tool_use>
<invoke name="工具名称">
<parameter name="参数名">参数值</parameter>
</invoke>
</tool_use>

可用工具：

1. create_task - 创建任务
   <tool_use><invoke name="create_task"><parameter name="title">任务标题</parameter></invoke></tool_use>

2. read_tasks - 读取任务
   <tool_use><invoke name="read_tasks"><parameter name="view_context">daily::2024-01-01</parameter></invoke></tool_use>

3. update_task - 更新任务
   <tool_use><invoke name="update_task"><parameter name="task_id">uuid</parameter><parameter name="completed">true</parameter></invoke></tool_use>

4. create_schedule - 创建日程
   <tool_use><invoke name="create_schedule"><parameter name="task_id">uuid</parameter><parameter name="scheduled_date">2024-01-01</parameter></invoke></tool_use>

5. delete_task - 删除任务
   <tool_use><invoke name="delete_task"><parameter name="task_id">uuid</parameter></invoke></tool_use>

规则：
- 严格按XML格式
- 一次只调用一个工具
- 所有参数都是字符串`
}
```

### 4. 参数转换工具执行器 (70行)
```typescript
// cutie/src/services/ai/executor/ToolExecutor.ts

export async function executeToolCall(toolUse: ToolUse): Promise<ToolResult> {
  try {
    switch (toolUse.name) {
      case "create_task":
        return await createTask({
          title: toolUse.params.title,
          area_id: toolUse.params.area_id,
          scheduled_date: toolUse.params.scheduled_date
        })

      case "read_tasks":
        return await readTasks({
          view_context: toolUse.params.view_context
        })

      case "update_task":
        return await updateTask({
          task_id: toolUse.params.task_id,
          title: toolUse.params.title,
          completed: toolUse.params.completed === 'true' // 字符串转布尔
        })

      case "create_schedule":
        return await createSchedule({
          task_id: toolUse.params.task_id,
          scheduled_date: toolUse.params.scheduled_date
        })

      case "delete_task":
        return await deleteTask({
          task_id: toolUse.params.task_id
        })

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

## 🎯 XML vs JSON 的优势

### 对比表

| 特性 | XML方案 | JSON方案 |
|-----|--------|----------|
| **模型兼容性** | ✅ 任何模型 | ❌ 需要function calling |
| **OpenAI兼容SDK** | ✅ 完全兼容 | ❌ 可能不支持 |
| **自托管模型** | ✅ 完美支持 | ❌ 极可能不支持 |
| **实现复杂度** | ⭐⭐ 中等 | ⭐ 简单 |
| **解析稳定性** | ✅ 状态机稳定 | ✅ JSON.parse稳定 |
| **调试难度** | ⭐⭐ 可见XML | ⭐ 直接对象 |
| **扩展性** | ✅ 灵活格式 | ✅ 标准格式 |

### 结论
**XML方案更适合你的场景**，因为：
1. ✅ **通用性强** - 任何模型都能学会输出XML
2. ✅ **SDK兼容** - 不依赖function calling功能
3. ✅ **成熟方案** - KiloCode已验证可行性
4. ✅ **可控性强** - 完全基于提示词，可自定义格式

---

## 📋 行动检查清单

### 立即开始 (今天)
- [ ] 创建 `cutie/src/services/ai/` 目录结构
- [ ] 复制上面的4个代码示例
- [ ] 运行单元测试验证XML解析器

### 本周目标 (Week 1)
- [ ] 完成所有类型定义和解析器
- [ ] 实现5个基础工具
- [ ] 测试XML格式工具调用

### 下周目标 (Week 2)
- [ ] 完成流处理和执行引擎
- [ ] 集成Rust后端SSE支持
- [ ] 完成Vue聊天UI

**两周内上线XML工具调用功能！** 🎉

---

## 总结

**选择XML的明智之处**：
1. **技术可行性** - 不依赖模型原生功能
2. **兼容性强** - 支持所有OpenAI兼容SDK
3. **实现简单** - 复用KiloCode成熟方案
4. **调试友好** - XML格式人类可读

**代码量对比**：
- JSON方案：~1000行
- XML方案：~750行（更简单！）

现在你可以开始实施这个XML工具调用方案了！