### **Cutie 后端设计与架构纲领 V1.0**

#### **前言：核心设计哲学**

*   **1.1. 产品理念:** 本文档所有技术决策均服务于Cutie的核心产品理念：
    *   **平静工作 (Calm Work):** 软件应减少用户的焦虑，而不是增加。设计应引导用户进入主动、专注、可持续的工作状态。
    *   **用户主导 (User in Control):** 用户永远是其数据和意图的最终主人。AI和自动化应作为助手，而非决策者。
    *   **最小化元工作 (Minimizing Meta-work):** 软件应尽可能减少用户在“管理工作”上花费的时间，让用户专注于“执行工作”。

*   **1.2. 工程原则:**
    *   **健壮性 (Robustness):** 系统必须保证数据的一致性和操作的原子性。业务逻辑的边界情况必须被充分考虑和处理。
    *   **可测试性 (Testability):** 系统的核心逻辑必须是可单元测试的。外部依赖（时间、数据库、网络）必须被解耦。
    *   **可演进性 (Evolvability):** 架构设计必须为未来的功能扩展（如云同步、手机App、更多AI功能）和技术迁移预留清晰的路径。

*   **1.3. 发布范围 (V1.0.0.0):**
    *   **核心实现:** `Task`, `TimeBlock`, `Area`, `Template`, `Task_Schedule` (日程安排), `Ordering` (排序)。
    *   **延迟实现 (仅创建数据库表，不提供API):** `TimePoint`, `Project`, `Reminder`, `Tag`。
    *   **特殊实现:** `Setting` 采用TOML文件适配器实现。
    *   **不实现:** 云同步、多人协作、完整的重复任务后台服务。

---

#### **第一部分：数据模型层 (The Foundation)**

