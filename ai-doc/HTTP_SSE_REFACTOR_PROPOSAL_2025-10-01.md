# Cutie 架构重构建议：HTTP 命令 + SSE 事件化副作用（可演进方案）

作者：AI 助手（2025-10-01）

## 1. 背景与目标

当前系统采用“HTTP 命令 + 响应回写主资源 + 前端按需补拉”的模式，单机单用户体验稳定。在引入第二窗口（AI 助手）与未来可能迁移至 Electron/独立服务端、多端协同的前提下，我们希望：

- 将“稳定面”和“变化面”解耦：命令回写主资源（稳定）、副作用通过事件流（变化）传播。
- 在业务副作用经常变化的情况下，降低对已有响应体与 UI 的耦合度，提升可演进性。
- 保持渐进式迁移、低风险落地，并兼容未来多设备/远程部署。

结论：采用“HTTP 负责命令与主资源同步回写 + SSE 负责副作用/领域事件推送”的混合方案，建立领域事件契约与幂等更新机制，逐步事件化现有副作用逻辑。

## 2. 现状评估（摘录）

- HTTP 端点（Axum）：`POST /api/tasks/{id}/completion` 等，事务内处理任务完成、日程更新、时间块截断/删除，返回 `task` 与 `deleted_time_block_ids`、`truncated_time_block_ids`。
- 前端（Vue + Pinia）：
  - 以 `TaskStore` 为单一数据源，`TimeBlockStore` 管理日历块；
  - 完成任务后用返回体更新任务、删除时间块、并按日期段补拉被截断块；
  - 多窗口一致性尚未在框架层抽象（当前以单窗口为主）。
- 推送机制：未实现跨端 WS/SSE；Tauri 事件仅用于端口发现等本地事件，无法满足未来 Electron/独服需求。

问题与机会：

- 当副作用列表变化（比如新增更多受影响对象）时，需要频繁调整响应体与前端补拉逻辑，耦合偏高；
- 多窗口/多端一致性需要一条统一的“后端→前端”广播通道；
- 目前对时间块截断后的可观测性与幂等更新有优化空间。

## 3. 设计决策

- 传输层：
  - HTTP：保留全部“命令”与“主资源快照的同步回写”。
  - SSE：新增 `/api/events` 事件流，用于广播“副作用 / 领域事件”，保证跨窗口/多端被动同步。
  - WS：仅在确需双向高频（协作光标、上行流）时增量引入，非本次重构必需。
- 契约层：
  - 建立“事件化”的领域契约，版本化与幂等化，支持“未知事件忽略”。
  - 命令响应体保持稳定，副作用经事件流独立演进，降低 API 变更频率与 UI 耦合。

## 4. 领域事件模型（Domain Events）

事件信封（Envelope）：

```json
{
  "id": "evt_...", // 事件ID（递增/时间序）
  "type": "task.completed", // 事件类型（kebab/snake/dot均可，建议 dot）
  "version": 1, // 事件契约版本
  "occurred_at": "2025-10-01T08:00:00Z",
  "aggregate_type": "task", // 聚合类型
  "aggregate_id": "...uuid...",
  "aggregate_version": 42, // 聚合版本或updated_at单调戳
  "correlation_id": "cmd_...", // 关联发起命令（HTTP 请求）
  "causation_id": "evt_...", // 可选：上游事件ID，用于事件链路追踪
  "payload": {
    /* 见各事件 */
  }
}
```

核心事件（初始集）：

- `task.completed`
  - payload：`{ task: TaskCardDto }`
- `task.reopened`
  - payload：`{ task: TaskCardDto }`
- `time_blocks.truncated`
  - payload：`{ blocks: TimeBlockViewDto[] }`（直接携带已截断后的完整块，避免前端再按区间补拉）
- `time_blocks.deleted`
  - payload：`{ block_ids: Uuid[] }`
- `task.updated`、`time_block.updated/created/deleted`（按需）

幂等与时序：

- 前端 Reducer 必须以 `aggregate_version`（或 `updated_at` 单调戳）为准，丢弃过期事件；
- 以 `correlation_id` 做命令端/事件端的去重与关联，避免重复应用。

