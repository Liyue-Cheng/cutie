<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { RRule, Frequency } from 'rrule'
import type { TimeBlockView } from '@/types/dtos'
import { pipeline } from '@/cpu'
import { getTodayDateString, toDateString } from '@/infra/utils/dateUtils'

const props = defineProps<{
  timeBlock: TimeBlockView
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
const skipConflicts = ref<boolean>(true)

// 监听对话框打开，自动设置初始值
watch(
  () => props.open,
  (isOpen) => {
    if (isOpen && props.timeBlock) {
      // 🔥 从时间块的 start_time 提取本地日期
      // 使用 dateUtils.toDateString 确保符合 TIME_CONVENTION.md 规范
      const startTimeDate = new Date(props.timeBlock.start_time)
      const blockDate = toDateString(startTimeDate)
      startDate.value = blockDate || getTodayDateString()
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

// 从时间块中提取时长（分钟）
const durationMinutes = computed(() => {
  const start = new Date(props.timeBlock.start_time)
  const end = new Date(props.timeBlock.end_time)
  return Math.round((end.getTime() - start.getTime()) / (1000 * 60))
})

// 从时间块中提取开始时间 (HH:MM:SS)
const startTimeLocal = computed(() => {
  if (props.timeBlock.start_time_local) {
    return props.timeBlock.start_time_local
  }
  // 如果没有本地时间，从 UTC 时间转换
  const date = new Date(props.timeBlock.start_time)
  const hours = date.getHours().toString().padStart(2, '0')
  const minutes = date.getMinutes().toString().padStart(2, '0')
  return `${hours}:${minutes}:00`
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
    // 使用 CPU 指令创建时间块循环规则
    await pipeline.dispatch('timeblock-recurrence.create', {
      // 模板信息（从当前时间块复制）
      title: props.timeBlock.title,
      glance_note_template: props.timeBlock.glance_note ?? undefined,
      detail_note_template: props.timeBlock.detail_note ?? undefined,
      duration_minutes: durationMinutes.value,
      start_time_local: startTimeLocal.value,
      time_type: props.timeBlock.time_type,
      is_all_day: props.timeBlock.is_all_day,
      area_id: props.timeBlock.area_id ?? undefined,

      // 循环规则信息
      rule: ruleString.value,
      start_date: startDate.value,
      end_date: endDate.value,
      skip_conflicts: skipConflicts.value,

      // 将当前时间块作为第一个实例
      source_time_block_id: props.timeBlock.id,
    })

    emit('success')
    emit('close')
  } catch (error) {
    console.error('Failed to create time block recurrence:', error)
    alert(t('message.error.createRecurrenceFailed'))
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
      <h3>{{ $t('recurrence.title.timeBlockConfig') }}</h3>
      <p class="block-info">
        {{
          $t('recurrence.title.timeBlockConfigDesc', {
            title: timeBlock.title || $t('timeBlock.label.untitled'),
          })
        }}
      </p>

      <!-- 时间块摘要 -->
      <section class="time-block-summary">
        <div class="summary-item">
          <span class="label">{{ $t('timeBlock.label.startTime') }}:</span>
          <span class="value">{{ startTimeLocal }}</span>
        </div>
        <div class="summary-item">
          <span class="label">{{ $t('timeBlock.label.duration') }}:</span>
          <span class="value">{{ durationMinutes }} {{ $t('common.unit.minutes') }}</span>
        </div>
      </section>

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
          <option v-for="day in 31" :key="day" :value="day">
            {{ day }} {{ $t('recurrence.label.monthDaySuffix') }}
          </option>
        </select>
      </section>

      <!-- 冲突处理 -->
      <section class="form-section">
        <label class="section-label">{{ $t('recurrence.label.conflictBehavior') }}</label>
        <div class="radio-group">
          <label class="radio-item">
            <input type="radio" :value="true" v-model="skipConflicts" />
            <span>
              <strong>{{ $t('recurrence.conflict.skip') }}</strong>
              <div class="radio-description">
                {{ $t('recurrence.conflict.skipDesc') }}
              </div>
            </span>
          </label>
          <label class="radio-item">
            <input type="radio" :value="false" v-model="skipConflicts" />
            <span>
              <strong>{{ $t('recurrence.conflict.error') }}</strong>
              <div class="radio-description">
                {{ $t('recurrence.conflict.errorDesc') }}
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

/* 时间块信息提示 */
.block-info {
  color: var(--color-text-secondary);
  font-size: 1.4rem;
  margin-bottom: 1.6rem;
  line-height: 1.5;
}

/* 时间块摘要 */
.time-block-summary {
  display: flex;
  gap: 2rem;
  padding: 1.2rem;
  background: var(--color-background-secondary);
  border-radius: 0.6rem;
  margin-bottom: 2.4rem;
}

.summary-item {
  display: flex;
  gap: 0.6rem;
  font-size: 1.4rem;
}

.summary-item .label {
  color: var(--color-text-tertiary);
}

.summary-item .value {
  color: var(--color-text-primary);
  font-weight: 500;
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
