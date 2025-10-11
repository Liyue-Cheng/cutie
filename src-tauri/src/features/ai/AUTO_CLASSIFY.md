# AI 自动分类功能

## 🎯 功能概述

当创建任务时，如果满足以下条件，系统会**异步**调用 AI 自动为任务选择合适的 Area（分类）：

1. ✅ **未指定 area_id** - 用户没有手动选择分类
2. ✅ **不是从模板创建** - 确保模板创建的任务不被自动分类

## ⚡ 工作流程

```
用户创建任务 (无 area_id)
    ↓
HTTP 请求立即返回 (不阻塞) ← ✅ 用户立即得到响应
    ↓
[后台异步任务开始]
    ↓
1. 获取所有可用的 Area 列表
    ↓
2. 调用 AI (超时 5 秒)
   - 发送任务标题
   - 发送可用的 Area 列表
   - AI 分析并选择最合适的
    ↓
3. 更新任务的 area_id
    ↓
4. 前端通过 SSE 自动更新（如果有监听）
```

## 📝 AI Prompt

系统使用以下 prompt（写死在代码中）：

```
你是一个任务分类助手。请根据任务标题，从给定的分类列表中选择最合适的一个。

任务标题: "{title}"

可用分类:
- {area_name} (ID: {uuid})
- ...

请仔细分析任务标题，选择最恰当的分类。如果没有合适的分类，输出 <none>。

只输出分类的 ID（UUID 格式），不要有任何其他内容。
```

## 🎛️ 配置参数

| 参数         | 值       | 说明                               |
| ------------ | -------- | ---------------------------------- |
| **超时时间** | 5 秒     | 超时后放弃分类，不影响任务创建     |
| **重试次数** | 0        | 超时或失败不重试                   |
| **阻塞行为** | 否       | 异步执行，不阻塞 HTTP 响应         |
| **失败策略** | 静默失败 | 分类失败不影响任务创建，只记录日志 |

## 📊 示例场景

### 场景 1：成功分类

```
用户创建: "买牛奶和鸡蛋"
可用 Areas:
  - 工作 (UUID-1)
  - 生活 (UUID-2)
  - 学习 (UUID-3)

AI 分析: "买牛奶和鸡蛋" → 日常生活相关
AI 返回: UUID-2
结果: 任务自动归类到 "生活"
```

### 场景 2：无合适分类

```
用户创建: "研究量子物理"
可用 Areas:
  - 工作 (UUID-1)
  - 生活 (UUID-2)

AI 分析: 都不太合适
AI 返回: <none>
结果: 任务保持未分类状态
```

### 场景 3：超时

```
用户创建: "准备周报"
AI 请求: 5 秒后超时
结果: 任务保持未分类状态，记录警告日志
```

## 🔍 日志追踪

所有 AI 分类操作都有详细的日志：

```rust
// 开始分类
tracing::info!(
    target: "SERVICE:TASKS:auto_classify",
    task_id = %task_id,
    title = %task_title,
    "Starting AI classification"
);

// 成功分类
tracing::info!(
    target: "SERVICE:TASKS:auto_classify",
    task_id = %task_id,
    area_id = %area_id,
    "Task area updated successfully"
);

// 超时
tracing::warn!(
    target: "AI:CLASSIFY",
    "AI classification timeout (5s)"
);
```

可以通过日志搜索 `auto_classify` 或 `AI:CLASSIFY` 查看所有分类活动。

## 🎨 自定义 Prompt

如需修改 prompt，编辑：

`src-tauri/src/features/ai/shared/auto_classify.rs`

```rust
let prompt = format!(
    r#"你是一个任务分类助手。请根据任务标题，从给定的分类列表中选择最合适的一个。

任务标题: "{}"

可用分类:
{}

// 在这里修改 prompt
"#,
    task_title, areas_list
);
```

## ⚙️ 调整超时时间

编辑 `src-tauri/src/features/ai/shared/auto_classify.rs`:

```rust
// 设置 5 秒超时 ← 修改这里
let result = tokio::time::timeout(
    std::time::Duration::from_secs(5),  // 改成你想要的秒数
    client.chat(messages, None, Some(100)),
)
.await;
```

## 🚨 注意事项

1. **异步执行** - 不会阻塞任务创建，用户体验流畅
2. **静默失败** - AI 分类失败不影响核心功能
3. **超时保护** - 5 秒超时确保不会无限等待
4. **模板任务** - 从模板创建的任务不会被自动分类（保持模板预设）
5. **手动分类** - 如果用户手动选择了 Area，不会触发自动分类

## 🔮 未来增强

- [ ] 支持自定义 prompt（通过配置文件）
- [ ] 支持多语言 prompt
- [ ] 学习用户习惯（记录历史分类决策）
- [ ] 支持更多上下文（任务描述、子任务等）
- [ ] 提供 UI 开关（允许用户禁用自动分类）

## 📈 性能影响

- **HTTP 响应时间**: **0ms** （异步执行，不阻塞）
- **AI 分类时间**: 通常 500-2000ms（取决于模型和网络）
- **数据库开销**: 2 次查询（获取 Areas + 更新任务）
- **内存开销**: 极小（仅一个后台任务）

## ✅ 测试建议

1. **正常分类**:

   ```bash
   # 创建没有 area_id 的任务
   POST /api/tasks
   { "title": "买菜做饭" }

   # 查看日志，应该能看到 AI 分类过程
   # 等待 1-2 秒后，任务应该已经自动分类
   ```

2. **超时测试**:

   ```bash
   # 临时断网或使用无效的 API 地址
   # 创建任务，5 秒后应该看到超时日志
   ```

3. **手动分类**:

   ```bash
   # 创建带 area_id 的任务
   POST /api/tasks
   { "title": "买菜做饭", "area_id": "..." }

   # 不应该触发 AI 分类
   ```

## 🎉 享受智能分类！

现在你的任务会自动归类到合适的 Area，无需手动选择！✨
