<template>
  <div class="cpu-debug-view">
    <div class="debug-header">
      <h1>CPU流水线调试器</h1>
      <div class="header-controls">
        <CuteButton @click="handleStart" :disabled="isRunning">
          <CuteIcon name="Play" :size="16" />
          启动
        </CuteButton>
        <CuteButton @click="handleStop" :disabled="!isRunning">
          <CuteIcon name="Pause" :size="16" />
          停止
        </CuteButton>
        <CuteButton @click="handleReset">
          <CuteIcon name="RotateCcw" :size="16" />
          重置
        </CuteButton>
      </div>
    </div>

    <!-- 流水线状态卡片 -->
    <div class="pipeline-status">
      <div class="status-card">
        <div class="status-icon if">IF</div>
        <div class="status-info">
          <div class="status-label">缓冲区</div>
          <div class="status-value">{{ pipelineStatus.ifBufferSize }}</div>
        </div>
      </div>
      <div class="status-arrow">→</div>
      <div class="status-card">
        <div class="status-icon sch">SCH</div>
        <div class="status-info">
          <div class="status-label">Pending</div>
          <div class="status-value">{{ pipelineStatus.schPendingSize }}</div>
          <div class="status-label">Active</div>
          <div class="status-value">{{ pipelineStatus.schActiveSize }}</div>
        </div>
      </div>
      <div class="status-arrow">→</div>
      <div class="status-card">
        <div class="status-icon ex">EX</div>
        <div class="status-info">
          <div class="status-label">执行中</div>
          <div class="status-value">{{ executingCount }}</div>
        </div>
      </div>
      <div class="status-arrow">→</div>
      <div class="status-card">
        <div class="status-icon res">RES</div>
        <div class="status-info">
          <div class="status-label">响应中</div>
          <div class="status-value">{{ respondingCount }}</div>
        </div>
      </div>
      <div class="status-arrow">→</div>
      <div class="status-card">
        <div class="status-icon wb">WB</div>
        <div class="status-info">
          <div class="status-label">已完成</div>
          <div class="status-value">{{ pipelineStatus.totalCompleted }}</div>
          <div class="status-label">失败</div>
          <div class="status-value error">{{ pipelineStatus.totalFailed }}</div>
        </div>
      </div>
      <div class="status-arrow">→</div>
      <div class="status-card">
        <div class="status-icon int">INT</div>
        <div class="status-info">
          <div class="status-label">中断表</div>
          <div class="status-value">{{ intStats.tableSize }}</div>
        </div>
      </div>
    </div>

    <!-- 控制台控制 -->
    <div class="console-controls">
      <h2>控制台设置</h2>
      <div class="control-group">
        <label>控制台级别：</label>
        <select v-model="consoleLevel" @change="onConsoleLevelChange">
          <option :value="0">关闭 (SILENT)</option>
          <option :value="1">最小 (MINIMAL)</option>
          <option :value="2">正常 (NORMAL)</option>
          <option :value="3">详细 (VERBOSE)</option>
          <option :value="4">调试 (DEBUG)</option>
        </select>
        <span class="hint">{{ getConsoleLevelHint() }}</span>
      </div>
      <div class="action-buttons">
        <CuteButton @click="printStats">
          <CuteIcon name="Activity" :size="16" />
          打印统计信息
        </CuteButton>
        <CuteButton @click="printSeparator">
          <CuteIcon name="Minus" :size="16" />
          打印分隔线
        </CuteButton>
      </div>
    </div>

    <!-- 链式操作测试 -->
    <div class="chain-actions">
      <h2>🔗 链式操作测试（Awaitable Dispatch）</h2>
      <div class="control-group">
        <label>登录结果：</label>
        <div class="toggle-switch">
          <label class="switch">
            <input type="checkbox" v-model="loginShouldSucceed" />
            <span class="slider"></span>
          </label>
          <span class="toggle-label">{{ loginShouldSucceed ? '✅ 成功' : '❌ 失败' }}</span>
        </div>
      </div>
      <div class="action-buttons">
        <CuteButton @click="testLoginChain" :disabled="isLoggingIn">
          <CuteIcon name="LogIn" :size="16" />
          {{ isLoggingIn ? '登录中...' : '测试登录 → 欢迎' }}
        </CuteButton>
      </div>
      <div class="chain-info">
        <p>
          💡 此测试演示：
          <br />
          1. 先执行 <code>debug.login</code> 指令并 <strong>await</strong> 结果
          <br />
          2. 登录成功后，再执行 <code>debug.welcome</code> 指令
          <br />
          3. 如果登录失败，不会执行欢迎指令
          <br />
          <br />
          使用上方开关控制登录是否成功，观察控制台输出！
        </p>
      </div>
    </div>

    <!-- 快速发射指令 -->
    <div class="quick-actions">
      <h2>快速测试</h2>
      <div class="action-buttons">
        <CuteButton @click="dispatchInstruction('debug.fetch_baidu', {})">
          <CuteIcon name="Globe" :size="16" />
          请求百度
        </CuteButton>
        <CuteButton @click="dispatchInstruction('debug.quick_success', { data: 'test' })">
          <CuteIcon name="Zap" :size="16" />
          立即成功
        </CuteButton>
        <CuteButton @click="dispatchInstruction('debug.fetch_with_delay', { delay: 2000 })">
          <CuteIcon name="Clock" :size="16" />
          延迟2秒
        </CuteButton>
        <CuteButton @click="dispatchInstruction('debug.fetch_fail', { errorMessage: '测试失败' })">
          <CuteIcon name="X" :size="16" />
          必定失败
        </CuteButton>
        <CuteButton
          @click="
            dispatchInstruction('debug.conflicting_resource', { delay: 1500, id: Date.now() })
          "
        >
          <CuteIcon name="Lock" :size="16" />
          资源冲突
        </CuteButton>
        <CuteButton @click="dispatchInstruction('debug.test_timeout', {})">
          <CuteIcon name="Timer" :size="16" />
          测试超时（5秒）
        </CuteButton>
      </div>
      <div class="batch-test">
        <CuteButton @click="batchTest">
          <CuteIcon name="Layers" :size="16" />
          批量测试（10个指令）
        </CuteButton>
      </div>
    </div>

    <!-- 任务指令测试 -->
    <div class="task-actions">
      <h2>任务指令测试</h2>
      <div class="task-input-section">
        <div class="input-group">
          <label>任务标题：</label>
          <input v-model="testTaskTitle" type="text" placeholder="输入任务标题" />
        </div>
        <div class="input-group">
          <label>任务ID：</label>
          <input v-model="testTaskId" type="text" placeholder="输入现有任务ID" />
        </div>
      </div>
      <div class="action-buttons">
        <CuteButton @click="testCreateTask" :disabled="!testTaskTitle.trim()">
          <CuteIcon name="Plus" :size="16" />
          创建任务
        </CuteButton>
        <CuteButton @click="testCompleteTask" :disabled="!testTaskId.trim()">
          <CuteIcon name="Check" :size="16" />
          完成任务
        </CuteButton>
        <CuteButton @click="testReopenTask" :disabled="!testTaskId.trim()">
          <CuteIcon name="RotateCw" :size="16" />
          重新打开
        </CuteButton>
        <CuteButton @click="testUpdateTask" :disabled="!testTaskId.trim()">
          <CuteIcon name="Pencil" :size="16" />
          更新任务
        </CuteButton>
        <CuteButton @click="testDeleteTask" :disabled="!testTaskId.trim()">
          <CuteIcon name="Trash2" :size="16" />
          删除任务
        </CuteButton>
        <CuteButton @click="testArchiveTask" :disabled="!testTaskId.trim()">
          <CuteIcon name="Archive" :size="16" />
          归档任务
        </CuteButton>
      </div>
      <div class="task-list-section">
        <h3>可用任务列表</h3>
        <div class="task-list">
          <div
            v-for="task in availableTasks"
            :key="task.id"
            class="task-item"
            @click="testTaskId = task.id"
            :class="{ selected: testTaskId === task.id }"
          >
            <div class="task-info">
              <span class="task-title">{{ task.title }}</span>
              <span class="task-id">{{ task.id.substring(0, 8) }}...</span>
            </div>
            <div class="task-status">
              <span v-if="task.is_completed" class="badge completed">已完成</span>
              <span v-else-if="task.is_archived" class="badge archived">已归档</span>
              <span v-else class="badge active">进行中</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 指令追踪表格 -->
    <div class="trace-table-section">
      <div class="section-header">
        <h2>指令追踪记录</h2>
        <div class="filter-buttons">
          <button :class="{ active: filter === 'all' }" @click="filter = 'all'">
            全部 ({{ traces.length }})
          </button>
          <button :class="{ active: filter === 'committed' }" @click="filter = 'committed'">
            成功 ({{ successCount }})
          </button>
          <button :class="{ active: filter === 'failed' }" @click="filter = 'failed'">
            失败 ({{ failCount }})
          </button>
          <button :class="{ active: filter === 'executing' }" @click="filter = 'executing'">
            执行中 ({{ executingTraceCount }})
          </button>
        </div>
      </div>
      <div class="trace-table-wrapper">
        <table class="trace-table">
          <thead>
            <tr>
              <th>指令ID</th>
              <th>类型</th>
              <th>状态</th>
              <th>IF→SCH</th>
              <th>SCH→EX</th>
              <th>EX→RES</th>
              <th>RES→WB</th>
              <th>总耗时</th>
              <th>结果</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="trace in filteredTraces"
              :key="trace.instructionId"
              :class="getRowClass(trace)"
            >
              <td class="instruction-id">{{ formatInstructionId(trace.instructionId) }}</td>
              <td class="instruction-type">{{ formatInstructionType(trace.type) }}</td>
              <td>
                <span :class="['status-badge', trace.status]">{{
                  formatStatus(trace.status)
                }}</span>
              </td>
              <td>{{ formatDuration(trace.timestamps.IF, trace.timestamps.SCH) }}</td>
              <td>{{ formatDuration(trace.timestamps.SCH, trace.timestamps.EX) }}</td>
              <td>{{ formatDuration(trace.timestamps.EX, trace.timestamps.RES) }}</td>
              <td>{{ formatDuration(trace.timestamps.RES, trace.timestamps.WB) }}</td>
              <td class="total-duration">{{ trace.duration ? `${trace.duration}ms` : '-' }}</td>
              <td class="result-cell">
                <span v-if="trace.error" class="error-message">{{ trace.error.message }}</span>
                <span v-else-if="trace.networkResult" class="success-result">✓</span>
                <span v-else>-</span>
              </td>
            </tr>
          </tbody>
        </table>
        <div v-if="filteredTraces.length === 0" class="empty-state">
          <CuteIcon name="Inbox" :size="48" />
          <p>暂无指令记录</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { pipeline, instructionTracker } from '@/cpu'