*   **2.1. 最终数据库Schema (V1.8 “定稿版”):**

    *   **2.1.1. `Task` (任务表):**
        *   `id` (UUID, 主键), `title` (String), `glance_note` (String, nullable), `detail_note` (String, nullable)
        *   `estimated_duration` (Integer, nullable)
        *   `subtasks` (JSON, nullable) - `[{ "id": UUID, "title": String, "is_completed": Boolean, "sort_order": String }]`
        *   `project_id` (UUID, nullable, 外键 -> `Project.id`)
        *   `area_id` (UUID, nullable, 外键 -> `Area.id`)
        *   `due_date` (Timestamp, nullable), `due_date_type` (Enum: `SOFT`, `HARD`, nullable)
        *   `completed_at` (Timestamp, nullable)
        *   `created_at` (Timestamp), `updated_at` (Timestamp, INDEXED), `is_deleted` (Boolean, 默认 `false`, INDEXED)
        *   `source_info` (JSON, nullable), `external_source_id` (String, nullable, INDEXED), `external_source_provider` (String, nullable), `external_source_metadata` (JSON, nullable)
        *   `recurrence_rule` (String, nullable), `recurrence_parent_id` (UUID, nullable, 自关联 -> `Task.id`), `recurrence_original_date` (Timestamp, nullable), `recurrence_exclusions` (Array of Timestamps, nullable)

    *   **2.1.2. `TimeBlock` (时间块表):**
        *   `id` (UUID, 主键), `title` (String, nullable), `glance_note` (String, nullable), `detail_note` (String, nullable)
        *   `start_time` (Timestamp, NOT NULL), `end_time` (Timestamp, NOT NULL)
        *   `area_id` (UUID, nullable, 外键 -> `Area.id`)
        *   (包含所有`created_at`, `updated_at`, `is_deleted`, `external_...`, `recurrence_...`元数据和预留字段)

    *   **2.1.3. `Project` (项目表):**
        *   `id` (UUID, 主键), `name` (String)
        *   `status` (Enum: `ACTIVE`, `PAUSED`, `COMPLETED`, `ARCHIVED`), `type` (Enum: `PROJECT`, `EXPERIENCE`, 默认 `PROJECT`)
        *   `resources` (JSON, nullable), `area_id` (UUID, nullable, 外键 -> `Area.id`)
        *   (包含所有`created_at`, `updated_at`, `completed_at`, `is_deleted`, `external_...`元数据和预留字段)

    *   **2.1.4. `Area` (领域表):**
        *   `id` (UUID, 主键), `name` (String), `color` (String), `parent_area_id` (UUID, nullable, 自关联 -> `Area.id`)
        *   (包含`created_at`, `updated_at`, `is_deleted`元数据字段)
    
    *   **2.1.5. `Template` (模板表):**
        *   `id` (UUID, 主键), `name` (String), `title_template` (String), `glance_note_template` (String, nullable), `detail_note_template` (String, nullable), `estimated_duration_template` (Integer, nullable), `subtasks_template` (JSON, nullable), `area_id` (UUID, nullable, 外键 -> `Area.id`)
        *   (包含`created_at`, `updated_at`, `is_deleted`元数据字段)

    *   **2.1.6. `Task_Schedule` (任务日程表):**
        *   `id` (UUID, 主键), `task_id` (UUID, NOT NULL, 外键 -> `Task.id`), `scheduled_day` (Timestamp, NOT NULL, INDEXED), `outcome` (Enum: `PLANNED`, `PRESENCE_LOGGED`, `COMPLETED_ON_DAY`, `CARRIED_OVER`, 默认 `PLANNED`)

    *   **2.1.7. `Ordering` (统一排序表):**
        *   `id` (UUID, 主键), `context_type` (Enum: `DAILY_KANBAN`, `PROJECT_LIST`, `AREA_FILTER`, `MISC`), `context_id` (String, INDEXED), `task_id` (UUID, 外键 -> `Task.id`), `sort_order` (String, INDEXED), `updated_at` (Timestamp)
        *   (复合唯一约束: `context_type`, `context_id`, `task_id`)

    *   **2.1.8. `Task_TimeBlock_Link` (链接表):**
        *   `task_id` (UUID, 外键), `time_block_id` (UUID, 外键)

    *   **2.1.9. 延迟实现的表 (仅建表):** `TimePoint`, `Reminder`, `Tag`, `Task_Tag_Link`, `TimeBlock_Tag_Link`。

*   **2.2. 核心概念的数据表示:**
    *   **任务状态:** 任务的全局完成状态由`Task.completed_at`是否为`NULL`唯一确定。任务是否在Staging区由其是否存在于任何`Task_Schedule`记录中隐式确定。
    *   **每日结局:** 任务在某一天的局部状态由`Task_Schedule.outcome`字段明确记录，它独立于任务的全局完成状态，并用于忠实记录每日的工作历史。
    *   **上下文ID规范:** `Ordering.context_id`遵循以下规范：
        *   `DAILY_KANBAN`: Unix时间戳字符串 (e.g., `'1729555200'`)。
        *   `PROJECT_LIST`: `project::{project_id}`。
        *   `AREA_FILTER`: `area::{area_id}`。
        *   `MISC`: 纯小写蛇形命名 (e.g., `'floating'`, `'staging_all'`)。

---

#### **第二部分：后端架构设计 (The Framework)**

*   **3.1. 总体架构：Sidecar纯服务器模式**
    *   **3.1.1. 描述:** Tauri应用启动时，会附带启动一个独立的Rust Web服务器子进程 (Sidecar)。前端通过HTTP与此本地服务器通信。
    *   **3.1.2. 优势:** 实现了本地后端与未来云后端的代码同构，为手机App和Web App的开发铺平了道路，并实现了前后端的完全解耦。
    *   **3.1.3. 进程通信:** 采用动态端口分配与“握手”机制。后端服务器监听`0`端口，通过`stdout`将操作系统分配的实际端口号告知Tauri主进程，主进程再通过事件或状态管理将端口号传递给前端。