断线重连：

- SSE 使用 `Last-Event-ID` 续传；服务端维护 outbox 游标，支持从最近事件ID之后补发。

## 5. 后端改造（Axum + SQLx）

### 5.1 新增 SSE 端点

- `GET /api/events`；Header：`Accept: text/event-stream`；Keep-Alive + no-cache。
- 鉴权（如有需要）：可复用现有中间件（桌面端可简化）。
- 心跳：定期发送注释行或空事件，保持连接活性。

### 5.2 Outbox 模式（可靠投递）

- 事务内写入业务表与 `event_outbox`（同一事务）；提交后由后台 dispatcher 扫描 outbox，按序发送到 SSE 连接；成功后标记已发送或软删除。
- 表结构示例：

```sql
CREATE TABLE event_outbox (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  event_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  version INTEGER NOT NULL,
  aggregate_type TEXT NOT NULL,
  aggregate_id TEXT NOT NULL,
  aggregate_version INTEGER,
  correlation_id TEXT,
  occurred_at TEXT NOT NULL,
  payload TEXT NOT NULL,
  dispatched_at TEXT,
  created_at TEXT NOT NULL
);
CREATE INDEX idx_outbox_undispatched ON event_outbox(dispatched_at);
```

### 5.3 命令端点产出事件

- 以 `complete_task.rs` 为例：
  1. 事务内完成任务、更新日程、处理时间块；
  2. 写入 outbox：`task.completed`、`time_blocks.truncated`、`time_blocks.deleted`（blocks 用最新快照）；
  3. 提交事务；
  4. 返回响应体（保持稳定：`task` + 可选 `updated_time_blocks`）。

### 5.4 响应体微调（可选但推荐）

- 将 `CompleteTaskResponse` 扩展为：

```json
{
  "task": {
    /* TaskCardDto */
  },
  "deleted_time_block_ids": ["..."],
  "truncated_time_block_ids": ["..."],
  "updated_time_blocks": [
    /* TimeBlockViewDto[]（截断后的完整体）*/
  ],
  "correlation_id": "cmd_..."
}
```

这样发起命令的窗口可立即无补拉地精准更新；其他窗口依赖 SSE 事件完成被动同步。

## 6. 前端改造（Vue + Pinia）

### 6.1 事件订阅适配器（SSE）

- 新建 `services/events.ts`：
  - 负责连接 `/api/events`，处理 `EventSource` 生命周期、断线回退与 `Last-Event-ID`；
  - 提供订阅接口，向各 Store 广播事件（或通过全局 EventEmitter/Observable 分发）。
- 存储 `lastEventId` 于 `localStorage`，断线重连从上次位置续传。

示例（建议实现思路）：

```ts
// services/events.ts
export class DomainEventStream {
  private source?: EventSource
  private lastEventId = localStorage.getItem('lastEventId') ?? undefined

  connect(baseUrl: string) {
    const url = new URL('/api/events', baseUrl)
    const headers = { 'Last-Event-ID': this.lastEventId ?? '' }
    // 使用 polyfill 或后端允许查询参数携带 lastEventId
    this.source = new EventSource(url.toString(), { withCredentials: false })

    this.source.onmessage = (evt) => {
      this.lastEventId = evt.lastEventId || this.lastEventId
      if (this.lastEventId) localStorage.setItem('lastEventId', this.lastEventId)
      const event = JSON.parse(evt.data)
      this.dispatch(event)
    }
    this.source.onerror = () => {
      // 退避重试、断线重连
      this.reconnectWithBackoff()
    }
  }

  private dispatch(event: DomainEvent) {
    // 将事件分发给各 Store 的 Reducer
  }
}
```

### 6.2 Store 幂等 Reducer

- `TaskStore` 与 `TimeBlockStore` 各自提供 `applyEvent(event)`：
  - 基于 `aggregate_id` 与 `aggregate_version` 判断是否应用；
  - 支持 `task.completed`、`task.updated`、`time_blocks.truncated`、`time_blocks.deleted` 等；
  - 不影响现有 HTTP 回写路径，双轨并存，允许去重。

