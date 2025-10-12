# Vue.js Final Archive

**归档日期**: 2025-10-12  
**Git 标签**: `vue-final-version`  
**项目状态**: ✅ 完整且稳定

---

## 📦 项目概述

这是 Cutie Dashboard 项目在迁移到 Svelte 之前的最终 Vue.js 版本存档。

### 技术栈
- **后端**: Rust + Tauri + Axum + SQLite + SQLx
- **前端**: Vue 3 + TypeScript + Pinia + Vite
- **架构**: 分层架构 + 功能切片

---

## 🎯 已完成的主要功能

### 1. 后端架构重构 (Backend Architecture Refactor)

#### 模块重组
- ✅ 将 `shared` 模块迁移到 `infra`（基础设施层）
- ✅ 实现分层架构：
  - `endpoints/`: HTTP 端点处理
  - `services/`: 业务逻辑服务
  - `repositories/`: 数据访问层
  - `validators/`: 数据验证层

#### 核心改进
- ✅ 统一错误处理 (`AppError`, `ValidationError`)
- ✅ 添加 Repository Traits (`Repository`, `QueryableRepository`, `BatchRepository`)
- ✅ 实现验证器单元测试（`TaskValidator`, `TimeBlockValidator`）
- ✅ 修复所有端点路由注册问题

#### 文件结构
```
src-tauri/src/
├── infra/                    # 基础设施层（原 shared）
│   ├── core/                 # 核心错误类型
│   ├── database/             # 数据库连接
│   ├── events/               # SSE 事件系统
│   ├── http/                 # HTTP 基础设施
│   ├── logging/              # 日志系统
│   └── ports/                # 外部依赖抽象
├── features/                 # 功能切片
│   ├── endpoints/            # HTTP 端点
│   │   ├── area/
│   │   ├── tasks/
│   │   ├── time_blocks/
│   │   ├── recurrences/
│   │   ├── templates/
│   │   ├── trash/
│   │   ├── view_preferences/
│   │   └── views/
│   └── shared/               # 业务共享层
│       ├── repositories/     # 数据仓库
│       ├── assemblers/       # 数据组装器
│       ├── services/         # 业务服务
│       └── validators/       # 验证器
└── entities/                 # 实体和 DTOs
```

### 2. 前端功能实现

#### Daily Planning View（每日计划视图）
- ✅ 6 列布局：
  1. **Staging** - 未安排任务 (28rem)
  2. **Today** - 今日任务 (28rem)
  3. **Calendar** - 日历视图 (28rem)
  4. **Tomorrow/Upcoming** - 明日/即将到期 (28rem, 可切换)
  5. **Toolbar** - 工具栏 (6rem)
  6. **Daily Rituals** - 每日仪式 (28rem)
- ✅ 响应式布局，无边框分隔
- ✅ 拖放支持（schedule 模式）
- ✅ 任务输入框
- ✅ 上下行布局（标题 + 内容）

#### Daily Rituals Panel（每日仪式面板）
- ✅ 添加/删除仪式项
- ✅ 勾选完成状态
- ✅ 拖拽排序（⋮⋮ 手柄）
- ✅ 进度指示器（X/Y 完成）
- ✅ 每日午夜自动重置
- ✅ localStorage 持久化存储

#### Task Editor Modal（任务编辑器）
- ✅ 修复循环任务检测（使用 `recurrence_id` 而非标题匹配）
- ✅ 正确显示循环规则
- ✅ 子任务支持

#### Template Editor（模板编辑器）
- ✅ 修复使用废弃字段的问题
- ✅ 统一使用 `title` 字段
- ✅ `glance_note_template` 和 `detail_note_template`

#### View Preferences（视图偏好）
- ✅ RESTful API 设计
- ✅ Context Key 规范实现
- ✅ 路径参数传递（`/:context_key`）

### 3. Bug 修复清单

#### 后端修复
- ✅ 修复所有端点 HTTP 方法（PUT → PATCH）
- ✅ 修复 `/api/tasks/:id/schedules/:date` 路由
- ✅ 修复 `/api/view-preferences/:context_key` 路由
- ✅ 修复 `/api/views/daily/:date` 路由
- ✅ 修复完成任务端点（现在会同时完成所有子任务）
- ✅ 修复 SQL 注入风险（参数化查询）
- ✅ 修复跨天时间块验证

#### 前端修复
- ✅ 修复 204 No Content 响应处理
- ✅ 修复任务编辑器循环检测逻辑
- ✅ 修复模板编辑器字段映射
- ✅ 修复视图偏好保存请求格式

---

## 📊 项目统计

### 代码规模
- **后端文件**: ~215 个 Rust 文件
- **前端文件**: ~128 个 TypeScript/Vue 文件
- **测试文件**: 单元测试 + 集成测试

### Git 统计
- **总提交数**: 33 commits ahead of origin/dev
- **最近功能**:
  - Daily Planning View
  - Daily Rituals Panel
  - Backend Architecture Refactor
  - Endpoint Route Fixes

