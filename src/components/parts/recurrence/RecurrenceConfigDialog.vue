<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { RRule, Frequency } from 'rrule'
import type { TaskCard } from '@/types/dtos'
import { useTemplateStore } from '@/stores/template'
import { useRecurrenceStore } from '@/stores/recurrence'
import { useViewStore } from '@/stores/view'
import { pipeline } from '@/cpu'
import { getTodayDateString } from '@/infra/utils/dateUtils'

const props = defineProps<{
  task: TaskCard
  viewKey?: string // View context key (e.g., 'daily::2025-10-10', 'misc::staging')
  open: boolean
}>()

const emit = defineEmits<{
  close: []
  success: []
}>()

// 循环规则配置
const freq = ref<Frequency>(RRule.DAILY)
const interval = ref<number>(1)
const byweekday = ref<number[]>([]) // RRULE weekdays: 0=MO, 1=TU, ..., 6=SU
const bymonthday = ref<number | null>(null)
const bymonth = ref<number | null>(null)
const startDate = ref<string | null>(null)
const endDate = ref<string | null>(null)
const expiryBehavior = ref<'CARRYOVER_TO_STAGING' | 'EXPIRE'>('CARRYOVER_TO_STAGING') // 过期行为

const templateStore = useTemplateStore()
const recurrenceStore = useRecurrenceStore()
const viewStore = useViewStore()

// 从 viewKey 提取日期（如果是 daily 类型）
function extractDateFromViewKey(viewKey?: string): string | null {
  if (!viewKey) return null
  const parts = viewKey.split('::')
  if (parts[0] === 'daily' && parts[1]) {
    return parts[1] // 返回 YYYY-MM-DD 格式的日期
  }
  return null
}

// 监听对话框打开，自动设置 start_date
watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      const dateFromView = extractDateFromViewKey(props.viewKey)
      startDate.value = dateFromView || getTodayDateString()
    }
  },
  { immediate: true }
)

// 生成 RRULE 字符串
const ruleString = computed(() => {
  const options: any = {
    freq: freq.value,
    interval: interval.value,
  }

  if (freq.value === RRule.WEEKLY && byweekday.value.length > 0) {
    options.byweekday = byweekday.value
  }

  if (freq.value === RRule.MONTHLY && bymonthday.value) {
    options.bymonthday = bymonthday.value
  }

  if (freq.value === RRule.YEARLY && bymonth.value && bymonthday.value) {
    options.bymonth = bymonth.value
    options.bymonthday = bymonthday.value
  }

  const rule = new RRule(options)
  return rule.toString().replace('RRULE:', '') // 移除 RRULE: 前缀
})

// 人类可读的规则描述
const ruleDescription = computed(() => {
  try {
    const rule = new RRule({
      freq: freq.value,
      interval: interval.value,
      ...(freq.value === RRule.WEEKLY && byweekday.value.length > 0
        ? { byweekday: byweekday.value }
        : {}),
      ...(freq.value === RRule.MONTHLY && bymonthday.value ? { bymonthday: bymonthday.value } : {}),
      ...(freq.value === RRule.YEARLY && bymonth.value && bymonthday.value
        ? { bymonth: bymonth.value, bymonthday: bymonthday.value }
        : {}),
    })
    return rule.toText()
  } catch (e) {
    return '无效的规则'
  }
})

function toggleWeekday(day: number) {
  const index = byweekday.value.indexOf(day)
  if (index > -1) {
    byweekday.value.splice(index, 1)
  } else {
    byweekday.value.push(day)
  }
}

async function handleSave() {
  try {
    // 步骤1: 使用CPU指令创建循环模板（基于当前任务）
    const template = await pipeline.dispatch('template.create', {
      title: props.task.title,
      glance_note_template: props.task.glance_note ?? undefined,
      detail_note_template: undefined,
      estimated_duration_template: props.task.estimated_duration ?? undefined,
      subtasks_template: props.task.subtasks ?? undefined,
      area_id: props.task.area_id ?? undefined, // 🔥 修复：直接使用 area_id
      category: 'RECURRENCE',
    })

    // 步骤2: 使用CPU指令创建循环规则（传入原任务ID，避免重复创建）
    await pipeline.dispatch('recurrence.create', {
      template_id: template.id,
      rule: ruleString.value,
      time_type: 'FLOATING',
      start_date: startDate.value,
      end_date: endDate.value,
      expiry_behavior: expiryBehavior.value, // 🔥 传入过期行为
      is_active: true,
      source_task_id: props.task.id, // 🔥 传入原任务ID
    })
    // ✅ 刷新由 CPU 指令的 commit 阶段统一处理

    emit('success')
    emit('close')
  } catch (error) {
    console.error('Failed to create recurrence:', error)
    alert('创建循环规则失败，请检查配置')
  }
}

