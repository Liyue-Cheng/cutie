# Cutie 添加功能说明书

> 新维护者快速上手指南

---

## 🎯 添加新功能的完整流程

本文档以**添加一个新的 Tag 功能**为例，说明完整步骤。

---

## 📚 必读文档（按顺序）

1. **CUTIE_CONCEPTS.md** - 理解 Cutie 的设计哲学
2. **ARCHITECTURE.md** - 理解系统架构
3. **SFC_SPEC.md** - 学习后端开发规范
4. **PINIA_BEST_PRACTICES.md** - 学习前端状态管理

---

## 🔧 后端开发流程

### Step 1: 设计数据模型

**查看：** `src-tauri/migrations/xxx.sql`

**添加表：**
```sql
CREATE TABLE tags (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    color TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE
);
```

**注意：**
- 所有表名使用复数
- 时间字段使用 TEXT (ISO 8601 UTC)
- 添加 is_deleted 用于软删除
- 添加必要的索引

### Step 2: 创建实体模型

**文件：** `src-tauri/src/entities/tag/model.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

#[derive(Debug, FromRow)]
pub struct TagRow {
    pub id: String,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

impl TryFrom<TagRow> for Tag { ... }
```

### Step 3: 创建 DTOs

**文件：** `src-tauri/src/entities/tag/request_dtos.rs`

```rust
#[derive(Debug, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub color: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTagRequest {
    pub name: Option<String>,
    pub color: Option<String>,
}
```

**文件：** `src-tauri/src/entities/tag/response_dtos.rs`

```rust
#[derive(Debug, Serialize)]
pub struct TagDto {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**文件：** `src-tauri/src/entities/tag/mod.rs`

```rust
pub mod model;
pub mod request_dtos;
pub mod response_dtos;

pub use model::*;
pub use request_dtos::*;
pub use response_dtos::*;
```

### Step 4: 创建装配器（如果需要）

**文件：** `src-tauri/src/features/tags/shared/assembler.rs`

```rust
pub struct TagAssembler;

impl TagAssembler {
    pub fn tag_to_dto(tag: &Tag) -> TagDto {
        TagDto {
            id: tag.id,
            name: tag.name.clone(),
            color: tag.color.clone(),
            created_at: tag.created_at,
            updated_at: tag.updated_at,
        }
    }
}
```

### Step 5: 创建端点（SFC 模式）

**参考：** `src-tauri/src/features/SFC_SPEC.md`

**文件：** `src-tauri/src/features/tags/endpoints/create_tag.rs`

**模板结构：**
```rust
/// 创建 Tag API - 单文件组件

// 导入
use axum::{...};
use crate::{entities::{...}, ...};

// ==================== 文档层 ====================
/*
CABC for `create_tag`
... 按照 CABC 2.0 格式编写
*/

// ==================== HTTP 处理器 ====================
pub async fn handle(...) -> Response {
    match logic::execute(...).await {
        Ok(data) => created_response(data).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    pub fn validate_request(...) -> AppResult<()> {
        // 验证逻辑
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    pub async fn execute(...) -> AppResult<TagDto> {
        // 1. 验证
        // 2. 开启事务
        // 3. 生成 UUID (id_generator.new_uuid())
        // 4. 获取时间 (clock.now_utc())
        // 5. 创建实体
        // 6. 插入数据库
        // 7. 提交事务
        // 8. 返回 DTO
    }
}

// ==================== 数据访问层 ====================
mod database {
    pub async fn insert_tag_in_tx(...) -> AppResult<()> {
        // SQL 插入
    }
}
```

**关键检查：**
- [ ] 使用 `id_generator().new_uuid()` 生成 ID
- [ ] 使用 `clock().now_utc()` 获取时间
- [ ] 所有写操作在事务中
- [ ] 查看 Schema 确认表名

### Step 6: 注册路由

**文件：** `src-tauri/src/features/tags/mod.rs`

```rust
use axum::{routing::{get, post, patch, delete}, Router};
use crate::startup::AppState;

mod endpoints {
    pub mod create_tag;
    pub mod list_tags;
    pub mod update_tag;
    pub mod delete_tag;
}

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(endpoints::list_tags::handle).post(endpoints::create_tag::handle))
        .route("/:id", 
            get(endpoints::get_tag::handle)
                .patch(endpoints::update_tag::handle)
                .delete(endpoints::delete_tag::handle)
        )
}
```

**文件：** `src-tauri/src/features/mod.rs`

```rust
pub mod tags;  // ← 添加

pub fn create_api_router() -> Router<AppState> {
    Router::new()
        .nest("/tags", tags::create_routes())  // ← 添加
        // ... 其他路由
}
```

### Step 7: 编写 API 文档

**文件：** `src-tauri/src/features/tags/API_SPEC.md`

**参考：** 其他功能的 API_SPEC.md

**按照 CABC 2.0 格式**包含：
- 端点清单
- 每个端点的8个章节

---

## 💻 前端开发流程

### Step 1: 创建前端 DTO

**文件：** `src/types/dtos.ts`

```typescript
export interface Tag {
  id: string
  name: string
  color: string
  created_at: string
  updated_at: string
}
```

### Step 2: 创建 Pinia Store

**文件：** `src/stores/tag.ts`

**模板：**
```typescript
import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { waitForApiReady } from '@/composables/useApiConfig'

