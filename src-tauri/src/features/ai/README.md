# AI 功能模块

## 概述

为 Cutie 提供基于 OpenAI 的 AI 聊天功能，支持文本和图片输入。

## 架构

```
features/ai/
├── mod.rs                    # 路由配置
├── endpoints/
│   └── chat.rs              # 聊天端点 (SFC 模式)
└── shared/
    ├── client.rs            # OpenAI 客户端封装
    └── config.rs            # 配置（API Key、Base URL 等）
```

## 配置

**⚠️ 重要：修改 API Key**

在使用前，请修改 `src-tauri/src/features/ai/shared/config.rs` 中的配置：

```rust
/// OpenAI API Key
/// ⚠️ 替换为你的实际 API Key
pub const OPENAI_API_KEY: &str = "sk-your-api-key-here";
```

其他配置项：

- `OPENAI_BASE_URL`: OpenAI API 地址（默认为官方地址）
- `DEFAULT_MODEL`: 默认使用的模型（默认为 `gpt-4o-mini`）
- `DEFAULT_MAX_TOKENS`: 默认最大 Token 数（默认 2000）
- `REQUEST_TIMEOUT_SECS`: 请求超时时间（默认 60 秒）

## API 端点

### POST /api/ai/chat

发送消息给 AI 并获取回复。

**请求体：**

```json
{
  "messages": [
    {
      "role": "user",
      "text": "帮我写一首诗",
      "images": [
        { "kind": "url", "data": "https://example.com/image.png" },
        { "kind": "base64", "data": "data:image/png;base64,..." }
      ]
    }
  ],
  "system": "你是一个乐于助人的助手",
  "max_tokens": 500
}
```

**响应：**

```json
{
  "data": {
    "reply": "这是 AI 的回复内容",
    "usage": {
      "prompt_tokens": 123,
      "completion_tokens": 45,
      "total_tokens": 168
    },
    "model": "gpt-4o-mini"
  },
  "timestamp": "2025-10-10T10:00:00Z",
  "request_id": null
}
```

## 前端使用

### 1. 打开 AI 聊天对话框

在 `HomeView` 右侧工具栏点击 "AI 助手" 按钮（闪耀图标），或直接使用组件：

```vue
<script setup>
import AiChatDialog from '@/components/parts/ai/AiChatDialog.vue'
import { ref } from 'vue'

const isAiChatOpen = ref(false)
</script>

<template>
  <button @click="isAiChatOpen = true">打开 AI 聊天</button>
  <AiChatDialog v-if="isAiChatOpen" @close="isAiChatOpen = false" />
</template>
```

### 2. 使用 AI Store

```typescript
import { useAiStore } from '@/stores/ai'

const aiStore = useAiStore()

// 发送消息
await aiStore.sendMessage({
  role: 'user',
  text: 'Hello, AI!',
  images: [], // 可选
})

// 访问消息历史
console.log(aiStore.allMessages)

// 清空历史
aiStore.clearMessages()
```

### 3. 直接调用 API

```typescript
import { sendChatMessage } from '@/services/ai'

const response = await sendChatMessage({
  messages: [
    {
      role: 'user',
      text: 'Hello, AI!',
      images: [],
    },
  ],
  system: '你是一个乐于助人的助手',
  max_tokens: 1000,
})

console.log(response.reply)
console.log(response.usage)
```

## 功能特性

- ✅ 文本聊天
- ✅ 图片上传（URL 和 Base64）
- ✅ 多模态输入支持
- ✅ Token 使用统计
- ✅ 错误处理和超时控制
- ✅ 美观的对话界面
- ⏳ 流式响应（待实现）
- ⏳ 会话持久化（待实现）
- ⏳ 多会话管理（待实现）

## 注意事项

1. **API Key 安全**: API Key 仅在后端保存，前端无法访问
2. **图片大小限制**: 前端限制单张图片最大 5MB
3. **请求超时**: 默认 60 秒，可在 `config.rs` 中修改
4. **会话管理**: 当前版本不保存会话历史，刷新页面后清空
5. **模型限制**: 使用 `gpt-4o-mini` 模型，支持文本和视觉输入

## 开发指南

### 添加新的 AI 功能

1. 在 `endpoints/` 目录下创建新的端点文件（遵循 SFC 模式）
2. 在 `mod.rs` 中注册路由
3. 使用 `OpenAIClient` 进行 API 调用
4. 返回统一的 `ApiResponse<T>` 格式

### 自定义模型或参数

修改 `client.rs` 中的 `chat` 方法，或在调用时传入自定义参数。

## 故障排查

### 1. 401 Unauthorized

- 检查 API Key 是否正确
- 确认 API Key 有效且有足够的额度

### 2. 超时错误

- 增加 `REQUEST_TIMEOUT_SECS` 的值
- 检查网络连接
- 如果使用代理，检查 `OPENAI_BASE_URL` 配置

### 3. 图片无法发送

- 确认图片格式正确（JPEG、PNG、GIF、WebP）
- 检查图片大小（建议小于 5MB）
- Base64 格式需包含完整的 Data URL 前缀

## 未来计划

- [ ] 支持流式响应（打字机效果）
- [ ] 会话持久化到本地数据库
- [ ] 多会话管理
- [ ] 支持更多模型选择
- [ ] 函数调用/工具集成
- [ ] 与任务系统的 RAG 集成