*   **3.2. 分层架构：端口与适配器 (六边形架构)**
    *   **3.2.1. 路由层 (Router/Controller):** 负责解析HTTP请求、验证输入、调用服务层。不包含业务逻辑。使用`axum`或`actix-web`的Handler实现。
    *   **3.2.2. 服务层 (Service/Use Case):** 系统的核心，封装所有业务逻辑和事务。方法名直接反映用户意图（如`complete_task`）。它持有对仓库层抽象(Trait)的引用。
    *   **3.2.3. 仓库层 (Repository):** 抽象化所有数据访问。通过定义`Trait`（端口）和具体的数据库实现（适配器）来隔离业务逻辑与数据库技术细节。
    *   **3.2.4. 数据源层 (Data Source):** 数据库本身 (SQLite) 和数据库驱动 (`sqlx`)。

*   **3.3. 依赖管理：依赖注入 (DI)**
    *   **3.3.1. 依赖注入容器:** 在应用启动时，创建一个全局的`AppState`结构体，该结构体持有所有服务和仓库的实例 (`Arc<...>`包裹)。`AppState`通过`axum`的`State`提取器注入到所有路由处理器中。
    *   **3.3.2. 核心抽象定义:** 必须为所有外部依赖定义`Trait`（端口），包括但不限于`Clock` (时间), `IdGenerator` (UUID生成), 以及所有`Repository`。

---

#### **第三部分：可测试性设计 (Design for Testability)**

*   **4.1. 核心策略：解耦外部依赖**
    *   **4.1.1. 时间解耦:** `TaskService`等服务必须通过注入`Arc<dyn Clock>`来获取当前时间。提供`SystemClock`（生产用）和`FixedClock`（测试用）两种适配器。
    *   **4.1.2. 数据库解耦:** 所有服务必须通过注入`Arc<dyn ...Repository>`来访问数据。提供`Sqlx...Repository`（生产用）和`InMemory...Repository`（测试用）两种适配器。
    *   **4.1.3. 设置存储解耦:** `SettingService`通过注入`Arc<dyn SettingRepository>`来访问设置。V1.0提供`TomlSettingRepository`作为适配器，未来可无缝替换为`SqlxSettingRepository`。

*   **4.2. 其他可测试性设计范式:**
    *   **4.2.1. 函数式核心，命令式外壳:** 鼓励在服务层中，将无副作用的、复杂的计算逻辑（如`RRULE`解析）提炼成可独立测试的纯函数。
    *   **4.2.2. 测试策略:**
        *   **单元测试:** 重点测试服务层的业务逻辑（使用内存仓库和固定时钟）、纯函数以及仓库层的数据库查询逻辑。鼓励使用表驱动测试。
        *   **集成测试:** 测试从API端点到数据库的完整流程。鼓励使用快照测试来验证API响应的稳定性。

---

#### **第四部分：业务逻辑与API设计 (The Interface)**

*   **5.1. API设计哲学：面向用例 (Use Case-Driven)**
    *   所有API端点必须代表一个完整的用户意图，封装所有相关的数据库操作和业务规则于一个事务中。严禁创建简单的、直接暴露数据库表结构的CRUD API。
*   **5.2. V1.0.0.0 核心业务事件清单:**
    *   **任务生命周期:** `TaskCreatedInContext`, `TaskScheduledToDay`, `TaskUnscheduledCompletely`, `TaskRescheduled`, `TaskGloballyCompleted`, `TaskReopened`。
    *   **任务编辑:** `TaskDetailsUpdated`, `TaskAreaChanged`, `SubtasksUpdated`。
    *   **时间块管理:** `TimeBlockCreated`, `TimeBlockUpdated`, `TimeBlockDeleted`, `TaskLinkedToTimeBlock`, `TaskUnlinkedFromTimeBlock`。
    *   **日程与排序:** `PresenceLoggedForDay`, `DailySortOrderUpdated`, `MiscSortOrderUpdated`。
    *   **模板:** `TemplateCRUD`, `TaskCreatedFromTemplate`。