import { cpuConsole, ConsoleLevel } from '@/cpu/logging'
import type { InstructionTrace } from '@/cpu'
import CuteButton from '@/components/parts/CuteButton.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import { useTaskStore } from '@/stores/task'
import { storeToRefs } from 'pinia'
import { interruptHandler } from '@/cpu/interrupt/InterruptHandler'

const isRunning = ref(false)
const traces = ref<InstructionTrace[]>([])
const filter = ref<'all' | 'committed' | 'failed' | 'executing'>('all')

// 控制台设置
const consoleLevel = ref<ConsoleLevel>(cpuConsole.getLevel())

// 链式操作测试
const loginShouldSucceed = ref(true)
const isLoggingIn = ref(false)

// 任务测试相关
const testTaskTitle = ref('')
const testTaskId = ref('')
const taskStore = useTaskStore()
const { allTasks } = storeToRefs(taskStore) // 🔥 解构为响应式引用
const availableTasks = computed(() => {
  return allTasks.value.slice(0, 10) // 显示前10个任务，响应式更新
})

// 流水线状态
const pipelineStatus = computed(() => pipeline.status.value)

// INT 中断处理器状态
const intStats = ref({
  tableSize: 0,
  entries: [] as Array<{ correlationId: string; type: string; age: number }>,
})

