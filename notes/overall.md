# Cutie —— 每个小可爱的生活操作系统 ❤️

**Cutie (暂定名)** 是一款为你量身打造的、最可爱的生活操作系统。

我们相信，规划生活不应该是一件充满压力和焦虑的事情。它应该像装点自己的手帐本一样，充满乐趣、创造力和温暖。Cutie 旨在成为你的贴心伙伴，用一种“可可爱爱”的方式，帮助你轻松地组织任务、规划时间、记录生活中的“小确幸”，让你在高效完成目标的同时，也能感受到生活的美好。

---

## ✨ 核心开发范式：一种自上而下的设计哲学

Cutie 的开发，严格遵循一套从抽象到具体的“心智优先”范式。我们相信，一个伟大的软件，其优雅的体验源于其内在和谐统一的哲学思想。这个范式分为三个核心层次：

1.  **心智模型 (Mental Model):** 定义软件的“灵魂”，即我们希望用户如何思考和感受。
2.  **数据模型 (Data Model):** 将抽象的“心智模型”翻译成结构化的、计算机可以理解的“骨架”。
3.  **UI/UX 实现 (UI/UX Implementation):** 为“骨架”穿上美丽的“容貌”，并赋予其流畅、愉悦的交互体验。

这种自顶向下的方法，确保了 Cutie 的每一个功能、每一个按钮、每一个像素，都服务于其核心的设计哲学，而不是功能的无序堆砌。

---

## 🧠 心智模型：Cutie 的世界观

“心智模型”是我们对 Cutie 工作方式的最核心、最底层的构想。它定义了我们如何理解和组织现实世界中的各种信息。Cutie 的心智模型由三个既相互独立又可灵活连接的“域”(Domains) 和一个统一的“连接系统”构成。

### 1. 三大核心域 (The Three Core Domains)

我们认为，你需要管理的一切，都可以被优雅地归入以下三个纯粹的领域：

*   **Task 域 (你要做什么？):** 这里存放着你所有的**意图和目标**。它不是一个扁平的待办列表，而是被深刻地分层，以容纳你生活中的不同“待办”类型：
    *   **项目 (Project):** 代表一段**无法预期时长**的大型工作或**人生探索的经历**。它接纳不确定性，鼓励你去冒险。
        *   *例子：“读完《刀剑神域》”、“设计一个‘袖剑’”、“学习一门新语言”。*
    *   **任务 (Task):** 代表一个**可以预估时间**的、需要被**完成**的具体目标。它是你计划中的一个个坚实的踏板。
        *   *例子：“写完本周周报”、“预定周末的餐厅”。*
    *   **检查点 (Checkpoint):** 代表完成一个 `Task` 的**引导性步骤**。它不是一个硬性的子任务，而是一个友好的路标，帮助你轻松启动。
        *   *例子：(在“写周报”任务下) “下载公司模板”、“核对上周数据”。*

*   **Time 域 (你何时去做？):** 这里掌管着你最宝贵的资源——**时间**。它由两种基本单位构成：
    *   **活动 (Activity):** 代表日历上的一段**纯粹的时间**（时间块）。它中立、客观，只描述“从几点到几点”。
    *   **时间点 (Time Point):** 代表一个**具体的时刻**，如截止日期 (Deadline) 或提醒。

*   **Note 域 (你需要记住什么？):** 这里是你的**“记忆外挂”**，存放着所有非结构化的上下文信息。它的核心价值在于**“让你能无缝地继续今天的工作”**，避免因遗忘而产生的重复劳动和挫败感。
    *   *例子：“我跟着‘你的影月月’的稻妻探索视频看到了12P，下次从13P开始。”*

### 2. 核心交互机制：解耦与连接 (Decoupling & Linking)

Cutie 最核心的魔法在于，我们认为**“做什么”(Task 域) 和“何时做”(Time 域) 是两件独立的事情。**

*   **解耦：** 一个 `Task` 被创建时，它不关心自己何时被执行。一个 `Time Block` (活动) 被创建时，它也不关心自己要用来做什么。
*   **链接 (Link):** 你可以通过创建一个**多对多、双向的“链接”**，来将 `Task` 和 `Time Block` 优雅地关联起来。
    *   一个复杂的 `Task` 可以链接到**多个**分散在不同天的 `Time Block` 上。
    *   一个 `Time Block` 也可以同时链接到**多个**小 `Task`。