function handleCancel() {
  emit('close')
}

// 预设选项
function setWeekdays() {
  freq.value = RRule.WEEKLY
  byweekday.value = [0, 1, 2, 3, 4] // 周一到周五
}
</script>

<template>
  <div v-if="open" class="dialog-backdrop" @click.self="handleCancel">
    <div class="dialog-content">
      <h3>配置循环规则</h3>
      <p class="task-info">为任务 "{{ task.title }}" 设置循环</p>

      <!-- REPEATS 部分 -->
      <section class="form-section">
        <label class="section-label">重复频率</label>
        <div class="radio-group">
          <label class="radio-item">
            <input type="radio" :value="RRule.DAILY" v-model="freq" />
            <span>每天</span>
          </label>
          <label class="radio-item" @click="setWeekdays">
            <input type="radio" :checked="freq === RRule.WEEKLY && byweekday.length === 5" />
            <span>工作日（周一至周五）</span>
          </label>
          <label class="radio-item">
            <input type="radio" :value="RRule.WEEKLY" v-model="freq" />
            <span>每周</span>
          </label>
          <label class="radio-item">
            <input type="radio" :value="RRule.MONTHLY" v-model="freq" />
            <span>每月特定日期</span>
          </label>
          <label class="radio-item">
            <input type="radio" :value="RRule.YEARLY" v-model="freq" />
            <span>每年</span>
          </label>
        </div>
      </section>

      <!-- 每周选项 -->
      <section v-if="freq === RRule.WEEKLY" class="form-section">
        <label class="section-label">选择星期</label>
        <div class="weekday-buttons">
          <button
            v-for="(day, index) in ['周一', '周二', '周三', '周四', '周五', '周六', '周日']"
            :key="index"
            :class="{ active: byweekday.includes(index) }"
            @click="toggleWeekday(index)"
            type="button"
            class="weekday-btn"
          >
            {{ day }}
          </button>
        </div>
        <div class="interval-control">
          <label>
            每
            <input type="number" v-model.number="interval" min="1" max="4" class="interval-input" />
            周
          </label>
        </div>
      </section>

      <!-- 每月选项 -->
      <section v-if="freq === RRule.MONTHLY" class="form-section">
        <label class="section-label">每月几号</label>
        <select v-model.number="bymonthday" class="select-input">
          <option :value="null" disabled>请选择</option>
          <option v-for="day in 31" :key="day" :value="day">{{ day }} 号</option>
        </select>
      </section>

      <!-- 每年选项 -->
      <section v-if="freq === RRule.YEARLY" class="form-section">
        <label class="section-label">每年</label>
        <div class="inline-inputs">
          <select v-model.number="bymonth" class="select-input">
            <option :value="null" disabled>选择月份</option>
            <option v-for="month in 12" :key="month" :value="month">{{ month }} 月</option>
          </select>
          <select v-model.number="bymonthday" class="select-input">
            <option :value="null" disabled>选择日期</option>
            <option v-for="day in 31" :key="day" :value="day">{{ day }} 号</option>
          </select>
        </div>
      </section>

      <!-- 高级选项 -->
      <details class="advanced-options">
        <summary>高级选项</summary>
        <div class="form-section">
          <label class="section-label">开始日期（可选）</label>
          <input type="date" v-model="startDate" class="date-input" />
        </div>
        <div class="form-section">
          <label class="section-label">结束日期（可选）</label>
          <input type="date" v-model="endDate" class="date-input" />
        </div>
        <div class="form-section">
          <label class="section-label">过期后的处理方式</label>
          <div class="radio-group">
            <label class="radio-item">
              <input type="radio" value="CARRYOVER_TO_STAGING" v-model="expiryBehavior" />
              <span>
                <strong>结转到暂存区</strong>
                <div class="radio-description">
                  如果今天忘记完成，任务会进入暂存区等待处理（如：交水电费）
                </div>
              </span>
            </label>
            <label class="radio-item">
              <input type="radio" value="EXPIRE" v-model="expiryBehavior" />
              <span>
                <strong>自动过期</strong>
                <div class="radio-description">
                  如果今天没完成，任务自动失效，不再提醒（如：每日签到、游戏日常）
                </div>
              </span>
            </label>
          </div>
        </div>
      </details>

      <!-- 预览 -->
      <div class="rule-preview">
        <div class="preview-label">规则预览</div>
        <div class="preview-content">{{ ruleDescription }}</div>
        <div class="preview-code">{{ ruleString }}</div>
      </div>

      <!-- 按钮 -->
      <div class="dialog-actions">
        <button @click="handleCancel" class="btn-cancel">取消</button>
        <button @click="handleSave" class="btn-primary">确定</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog-backdrop {
  position: fixed;
  inset: 0;
  background: rgb(0 0 0 / 50%);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.dialog-content {
  background: white;
  border-radius: 12px;
  padding: 24px;
  max-width: 500px;
  width: 90%;
  max-height: 80vh;
  overflow-y: auto;
  box-shadow: 0 4px 20px rgb(0 0 0 / 15%);
}

h3 {
  margin: 0 0 8px;
  font-size: 1.5em;
}

.task-info {
  color: var(--color-text-secondary);
  font-size: 0.9em;
  margin-bottom: 20px;
}

.form-section {
  margin-bottom: 20px;
}

.section-label {
  display: block;
  font-weight: 600;
  margin-bottom: 8px;
  color: var(--color-text-primary);
}

.radio-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.radio-item {
  display: flex;
  align-items: center;
  padding: 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.2s;
}

.radio-item:hover {
  background: var(--color-background-hover);
}

.radio-item input[type='radio'] {
  margin-right: 8px;
  flex-shrink: 0;
}

.radio-item span {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.radio-description {
  font-size: 0.85em;
  color: var(--color-text-tertiary);
  font-weight: normal;
  line-height: 1.4;
}

.weekday-buttons {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.weekday-btn {
  padding: 8px 16px;
  border: 2px solid #ddd;
  border-radius: 20px;
  background: white;
  cursor: pointer;
  transition: all 0.2s;
}

.weekday-btn:hover {
  border-color: var(--color-border-hover);
}

.weekday-btn.active {
  background: var(--color-background-accent);
  color: var(--color-text-on-accent);
  border-color: var(--color-background-accent);
}

.interval-control {
  margin-top: 12px;
}

.interval-input {
  width: 60px;
  padding: 4px 8px;
  margin: 0 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  text-align: center;
}

.select-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 1em;
}

.inline-inputs {
  display: flex;
  gap: 12px;
}

.inline-inputs .select-input {
  flex: 1;
}

.date-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
}

.advanced-options {
  margin: 20px 0;
  padding: 16px;
  background: var(--color-background-secondary);
  border-radius: 8px;
}

.advanced-options summary {
  cursor: pointer;
  font-weight: 600;
  color: var(--color-text-accent);
}

.rule-preview {
  margin: 20px 0;
  padding: 16px;
  background: var(--color-background-secondary);
  border-radius: 8px;
}

.preview-label {
  font-weight: 600;
  margin-bottom: 8px;
  color: var(--color-text-primary);
}

.preview-content {
  margin-bottom: 8px;
  color: var(--color-text-secondary);
}

.preview-code {
  font-family: 'Courier New', monospace;
  font-size: 0.85em;
  color: var(--color-text-secondary);
  padding: 8px;
  background: var(--color-background-primary);
  border-radius: 4px;
  word-break: break-all;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 24px;
}

.btn-cancel,
.btn-primary {
  padding: 10px 24px;
  border-radius: 8px;
  font-size: 1em;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-cancel {
  background: var(--color-background-primary);
  border: 1px solid var(--color-border-default);
  color: var(--color-text-secondary);
}

.btn-cancel:hover {
  background: var(--color-background-hover);
}

.btn-primary {
  background: var(--color-background-accent);
  border: none;
  color: var(--color-text-on-accent);
}

.btn-primary:hover {
  background: var(--color-background-accent);
  filter: brightness(0.9);
}
</style>