// 统计
const successCount = computed(() => traces.value.filter((t) => t.status === 'committed').length)
const failCount = computed(() => traces.value.filter((t) => t.status === 'failed').length)
const executingTraceCount = computed(
  () =>
    traces.value.filter((t) => ['pending', 'issued', 'executing', 'responded'].includes(t.status))
      .length
)
const executingCount = computed(() => traces.value.filter((t) => t.status === 'executing').length)
const respondingCount = computed(() => traces.value.filter((t) => t.status === 'responded').length)

// 过滤后的追踪记录
const filteredTraces = computed(() => {
  if (filter.value === 'all') return traces.value
  if (filter.value === 'committed') return traces.value.filter((t) => t.status === 'committed')
  if (filter.value === 'failed') return traces.value.filter((t) => t.status === 'failed')
  if (filter.value === 'executing')
    return traces.value.filter((t) =>
      ['pending', 'issued', 'executing', 'responded'].includes(t.status)
    )
  return traces.value
})

let updateInterval: number | null = null

// 生命周期
onMounted(async () => {
  // 加载任务数据
  await taskStore.fetchAllTasks_DMA()

  // 启动流水线
  pipeline.start()
  isRunning.value = true

  // 定期更新追踪记录和 INT 状态
  updateInterval = window.setInterval(() => {
    traces.value = instructionTracker.getAllTraces()
    intStats.value = interruptHandler.getStats()
  }, 100)
})