export const useTagStore = defineStore('tag', () => {
  // State
  const tags = ref(new Map<string, Tag>())
  
  // Getters
  const allTags = computed(() => Array.from(tags.value.values()))
  
  // Actions
  async function fetchTags() {
    const response = await fetch(`${apiBaseUrl}/tags`)
    const result = await response.json()
    const tagList: Tag[] = result.data  // ← 提取 data
    // 更新 state
  }
  
  return { tags, allTags, fetchTags, ... }
})
```

**关键规则：**
- State 用 Map 存储
- 操作时创建新 Map
- 提取 `result.data`

### Step 3: 创建 UI 组件

**示例：TagManager 组件**

```vue
<script setup lang="ts">
import { onMounted } from 'vue'
import { useTagStore } from '@/stores/tag'

const tagStore = useTagStore()

onMounted(async () => {
  await tagStore.fetchTags()
})

async function handleCreate() {
  await tagStore.createTag({ name, color })
}
</script>

<template>
  <div>
    <!-- CRUD UI -->
  </div>
</template>
```

### Step 4: 集成到路由

**文件：** `src/router/index.ts`

```typescript
{
  path: 'tags',
  name: 'tags',
  component: () => import('../views/TagsView.vue'),
}
```

### Step 5: 添加导航链接

**文件：** `src/views/MainLayout.vue`

```vue
<li @click="$router.push('/tags')">
  <CuteIcon name="Tag" :size="16" />
  <span>Tags</span>