*   **优势：** 这个模型完美地解决了“计划赶不上变化”的问题。“没做完”不再是需要被“推迟”的失败，而仅仅是“为同一个目标创建一条新链接”的正常记录。

### 3. 统一的组织系统：标签 (Tags)

为了将这三个独立的域有机地组织起来，我们引入了一个**统一的、跨域的标签系统**。

*   你可以为任何 `Project`, `Task`, `Note`, `Activity` 打上同一个标签（如 `#学习` 或 `#项目:袖剑设计`）。
*   通过标签，你可以瞬间**聚合**所有与某个“情境”相关的信息，形成一个动态的、智能的视图。

---

**总结：** Cutie 的心智模型，是一个**尊重现实复杂性、充满人性关怀、且架构优雅灵活的系统**。它旨在成为你最聪明的“第二大脑”，也是你最可爱的“数字伙伴”。

---

## 🛠️ 技术栈 (Tech Stack)

Cutie 致力于采用现代、高效且体验优秀的技术栈，以确保产品的性能、可维护性和未来的可扩展性。

*   **核心框架 (Core Framework):** [Vue 3](https://vuejs.org/)
    *   我们选择 Vue 3 的组合式 API (Composition API) 和单文件组件 (SFC) 模式，因其极佳的开发体验、高度内聚的代码组织方式，以及对 AI 辅助开发极其友好的“上下文局部性”。

*   **构建工具 (Build Tool):** [Vite](https://vitejs.dev/)
    *   利用其快如闪电的冷启动和热更新 (HMR) 能力，提供极致的开发效率。

*   **语言 (Language):** [TypeScript](https://www.typescriptlang.org/)
    *   为大型应用提供必要的类型安全，减少运行时错误，并为 AI 提供更丰富的代码上下文。

*   **跨平台方案 (Cross-Platform Solution):** [Tauri](https://tauri.app/)
    *   使用 Tauri 将 Web 应用打包成轻量、高性能、安全的跨平台桌面应用 (Windows, macOS, Linux)。其 Rust 后端保证了性能和安全性。

*   **状态管理 (State Management):** [Pinia](https://pinia.vuejs.org/)
    *   作为 Vue 官方推荐的下一代状态管理库，Pinia 以其简洁的 API、强大的 TypeScript 支持和模块化的设计，作为我们应用的“中央记忆库”。

*   **样式方案 (Styling):**
    *   **核心原则:** 遵循我们自定义的、严格的样式规范，以实现主题切换和 UI 缩放。
    *   **底层技术:** (待定) 计划采用 [Tailwind CSS](https://tailwindcss.com/) 或其他原子化 CSS 方案，配合 Headless UI 组件库（如 [Radix Vue](https://www.radix-vue.com/)），以在保证开发效率的同时，获得最大的设计自由度。

*   **代码规范工具 (Linting & Formatting):**
    *   **ESLint:** 用于保证代码质量和一致性。
    *   **Prettier:** 用于自动化代码格式化。

---

## 📜 开发规范 (Development Guidelines)

为保证 Cutie 的代码质量、可维护性，以及实现我们的核心设计哲学（如主题切换、UI缩放、解耦），所有贡献者（包括我们自己和 AI 伙伴）都必须遵循以下“法典”。

### 1. 代码风格与格式化 (Code Style & Formatting)

*   **单一标准:** 本项目使用 **Prettier** 作为唯一的代码格式化工具。所有代码在保存时都应被自动格式化。
*   **配置文件:** 具体的格式化规则定义在根目录的 `.prettierrc.json` 文件中。
*   **质量检查:** **ESLint** 用于静态代码分析，帮助发现潜在问题。规则集在 `.eslintrc.cjs` 中定义，并与 Prettier 协同工作。

### 2. 样式规范 (The Styling Bible)

这是实现动态主题和缩放的基石，必须严格遵守。

*   ✅ **【尺寸】只用 `rem`:**
    *   在全局 `style.css` 中已设定 `font-size: 62.5%`，建立了 `1rem = 10px` 的基准。
    *   **铁律:** 所有 CSS 属性中的尺寸单位（`font-size`, `margin`, `padding`, `width`, `height`, `border-radius` 等）**必须**使用 `rem`。**严禁**使用 `px`。

*   ✅ **【颜色】只用 CSS 变量:**
    *   所有颜色值都在全局 `style.css` 的 `:root` 和 `body.theme-xxx` 中以 CSS 变量（如 `--color-primary`, `--color-background`）的形式统一定义。
    *   **铁律:** 在组件样式中，所有颜色属性（`color`, `background-color`, `border-color` 等）**必须**使用 `var(--variable-name)`。**严禁**使用硬编码的颜色值（如 `#FFF` 或 `red`）。

*   ✅ **【作用域】强制使用 Scoped CSS:**
    *   在所有 Vue 单文件组件中，`<style>` 标签**必须**添加 `scoped` 属性 (`<style scoped>`)，以防止组件间样式污染。

### 3. 组件设计架构 (Component Architecture)

我们遵循“高内聚，低耦合”的原则，保证系统的解耦和可维护性。

*   ✅ **【第一原则：语义化封装】**
    *   **何时封装？** 当一个 UI 元素（即使是简单的 `<div>`）在应用中代表了一个**可被“命名”的、会“重复出现”的“概念”**时，就必须将其封装成一个独立的基础组件。
        *   *例如：一个带有特定圆角、边框和内边距的容器，应该被封装成 `<CuteCard.vue>`。*
        *   *一个常用的页面布局结构（如“侧边栏+主内容区”），应该被封装成 `<CutePageLayout.vue>`。*
    *   **何时不封装？** 如果一个 `<div>` 或 `<span>` 仅仅是用于**某个特定组件内部的、临时的布局目的**，则不应过度封装，直接在组件内部使用即可。
    *   **目标：** 我们的 `src/components/ui/` 和 `src/components/layout/` 目录，应该是一个由这些充满“语义”的、可复用的“概念”组件构成的积木盒。
*   ✅ **【封装】建立自有基础组件库:**
    *   所有通用的、底层的 UI 元素（按钮、弹窗等）都应被封装为项目自身的**基础组件**，存放于 `src/components/ui/` 目录下。
    *   **铁律:** 业务页面和功能性组件中，**只允许**调用这些内部基础组件，**严禁**直接调用任何第三方 UI 库（如 Naive UI）的组件。这是为了保证未来可以无痛替换底层实现。

*   ✅ **【数据流】严格单向:**
    *   遵循标准的 **"Props down, Events up"** 模式。子组件通过 Props 接收数据，通过 Events (`emit`) 通知父组件状态变更请求。
    *   **严禁**子组件直接修改父组件传递的 Props。

*   ✅ **【逻辑分离】善用组合式函数 (Composables):**
    *   对于可复用的、或者复杂的业务逻辑（如拖放、本地数据库交互、AI 调用等），应将其抽离到 `src/composables/` 目录下的 `useXXX.ts` 文件中。
    *   保持组件的 `<script>` 部分简洁，主要负责状态管理和逻辑编排。

---

## 📂 文件结构 (Folder Structure)

为了保证项目的清晰性、可维护性和可扩展性，Cutie 采用“按功能领域划分”的模块化文件结构。清晰的结构能帮助我们（以及我们的 AI 伙伴）快速定位和组织代码。

`src/` 目录是我们所有前端代码的核心，其主要结构如下：

```
src/
│
├── assets/             # 存放静态资源 (图片, 字体等)
│
├── components/         # 存放全局可复用的、纯粹的UI组件
│   ├── layout/         #   - 应用的整体布局组件 (如: TheSidebar.vue, TheHeader.vue)
│   └── ui/             #   - 基础UI元素库 (如: CuteButton.vue, CuteDialog.vue)
│
├── composables/        # 存放可复用的 Vue 组合式函数 (逻辑单元, 如: useDatabase.ts)
│
├── directives/         # 存放自定义 Vue 指令 (如: v-tutorial.ts)
│
├── router/             # 存放 Vue Router 的路由配置
│
├── stores/             # 存放 Pinia 的全局状态管理模块
│
├── types/              # 存放全局 TypeScript 类型与接口定义【极其重要】
│
├── utils/              # 存放通用的、与框架无关的工具函数
│
└── views/              # 存放页面级组件，由路由直接渲染 (如: HomeView.vue)
```

### 各目录职责详解：

*   **`assets/`**:
    *   **用途:** 存放所有非代码的静态文件，如 SVG 图标、背景图片、自定义字体文件等。
    *   **AI 指令示例:** "将我提供的 `logo.svg` 文件保存到项目中。" -> AI 应将其放入 `src/assets/images/`。

*   **`components/`**:
    *   **用途:** 构建应用的“积木盒”。这里只存放**展示性 (Presentational)** 的、可在任何地方复用的组件。
    *   **`components/ui/`** 是我们的自有基础组件库，是实践“开发规范”中**组件封装原则**的地方。
    *   **AI 指令示例:** "创建一个新的、可爱的按钮组件，命名为 CuteSwitch。" -> AI 应在 `src/components/ui/` 目录下创建 `CuteSwitch.vue`。

*   **`composables/`**:
    *   **用途:** **逻辑复用的核心。** 所有复杂的、可抽离的业务逻辑或功能逻辑，都应封装成 `useXXX` 格式的组合式函数。
    *   **AI 指令示例:** "为与 SQLite 数据库的交互创建一个组合式函数。" -> AI 应在 `src/composables/` 目录下创建 `useDatabase.ts`。

*   **`router/`**:
    *   **用途:** 定义应用的页面导航结构。将 URL 路径映射到对应的 `views/` 组件。

*   **`stores/`**:
    *   **用途:** 应用的**“中央大脑”**。存放所有 Pinia store。建议按领域（如 `ui`, `user`, `data`）拆分文件。
    *   **AI 指令示例:** "创建一个 store 来管理用户的认证状态。" -> AI 应在 `src/stores/` 目录下创建 `user.ts`。

*   **`types/`**:
    *   **用途:** **项目的“数据字典”和“蓝图”！** 在此定义所有核心业务实体（如 `Project`, `Task`, `Note`）的 TypeScript `interface` 或 `type`。
    *   **AI 指令示例:** "为我们的'项目'心智模型定义 TypeScript 类型。" -> AI 应在 `src/types/index.d.ts` (或 `models.ts`) 中进行定义。

*   **`utils/`**:
    *   **用途:** 存放纯粹的、与具体业务无关的辅助函数，如日期格式化、字符串处理等。

*   **`views/`**:
    *   **用途:** 存放**容器性 (Container)** 的页面级组件。它们负责组织一个页面的布局，并从 `stores` 或 `composables` 获取数据，再将数据传递给 `components` 中的展示性组件。
    *   **AI 指令示例:** "创建一个新的页面用于展示用户设置。" -> AI 应在 `src/views/` 目录下创建 `SettingsView.vue`，并在 `router` 中为其配置路由。

---
---

## 💾 数据库结构 (Database Schema)

Cutie 的所有本地数据都存储在一个单一的、关系设计的 **SQLite** 数据库中。这种设计保证了数据的完整性、一致性以及强大的查询能力。以下是核心的表结构设计，它忠实地反映了我们的心智模型。

### 核心设计原则

*   **唯一标识符:** 所有主键 (`id`) 均使用文本类型的 **UUID**，以支持离线创建和未来的多设备同步。
*   **关系完整性:** 使用**外键 (Foreign Keys)** 明确实体间的关系。
*   **软删除:** 使用 `deleted_at` 时间戳实现软删除，便于数据恢复和同步。
*   **同步预留:** `remote_updated_at` 等字段为未来的云同步功能预留了接口。
*   **灵活性:** 关键表使用 `metadata` JSON 字段，以容纳未来可能增加的非结构化数据。

---

### 核心实体表 (Core Entity Tables)

#### 1. `projects`
*   **用途:** 存储“项目/探索经历”。作为大型目标或长期探索的容器。

```sql
CREATE TABLE projects (
    id TEXT PRIMARY KEY NOT NULL,          -- 主键 (UUID)
    title TEXT NOT NULL,                   -- 标题
    description TEXT,                      -- 详细描述 (支持Markdown)
    icon TEXT,                             -- 图标/Emoji
    color TEXT,                            -- 代表色
    status TEXT NOT NULL DEFAULT 'active', -- 状态: 'active', 'on_hold', 'archived'
    metadata JSON,                         -- 灵活的元数据存储
    
    -- 同步与时间戳
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    deleted_at INTEGER,                    -- 软删除标记
    remote_updated_at INTEGER
);
```

#### 2. `tasks`
*   **用途:** 存储具体的“行动单元”。一个任务可以独立存在，也可以从属于一个项目。

```sql
CREATE TABLE tasks (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT,                       -- 外键 (可为空), 关联到 projects 表
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'todo',   -- 'todo', 'in_progress', 'done', 'canceled'
    due_date INTEGER,                      -- 截止日期 (时间点)
    completed_at INTEGER,                  -- 完成时间
    sort_key TEXT NOT NULL,                -- 在其上下文中的排序键
    metadata JSON,
    
    -- 同步与时间戳
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    deleted_at INTEGER,
    remote_updated_at INTEGER,
    
    -- 如果项目被删除，此任务的 project_id 会被设为 NULL，任务本身保留
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE SET NULL 
);
```

#### 3. `checkpoints`
*   **用途:** 存储任务的引导性步骤或子任务。

```sql
CREATE TABLE checkpoints (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL,                 -- 外键, 关联到 tasks 表
    title TEXT NOT NULL,
    is_completed BOOLEAN NOT NULL DEFAULT 0,
    sort_key TEXT NOT NULL,                -- 在其所属任务中的排序键
    
    -- 同步与时间戳
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    deleted_at INTEGER,
    remote_updated_at INTEGER,
    
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);
```

#### 4. `activities`
*   **用途:** 存储日历上的“时间块”，是纯粹的时间容器。

```sql
CREATE TABLE activities (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT,                            -- 活动标题 (如 "团队会议")
    start_time INTEGER NOT NULL,           -- 开始时间戳
    end_time INTEGER NOT NULL,             -- 结束时间戳
    timezone TEXT,
    is_all_day BOOLEAN NOT NULL DEFAULT 0,
    color TEXT,
    metadata JSON,
    
    -- 同步与时间戳
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    deleted_at INTEGER,
    remote_updated_at INTEGER,
    
    -- 用于与外部日历同步
    origin_id TEXT,
    connector_id TEXT
);
```

#### 5. `tags`
*   **用途:** 存储可复用的、跨项目的分类标签。

```sql
CREATE TABLE tags (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    color TEXT,
    sort_key TEXT,                         -- 标签本身的排序键
    
    -- 同步与时间戳
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    deleted_at INTEGER,
    remote_updated_at INTEGER
);

-- 标签名应该是唯一的
CREATE UNIQUE INDEX idx_tags_title ON tags(title);
```

---

### 关系链接表 (Link Tables)

这些表是实现我们“多对多”灵活模型的关键，它们只存储关系，不存储实体数据。

#### 6. `task_activity_links`
*   **用途:** 连接 `tasks` (做什么) 和 `activities` (何时做)。

```sql
CREATE TABLE task_activity_links (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL,
    activity_id TEXT NOT NULL,
    
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (activity_id) REFERENCES activities(id) ON DELETE CASCADE
);

-- 确保同一对 task 和 activity 不会重复链接
CREATE UNIQUE INDEX idx_task_activity_unique ON task_activity_links(task_id, activity_id);
```

#### 7. `project_tags`
*   **用途:** 将标签应用到项目上。

```sql
CREATE TABLE project_tags (
    project_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    
    PRIMARY KEY (project_id, tag_id),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
```

#### 8. `task_tags`
*   **用途:** 将标签应用到任务上。

```sql
CREATE TABLE task_tags (
    task_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    
    PRIMARY KEY (task_id, tag_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
```

---

### 辅助系统表 (Auxiliary System Tables)

#### 9. `settings`
*   **用途:** 键值对形式存储所有用户可配置的应用设置。

```sql
CREATE TABLE settings (
    key TEXT PRIMARY KEY NOT NULL,
    value JSON NOT NULL
);
```

#### 10. `__migrations`
*   **用途:** 由数据库迁移工具自动管理，用于跟踪 Schema 的版本历史。
*   *注意: 表名和结构将由所选的迁移工具（如 `diesel_cli`）自动生成。*
```sql
/* Example for Diesel ORM */
CREATE TABLE __diesel_schema_migrations (
    version VARCHAR(50) PRIMARY KEY NOT NULL,
    run_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```