onBeforeUnmount(() => {
  if (updateInterval !== null) {
    clearInterval(updateInterval)
  }
})

// 控制按钮
function handleStart() {
  pipeline.start()
  isRunning.value = true
}

function handleStop() {
  pipeline.stop()
  isRunning.value = false
}

function handleReset() {
  pipeline.reset()
  traces.value = []
  filter.value = 'all'
  isRunning.value = false // 同步流水线状态
}

// 控制台控制
function onConsoleLevelChange() {
  cpuConsole.setLevel(consoleLevel.value)
  console.log(
    `%c✅ 控制台级别已设置为: ${getConsoleLevelName(consoleLevel.value)}`,
    'color: #10b981; font-weight: bold'
  )
}

function getConsoleLevelName(level: ConsoleLevel): string {
  const names = ['SILENT', 'MINIMAL', 'NORMAL', 'VERBOSE', 'DEBUG']
  return names[level] || 'UNKNOWN'
}

function getConsoleLevelHint(): string {
  const hints = [
    '不输出任何内容',
    '只输出成功/失败',
    '输出关键阶段',
    '输出所有细节',
    '输出调试信息（包括 payload）',
  ]
  return hints[consoleLevel.value] || ''
}

function printStats() {
  const stats = {
    total: traces.value.length,
    success: successCount.value,
    failed: failCount.value,
    avgLatency:
      traces.value.reduce((sum, t) => sum + (t.duration || 0), 0) / traces.value.length || 0,
  }
  cpuConsole.printStats(stats)
}

function printSeparator() {
  cpuConsole.printSeparator('CPU 流水线调试')
}

