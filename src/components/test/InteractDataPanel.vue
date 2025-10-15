<!--
  新拖放系统数据面板
  
  实时显示拖放系统的元数据和状态
-->

<template>
  <div class="interact-data-panel">
    <div class="panel-header">
      <h3>📊 拖放数据面板</h3>
      <div class="status-dot" :class="{ active: hasPreview }"></div>
    </div>

    <div class="panel-content">
      <!-- 当前预览状态 -->
      <div class="data-section">
        <h4>🎯 预览状态</h4>
        <div v-if="!hasPreview" class="no-data">
          <p>暂无拖放活动</p>
          <small>开始拖动任务查看实时数据</small>
        </div>
        <div v-else class="data-grid">
          <div class="data-item">
            <label>类型</label>
            <span class="value type">{{ previewType }}</span>
          </div>
          <div class="data-item">
            <label>源区域</label>
            <span class="value source">{{ previewData?.raw.sourceZoneId }}</span>
          </div>
          <div class="data-item">
            <label>目标区域</label>
            <span class="value target" :class="{ rebounding: isRebounding }">
              {{ previewData?.raw.targetZoneId || '无效区域' }}
            </span>
          </div>
          <div class="data-item">
            <label>任务</label>
            <span class="value task">{{ previewData?.raw.ghostTask.title }}</span>
          </div>
          <div v-if="previewData?.computed.dropIndex !== undefined" class="data-item">
            <label>插入位置</label>
            <span class="value index">{{ previewData.computed.dropIndex }}</span>
          </div>
          <div class="data-item">
            <label>鼠标位置</label>
            <span class="value position">
              ({{ previewData?.raw.mousePosition.x }}, {{ previewData?.raw.mousePosition.y }})
            </span>
          </div>
        </div>
      </div>

      <!-- 回弹状态 -->
      <div v-if="isRebounding" class="data-section rebound-alert">
        <h4>⚡ 越界回弹</h4>
        <p>任务已回到原始位置</p>
        <small>拖拽到有效区域继续操作</small>
      </div>

      <!-- 控制器状态 -->
      <div class="data-section">
        <h4>🎮 控制器状态</h4>
        <div class="data-grid">
          <div class="data-item">
            <label>阶段</label>
            <span class="value phase" :class="controllerDebug.phase?.toLowerCase()">
              {{ controllerDebug.phase }}
            </span>
          </div>
          <div class="data-item">
            <label>会话</label>
            <span class="value session" :class="{ active: controllerDebug.hasSession }">
              {{ controllerDebug.hasSession ? '活跃' : '无' }}
            </span>
          </div>
          <div class="data-item">
            <label>目标区域</label>
            <span class="value">{{ controllerDebug.targetZone || '无' }}</span>
          </div>
          <div class="data-item">
            <label>有效区域</label>
            <span class="value">{{ controllerDebug.validZones?.length || 0 }} 个</span>
          </div>
          <div class="data-item">
            <label>幽灵元素</label>
            <span class="value ghost" :class="{ active: controllerDebug.hasGhost }">
              {{ controllerDebug.hasGhost ? '存在' : '无' }}
            </span>
          </div>
        </div>
      </div>

      <!-- 有效区域列表 -->
      <div class="data-section">
        <h4>📍 有效区域</h4>
        <div class="zone-list">
          <div
            v-for="zone in controllerDebug.validZones"
            :key="zone"
            class="zone-item"
            :class="{ active: zone === controllerDebug.targetZone }"
          >
            {{ zone }}
          </div>
        </div>
      </div>

      <!-- 日志输出 -->
      <div class="data-section">
        <h4>📝 操作日志</h4>
        <div class="log-container">
          <div v-for="(log, index) in logs" :key="index" class="log-item" :class="log.type">
            <span class="log-time">{{ log.time }}</span>
            <span class="log-message">{{ log.message }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import {
  dragPreviewState,
  hasPreview,
  previewType,
  isRebounding,
  controllerDebugState,
} from '@/infra/drag-interact'

// 日志系统
interface LogEntry {
  time: string
  message: string
  type: 'info' | 'success' | 'warning' | 'error'
}

const logs = ref<LogEntry[]>([])

const addLog = (message: string, type: LogEntry['type'] = 'info') => {
  const time = new Date().toLocaleTimeString()
  logs.value.unshift({ time, message, type })

  // 限制日志数量
  if (logs.value.length > 20) {
    logs.value = logs.value.slice(0, 20)
  }
}

// 预览数据
const previewData = computed(() => dragPreviewState.value)

// 控制器调试信息（响应式）
const controllerDebug = computed(() => controllerDebugState.value)

// 监听预览状态变化
watch(hasPreview, (newValue, oldValue) => {
  if (newValue && !oldValue) {
    addLog('🎯 开始拖放', 'info')
  } else if (!newValue && oldValue) {
    addLog('✅ 拖放结束', 'success')
  }
})

watch(isRebounding, (newValue) => {
  if (newValue) {
    addLog('⚡ 触发越界回弹', 'warning')
  }
})

watch(
  () => previewData.value?.raw.targetZoneId,
  (newZone, oldZone) => {
    if (newZone && newZone !== oldZone) {
      if (newZone === null) {
        addLog('🚫 进入无效区域', 'warning')
      } else {
        addLog(`📍 进入区域: ${newZone}`, 'info')
      }
    }
  }
)

// 监听控制器阶段变化
watch(
  () => controllerDebug.value.phase,
  (newPhase, oldPhase) => {
    if (newPhase !== oldPhase) {
      addLog(`🎮 阶段变化: ${oldPhase} → ${newPhase}`, 'info')
    }
  }
)

// 初始化日志
addLog('📊 数据面板已初始化', 'success')
</script>

<style scoped>
.interact-data-panel {
  display: flex;
  flex-direction: column;
  background: var(--color-card-available);
  border-radius: 12px;
  box-shadow: 0 4px 6px -1px rgb(0 0 0 / 10%);
  overflow: hidden;
  height: 100%;
}

.panel-header {
  padding: 1rem;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.panel-header h3 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
}

.status-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: rgb(255 255 255 / 30%);
  transition: all 0.3s ease;
}

