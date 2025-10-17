# CPU DevTools 设计与实现文档

## 概述

CPU DevTools 是为 Cutie 项目 CPU 架构设计的专业调试和性能监控工具。基于现有强大的 CPU 追踪系统，提供实时的指令执行可视化、性能分析和问题诊断能力。

## 目录

- [现有追踪系统分析](#现有追踪系统分析)
- [DevTools架构设计](#devtools架构设计)
- [集成方案](#集成方案)
- [核心组件实现](#核心组件实现)
- [使用指南](#使用指南)
- [最佳实践](#最佳实践)

---

## 现有追踪系统分析

### 追踪能力概览

Cutie 项目已具备完善的 CPU 追踪基础设施：

#### ✅ 完整的事件收集 (CPUEventCollector)
- **指令生命周期追踪**: Created → Issued → Executing → Responded → Committed/Failed
- **网络请求追踪**: Request sent → Response received (包含延迟、状态码、大小)
- **资源冲突追踪**: 冲突检测、等待时间、冲突资源
- **乐观更新追踪**: 应用 → 回滚事件
- **性能警告**: 自动检测延迟超阈值

#### ✅ 强大的索引系统 (CPULogger)
- 按指令ID索引 (`eventsByInstruction`)
- 按correlation ID索引 (`eventsByCorrelation`)
- 按事件类型索引 (`eventsByType`)
- 多维度查询API (`query()`)

#### ✅ 分析能力
- 指令性能统计 (成功率、延迟分布、P50/P95/P99)
- 资源冲突热点分析
- 乐观更新回滚率分析
- 流水线吞吐量分析

### 现有数据结构

```typescript
// 核心事件结构
interface CPUEvent {
  eventId: string
  eventType: CPUEventType
  timestamp: number
  instructionId: string
  instructionType: string
  correlationId: string
  pipelineStage: PipelineStage
  instructionStatus: InstructionStatus
  latency?: number
  duration?: number
  payload: any
  metadata?: {
    resourceIds?: string[]
    priority?: number
    retryCount?: number
    tags?: string[]
  }
}

// 事件类型
enum CPUEventType {
  INSTRUCTION_CREATED = 'instruction.created',
  INSTRUCTION_ISSUED = 'instruction.issued',
  INSTRUCTION_EXECUTING = 'instruction.executing',
  INSTRUCTION_RESPONDED = 'instruction.responded',
  INSTRUCTION_COMMITTED = 'instruction.committed',
  INSTRUCTION_FAILED = 'instruction.failed',
  NETWORK_REQUEST_SENT = 'network.request_sent',
  NETWORK_RESPONSE_RECEIVED = 'network.response_received',
  SCHEDULER_CONFLICT_DETECTED = 'scheduler.conflict_detected',
  OPTIMISTIC_APPLIED = 'optimistic.applied',
  OPTIMISTIC_ROLLED_BACK = 'optimistic.rolled_back',
  PERFORMANCE_WARNING = 'performance.warning'
}
```

---

## DevTools架构设计

### 整体架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        CPU DevTools                            │
├─────────────────────────────────────────────────────────────────┤
│ 用户界面层                                                       │
│ ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐    │
│ │   瀑布图组件     │ │   统计面板      │ │   详情面板      │    │
│ └─────────────────┘ └─────────────────┘ └─────────────────┘    │
├─────────────────────────────────────────────────────────────────┤
│ 数据处理层                                                       │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │            CPUDevToolsDataProvider                         │ │
│ └─────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│ 追踪系统层 (现有)                                               │
│ ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐    │
│ │ CPUEventCollector│ │    CPULogger    │ │   CPU Pipeline  │    │
│ └─────────────────┘ └─────────────────┘ └─────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

### 核心数据类型

```typescript
/**
 * 瀑布图指令数据
 */
interface WaterfallInstruction {
  instructionId: string
  instructionType: string
  correlationId: string
  status: 'success' | 'failed' | 'pending'

  // 时间信息
  submitTime: number
  completionTime?: number
  totalDuration: number

  // 阶段时序
  stages: StageTimingInfo[]

  // 网络请求
  networkRequests: NetworkRequestInfo[]

  // 资源冲突
  conflicts: ConflictInfo[]

  // 元数据
  callSource?: any
  payload: any
  error?: string
}

/**
 * 阶段时序信息
 */
interface StageTimingInfo {
  stage: PipelineStage
  startTime: number
  endTime: number
  duration: number
  status: 'success' | 'failed' | 'pending'
}

/**
 * 网络请求信息
 */
interface NetworkRequestInfo {
  method: string
  url: string
  startTime: number
  endTime?: number
  duration?: number
  status?: number
  size?: number
  latency?: number
}

/**
 * 冲突信息
 */
interface ConflictInfo {
  timestamp: number
  resources: string[]
  conflictingInstructions: string[]
  waitTime: number
}

/**
 * 统计面板数据
 */
interface StatsPanelData {
  overview: {
    totalInstructions: number
    totalEvents: number
    successRate: number
    avgLatency: number
  }
  performance: {
    instructionsPerSecond: number
    eventsPerSecond: number
    pipelineUtilization: number
  }
  conflicts: Array<{
    resource: string
    conflictCount: number
    avgWaitTime: number
    involvedInstructions: string[]
  }>
  optimistic: {
    rollbackRate: number
    byInstructionType: Record<string, { total: number; rollbacks: number; rate: number }>
  }
}

/**
 * 指令详情数据
 */
interface InstructionDetails {
  instruction: WaterfallInstruction
  fullTrace: CPUEvent[]
  correlatedInstructions: string[]
  timeline: TimelineEntry[]
}

/**
 * 时间线条目
 */
interface TimelineEntry {
  timestamp: number
  eventType: CPUEventType
  stage: PipelineStage
  status: InstructionStatus
  duration?: number
  latency?: number
  description: string
  payload: any
  metadata?: any
}
```

---

## 集成方案

### 方案1：内嵌路由（推荐开发阶段）

直接在主应用中添加 DevTools 页面，便于开发时快速访问。

#### 路由配置
```typescript
// src/router/index.ts
import CPUDevToolsView from '@/views/CPUDevToolsView.vue'

const routes = [
  // ... 现有路由
  {
    path: '/cpu-devtools',
    name: 'CPUDevTools',
    component: CPUDevToolsView,
    meta: {
      title: 'CPU DevTools',
      requiresDev: true
    }
  }
]
```

#### 视图组件
```vue
<!-- src/views/CPUDevToolsView.vue -->
<template>
  <div class="cpu-devtools-view">
    <CPUDevTools />
  </div>
</template>

<script setup lang="ts">
import CPUDevTools from '@/components/dev/CPUDevTools.vue'
document.title = 'CPU DevTools - Cutie'
</script>

<style scoped>
.cpu-devtools-view {
  height: 100vh;
  overflow: hidden;
}
</style>
```

#### 导航集成
```vue
<!-- 在主导航中添加DevTools入口 -->
<template>
  <nav class="main-navigation">
    <!-- 现有导航项 -->
    <router-link to="/daily">日程</router-link>
    <router-link to="/staging">暂存</router-link>

    <!-- 开发工具入口（仅开发模式显示） -->
    <div v-if="isDevelopment" class="dev-tools-section">
      <hr class="nav-divider">
      <router-link to="/cpu-debug" class="dev-link">CPU调试</router-link>
      <router-link to="/cpu-devtools" class="dev-link">CPU DevTools</router-link>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const isDevelopment = computed(() => {
  return process.env.NODE_ENV === 'development' ||
         window.location.search.includes('dev=true')
})
</script>
```

### 方案2：独立DevTools窗口（推荐生产调试）

利用 Tauri 多窗口功能创建专用调试窗口，不干扰主应用界面。

#### Rust 后端实现
```rust
// src-tauri/src/main.rs
use tauri::{Window, Manager, WindowBuilder, WindowUrl};

#[tauri::command]
async fn open_devtools_window(app: tauri::AppHandle) -> Result<(), String> {
    // 检查DevTools窗口是否已存在
    if let Some(_) = app.get_window("devtools") {
        if let Some(window) = app.get_window("devtools") {
            window.set_focus().map_err(|e| e.to_string())?;
        }
        return Ok(());
    }

    // 创建新的DevTools窗口
    let devtools_window = WindowBuilder::new(
        &app,
        "devtools",
        WindowUrl::App("/cpu-devtools".into())
    )
    .title("CPU DevTools")
    .inner_size(1200.0, 800.0)
    .min_inner_size(800.0, 600.0)
    .resizable(true)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // ... 现有命令
            open_devtools_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

#### 前端窗口管理
```typescript
// src/services/devtools.ts
import { invoke } from '@tauri-apps/api/tauri'

export class DevToolsManager {
  /**
   * 打开DevTools窗口
   */
  static async openDevToolsWindow(): Promise<void> {
    try {
      await invoke('open_devtools_window')
    } catch (error) {
      console.error('Failed to open DevTools window:', error)
    }
  }

  /**
   * 检查是否在DevTools窗口中
   */
  static isDevToolsWindow(): boolean {
    return window.location.pathname === '/cpu-devtools'
  }
}
```

#### 快捷键支持
```vue
<!-- src/components/layout/GlobalShortcuts.vue -->
<template>
  <div @keydown="handleKeyDown" tabindex="-1" style="outline: none;">
    <slot />
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { DevToolsManager } from '@/services/devtools'

const handleKeyDown = (event: KeyboardEvent) => {
  // Ctrl/Cmd + Shift + D 打开DevTools
  if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === 'D') {
    event.preventDefault()
    DevToolsManager.openDevToolsWindow()
  }

  // F12 打开DevTools（传统习惯）
  if (event.key === 'F12') {
    event.preventDefault()
    DevToolsManager.openDevToolsWindow()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown)
})
</script>
```

### 方案3：浮动DevTools面板（推荐日常使用）

可折叠的悬浮 DevTools 面板，提供快速监控能力。

#### 浮动面板组件
```vue
<!-- src/components/dev/FloatingDevTools.vue -->
<template>
  <Teleport to="body">
    <div
      v-if="isVisible"
      class="floating-devtools"
      :class="{ collapsed: isCollapsed, docked: isDocked }"
      :style="panelStyle"
      @mousedown="startDrag"
    >
      <!-- 标题栏 -->
      <div class="devtools-header">
        <div class="devtools-title">
          <span class="title-text">CPU DevTools</span>
          <div class="cpu-status-indicator" :class="cpuStatus"></div>
        </div>

        <div class="devtools-controls">
          <button @click="toggleCollapse" class="control-btn">
            {{ isCollapsed ? '📈' : '📉' }}
          </button>
          <button @click="toggleDock" class="control-btn">
            {{ isDocked ? '🪟' : '📌' }}
          </button>
          <button @click="openFullDevTools" class="control-btn">🔧</button>
          <button @click="close" class="control-btn close-btn">✕</button>
        </div>
      </div>

      <!-- 内容区域 -->
      <div v-show="!isCollapsed" class="devtools-content">
        <!-- 迷你统计面板 -->
        <div class="mini-stats">
          <div class="stat-row">
            <span class="stat-label">指令/秒:</span>
            <span class="stat-value">{{ stats.instructionsPerSecond.toFixed(1) }}</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">成功率:</span>
            <span class="stat-value">{{ (stats.successRate * 100).toFixed(1) }}%</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">冲突:</span>
            <span class="stat-value conflict-count">{{ stats.conflicts }}</span>
          </div>
        </div>

        <!-- 最近指令列表 -->
        <div class="recent-instructions">
          <div class="section-title">最近指令</div>
          <div
            v-for="instruction in recentInstructions"
            :key="instruction.id"
            class="instruction-item"
            :class="instruction.status"
            @click="selectInstruction(instruction)"
          >
            <span class="instruction-type">{{ instruction.type }}</span>
            <span class="instruction-duration">{{ instruction.duration }}ms</span>
            <span class="instruction-status">{{ getStatusIcon(instruction.status) }}</span>
          </div>
        </div>

        <!-- 快速操作 -->
        <div class="quick-actions">
          <button @click="clearLogs" class="action-btn">清空日志</button>
          <button @click="exportLogs" class="action-btn">导出</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { DevToolsManager } from '@/services/devtools'
import { CPUDevToolsDataProvider } from './CPUDevToolsDataProvider'
import { cpuLogger } from '@/cpu/logging'

// 面板状态
const isVisible = ref(false)
const isCollapsed = ref(false)
const isDocked = ref(false)
const position = ref({ x: 20, y: 20 })

// 数据
const dataProvider = new CPUDevToolsDataProvider(cpuLogger)
const stats = ref({
  instructionsPerSecond: 0,
  successRate: 0,
  conflicts: 0
})
const recentInstructions = ref([])

// 计算属性
const panelStyle = computed(() => {
  if (isDocked.value) {
    return {
      position: 'fixed',
      top: '0',
      right: '0'
    }
  }

  return {
    position: 'fixed',
    left: `${position.value.x}px`,
    top: `${position.value.y}px`
  }
})

const cpuStatus = computed(() => {
  if (stats.value.instructionsPerSecond > 10) return 'busy'
  if (stats.value.instructionsPerSecond > 1) return 'active'
  return 'idle'
})

// 方法
const show = () => {
  isVisible.value = true
  startDataRefresh()
}

const close = () => {
  isVisible.value = false
  stopDataRefresh()
}

const refreshData = () => {
  const statsData = dataProvider.getStatsPanelData()
  stats.value = {
    instructionsPerSecond: statsData.performance.instructionsPerSecond,
    successRate: statsData.overview.successRate,
    conflicts: statsData.conflicts.length
  }

  const waterfallData = dataProvider.getWaterfallData({
    sortBy: 'completionTime',
    maxInstructions: 10
  })

  recentInstructions.value = waterfallData.map(instr => ({
    id: instr.instructionId,
    type: instr.instructionType,
    duration: instr.totalDuration,
    status: instr.status
  }))
}

let refreshTimer: number

const startDataRefresh = () => {
  refreshData()
  refreshTimer = setInterval(refreshData, 1000)
}

const stopDataRefresh = () => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
}

// 暴露方法给外部调用
defineExpose({
  show,
  close,
  toggle: () => isVisible.value ? close() : show()
})

onMounted(() => {
  document.addEventListener('keydown', (event) => {
    // Ctrl/Cmd + ` 切换DevTools面板
    if ((event.ctrlKey || event.metaKey) && event.key === '`') {
      event.preventDefault()
      if (isVisible.value) {
        close()
      } else {
        show()
      }
    }
  })
})

onUnmounted(() => {
  stopDataRefresh()
})
</script>

<style scoped>
.floating-devtools {
  width: 300px;
  background: rgba(30, 30, 30, 0.95);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 11px;
  color: #fff;
  z-index: 10000;
  user-select: none;
  transition: all 0.3s ease;
}

.floating-devtools.collapsed {
  height: 32px;
}

.floating-devtools.docked {
  border-radius: 0 0 0 8px;
  border-top: none;
  border-right: none;
}

.devtools-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 8px;
  background: rgba(0, 0, 0, 0.3);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  cursor: move;
}

.cpu-status-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.cpu-status-indicator.idle { background: #4CAF50; }
.cpu-status-indicator.active { background: #FF9800; }
.cpu-status-indicator.busy { background: #f44336; }

.devtools-content {
  padding: 8px;
  max-height: 400px;
  overflow-y: auto;
}

.stat-row {
  display: flex;
  justify-content: space-between;
  padding: 2px 0;
}

.instruction-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 3px 6px;
  margin: 2px 0;
  border-radius: 3px;
  cursor: pointer;
}

.instruction-item.success {
  border-left: 2px solid #4CAF50;
}

.instruction-item.failed {
  border-left: 2px solid #f44336;
}
</style>
```

---

## 核心组件实现

### 数据提供者

```typescript
/**
 * CPU DevTools 数据处理层
 *
 * 职责：
 * 1. 基于CPULogger提供DevTools所需的数据格式
 * 2. 实现瀑布图、统计面板、详情面板的数据转换
 * 3. 提供实时数据查询和分析能力
 */
export class CPUDevToolsDataProvider {
  constructor(private cpuLogger: CPULogger) {}

  /**
   * 获取瀑布图数据
   */
  getWaterfallData(options: {
    sortBy: 'submitTime' | 'completionTime'
    timeRange?: { start: number; end: number }
    instructionType?: string
    maxInstructions?: number
  }): WaterfallInstruction[] {
    // 获取所有指令的完整事件链
    let events = this.cpuLogger.query({
      eventType: CPUEventType.INSTRUCTION_CREATED,
      timeRange: options.timeRange,
      instructionType: options.instructionType
    })

    // 按排序方式排序
    if (options.sortBy === 'submitTime') {
      events.sort((a, b) => a.timestamp - b.timestamp)
    } else {
      // 需要获取完成时间来排序
      events = events.filter(e => {
        const trace = this.cpuLogger.getInstructionTrace(e.instructionId)
        const hasCompletion = trace.some(evt =>
          evt.eventType === CPUEventType.INSTRUCTION_COMMITTED ||
          evt.eventType === CPUEventType.INSTRUCTION_FAILED
        )
        return hasCompletion
      }).sort((a, b) => {
        const traceA = this.cpuLogger.getInstructionTrace(a.instructionId)
        const traceB = this.cpuLogger.getInstructionTrace(b.instructionId)
        const completionA = this.getCompletionTime(traceA)
        const completionB = this.getCompletionTime(traceB)
        return completionA - completionB
      })
    }

    if (options.maxInstructions) {
      events = events.slice(0, options.maxInstructions)
    }

    // 构建瀑布图数据
    return events.map(createEvent => {
      const trace = this.cpuLogger.getInstructionTrace(createEvent.instructionId)
      return this.buildWaterfallInstruction(trace)
    })
  }

  /**
   * 构建单个指令的瀑布图数据
   */
  private buildWaterfallInstruction(events: CPUEvent[]): WaterfallInstruction {
    const stages = this.calculateStageTimings(events)
    const networkRequests = this.extractNetworkRequests(events)
    const conflicts = this.extractConflicts(events)

    const createEvent = events.find(e => e.eventType === CPUEventType.INSTRUCTION_CREATED)!
    const completionEvent = events.find(e =>
      e.eventType === CPUEventType.INSTRUCTION_COMMITTED ||
      e.eventType === CPUEventType.INSTRUCTION_FAILED
    )

    return {
      instructionId: createEvent.instructionId,
      instructionType: createEvent.instructionType,
      correlationId: createEvent.correlationId,
      status: completionEvent?.eventType === CPUEventType.INSTRUCTION_COMMITTED ? 'success' : 'failed',

      // 时间信息
      submitTime: createEvent.timestamp,
      completionTime: completionEvent?.timestamp || Date.now(),
      totalDuration: (completionEvent?.timestamp || Date.now()) - createEvent.timestamp,

      // 阶段时序
      stages,

      // 网络请求
      networkRequests,

      // 资源冲突
      conflicts,

      // 元数据
      callSource: createEvent.payload.callSource,
      payload: createEvent.payload,
      error: completionEvent?.eventType === CPUEventType.INSTRUCTION_FAILED ?
        completionEvent.payload.error : undefined
    }
  }

  /**
   * 计算各阶段时间
   */
  private calculateStageTimings(events: CPUEvent[]): StageTimingInfo[] {
    const stageMap = new Map<PipelineStage, { start: number; end?: number }>()

    // 找到各阶段的开始和结束时间
    for (const event of events) {
      const stage = event.pipelineStage

      if (!stageMap.has(stage)) {
        stageMap.set(stage, { start: event.timestamp })
      } else {
        stageMap.get(stage)!.end = event.timestamp
      }
    }

    return Array.from(stageMap.entries()).map(([stage, timing]) => ({
      stage,
      startTime: timing.start,
      endTime: timing.end || Date.now(),
      duration: (timing.end || Date.now()) - timing.start,
      status: this.getStageStatus(events, stage)
    }))
  }

  /**
   * 提取网络请求信息
   */
  private extractNetworkRequests(events: CPUEvent[]): NetworkRequestInfo[] {
    const requests: NetworkRequestInfo[] = []
    const requestMap = new Map<string, { request?: CPUEvent; response?: CPUEvent }>()

    for (const event of events) {
      if (event.eventType === CPUEventType.NETWORK_REQUEST_SENT) {
        const key = `${event.payload.method}:${event.payload.url}`
        if (!requestMap.has(key)) {
          requestMap.set(key, {})
        }
        requestMap.get(key)!.request = event
      } else if (event.eventType === CPUEventType.NETWORK_RESPONSE_RECEIVED) {
        // 需要根据URL匹配请求
        for (const [key, entry] of requestMap.entries()) {
          if (entry.request && !entry.response) {
            entry.response = event
            break
          }
        }
      }
    }

    for (const [key, { request, response }] of requestMap.entries()) {
      if (request) {
        requests.push({
          method: request.payload.method,
          url: request.payload.url,
          startTime: request.timestamp,
          endTime: response?.timestamp,
          duration: response ? response.timestamp - request.timestamp : undefined,
          status: response?.payload.status,
          size: response?.payload.size,
          latency: response?.payload.latency
        })
      }
    }

    return requests
  }

  /**
   * 提取冲突信息
   */
  private extractConflicts(events: CPUEvent[]): ConflictInfo[] {
    return events
      .filter(e => e.eventType === CPUEventType.SCHEDULER_CONFLICT_DETECTED)
      .map(event => ({
        timestamp: event.timestamp,
        resources: event.payload.conflictingResources,
        conflictingInstructions: event.payload.conflictingInstructions,
        waitTime: event.payload.waitTime
      }))
  }

  /**
   * 获取统计面板数据
   */
  getStatsPanelData(): StatsPanelData {
    const stats = this.cpuLogger.getStats()
    const throughput = this.cpuLogger.analyzeThroughput()
    const conflicts = this.cpuLogger.analyzeResourceConflicts()
    const rollbacks = this.cpuLogger.analyzeOptimisticRollbackRate()

    return {
      overview: {
        totalInstructions: stats.totalInstructions,
        totalEvents: stats.totalEvents,
        successRate: this.calculateOverallSuccessRate(),
        avgLatency: this.calculateOverallAvgLatency()
      },
      performance: {
        instructionsPerSecond: throughput.instructionsPerSecond,
        eventsPerSecond: throughput.eventsPerSecond,
        pipelineUtilization: throughput.avgPipelineUtilization
      },
      conflicts: conflicts.slice(0, 10), // Top 10 conflict hotspots
      optimistic: {
        rollbackRate: rollbacks.rollbackRate,
        byInstructionType: rollbacks.byInstructionType
      }
    }
  }

  /**
   * 获取指令详情
   */
  getInstructionDetails(instructionId: string): InstructionDetails {
    const trace = this.cpuLogger.getInstructionTrace(instructionId)
    const correlation = this.cpuLogger.getCorrelationTrace(trace[0]?.correlationId || '')

    return {
      instruction: this.buildWaterfallInstruction(trace),
      fullTrace: trace,
      correlatedInstructions: correlation.map(e => e.instructionId).filter(id => id !== instructionId),
      timeline: this.buildDetailedTimeline(trace)
    }
  }

  /**
   * 构建详细时间线
   */
  private buildDetailedTimeline(events: CPUEvent[]): TimelineEntry[] {
    return events.map(event => ({
      timestamp: event.timestamp,
      eventType: event.eventType,
      stage: event.pipelineStage,
      status: event.instructionStatus,
      duration: event.duration,
      latency: event.latency,
      description: this.getEventDescription(event),
      payload: event.payload,
      metadata: event.metadata
    }))
  }

  private getEventDescription(event: CPUEvent): string {
    switch (event.eventType) {
      case CPUEventType.INSTRUCTION_CREATED:
        return `指令创建: ${event.instructionType}`
      case CPUEventType.INSTRUCTION_ISSUED:
        return '指令发射到执行单元'
      case CPUEventType.INSTRUCTION_EXECUTING:
        return '开始执行指令'
      case CPUEventType.NETWORK_REQUEST_SENT:
        return `发送 ${event.payload.method} 请求: ${event.payload.url}`
      case CPUEventType.NETWORK_RESPONSE_RECEIVED:
        return `收到响应 (${event.payload.status}): ${event.payload.latency}ms`
      case CPUEventType.SCHEDULER_CONFLICT_DETECTED:
        return `资源冲突: ${event.payload.conflictingResources.join(', ')}`
      case CPUEventType.OPTIMISTIC_APPLIED:
        return '应用乐观更新'
      case CPUEventType.OPTIMISTIC_ROLLED_BACK:
        return `回滚乐观更新: ${event.payload.reason}`
      case CPUEventType.INSTRUCTION_COMMITTED:
        return '指令提交成功'
      case CPUEventType.INSTRUCTION_FAILED:
        return `指令失败: ${event.payload.error}`
      default:
        return event.eventType
    }
  }

  private calculateOverallSuccessRate(): number {
    // 实现整体成功率计算
    const stats = this.cpuLogger.getStats()
    return stats.totalCompleted / (stats.totalCompleted + stats.totalFailed) || 0
  }

  private calculateOverallAvgLatency(): number {
    // 实现整体平均延迟计算
    return 0 // 简化实现
  }

  private getCompletionTime(events: CPUEvent[]): number {
    const completion = events.find(e =>
      e.eventType === CPUEventType.INSTRUCTION_COMMITTED ||
      e.eventType === CPUEventType.INSTRUCTION_FAILED
    )
    return completion?.timestamp || Date.now()
  }

  private getStageStatus(events: CPUEvent[], stage: PipelineStage): 'success' | 'failed' | 'pending' {
    const stageEvents = events.filter(e => e.pipelineStage === stage)
    if (stageEvents.some(e => e.eventType === CPUEventType.INSTRUCTION_FAILED)) {
      return 'failed'
    }
    const hasCompletion = stageEvents.some(e =>
      e.eventType === CPUEventType.INSTRUCTION_COMMITTED ||
      e.eventType === CPUEventType.INSTRUCTION_RESPONDED
    )
    return hasCompletion ? 'success' : 'pending'
  }
}
```

### 主DevTools组件

```vue
<!-- src/components/dev/CPUDevTools.vue -->
<template>
  <div class="cpu-devtools">
    <!-- 工具栏 -->
    <div class="devtools-toolbar">
      <div class="controls">
        <button @click="toggleRecording" :class="{ active: isRecording }">
          {{ isRecording ? '停止记录' : '开始记录' }}
        </button>
        <button @click="clearData">清空数据</button>
        <button @click="exportData">导出数据</button>
      </div>

      <div class="filters">
        <select v-model="sortBy">
          <option value="submitTime">按提交时间排序</option>
          <option value="completionTime">按完成时间排序</option>
        </select>

        <select v-model="instructionTypeFilter">
          <option value="">所有指令类型</option>
          <option v-for="type in availableTypes" :key="type" :value="type">
            {{ type }}
          </option>
        </select>

        <input
          v-model="maxInstructions"
          type="number"
          placeholder="最大显示数量"
          min="1"
          max="1000"
        >
      </div>
    </div>

    <!-- 主要内容区域 -->
    <div class="devtools-content">
      <!-- 统计面板 -->
      <div class="stats-panel">
        <div class="stat-group">
          <h3>概览</h3>
          <div class="stats-grid">
            <div class="stat-item">
              <span class="label">总指令数</span>
              <span class="value">{{ statsData.overview.totalInstructions }}</span>
            </div>
            <div class="stat-item">
              <span class="label">成功率</span>
              <span class="value">{{ (statsData.overview.successRate * 100).toFixed(1) }}%</span>
            </div>
            <div class="stat-item">
              <span class="label">平均延迟</span>
              <span class="value">{{ statsData.overview.avgLatency.toFixed(1) }}ms</span>
            </div>
            <div class="stat-item">
              <span class="label">吞吐量</span>
              <span class="value">{{ statsData.performance.instructionsPerSecond.toFixed(1) }}/s</span>
            </div>
          </div>
        </div>

        <div class="stat-group">
          <h3>冲突热点</h3>
          <div class="conflict-list">
            <div
              v-for="conflict in statsData.conflicts.slice(0, 5)"
              :key="conflict.resource"
              class="conflict-item"
            >
              <span class="resource">{{ conflict.resource }}</span>
              <span class="count">{{ conflict.conflictCount }}次</span>
              <span class="wait">{{ conflict.avgWaitTime.toFixed(1) }}ms</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 瀑布图 -->
      <div class="waterfall-container">
        <div class="waterfall-header">
          <div class="timeline-ruler">
            <!-- 时间轴刻度 -->
            <div
              v-for="tick in timelineTicks"
              :key="tick.timestamp"
              class="timeline-tick"
              :style="{ left: tick.position + '%' }"
            >
              {{ formatTime(tick.timestamp) }}
            </div>
          </div>
        </div>

        <div class="waterfall-body">
          <div
            v-for="instruction in waterfallData"
            :key="instruction.instructionId"
            class="instruction-row"
            @click="selectInstruction(instruction)"
            :class="{ selected: selectedInstruction?.instructionId === instruction.instructionId }"
          >
            <!-- 指令基本信息 -->
            <div class="instruction-info">
              <div class="instruction-type">{{ instruction.instructionType }}</div>
              <div class="instruction-id">{{ instruction.instructionId.slice(-8) }}</div>
              <div class="instruction-duration">{{ instruction.totalDuration }}ms</div>
            </div>

            <!-- 时间线图表 -->
            <div class="instruction-timeline">
              <!-- 各阶段条形图 -->
              <div
                v-for="stage in instruction.stages"
                :key="stage.stage"
                class="stage-bar"
                :class="[`stage-${stage.stage}`, `status-${stage.status}`]"
                :style="getStageBarStyle(stage, instruction)"
                :title="`${stage.stage}: ${stage.duration}ms`"
              ></div>

              <!-- 网络请求标记 -->
              <div
                v-for="request in instruction.networkRequests"
                :key="request.url"
                class="network-marker"
                :style="getNetworkMarkerStyle(request, instruction)"
                :title="`${request.method} ${request.url}: ${request.latency || '?'}ms`"
              ></div>

              <!-- 冲突标记 -->
              <div
                v-for="conflict in instruction.conflicts"
                :key="conflict.timestamp"
                class="conflict-marker"
                :style="getConflictMarkerStyle(conflict, instruction)"
                :title="`资源冲突: ${conflict.resources.join(', ')}`"
              ></div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 详情面板 -->
    <div v-if="selectedInstruction" class="details-panel">
      <InstructionDetailsPanel
        :instruction="selectedInstruction"
        :details="selectedInstructionDetails"
        @close="selectedInstruction = null"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { CPUDevToolsDataProvider } from './CPUDevToolsDataProvider'
import { cpuLogger } from '@/cpu/logging'
import InstructionDetailsPanel from './InstructionDetailsPanel.vue'

// 数据提供者
const dataProvider = new CPUDevToolsDataProvider(cpuLogger)

// 响应式数据
const isRecording = ref(true)
const sortBy = ref<'submitTime' | 'completionTime'>('submitTime')
const instructionTypeFilter = ref('')
const maxInstructions = ref(100)

const waterfallData = ref<WaterfallInstruction[]>([])
const statsData = ref<StatsPanelData>()
const selectedInstruction = ref<WaterfallInstruction | null>(null)
const selectedInstructionDetails = ref<InstructionDetails | null>(null)

// 计算属性
const availableTypes = computed(() => {
  const types = new Set<string>()
  waterfallData.value.forEach(instr => types.add(instr.instructionType))
  return Array.from(types).sort()
})

const timelineTicks = computed(() => {
  if (waterfallData.value.length === 0) return []

  const minTime = Math.min(...waterfallData.value.map(i => i.submitTime))
  const maxTime = Math.max(...waterfallData.value.map(i => i.completionTime || i.submitTime))
  const duration = maxTime - minTime

  const ticks = []
  const tickCount = 10
  for (let i = 0; i <= tickCount; i++) {
    const timestamp = minTime + (duration * i / tickCount)
    const position = (timestamp - minTime) / duration * 100
    ticks.push({ timestamp, position })
  }

  return ticks
})

// 方法
const refreshData = () => {
  if (!isRecording.value) return

  waterfallData.value = dataProvider.getWaterfallData({
    sortBy: sortBy.value,
    instructionType: instructionTypeFilter.value || undefined,
    maxInstructions: maxInstructions.value
  })

  statsData.value = dataProvider.getStatsPanelData()
}

const selectInstruction = (instruction: WaterfallInstruction) => {
  selectedInstruction.value = instruction
  selectedInstructionDetails.value = dataProvider.getInstructionDetails(instruction.instructionId)
}

const getStageBarStyle = (stage: StageTimingInfo, instruction: WaterfallInstruction) => {
  if (waterfallData.value.length === 0) return {}

  const minTime = Math.min(...waterfallData.value.map(i => i.submitTime))
  const maxTime = Math.max(...waterfallData.value.map(i => i.completionTime || i.submitTime))
  const totalDuration = maxTime - minTime

  const left = ((stage.startTime - minTime) / totalDuration) * 100
  const width = (stage.duration / totalDuration) * 100

  return {
    left: `${left}%`,
    width: `${width}%`
  }
}

const formatTime = (timestamp: number) => {
  const date = new Date(timestamp)
  return `${date.getMinutes()}:${date.getSeconds().toString().padStart(2, '0')}`
}

// 生命周期
let refreshTimer: number

onMounted(() => {
  refreshData()
  refreshTimer = setInterval(refreshData, 1000) // 每秒刷新
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
})

// 监听器
watch([sortBy, instructionTypeFilter, maxInstructions], refreshData)
</script>

<style scoped>
.cpu-devtools {
  display: flex;
  flex-direction: column;
  height: 100vh;
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 12px;
  background: #1a1a1a;
  color: #e0e0e0;
}

.devtools-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  border-bottom: 1px solid #333;
  background: #2a2a2a;
}

.controls {
  display: flex;
  gap: 8px;
}

.controls button {
  padding: 4px 8px;
  border: 1px solid #555;
  background: #333;
  color: #e0e0e0;
  border-radius: 3px;
  cursor: pointer;
}

.controls button.active {
  background: #007acc;
  border-color: #007acc;
}

.waterfall-container {
  flex: 1;
  overflow: auto;
  border: 1px solid #333;
}

.instruction-row {
  display: flex;
  border-bottom: 1px solid #333;
  cursor: pointer;
  transition: background-color 0.2s;
}

.instruction-row:hover {
  background-color: #333;
}

.instruction-row.selected {
  background-color: #2d4a6b;
}

.instruction-info {
  flex: 0 0 200px;
  padding: 8px;
  border-right: 1px solid #333;
}

.instruction-timeline {
  flex: 1;
  position: relative;
  height: 40px;
}

.stage-bar {
  position: absolute;
  height: 20px;
  top: 10px;
  border-radius: 2px;
  opacity: 0.8;
}

.stage-IF { background-color: #4CAF50; }
.stage-SCH { background-color: #FF9800; }
.stage-EX { background-color: #2196F3; }
.stage-RES { background-color: #9C27B0; }
.stage-WB { background-color: #607D8B; }

.status-failed {
  background-color: #f44336 !important;
}

.network-marker {
  position: absolute;
  width: 2px;
  height: 30px;
  top: 5px;
  background-color: #FF5722;
}

.conflict-marker {
  position: absolute;
  width: 6px;
  height: 6px;
  top: 17px;
  background-color: #FFEB3B;
  border-radius: 50%;
  border: 1px solid #FFC107;
}

.stats-panel {
  display: flex;
  gap: 20px;
  padding: 16px;
  background: #262626;
  border-bottom: 1px solid #333;
}

.stat-group h3 {
  margin: 0 0 8px 0;
  color: #ccc;
  font-size: 14px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stat-item .label {
  font-size: 10px;
  color: #999;
  text-transform: uppercase;
}

.stat-item .value {
  font-size: 14px;
  font-weight: bold;
  color: #e0e0e0;
}
</style>
```

---

## 使用指南

### 开发环境

#### 启用DevTools
```typescript
// 方式1：开发模式自动启用
if (process.env.NODE_ENV === 'development') {
  // DevTools自动可用
}

// 方式2：通过localStorage启用
localStorage.setItem('enable-devtools', 'true')

// 方式3：通过URL参数启用
window.location.search = '?dev=true'
```

#### 快捷键
- **Ctrl/Cmd + `**: 切换浮动DevTools面板
- **F12**: 打开完整DevTools窗口
- **Ctrl/Cmd + Shift + D**: 打开完整DevTools窗口

#### 路由访问
- 直接访问 `/cpu-devtools` 路由
- 从主导航的开发工具区域进入

### 生产环境

#### 隐藏入口启用
```typescript
// 在浏览器控制台中执行
localStorage.setItem('enable-devtools', 'true')
// 然后刷新页面

// 或者通过特殊URL参数
window.location.href += '?dev=true'
```

#### 全局调试对象
```typescript
// 在开发环境下，全局对象上暴露了调试方法
window.__CUTIE_DEVTOOLS__ = {
  show: () => devToolsRef.value?.show(),
  hide: () => devToolsRef.value?.close(),
  toggle: () => devToolsRef.value?.toggle()
}

window.__CUTIE_DEBUG__ = {
  cpuLogger: () => import('@/cpu/logging').then(m => m.cpuLogger),
  pipeline: () => import('@/cpu').then(m => m.pipeline),
  stores: () => import('@/stores').then(m => m)
}
```

### 功能使用

#### 瀑布图分析
1. **查看指令执行时序**: 每个指令显示为一行，包含各阶段的时间条
2. **识别性能瓶颈**: 查看哪个阶段耗时最长
3. **网络请求监控**: 红色标记显示网络请求时间点
4. **资源冲突标识**: 黄色圆点标记资源冲突位置

#### 统计面板监控
1. **实时性能指标**: 吞吐量、成功率、平均延迟
2. **冲突热点分析**: 识别最频繁冲突的资源
3. **乐观更新监控**: 回滚率和成功率统计

#### 指令详情分析
1. **点击指令行**: 查看详细的事件时间线
2. **关联指令追踪**: 查看相同correlation ID的其他指令
3. **完整事件链**: 从创建到完成的所有事件详情

---

## 最佳实践

### 开发阶段
1. **使用浮动面板进行日常监控**
   - 快捷键 `Ctrl+`` 快速切换
   - 关注CPU状态指示器颜色变化
   - 监控指令成功率和冲突情况

2. **性能调优时使用完整DevTools**
   - F12打开独立窗口
   - 按完成时间排序查看延迟分布
   - 重点关注P95、P99延迟指标

3. **问题调试时的工作流**
   - 清空历史数据 → 重现问题 → 导出数据分析
   - 查看失败指令的详细事件链
   - 分析resource冲突模式

### 生产环境
1. **隐藏式启用**
   - 通过localStorage或URL参数临时启用
   - 避免在正常用户界面中暴露入口

2. **性能监控**
   - 定期检查吞吐量趋势
   - 监控资源冲突热点
   - 关注乐观更新回滚率

3. **问题诊断**
   - 导出性能数据进行离线分析
   - 与用户反馈的时间点进行关联
   - 查看correlation ID追踪完整用户操作链

### 数据导出与分析
```typescript
// 导出特定时间段的数据
const data = dataProvider.exportData({
  timeRange: { start: startTime, end: endTime },
  instructionType: 'task.update'
})

// 保存到文件
const blob = new Blob([JSON.stringify(data, null, 2)], {
  type: 'application/json'
})
const url = URL.createObjectURL(blob)
const a = document.createElement('a')
a.href = url
a.download = `cpu-devtools-${Date.now()}.json`
a.click()
```

### 性能影响最小化
1. **数据采集优化**
   - 生产环境禁用详细事件采集
   - 使用采样模式减少数据量
   - 设置合理的事件保留上限

2. **UI渲染优化**
   - 虚拟滚动处理大量指令列表
   - 按需加载详情数据
   - 使用Web Worker处理复杂计算

3. **内存管理**
   - 定期清理过期事件数据
   - 实现数据压缩存储
   - 监控DevTools自身的性能开销

---

## 总结

CPU DevTools 基于 Cutie 项目现有的强大追踪系统，提供了完整的指令执行可视化和性能分析能力。通过三种不同的集成方案，可以满足开发、调试、生产监控的不同需求。

### 核心优势
1. **零侵入集成** - 基于现有追踪系统，无需修改核心CPU代码
2. **实时可视化** - 瀑布图直观展示指令执行时序和性能瓶颈
3. **多场景适配** - 内嵌路由、独立窗口、浮动面板三种模式
4. **深度分析** - 支持指令级别的详细事件链分析

### 实施建议
1. **优先实现浮动面板** - 提供日常开发的快速监控能力
2. **逐步完善瀑布图** - 核心的可视化分析功能
3. **增强统计面板** - 提供关键性能指标监控
4. **完善生产集成** - 确保生产环境的可调试性

通过这套完整的DevTools系统，可以大大提升CPU架构的可观测性和调试效率，为系统优化提供强有力的数据支持。

---

*文档生成时间: 2025-10-17*
*版本: v1.0*