// 链式操作测试：登录 → 欢迎
async function testLoginChain() {
  isLoggingIn.value = true

  try {
    cpuConsole.printSeparator('链式操作测试：登录 → 欢迎')

    console.log('%c📋 步骤 1: 开始登录...', 'color: #3b82f6; font-weight: bold')

    // 🔥 步骤 1: 执行登录指令并 await 结果
    const loginResult = await pipeline.dispatch('debug.login', {
      shouldSucceed: loginShouldSucceed.value,
    })

    console.log('%c✅ 步骤 1 完成: 登录成功！', 'color: #10b981; font-weight: bold', loginResult)

    // 🔥 步骤 2: 登录成功后，发送欢迎指令
    console.log('%c📋 步骤 2: 发送欢迎消息...', 'color: #3b82f6; font-weight: bold')

    const welcomeResult = await pipeline.dispatch('debug.welcome', {
      userId: loginResult.user.id,
      userName: loginResult.user.name,
    })

    console.log(
      '%c✅ 步骤 2 完成: 欢迎消息已发送！',
      'color: #10b981; font-weight: bold',
      welcomeResult
    )

    console.log(
      '%c🎉 链式操作完成！登录 → 欢迎',
      'color: #10b981; font-weight: bold; font-size: 16px'
    )
    console.log(`%c${welcomeResult.message}`, 'color: #8b5cf6; font-size: 14px')
    console.log('%c提示:', 'color: #666; font-weight: bold')
    welcomeResult.tips.forEach((tip: string) => {
      console.log(`  • ${tip}`)
    })
  } catch (error) {
    console.log('%c❌ 链式操作失败！', 'color: #ef4444; font-weight: bold; font-size: 16px')
    console.error('失败原因:', error)

    if ((error as Error).message.includes('登录失败')) {
      console.log('%c💡 登录失败，欢迎指令不会执行', 'color: #f59e0b; font-weight: bold')
    }
  } finally {
    isLoggingIn.value = false
  }
}

// 发射指令
function dispatchInstruction(type: string, payload: any) {
  pipeline.dispatch(type, payload, 'test')
}

// 批量测试
function batchTest() {
  const instructions = [
    { type: 'debug.quick_success', payload: { data: 'batch-1' } },
    { type: 'debug.quick_success', payload: { data: 'batch-2' } },
    { type: 'debug.fetch_with_delay', payload: { delay: 500 } },
    { type: 'debug.fetch_with_delay', payload: { delay: 1000 } },
    { type: 'debug.quick_success', payload: { data: 'batch-3' } },
    { type: 'debug.conflicting_resource', payload: { delay: 800 } },
    { type: 'debug.conflicting_resource', payload: { delay: 800 } },
    { type: 'debug.fetch_with_delay', payload: { delay: 1500 } },
    { type: 'debug.quick_success', payload: { data: 'batch-4' } },
    { type: 'debug.fetch_fail', payload: { errorMessage: '批量测试失败' } },
  ]

  instructions.forEach((instr, index) => {
    setTimeout(() => {
      dispatchInstruction(instr.type, instr.payload)
    }, index * 50)
  })
}

// 任务测试函数
function testCreateTask() {
  if (!testTaskTitle.value.trim()) return
  dispatchInstruction('task.create', {
    title: testTaskTitle.value.trim(),
  })
  testTaskTitle.value = '' // 清空输入
}

function testCompleteTask() {
  if (!testTaskId.value.trim()) return
  dispatchInstruction('task.complete', {
    id: testTaskId.value.trim(),
  })
}

function testReopenTask() {
  if (!testTaskId.value.trim()) return
  dispatchInstruction('task.reopen', {
    id: testTaskId.value.trim(),
  })
}

function testUpdateTask() {
  if (!testTaskId.value.trim()) return
  dispatchInstruction('task.update', {
    id: testTaskId.value.trim(),
    updates: {
      title: `[CPU更新] ${Date.now()}`,
    },
  })
}

function testDeleteTask() {
  if (!testTaskId.value.trim()) return
  dispatchInstruction('task.delete', {
    id: testTaskId.value.trim(),
  })
}

function testArchiveTask() {
  if (!testTaskId.value.trim()) return
  dispatchInstruction('task.archive', {
    id: testTaskId.value.trim(),
  })
}

// 格式化函数
function formatInstructionId(id: string): string {
  return id.split('-').slice(-1)[0] || ''
}

function formatInstructionType(type: string): string {
  return type.replace('debug.', '')
}

function formatStatus(status: string): string {
  const statusMap: Record<string, string> = {
    pending: '等待',
    issued: '已发射',
    executing: '执行中',
    responded: '已响应',
    committed: '成功',
    failed: '失败',
  }
  return statusMap[status] || status
}

function formatDuration(start?: number, end?: number): string {
  if (!start || !end) return '-'
  return `${end - start}ms`
}

