# Cutie API Reference

> 本文档由 `doc-composer` 工具自动生成，请勿手动编辑。
> 源文件位置：`src-tauri/src/features/*/endpoints/*.rs`

## Table of Contents

- [Ai](#ai)
  - [POST /api/ai/chat](#post-apiaichat)

---

## Ai

### POST /api/ai/chat

<details>
<summary>源文件: <code>src\features\ai\endpoints\chat.rs</code></summary>
</details>

## 1. 端点签名
POST /api/ai/chat

## 2. 预期行为简介

### 2.1 用户故事
> 作为用户，我想要与 AI 聊天，可以发送文本和图片，以便获得智能回复

### 2.2 核心业务逻辑
接收用户消息（支持文本和图片），调用 OpenAI API，返回 AI 回复和 token 使用情况

## 3. 输入输出规范

### 3.1 请求 (Request)
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

### 3.2 响应 (Responses)
**200 OK:**
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

## 4. 验证规则
- messages: 必须，非空数组
- messages[].text: 必须，非空字符串

## 5. 业务逻辑详解
1. 验证输入
2. 创建 OpenAI 客户端
3. 调用 OpenAI API
4. 返回结果

## 6. 边界情况
- messages 为空: 返回 422
- OpenAI API 超时: 返回 503
- OpenAI API 错误: 返回 502

## 7. 预期副作用
无数据库操作，无 SSE 事件

## 8. 契约
### 前置条件:
- 配置了有效的 OpenAI API Key

### 后置条件:
- 返回 AI 回复和 token 使用情况

### 不变量:
- 不修改任何持久化数据