*   **5.3. 关键业务逻辑分解 (示例: `CompleteTaskCommand`):**
    *   **命令:** `POST /api/tasks/{id}/complete`
    *   **前置条件:** 任务存在；任务未完成（幂等处理）。
    *   **过程 (事务内):**
        1.  获取当前时间`now` (通过`Clock`服务)。
        2.  更新`Task.completed_at = now`。
        3.  截断正在进行的、仅与此任务耦合的`TimeBlock`。
        4.  删除所有未来的`Task_Schedule`记录。
        5.  删除所有未来的、仅与此任务耦合的`TimeBlock`。
        6.  更新**当天**的`Task_Schedule`记录的`outcome`为`COMPLETED_ON_DAY`。**不修改**过去日期的`outcome`。
    *   **结果:** 返回更新后的`Task`对象。通过WebSocket推送`TASK_UPDATED`事件。
*   **5.4. AI集成策略:**
    *   **调用位置:** 所有对外部AI服务的调用均在后端执行。
    *   **执行模式:**
        *   **Area推导:** 采用“后端异步执行”模式。创建任务的API立即返回，AI推导结果通过WebSocket事后更新到前端。
        *   **任务细化 (“少女的明信片”):** 采用“后端同步执行”模式。前端调用专门的API (`/api/ai/refine-task`)并等待返回，因为用户预期这是一个即时操作。

#### 第五部分：开发流程

**每开发完一层，都必须完成相应的测试和文档，并通过评审，才能进入下一层。** 这就是我们的检查点。

以下是我为您设计的、与我们八层开发顺序严格对应的检查点和交付物清单。

---

### **Cutie开发流程的“质量门”检查点**

我们将整个开发过程视为一系列的“关卡”，每个关卡都有明确的“通关条件”。

#### **关卡 1-4: 奠基阶段 (Laying the Foundation)**
*(这前四层是基础库和定义，它们可以并行或快速串行完成)*

*   **开发内容:**
    1.  **核心领域模型层:** 完成所有`struct`和`enum`的定义。
    2.  **数据源层:** 完成V1.0的所有数据库迁移脚本。
    3.  **通用工具/错误处理层:** 完成`AppError`和所有`utils`模块。
    4.  **外部依赖抽象层:** 完成所有`Trait`（端口）的定义。
*   **检查点/质量门:**
    *   **交付物:**
        *   所有领域模型代码 (`models/*.rs`)。
        *   所有数据库迁移SQL文件 (`migrations/*.sql`)。
        *   所有工具和错误处理代码 (`utils/*.rs`, `error.rs`)。
        *   所有外部依赖`Trait`定义代码 (`ports/*.rs`)。
        *   **开发文档:** 对每个核心模型、错误类型和`Trait`的RustDoc注释（即我们之前编写的CABC）。
    *   **验收标准 (通关条件):**
        1.  **代码审查 (Code Review):** 所有代码必须通过团队的代码审查，确保其符合我们定义的规范。
        2.  **数据库迁移测试:** 在一个空的测试数据库上，能够从头到尾、无错误地成功运行所有迁移脚本。
        3.  **单元测试:** `utils`模块中的所有纯函数必须有100%的单元测试覆盖率。
        4.  **文档生成:** `cargo doc --open`能够成功生成清晰、可读的文档。

---

#### **关卡 5: 仓库层 (The Data Access Gate)**