function getRowClass(trace: InstructionTrace): string {
  if (trace.status === 'failed') return 'row-failed'
  if (trace.status === 'committed') return 'row-success'
  return 'row-executing'
}
</script>

<style scoped>
.cpu-debug-view {
  padding: 24px;
  height: 100%;
  overflow-y: auto;
  background: var(--color-background);
}

.debug-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.debug-header h1 {
  font-size: 24px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.header-controls {
  display: flex;
  gap: 8px;
}

/* 流水线状态卡片 */
.pipeline-status {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 32px;
  padding: 20px;
  background: var(--color-surface);
  border-radius: 12px;
  overflow-x: auto;
}

.status-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--color-background);
  border-radius: 8px;
  min-width: 120px;
}

.status-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  font-weight: 700;
  font-size: 14px;
  color: white;
}

.status-icon.if {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.status-icon.sch {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
}

.status-icon.ex {
  background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
}

.status-icon.res {
  background: linear-gradient(135deg, #43e97b 0%, #38f9d7 100%);
}

.status-icon.wb {
  background: linear-gradient(135deg, #fa709a 0%, #fee140 100%);
}

.status-icon.int {
  background: linear-gradient(135deg, #a18cd1 0%, #fbc2eb 100%);
}

.status-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.status-label {
  font-size: 11px;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.status-value {
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.status-value.error {
  color: #f44336;
}

.status-arrow {
  font-size: 24px;
  color: var(--color-text-tertiary);
  user-select: none;
}

/* 控制台控制 */
.console-controls {
  margin-bottom: 32px;
  padding: 20px;
  background: var(--color-surface);
  border-radius: 12px;
}

.console-controls h2 {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: 16px;
}

/* 链式操作测试 */
.chain-actions {
  margin-bottom: 32px;
  padding: 20px;
  background: linear-gradient(135deg, #667eea15 0%, #764ba215 100%);
  border: 2px solid #667eea30;
  border-radius: 12px;
}

.chain-actions h2 {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: 16px;
}

.toggle-switch {
  display: flex;
  align-items: center;
  gap: 12px;
}

.switch {
  position: relative;
  display: inline-block;
  width: 48px;
  height: 24px;
}

.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  inset: 0;
  background-color: #ef4444;
  transition: 0.3s;
  border-radius: 24px;
}

.slider::before {
  position: absolute;
  content: '';
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: 0.3s;
  border-radius: 50%;
}

input:checked + .slider {
  background-color: #10b981;
}

input:checked + .slider::before {
  transform: translateX(24px);
}

.toggle-label {
  font-size: 14px;
  font-weight: 600;
  min-width: 80px;
}

.chain-info {
  margin-top: 16px;
  padding: 16px;
  background: var(--color-background);
  border-radius: 8px;
  border-left: 4px solid #667eea;
}

.chain-info p {
  font-size: 13px;
  line-height: 1.8;
  color: var(--color-text-secondary);
  margin: 0;
}

.chain-info code {
  padding: 2px 6px;
  background: #667eea15;
  border-radius: 4px;
  font-family: 'Fira Code', monospace;
  font-size: 12px;
  color: #667eea;
}

.control-group {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}

.control-group label {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-secondary);
  min-width: 100px;
}

.control-group select {
  padding: 8px 12px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-background);
  color: var(--color-text-primary);
  font-size: 14px;
  font-family: inherit;
  cursor: pointer;
  transition: border-color 0.2s;
}

.control-group select:focus {
  outline: none;
  border-color: var(--color-primary);
}

.control-group .hint {
  font-size: 12px;
  color: var(--color-text-tertiary);
  font-style: italic;
}

/* 快速操作 */
.quick-actions {
  margin-bottom: 32px;
  padding: 20px;
  background: var(--color-surface);
  border-radius: 12px;
}

.quick-actions h2 {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: 16px;
}

.action-buttons {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 16px;
}

.batch-test {
  padding-top: 16px;
  border-top: 1px solid var(--color-border);
}

/* 任务测试区域 */
.task-actions {
  margin-bottom: 32px;
  padding: 20px;
  background: var(--color-surface);
  border-radius: 12px;
}

.task-actions h2 {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: 16px;
}

.task-actions h3 {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin-bottom: 12px;
  margin-top: 20px;
}

.task-input-section {
  display: flex;
  gap: 16px;
  margin-bottom: 16px;
}

.input-group {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.input-group label {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.input-group input {
  padding: 8px 12px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-background);
  color: var(--color-text-primary);
  font-size: 14px;
  font-family: inherit;
  transition: border-color 0.2s;
}

.input-group input:focus {
  outline: none;
  border-color: var(--color-primary);
}

.input-group input::placeholder {
  color: var(--color-text-tertiary);
}

.task-list-section {
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid var(--color-border);
}

.task-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 300px;
  overflow-y: auto;
}

.task-item {
  padding: 12px;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.task-item:hover {
  background: rgb(33 150 243 / 5%);
  border-color: var(--color-primary);
}

.task-item.selected {
  background: rgb(33 150 243 / 10%);
  border-color: var(--color-primary);
}

.task-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.task-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--color-text-primary);
}

.task-id {
  font-size: 12px;
  font-family: 'Courier New', monospace;
  color: var(--color-text-tertiary);
}

.task-status {
  display: flex;
  gap: 8px;
}

.task-status .badge {
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
}

.task-status .badge.completed {
  background: rgb(76 175 80 / 10%);
  color: rgb(76 175 80);
}

.task-status .badge.archived {
  background: rgb(156 39 176 / 10%);
  color: rgb(156 39 176);
}

.task-status .badge.active {
  background: rgb(33 150 243 / 10%);
  color: rgb(33 150 243);
}

/* 追踪表格 */
.trace-table-section {
  background: var(--color-surface);
  border-radius: 12px;
  padding: 20px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.section-header h2 {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.filter-buttons {
  display: flex;
  gap: 8px;
}

.filter-buttons button {
  padding: 6px 12px;
  border: 1px solid var(--color-border);
  background: var(--color-background);
  border-radius: 6px;
  font-size: 13px;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.filter-buttons button:hover {
  background: var(--color-surface);
  color: var(--color-text-primary);
}

.filter-buttons button.active {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
}

.trace-table-wrapper {
  overflow: auto;
  max-height: 500px;
}

.trace-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.trace-table thead {
  position: sticky;
  top: 0;
  background: var(--color-surface);
  z-index: 1;
}

.trace-table th {
  padding: 12px 8px;
  text-align: left;
  font-weight: 600;
  color: var(--color-text-secondary);
  border-bottom: 2px solid var(--color-border);
  text-transform: uppercase;
  font-size: 11px;
  letter-spacing: 0.5px;
}

.trace-table td {
  padding: 12px 8px;
  border-bottom: 1px solid var(--color-border);
  color: var(--color-text-primary);
}

.trace-table tbody tr:hover {
  background: var(--color-background);
}

.row-success {
  background: rgb(76 175 80 / 5%);
}

.row-failed {
  background: rgb(244 67 54 / 5%);
}

.row-executing {
  background: rgb(33 150 243 / 5%);
}

.instruction-id {
  font-family: 'Courier New', monospace;
  color: var(--color-text-secondary);
  font-size: 12px;
}

.instruction-type {
  font-weight: 500;
}

.status-badge {
  display: inline-block;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.status-badge.pending,
.status-badge.issued {
  background: #e3f2fd;
  color: #1976d2;
}

.status-badge.executing,
.status-badge.responded {
  background: #fff3e0;
  color: #f57c00;
}

.status-badge.committed {
  background: #e8f5e9;
  color: #388e3c;
}

.status-badge.failed {
  background: #ffebee;
  color: #d32f2f;
}

.total-duration {
  font-weight: 600;
  color: var(--color-primary);
}

.result-cell {
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.error-message {
  color: #f44336;
  font-size: 12px;
}

.success-result {
  color: #4caf50;
  font-size: 16px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px;
  color: var(--color-text-tertiary);
}

.empty-state p {
  margin-top: 16px;
  font-size: 14px;
}
</style>