### 最近 10 次提交
```
5acef14 refactor: simplify DailyRitualPanel UI and layout
5a1ac1e chore: remove test pages (AreaTestView and DebugView)
6e4c53f feat: add Daily Rituals panel to DailyPlanningView
d612e38 style: remove borders between columns in DailyPlanningView
63f9d62 fix: center align content in DailyPlanningView
b2afe37 fix: make calendar always visible and add Upcoming view option
27c02fd fix: use correct view context key format in DailyPlanningView
3968b4c fix: change kanban and calendar width to 28rem in DailyPlanningView
2740cd7 fix: match kanban and calendar widths to HomeView proportions
091212b fix: restore background color and border to DailyPlanningView
```

---

## 🔑 核心概念与规范

### Context Key 规范
```typescript
// 格式: {type}::{identifier}
'misc::staging'           // 未安排任务
'misc::all'               // 所有任务
'daily::2025-10-12'       // 每日看板
'area::{uuid}'            // 区域筛选
'project::{uuid}'         // 项目筛选
```

### API 端点规范
```
GET    /api/tasks                    # 列表
POST   /api/tasks                    # 创建
GET    /api/tasks/:id                # 详情
PATCH  /api/tasks/:id                # 更新（部分）
DELETE /api/tasks/:id                # 删除

POST   /api/tasks/:id/schedules      # 添加日程
PATCH  /api/tasks/:id/schedules/:date # 更新日程
DELETE /api/tasks/:id/schedules/:date # 删除日程

POST   /api/tasks/:id/completion     # 标记完成
DELETE /api/tasks/:id/completion     # 重新打开

GET    /api/views/daily/:date        # 每日视图
PUT    /api/view-preferences/:context_key  # 保存视图偏好
```

### 数据库 Schema
```sql
-- 主要表
tasks                   # 任务
task_schedules          # 任务日程
time_blocks             # 时间块
areas                   # 区域
task_recurrences        # 循环规则
templates               # 模板
view_preferences        # 视图偏好

-- 关系表
task_recurrence_links   # 任务-循环关联
task_time_block_links   # 任务-时间块关联
```

---

## 📚 重要文档

### 开发指南
- `references/COMPLETE_FEATURE_DEVELOPMENT_GUIDE.md` - 完整功能开发指南
- `references/VIEW_CONTEXT_KEY_SPEC.md` - 视图上下文键规范
- `references/DEVELOPMENT_GUIDELINES.md` - 开发规范
- `references/SFC_SPEC.md` - 单文件组件规范

### 架构文档
- `ai-doc/ARCHITECTURE.md` - 架构概览
- `ai-doc/BACKEND_DATA_CONSISTENCY_AUDIT_REPORT.md` - 后端数据一致性审计
- `ai-doc/FRONTEND_LOGGER_SYSTEM.md` - 前端日志系统

### 功能文档
- `docs/RECURRENCE_FEATURE_GUIDE.md` - 循环任务功能指南
- `notes/业务逻辑.md` - 业务逻辑说明

---

## 🚀 如何恢复此版本

### 1. 检出标签
```bash
git checkout vue-final-version
```

### 2. 安装依赖
```bash
# 前端依赖
pnpm install

# 后端依赖（自动通过 Cargo.toml）
```

### 3. 运行项目
```bash
# 开发模式
pnpm tauri dev

# 构建生产版本
pnpm tauri build
```

### 4. 数据库迁移
```bash
# SQLx 迁移位于 src-tauri/migrations/
# Tauri 启动时会自动运行
```

---

## 🎨 UI/UX 特性

### 设计风格
- **主题**: 简洁现代，Cutie 风格
- **颜色**: CSS 变量系统 (`--color-*`)
- **字体**: 1rem = 10px，使用相对单位
- **圆角**: 0.6rem ~ 0.8rem
- **间距**: 0.8rem ~ 1.5rem

### 交互特性
- ✅ 拖放支持（Task cards, Rituals）
- ✅ 实时更新（SSE）
- ✅ 响应式设计
- ✅ 键盘快捷键（Enter 提交等）
- ✅ Hover 效果
- ✅ 平滑动画

---

## 🔮 未来展望（Svelte 迁移）

### 迁移原因
- 更小的打包体积
- 更好的性能
- 更简洁的语法
- 更好的 TypeScript 支持

### 保留内容
- ✅ 后端架构（Rust + Tauri）
- ✅ 数据库 Schema
- ✅ API 端点设计
- ✅ 业务逻辑
- ✅ 设计风格

### 需要迁移
- ⚠️ Vue 组件 → Svelte 组件
- ⚠️ Pinia Store → Svelte Store
- ⚠️ Vue Router → SvelteKit Router
- ⚠️ Vue Composables → Svelte Actions

---

## 📞 联系与支持

### 代码仓库
- Git 标签: `vue-final-version`
- 分支: `dev`

### 相关资源
- Tauri 文档: https://tauri.app
- Vue 3 文档: https://vuejs.org
- Axum 文档: https://docs.rs/axum

---

## ✨ 致谢

感谢所有为此项目做出贡献的开发者！

此版本标志着 Vue.js 时代的圆满结束，期待 Svelte 版本的到来！🎉

---

**归档完成** - 2025-10-12