### 6.3 多窗口/AI 助手

- 两个窗口各自发命令（HTTP），各自立即按响应体更新本地 Store；
- 其它窗口通过 SSE 事件被动同步，确保最终一致；
- 使用 `correlation_id` 避免“命令窗口”再次应用同一事件。

## 7. 迁移计划（分阶段、低风险）

Phase 0（1-2 天）：

- 后端：为关键端点响应体增加 `correlation_id`；`CompleteTaskResponse` 可选返回 `updated_time_blocks`；
- 前端：在命令后优先使用响应体对象更新，减少补拉；为将来事件 Reducer 预留接口。

Phase 1（3-5 天）：

- 后端：实现 outbox 与 `/api/events` SSE（基础可靠投递、心跳、断线续传）；
- 前端：实现事件订阅适配器，但先仅日志观察，不改 UI（灰度）。

Phase 2（5-8 天）：

- 前端：实现 Task/TimeBlock Reducer 并灰度打开；覆盖“完成任务→时间块截断/删除”等链路；
- 集成测试：双窗口一致性、断网重连、幂等去重。

Phase 3（3-5 天）：

- 扩展事件覆盖面（task.updated、time_block.updated/created/deleted 等）；
- 观测与调优（节流/合并）。

回滚预案：

- 关闭 SSE 订阅与事件应用，保留现有 HTTP 行为即可；outbox 与端点互不影响，安全回退。

## 8. 测试策略

- 单元测试：
  - 事件生成（完成任务、截断/删除块）与 outbox 写入；
  - Reducer 幂等应用（顺序乱序、重复事件）。
- 集成测试：
  - API → DB → outbox → SSE 流全链路；
  - 双窗口场景，命令窗口与观察窗口的最终一致性。
- 可靠性：
  - 断线重连、`Last-Event-ID`、事件丢失与重放。

## 9. 运维与安全

- 连接数与背压：限制每客户端连接数，批量/合并高频事件，压缩可后续评估（SSE 不天然压缩，需反向代理层处理）。
- 事件留存：outbox 保留最近 N 分钟/小时事件即可，定期清理；关键事件可归档。
- 监控：SSE 连接数、平均滞后、重试次数、事件吞吐、丢弃率；命令-事件延迟直方图。

## 10. 与现状对比与收益

- 可演进性：副作用新增/调整无需改动命令响应体与 UI 主流程；
- 一致性：多窗口/多端通过事件广播达成最终一致；
- 降低耦合：把“变化面”放进事件层，契约版本可控，支持灰度与回滚；
- 渐进替换：无需一次性迁移到 WS，后续如需双向高频再引入。

## 11. 工作量评估（粗略）

- 后端：SSE 端点 + outbox + dispatcher + 事件落地与发布（3-5 人日）。
- 前端：事件订阅适配器 + Reducer（Task/TimeBlock）+ 双窗口集成（3-5 人日）。
- 测试与灰度：3-5 人日。

## 12. 附录：示例事件载荷

`task.completed@v1`

```json
{
  "id": "evt_1000123",
  "type": "task.completed",
  "version": 1,
  "occurred_at": "2025-10-01T08:00:00Z",
  "aggregate_type": "task",
  "aggregate_id": "a1b2...",
  "aggregate_version": 42,
  "correlation_id": "cmd_98765",
  "payload": {
    "task": {
      /* TaskCardDto */
    }
  }
}
```

`time_blocks.truncated@v1`

```json
{
  "id": "evt_1000124",
  "type": "time_blocks.truncated",
  "version": 1,
  "occurred_at": "2025-10-01T08:00:00Z",
  "aggregate_type": "time_block",
  "aggregate_id": "-",
  "aggregate_version": 0,
  "correlation_id": "cmd_98765",
  "payload": {
    "blocks": [
      /* TimeBlockViewDto[]（截断后的完整体）*/
    ]
  }
}
```

---

实施建议：先以完成任务链路为试点落地（最小可行集：`task.completed` + `time_blocks.truncated/deleted`），稳定后扩展到更新、删除、创建等路径，逐步替换前端的“区间补拉”与耦合逻辑。
