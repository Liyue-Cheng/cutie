<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { RRule, Frequency } from 'rrule'
import type { TaskCard } from '@/types/dtos'
import { useTemplateStore } from '@/stores/template'
import { useRecurrenceStore } from '@/stores/recurrence'
import { pipeline } from '@/cpu'
import { getTodayDateString } from '@/infra/utils/dateUtils'
import { dialog } from '@/composables/useDialog'

const props = defineProps<{
  task: TaskCard
  viewKey?: string // View context key (e.g., 'daily::2025-10-10', 'misc::staging')
  open: boolean
}>()

const emit = defineEmits<{
  close: []
  success: []
}>()

const { t } = useI18n()

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

// 判断当前是否精确匹配"工作日"（周一到周五）
const isExactlyWeekdays = computed(() => {
  if (freq.value !== RRule.WEEKLY) return false
  if (byweekday.value.length !== 5) return false
  const sorted = [...byweekday.value].sort((a, b) => a - b)
  return sorted.join(',') === '0,1,2,3,4'
})

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
    await dialog.alert(t('message.error.createRecurrenceFailed'))
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

function setWeekly() {
  freq.value = RRule.WEEKLY
  byweekday.value = [] // 清空选择，让用户自己选
}
</script>

<template>
  <div v-if="open" class="dialog-backdrop" @click.self="handleCancel">
    <div class="dialog-content">
      <h3>{{ $t('recurrence.title.config') }}</h3>
      <p class="task-info">{{ $t('recurrence.title.taskConfig', { title: task.title }) }}</p>

      <!-- REPEATS 部分 -->
      <section class="form-section">
        <label class="section-label">{{ $t('recurrence.label.frequency') }}</label>
        <div class="radio-group">
          <label class="radio-item">
            <input type="radio" :value="RRule.DAILY" v-model="freq" />
            <span>{{ $t('recurrence.freq.daily') }}</span>
          </label>
          <label class="radio-item" @click="setWeekdays">
            <input type="radio" :checked="isExactlyWeekdays" />
            <span>{{ $t('recurrence.freq.weekdays') }}</span>
          </label>
          <label class="radio-item" @click="setWeekly">
            <input type="radio" :checked="freq === RRule.WEEKLY && !isExactlyWeekdays" />
            <span>{{ $t('recurrence.freq.weekly') }}</span>
          </label>
          <label class="radio-item">
            <input type="radio" :value="RRule.MONTHLY" v-model="freq" />
            <span>{{ $t('recurrence.freq.monthly') }}</span>
          </label>
          <label class="radio-item">
            <input type="radio" :value="RRule.YEARLY" v-model="freq" />
            <span>{{ $t('recurrence.freq.yearly') }}</span>
          </label>
        </div>
      </section>

      <!-- 每周选项 -->
      <section v-if="freq === RRule.WEEKLY" class="form-section">
        <label class="section-label">{{ $t('recurrence.label.selectWeekday') }}</label>
        <div class="weekday-buttons">
          <button
            v-for="(key, index) in ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun']"
            :key="index"
            :class="{ active: byweekday.includes(index) }"
            @click="toggleWeekday(index)"
            type="button"
            class="weekday-btn"
          >
            {{ $t(`recurrence.weekday.${key}`) }}
          </button>
        </div>
        <div class="interval-control">
          <label>
            {{ $t('recurrence.label.interval') }}
            <input type="number" v-model.number="interval" min="1" max="4" class="interval-input" />
            {{ $t('recurrence.label.intervalSuffix') }}
          </label>
        </div>
      </section>

      <!-- 每月选项 -->
      <section v-if="freq === RRule.MONTHLY" class="form-section">
        <label class="section-label">{{ $t('recurrence.label.monthDay') }}</label>
        <select v-model.number="bymonthday" class="select-input">
          <option :value="null" disabled>{{ $t('common.action.select') }}</option>
          <option v-for="day in 31" :key="day" :value="day">{{ day }} {{ $t('recurrence.label.monthDaySuffix') }}</option>
        </select>
      </section>

      <!-- 每年选项 -->
      <section v-if="freq === RRule.YEARLY" class="form-section">
        <label class="section-label">{{ $t('recurrence.freq.yearly') }}</label>
        <div class="inline-inputs">
          <select v-model.number="bymonth" class="select-input">
            <option :value="null" disabled>{{ $t('common.action.selectMonth') }}</option>
            <option v-for="month in 12" :key="month" :value="month">{{ month }} {{ $t('recurrence.label.month') }}</option>
          </select>
          <select v-model.number="bymonthday" class="select-input">
            <option :value="null" disabled>{{ $t('common.action.selectDate') }}</option>
            <option v-for="day in 31" :key="day" :value="day">{{ day }} {{ $t('recurrence.label.monthDaySuffix') }}</option>
          </select>
        </div>
      </section>

      <!-- 过期行为 -->
      <section class="form-section">
        <label class="section-label">{{ $t('recurrence.label.expiryBehavior') }}</label>
        <div class="radio-group">
          <label class="radio-item">
            <input type="radio" value="CARRYOVER_TO_STAGING" v-model="expiryBehavior" />
            <span>
              <strong>{{ $t('recurrence.expiry.carryoverFull') }}</strong>
              <div class="radio-description">
                {{ $t('recurrence.expiry.carryoverDesc') }}
              </div>
            </span>
          </label>
          <label class="radio-item">
            <input type="radio" value="EXPIRE" v-model="expiryBehavior" />
            <span>
              <strong>{{ $t('recurrence.expiry.expire') }}</strong>
              <div class="radio-description">
                {{ $t('recurrence.expiry.expireDesc') }}
              </div>
            </span>
          </label>
        </div>
      </section>

      <!-- 结束日期 -->
      <section class="form-section">
        <label class="section-label">{{ $t('recurrence.label.endDate') }}</label>
        <input type="date" v-model="endDate" class="date-input" />
      </section>

      <!-- 按钮 -->
      <div class="dialog-actions">
        <button @click="handleCancel" class="btn-cancel">{{ $t('common.action.cancel') }}</button>
        <button @click="handleSave" class="btn-primary">{{ $t('common.action.confirm') }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 模态框背景遮罩 */
.dialog-backdrop {
  position: fixed;
  inset: 0;
  background: var(--color-overlay-heavy);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

/* 对话框主体 */
.dialog-content {
  background: var(--color-background-content);
  border: 1px solid var(--color-border-light);
  border-radius: 0.8rem;
  padding: 2.4rem;
  max-width: 54rem;
  width: 90%;
  max-height: 85vh;
  overflow-y: auto;
  box-shadow: var(--shadow-lg);
}

/* 标题 */
h3 {
  margin: 0 0 0.8rem;
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

/* 任务信息提示 */
.task-info {
  color: var(--color-text-secondary);
  font-size: 1.4rem;
  margin-bottom: 2.4rem;
  line-height: 1.5;
}

/* 表单区块 */
.form-section {
  margin-bottom: 2.4rem;
}

/* 区块标签 */
.section-label {
  display: block;
  font-weight: 600;
  font-size: 1.4rem;
  margin-bottom: 1.2rem;
  color: var(--color-text-secondary);
}

/* 单选组 */
.radio-group {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

/* 单选项 */
.radio-item {
  display: flex;
  align-items: flex-start;
  gap: 1rem;
  padding: 1.2rem;
  background: var(--color-background-secondary);
  border: 1px solid var(--color-border-light);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.radio-item:hover {
  background: var(--color-background-hover);
  border-color: var(--color-border-hover);
}

.radio-item input[type='radio'] {
  margin-top: 0.2rem;
  cursor: pointer;
  flex-shrink: 0;
  width: 1.6rem;
  height: 1.6rem;
}

.radio-item span {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  font-size: 1.4rem;
  color: var(--color-text-primary);
}

.radio-description {
  font-size: 1.2rem;
  color: var(--color-text-tertiary);
  font-weight: normal;
  line-height: 1.6;
}

/* 星期按钮组 */
.weekday-buttons {
  display: flex;
  gap: 0.8rem;
  flex-wrap: wrap;
}

.weekday-btn {
  padding: 0.8rem 1.6rem;
  border: 1px solid var(--color-border-default);
  border-radius: 0.6rem;
  background: var(--color-background-secondary);
  color: var(--color-text-primary);
  font-size: 1.4rem;
  cursor: pointer;
  transition: all 0.2s ease;
  user-select: none;
}

.weekday-btn:hover {
  border-color: var(--color-border-hover);
  background: var(--color-background-hover);
}

.weekday-btn.active {
  background: var(--color-button-primary-bg);
  color: var(--color-button-primary-text);
  border-color: var(--color-button-primary-bg);
}

/* 间隔控件 */
.interval-control {
  margin-top: 1.2rem;
  font-size: 1.4rem;
  color: var(--color-text-primary);
}

.interval-input {
  width: 6rem;
  padding: 0.6rem 1rem;
  margin: 0 0.8rem;
  border: 1px solid var(--color-border-input);
  border-radius: 0.4rem;
  background: var(--color-background-input);
  color: var(--color-text-primary);
  font-size: 1.4rem;
  text-align: center;
  transition: border-color 0.2s ease;
}

.interval-input:hover {
  border-color: var(--color-border-input-hover);
}

.interval-input:focus {
  outline: none;
  border-color: var(--color-border-input-focus);
  box-shadow: var(--shadow-focus);
}

/* 下拉选择框 */
.select-input {
  width: 100%;
  padding: 1rem 1.2rem;
  border: 1px solid var(--color-border-input);
  border-radius: 0.6rem;
  background: var(--color-background-input);
  color: var(--color-text-primary);
  font-size: 1.4rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.select-input:hover {
  border-color: var(--color-border-input-hover);
  background: var(--color-background-input-hover);
}

.select-input:focus {
  outline: none;
  border-color: var(--color-border-input-focus);
  box-shadow: var(--shadow-focus);
}

/* 内联输入组 */
.inline-inputs {
  display: flex;
  gap: 1.2rem;
}

.inline-inputs .select-input {
  flex: 1;
}

/* 日期输入框 */
.date-input {
  width: 100%;
  padding: 1rem 1.2rem;
  border: 1px solid var(--color-border-input);
  border-radius: 0.6rem;
  background: var(--color-background-input);
  color: var(--color-text-primary);
  font-size: 1.4rem;
  transition: all 0.2s ease;
}

.date-input:hover {
  border-color: var(--color-border-input-hover);
  background: var(--color-background-input-hover);
}

.date-input:focus {
  outline: none;
  border-color: var(--color-border-input-focus);
  box-shadow: var(--shadow-focus);
}

/* 操作按钮组 */
.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 1.2rem;
  margin-top: 2.4rem;
  padding-top: 2.4rem;
  border-top: 1px solid var(--color-divider);
}

/* 按钮基础样式 */
.btn-cancel,
.btn-primary {
  padding: 1rem 2.4rem;
  border-radius: 0.6rem;
  font-size: 1.4rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  border: none;
}

/* 取消按钮 */
.btn-cancel {
  background: var(--color-button-secondary-bg);
  border: 1px solid var(--color-button-secondary-border);
  color: var(--color-text-secondary);
}

.btn-cancel:hover {
  background: var(--color-button-secondary-hover);
  color: var(--color-text-primary);
}

/* 主要按钮 */
.btn-primary {
  background: var(--color-button-primary-bg);
  color: var(--color-button-primary-text);
}

.btn-primary:hover {
  background: var(--color-button-primary-hover);
}

.btn-primary:active {
  transform: scale(0.98);
}
</style>