.status-dot.active {
  background: #10b981;
  box-shadow: 0 0 0 3px rgb(16 185 129 / 30%);
}

.panel-content {
  flex: 1;
  padding: 1rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.data-section {
  background: var(--color-background-muted);
  border: 1px solid var(--color-border-default);
  border-radius: 8px;
  padding: 1rem;
}

.data-section h4 {
  margin: 0 0 0.75rem;
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.data-section.rebound-alert {
  background: #fef3c7;
  border-color: #f59e0b;
  color: #92400e;
}

.no-data {
  text-align: center;
  color: var(--color-text-secondary);
  padding: 1rem 0;
}

.no-data p {
  margin: 0 0 0.25rem;
  font-weight: 500;
}

.no-data small {
  font-size: 0.75rem;
}

.data-grid {
  display: grid;
  gap: 0.75rem;
}

.data-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem;
  background: var(--color-card-available);
  border: 1px solid var(--color-border-default);
  border-radius: 6px;
}

.data-item label {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.025em;
}

.value {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-primary);
  font-family: Monaco, Menlo, monospace;
}

.value.type {
  background: #ddd6fe;
  color: #7c3aed;
  padding: 0.125rem 0.375rem;
  border-radius: 4px;
}

.value.source {
  background: #dbeafe;
  color: #2563eb;
  padding: 0.125rem 0.375rem;
  border-radius: 4px;
}

.value.target {
  background: #dcfce7;
  color: #16a34a;
  padding: 0.125rem 0.375rem;
  border-radius: 4px;
}

.value.target.rebounding {
  background: #fee2e2;
  color: #dc2626;
}

.value.task {
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.value.phase {
  text-transform: uppercase;
  font-size: 0.75rem;
  padding: 0.125rem 0.375rem;
  border-radius: 4px;
  background: var(--color-background-muted);
  color: var(--color-text-secondary);
}

.value.phase.idle {
  background: var(--color-background-muted);
  color: var(--color-text-secondary);
}

.value.phase.preparing {
  background: #fef3c7;
  color: #92400e;
}

.value.phase.dragging {
  background: #dbeafe;
  color: #2563eb;
}

.value.phase.over-target {
  background: #dcfce7;
  color: #16a34a;
}

.value.phase.dropping {
  background: #e0e7ff;
  color: #4338ca;
}

.value.session.active,
.value.ghost.active {
  color: #16a34a;
}

.zone-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.zone-item {
  padding: 0.25rem 0.5rem;
  background: var(--color-background-muted);
  color: var(--color-text-secondary);
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 500;
  font-family: Monaco, Menlo, monospace;
  transition: all 0.2s ease;
}

.zone-item.active {
  background: #dcfce7;
  color: #16a34a;
  box-shadow: 0 0 0 2px rgb(16 185 129 / 20%);
}

.log-container {
  max-height: 200px;
  overflow-y: auto;
  border: 1px solid var(--color-border-default);
  border-radius: 6px;
  background: var(--color-card-available);
}

.log-item {
  display: flex;
  gap: 0.5rem;
  padding: 0.5rem;
  border-bottom: 1px solid var(--color-border-default);
  font-size: 0.75rem;
  line-height: 1.4;
}

.log-item:last-child {
  border-bottom: none;
}

.log-time {
  color: var(--color-text-secondary);
  font-family: Monaco, Menlo, monospace;
  flex-shrink: 0;
  width: 60px;
}

.log-message {
  flex: 1;
  color: var(--color-text-primary);
}

.log-item.info .log-message {
  color: #2563eb;
}

.log-item.success .log-message {
  color: #16a34a;
}

.log-item.warning .log-message {
  color: #d97706;
}

.log-item.error .log-message {
  color: #dc2626;
}

/* 滚动条样式 */
.panel-content::-webkit-scrollbar,
.log-container::-webkit-scrollbar {
  width: 6px;
}

.panel-content::-webkit-scrollbar-track,
.log-container::-webkit-scrollbar-track {
  background: var(--color-background-muted);
  border-radius: 3px;
}

.panel-content::-webkit-scrollbar-thumb,
.log-container::-webkit-scrollbar-thumb {
  background: var(--color-border-default);
  border-radius: 3px;
}

.panel-content::-webkit-scrollbar-thumb:hover,
.log-container::-webkit-scrollbar-thumb:hover {
  background: var(--color-text-tertiary);
}
</style>
