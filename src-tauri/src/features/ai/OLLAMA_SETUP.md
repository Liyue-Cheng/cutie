# Ollama 本地部署指南

## 🚀 快速开始

### 1. 安装 Ollama

**Windows:**

```powershell
# 下载安装器
# https://ollama.com/download/windows

# 或使用 winget
winget install Ollama.Ollama
```

**macOS:**

```bash
brew install ollama
```

**Linux:**

```bash
curl -fsSL https://ollama.com/install.sh | sh
```

### 2. 下载并运行 Qwen2.5 模型

```bash
# 启动 Ollama 服务（如果未自动启动）
ollama serve

# 下载并运行 Qwen2.5-0.5B（最小最快版本）
ollama run qwen2.5:0.5b
```

### 3. 验证服务

打开浏览器访问：`http://localhost:11434/`

你应该看到：`Ollama is running`

### 4. 启动 Cutie

现在启动 Cutie，AI 功能会自动连接到本地 Ollama！

## 📊 可用模型

| 模型             | 参数量 | 显存需求 | 下载大小 | 速度   | 推荐用途           |
| ---------------- | ------ | -------- | -------- | ------ | ------------------ |
| **qwen2.5:0.5b** | 5亿    | ~2GB     | ~400MB   | ⚡⚡⚡ | 极速对话、分类     |
| **qwen2.5:1.5b** | 15亿   | ~4GB     | ~1GB     | ⚡⚡   | 通用对话           |
| **qwen2.5:3b**   | 30亿   | ~8GB     | ~2GB     | ⚡     | 复杂推理           |
| **qwen2.5:7b**   | 70亿   | ~16GB    | ~4.7GB   | 🐢     | 代码生成、深度分析 |

## 🎯 推荐配置

### 入门配置（集成显卡）

```bash
ollama run qwen2.5:0.5b
```

- **显存**: 2GB
- **响应**: 100-300ms
- **适合**: 快速对话、任务助手

### 标准配置（独显 6-8GB）

```bash
ollama run qwen2.5:1.5b
```

- **显存**: 4GB
- **响应**: 300-500ms
- **适合**: 日常使用、中文任务

### 高级配置（独显 12GB+）

```bash
ollama run qwen2.5:7b
```

- **显存**: 16GB
- **响应**: 1-2s
- **适合**: 代码生成、复杂推理

## 🔧 切换模型

编辑 `src-tauri/src/features/ai/shared/config.rs`:

```rust
// 改这一行
pub const DEFAULT_MODEL: &str = "qwen2.5:1.5b";  // 换成你想要的模型
```

然后重新编译运行。

## 🎨 其他推荐模型

### Llama 3.2（Meta 官方）

```bash
ollama run llama3.2:3b  # 30亿参数，英文强
```

### Phi-3（Microsoft）

```bash
ollama run phi3:mini    # 38亿参数，小而强
```

### Gemma 2（Google）

```bash
ollama run gemma2:2b    # 20亿参数，多任务
```

## 📈 性能测试

在我的机器（RTX 3060 12GB）上测试：

| 模型         | 首次响应 | 每秒生成     | 显存占用 |
| ------------ | -------- | ------------ | -------- |
| qwen2.5:0.5b | 150ms    | ~80 tokens/s | 1.8GB    |
| qwen2.5:1.5b | 280ms    | ~50 tokens/s | 3.2GB    |
| qwen2.5:7b   | 800ms    | ~25 tokens/s | 8.5GB    |

## 🐛 故障排查

### 1. 连接失败

```bash
# 检查 Ollama 是否运行
curl http://localhost:11434/

# 重启 Ollama
ollama serve
```

### 2. 模型未找到

```bash
# 查看已下载的模型
ollama list

# 重新拉取模型
ollama pull qwen2.5:0.5b
```

### 3. 响应慢

- 考虑换用更小的模型（0.5b）
- 检查显存是否不足（模型被移到 CPU）
- 关闭其他占用显存的程序

### 4. Windows 防火墙

如果连接被拒绝，允许 Ollama 通过防火墙：

```powershell
# 以管理员运行 PowerShell
New-NetFirewallRule -DisplayName "Ollama" -Direction Inbound -Action Allow -Protocol TCP -LocalPort 11434
```

## 📚 更多信息

- Ollama 官网: https://ollama.com/
- Qwen2.5 模型卡: https://ollama.com/library/qwen2.5
- Ollama API 文档: https://github.com/ollama/ollama/blob/main/docs/api.md

## 🎉 享受本地 AI！

现在你的 Cutie 完全本地运行，无需联网，无需 API Key，完全免费！⚡