*   **开发内容:** 编写所有`Repository Trait`的`Sqlx`适配器实现。
*   **检查点/质量门:**
    *   **交付物:**
        *   所有`Sqlx...Repository`的实现代码 (`repositories/*.rs`)。
        *   **开发文档:** 对`Repository`实现中一些复杂SQL查询的必要注释。
    *   **验收标准 (通关条件):**
        1.  **代码审查:** 重点审查SQL语句的正确性、效率和安全性（防止SQL注入，虽然`sqlx`的参数化查询已能很好地处理）。
        2.  **集成测试 (关键！):**
            *   **必须为每一个`Repository`的每一个公共方法编写集成测试。**
            *   这些测试**必须**连接到一个真实的（但用于测试的）数据库。
            *   测试流程：准备数据 -> 调用仓库方法 -> 从数据库中断言状态的改变是否符合预期。
            *   **测试覆盖率：** `Repository`层的集成测试覆盖率应达到一个非常高的标准（例如95%+）。

---

#### **关卡 6: 应用配置与启动层 (The Assembly Gate)**

*   **开发内容:** 编写应用的`main.rs`，构建`AppState`依赖注入容器，配置Sidecar启动。
*   **检查点/质量门:**
    *   **交付物:** `main.rs`和相关的配置模块代码 (`config.rs`)。
    *   **验收标准 (通关条件):**
        1.  **能跑起来:** `cargo run`命令必须能够成功启动应用主进程和Sidecar子进程。
        2.  **日志正常:** 启动时，日志系统必须被正确初始化，并能看到各模块启动的日志信息。
        3.  **依赖注入验证:** 在一个临时的测试路由中，能够成功注入并访问`AppState`中的服务实例（即使这些服务此时还是“空”的）。
        4.  **配置加载:** 应用能够正确地从环境变量或配置文件中读取配置。

---

#### **关卡 7: 业务/服务层 (The Logic Gate)**

*   **开发内容:** 实现所有`Service`层的业务逻辑。
*   **检查点/质量门:**
    *   **交付物:**
        *   所有`...Service`的实现代码 (`services/*.rs`)。
        *   **开发文档:** **这是最重要的文档交付物！** 我们之前编写的、完整的CABC文档（函数签名、行为简介、契约、边界、副作用）必须作为RustDoc注释，写在每一个公共方法之上。
    *   **验收标准 (通-关条件):**
        1.  **代码审查:** **极其严格的代码审查。** 审查者必须以CABC文档为“法律”，逐行检查代码实现是否100%符合文档的定义。
        2.  **单元测试 (关键！):**
            *   **必须为每一个`Service`的每一个公共方法编写详尽的单元测试。**
            *   这些测试**必须**使用**内存适配器**（`InMemoryRepository`, `FixedClock`等）来运行，确保测试速度飞快且与外部世界隔离。
            *   测试用例的设计**必须**直接来源于CABC文档中的“边界情况”部分，确保所有阳光和阴雨天路径都被覆盖。
            *   **测试覆盖率：** `Service`层的单元测试覆盖率必须达到100%或接近100%。

---

#### **关卡 8: 网络/路由层 (The Final Gate - API Freeze)**

*   **开发内容:** 编写所有API端点处理器，定义DTO，完成最终的OpenAPI文档。
*   **检查点/质量门:**
    *   **交付物:**
        *   所有路由处理器代码 (`handlers/*.rs` 或 `routes/*.rs`)。
        *   所有DTO结构体定义 (`dtos.rs`)。
        *   **最终的、完整的`openapi.yaml`或`.json`文件。**
    *   **验收标准 (通关条件):**
        1.  **API契约审查:** `openapi.yaml`文件必须通过前后端团队的共同评审，并被“冻结”，作为V1.0的最终契约。
        2.  **端到端(E2E) / API集成测试:**
            *   使用一个HTTP客户端（如`reqwest`）和运行中的测试服务器，对**每一个**API端点发起真实的HTTP请求。
            *   断言HTTP响应的状态码、头部和响应体是否与OpenAPI文档的定义完全一致。
            *   这些测试会贯穿所有层，最终写入真实的测试数据库。
        3.  **文档一致性:** 自动生成的Swagger UI或Redoc文档必须清晰、可用，且与`openapi.yaml`文件完全一致。