</li>
```

---

## 🔄 完整示例：添加 Tag 功能

### 后端文件清单（9个）

```
src-tauri/src/
├── migrations/xxx.sql                           # 添加 tags 表
├── entities/tag/
│   ├── model.rs                                 # Tag, TagRow
│   ├── request_dtos.rs                          # CreateTagRequest, UpdateTagRequest
│   ├── response_dtos.rs                         # TagDto
│   └── mod.rs                                   # 导出
├── features/tags/
│   ├── endpoints/
│   │   ├── create_tag.rs                        # POST /tags
│   │   ├── list_tags.rs                         # GET /tags
│   │   ├── update_tag.rs                        # PATCH /tags/:id
│   │   └── delete_tag.rs                        # DELETE /tags/:id
│   ├── mod.rs                                   # 路由注册
│   └── API_SPEC.md                              # 文档
└── features/mod.rs                              # 注册 tags 模块
```

### 前端文件清单（4-6个）

```
src/
├── types/dtos.ts                                # interface Tag
├── stores/tag.ts                                # useTagStore
├── components/parts/
│   ├── TagManager.vue                           # 管理器弹窗
│   └── TagSelector.vue                          # 标签选择器
├── views/
│   └── TagTestView.vue                          # 测试页面
└── router/index.ts                              # 路由配置
```

---

## 📝 开发检查清单

### 后端开发

- [ ] 查看并理解 Schema
- [ ] 创建实体 model
- [ ] 创建 request/response DTOs
- [ ] 创建装配器（如果需要）
- [ ] 实现所有端点（遵循 SFC 模式）
- [ ] 注册路由
- [ ] 编写 API_SPEC.md
- [ ] 运行 `cargo check`
- [ ] 测试所有端点

### 前端开发

- [ ] 创建 DTO interface
- [ ] 创建 Pinia Store
- [ ] 创建 UI 组件
- [ ] 添加路由
- [ ] 添加导航链接
- [ ] 检查 linter 错误
- [ ] 测试响应式更新
- [ ] 测试完整工作流

---

## 🚀 快速启动

### 我想添加一个新的 XXX 功能

**1. 找到参考实现：**
- 简单功能 → 参考 Area
- 复杂功能 → 参考 Task
- 视图功能 → 参考 Views

**2. 复制并修改：**
- 后端：复制整个 `features/areas/` 目录
- 前端：复制 `stores/area.ts`
- 全局替换：`Area` → `XXX`, `area` → `xxx`

**3. 根据需求调整：**
- 修改 Schema
- 修改业务逻辑
- 修改 UI

**4. 遵循规范：**
- 后端：`SFC_SPEC.md` 的所有检查清单
- 前端：`PINIA_BEST_PRACTICES.md` 的模式

**5. 编写文档：**
- `features/xxx/API_SPEC.md`（CABC 2.0 格式）

---

## 📍 关键文件位置

### 后端核心文件

| 文件                          | 用途                   |
| ----------------------------- | ---------------------- |
| `migrations/xxx.sql`          | 数据库 Schema（必看！）|
| `features/*/endpoints/*.rs`   | API 端点实现           |
| `features/*/shared/assembler.rs` | DTO 转换逻辑       |
| `features/*/mod.rs`           | 路由注册               |
| `features/mod.rs`             | 功能模块注册           |

### 前端核心文件

| 文件                     | 用途               |
| ------------------------ | ------------------ |
| `src/types/dtos.ts`      | DTO 定义           |
| `src/stores/*.ts`        | Pinia stores       |
| `src/views/*.vue`        | 页面组件           |
| `src/components/**/*.vue`| UI 组件            |
| `src/router/index.ts`    | 路由配置           |

---

## 🎓 学习路径

### 第1天：理解架构

- [ ] 阅读 CUTIE_CONCEPTS.md
- [ ] 阅读 ARCHITECTURE.md
- [ ] 理解数据流
- [ ] 查看 Schema

### 第2天：学习后端

- [ ] 阅读 SFC_SPEC.md
- [ ] 查看现有端点实现
- [ ] 理解单文件组件模式
- [ ] 学习装配器模式

### 第3天：学习前端

- [ ] 阅读 PINIA_BEST_PRACTICES.md
- [ ] 查看现有 Store 实现
- [ ] 理解响应式更新链路
- [ ] 查看组件如何使用 Store

### 第4天：实践

- [ ] 修改一个现有端点
- [ ] 添加一个新字段
- [ ] 测试完整流程
- [ ] 理解数据流

### 第5天：独立开发

- [ ] 从头实现一个简单功能
- [ ] 遵循所有检查清单
- [ ] 编写文档
- [ ] Code Review

---

## 💡 常见问题

### Q: 我应该从哪个文件开始看代码？

**A: 按照这个顺序：**
1. `migrations/xxx.sql` - 理解数据结构
2. `entities/task/model.rs` - 理解实体
3. `features/tasks/endpoints/create_task.rs` - 理解端点
4. `src/types/dtos.ts` - 理解前端数据
5. `src/stores/task.ts` - 理解状态管理
6. `src/components/parts/kanban/KanbanTaskCard.vue` - 理解 UI

### Q: 单文件组件的模板在哪里？

**A: 参考现有实现：**
- 简单 CRUD：`features/areas/endpoints/create_area.rs`
- 复杂逻辑：`features/tasks/endpoints/complete_task.rs`
- 拖动专用：`features/time_blocks/endpoints/create_from_task.rs`

**文档：** `SFC_SPEC.md`

### Q: 如何确保数据一致性？

**A: 遵循这些原则：**
1. 后端返回真实状态（查询 DB，不用默认值）
2. 后端返回完整数据（包含受影响的关联对象）
3. 前端提取 `result.data`
4. 前端创建新对象触发响应式

**参考：** `SFC_SPEC.md` 4.7 数据真实性原则

### Q: 如何调试响应式更新问题？

**A: 检查链路：**
1. API 返回了什么？（Network tab）
2. Store 更新了吗？（`$pinia.state.value.xxx`）
3. Getter 重新计算了吗？（添加 console.log）
4. Computed 触发了吗？（添加 console.log）

**工具：** 任务编辑器底部的调试数据展示

### Q: 我改了 Schema，需要改哪些文件？

**A: 查看：** `DATA_SCHEMA_COUPLING.md`

---

## 🛠️ 实用命令

### 后端开发

```bash
# 检查编译
cd src-tauri && cargo check

# 运行应用
cargo tauri dev

# 查找引用
grep -rn "function_name" src-tauri/src
```

### 前端开发

```bash
# 类型检查
npm run type-check

# Linter
npm run lint

# 开发服务器
npm run dev
```

### 数据库

```bash
# 删除旧数据库（重新运行 migrations）
rm src-tauri/*.db*

# 查看 Schema
cat src-tauri/migrations/xxx.sql
```

---

## 📖 代码风格

### 后端

- 遵循 Rust 标准
- 使用 `rustfmt`
- 文档注释必须完整
- 错误处理使用 `?`

### 前端

- 遵循 Vue 3 Composition API
- TypeScript strict mode
- 组件使用 `<script setup>`
- 样式使用 scoped

---

## 🎯 成功标准

**一个功能算完成当：**
- ✅ 所有端点实现且编译通过
- ✅ 所有端点有 CABC 2.0 文档
- ✅ Pinia Store 实现
- ✅ UI 组件实现
- ✅ 路由和导航配置
- ✅ 无 linter 错误
- ✅ 响应式更新正常
- ✅ 手动测试通过

---

## 🆘 遇到问题？

1. **查文档：** 相关的 `*_SPEC.md` 或 `*.md`
2. **看代码：** 参考类似功能的实现
3. **检查清单：** 确保没有遗漏步骤
4. **查 Schema：** 确认数据库结构
5. **调试数据流：** 使用调试工具

---

**记住：Cutie 的架构是经过深思熟虑的，遵循规范可以避免 90% 的问题！** 📚✨

