<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { RRule, Frequency } from 'rrule'
import type { TaskRecurrence } from '@/types/dtos'
import { pipeline } from '@/cpu'
import { dialog } from '@/composables/useDialog'

const props = defineProps<{
  recurrence: TaskRecurrence | null
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

// 当打开对话框时，从现有规则中解析参数
watch(
  () => props.open,
  (isOpen) => {
    if (isOpen && props.recurrence) {
      parseExistingRule(props.recurrence)
    }
  },
  { immediate: true }
)

// 解析现有的 RRULE
function parseExistingRule(recurrence: TaskRecurrence) {
  try {
    const rule = RRule.fromString(recurrence.rule)
    const options = rule.origOptions

    // 频率/间隔
    freq.value = options.freq ?? RRule.DAILY
    interval.value = options.interval || 1

    // byweekday 归一化为 number[] (0=MO ... 6=SU)
    const normalizeWeekday = (d: unknown): number | null => {
      if (typeof d === 'number') return d
      if (typeof d === 'string') {
        const map: Record<string, number> = {
          MO: 0,
          TU: 1,
          WE: 2,
          TH: 3,
          FR: 4,
          SA: 5,
          SU: 6,
        }
        return map[d] ?? null
      }
      if (typeof d === 'object' && d !== null && 'weekday' in (d as any)) {
        return (d as any).weekday ?? null
      }
      return null
    }

    if (options.byweekday) {
      const raw = Array.isArray(options.byweekday) ? options.byweekday : [options.byweekday]
      byweekday.value = raw
        .map((d) => normalizeWeekday(d))
        .filter((x): x is number => typeof x === 'number')
    } else {
      byweekday.value = []
    }

    // bymonthday / bymonth 归一化
    if (options.bymonthday) {
      bymonthday.value = Array.isArray(options.bymonthday)
        ? (options.bymonthday[0] ?? null)
        : (options.bymonthday ?? null)
    } else {
      bymonthday.value = null
    }

    if (options.bymonth) {
      bymonth.value = Array.isArray(options.bymonth)
        ? (options.bymonth[0] ?? null)
        : (options.bymonth ?? null)
    } else {
      bymonth.value = null
    }
    startDate.value = recurrence.start_date
    endDate.value = recurrence.end_date
    expiryBehavior.value = recurrence.expiry_behavior // 加载过期行为
  } catch (e) {
    console.error('Failed to parse RRULE:', e)
    // 使用默认值
    freq.value = RRule.DAILY
    interval.value = 1
    byweekday.value = []
    bymonthday.value = null
    bymonth.value = null
  }
}

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
  if (!props.recurrence) return

  try {
    // 🔥 构造符合后端三态字段要求的 payload
    const payload: any = {
      rule: ruleString.value,
    }

    // 🔥 注意：后端禁止修改 start_date，所以不发送该字段
    // if (startDate.value !== props.recurrence.start_date) {
    //   payload.start_date = startDate.value || null
    // }

    // 🔥 只有当 end_date 发生变化时才包含该字段
    if (endDate.value !== props.recurrence.end_date) {
      payload.end_date = endDate.value || null // 空字符串转为 null
    }

    // 🔥 expiry_behavior 不允许在编辑时修改，所以不发送该字段

    console.log('Updating recurrence with payload:', payload)

    // 使用CPU指令更新循环规则
    await pipeline.dispatch('recurrence.update', {
      id: props.recurrence.id,
      ...payload,
    })

    emit('success')
    emit('close')
    // ✅ 视图刷新由 CPU 指令的 commit 阶段统一处理
  } catch (error) {
    console.error('Failed to update recurrence:', error)
    await dialog.alert('更新循环规则失败')
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
  <div v-if="open && recurrence" class="dialog-backdrop" @click.self="handleCancel">
    <div class="dialog-content">
      <h3>编辑循环规则</h3>
      <p class="info-text">编辑循环规则，已生成的任务不会受影响</p>

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

      <!-- 开始/结束日期 -->
      <section class="form-section">
        <label class="section-label">生效时间</label>
        <div class="date-inputs">
          <div class="date-input-wrapper">
            <label>开始日期</label>
            <input
              type="date"
              v-model="startDate"
              class="date-input"
              disabled
              title="开始日期不可修改"
            />
          </div>
          <div class="date-input-wrapper">
            <label>结束日期（可选）</label>
            <input type="date" v-model="endDate" class="date-input" />
          </div>
        </div>
      </section>

      <!-- 过期行为（只读显示，不可编辑） -->
      <section class="form-section">
        <label class="section-label">过期后的处理方式（不可修改）</label>
        <div class="expiry-readonly">
          <template v-if="expiryBehavior === 'CARRYOVER_TO_STAGING'">
            <strong>结转到暂存区</strong>
            <div class="radio-description">
              如果今天忘记完成，任务会进入暂存区等待处理（如：交水电费）
            </div>
          </template>
          <template v-else>
            <strong>自动过期</strong>
            <div class="radio-description">
              如果今天没完成，任务自动失效，不再提醒（如：每日签到、游戏日常）
            </div>
          </template>
        </div>
      </section>

      <!-- 规则预览 -->
      <section class="form-section preview-section">
        <label class="section-label">规则预览</label>
        <div class="rule-preview">{{ ruleDescription }}</div>
      </section>

      <!-- 操作按钮 -->
      <div class="dialog-actions">
        <button @click="handleCancel" class="btn btn-secondary">取消</button>
        <button @click="handleSave" class="btn btn-primary">保存</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 模态框背景遮罩 */
.dialog-backdrop {
  position: fixed;
  inset: 0;
  background: var(--color-overlay-heavy, #f0f);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

/* 对话框主体 */
.dialog-content {
  background: var(--color-background-content, #f0f);
  border: 1px solid var(--color-border-light, #f0f);
  border-radius: 0.8rem;
  padding: 2.4rem;
  max-width: 54rem;
  width: 90%;
  max-height: 85vh;
  overflow-y: auto;
  box-shadow: var(--shadow-lg, #f0f);
}

/* 标题 */
h3 {
  margin: 0 0 0.8rem;
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
}

/* 信息提示 */
.info-text {
  color: var(--color-text-secondary, #f0f);
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
  color: var(--color-text-secondary, #f0f);
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
  background: var(--color-background-secondary, #f0f);
  border: 1px solid var(--color-border-light, #f0f);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.radio-item:hover {
  background: var(--color-background-hover, #f0f);
  border-color: var(--color-border-hover, #f0f);
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
  color: var(--color-text-primary, #f0f);
}

.radio-description {
  font-size: 1.2rem;
  color: var(--color-text-tertiary, #f0f);
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
  border: 1px solid var(--color-border-default, #f0f);
  border-radius: 0.6rem;
  background: var(--color-background-secondary, #f0f);
  color: var(--color-text-primary, #f0f);
  font-size: 1.4rem;
  cursor: pointer;
  transition: all 0.2s ease;
  user-select: none;
}

.weekday-btn:hover {
  border-color: var(--color-border-hover, #f0f);
  background: var(--color-background-hover, #f0f);
}

.weekday-btn.active {
  background: var(--color-button-primary-bg, #f0f);
  color: var(--color-button-primary-text, #f0f);
  border-color: var(--color-button-primary-bg, #f0f);
}

/* 间隔控件 */
.interval-control {
  margin-top: 1.2rem;
  font-size: 1.4rem;
  color: var(--color-text-primary, #f0f);
}

.interval-input {
  width: 6rem;
  padding: 0.6rem 1rem;
  margin: 0 0.8rem;
  border: 1px solid var(--color-border-input, #f0f);
  border-radius: 0.4rem;
  background: var(--color-background-input, #f0f);
  color: var(--color-text-primary, #f0f);
  font-size: 1.4rem;
  text-align: center;
  transition: border-color 0.2s ease;
}

.interval-input:hover {
  border-color: var(--color-border-input-hover, #f0f);
}

.interval-input:focus {
  outline: none;
  border-color: var(--color-border-input-focus, #f0f);
  box-shadow: var(--shadow-focus, #f0f);
}

/* 下拉选择框 */
.select-input {
  width: 100%;
  padding: 1rem 1.2rem;
  border: 1px solid var(--color-border-input, #f0f);
  border-radius: 0.6rem;
  background: var(--color-background-input, #f0f);
  color: var(--color-text-primary, #f0f);
  font-size: 1.4rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.select-input:hover {
  border-color: var(--color-border-input-hover, #f0f);
  background: var(--color-background-input-hover, #f0f);
}

.select-input:focus {
  outline: none;
  border-color: var(--color-border-input-focus, #f0f);
  box-shadow: var(--shadow-focus, #f0f);
}

/* 内联输入组 */
.inline-inputs {
  display: flex;
  gap: 1.2rem;
}

.inline-inputs .select-input {
  flex: 1;
}

/* 日期输入区域 */
.date-inputs {
  display: flex;
  gap: 1.2rem;
}

.date-input-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.date-input-wrapper label {
  font-size: 1.2rem;
  color: var(--color-text-secondary, #f0f);
}

/* 日期输入框 */
.date-input {
  width: 100%;
  padding: 1rem 1.2rem;
  border: 1px solid var(--color-border-input, #f0f);
  border-radius: 0.6rem;
  background: var(--color-background-input, #f0f);
  color: var(--color-text-primary, #f0f);
  font-size: 1.4rem;
  transition: all 0.2s ease;
}

.date-input:hover:not(:disabled) {
  border-color: var(--color-border-input-hover, #f0f);
  background: var(--color-background-input-hover, #f0f);
}

.date-input:focus {
  outline: none;
  border-color: var(--color-border-input-focus, #f0f);
  box-shadow: var(--shadow-focus, #f0f);
}

.date-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  background: var(--color-background-secondary, #f0f);
}

/* 过期行为只读显示 */
.expiry-readonly {
  padding: 1.2rem;
  background: var(--color-background-secondary, #f0f);
  border: 1px solid var(--color-border-light, #f0f);
  border-radius: 0.6rem;
}

.expiry-readonly strong {
  display: block;
  font-size: 1.4rem;
  color: var(--color-text-primary, #f0f);
  margin-bottom: 0.4rem;
}

.expiry-readonly .radio-description {
  font-size: 1.2rem;
  color: var(--color-text-tertiary, #f0f);
  line-height: 1.6;
}

/* 规则预览区块 */
.preview-section {
  background: var(--color-background-secondary, #f0f);
  padding: 1.6rem;
  border-radius: 0.6rem;
  border: 1px solid var(--color-border-light, #f0f);
}

.rule-preview {
  font-size: 1.4rem;
  color: var(--color-text-primary, #f0f);
  line-height: 1.5;
}

/* 操作按钮组 */
.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 1.2rem;
  margin-top: 2.4rem;
  padding-top: 2.4rem;
  border-top: 1px solid var(--color-divider, #f0f);
}

/* 按钮基础样式 */
.btn {
  padding: 1rem 2.4rem;
  border-radius: 0.6rem;
  font-size: 1.4rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  border: none;
}

/* 次要按钮 */
.btn-secondary {
  background: var(--color-button-secondary-bg, #f0f);
  border: 1px solid var(--color-button-secondary-border, #f0f);
  color: var(--color-text-secondary, #f0f);
}

.btn-secondary:hover {
  background: var(--color-button-secondary-hover, #f0f);
  color: var(--color-text-primary, #f0f);
}

/* 主要按钮 */
.btn-primary {
  background: var(--color-button-primary-bg, #f0f);
  color: var(--color-button-primary-text, #f0f);
}

.btn-primary:hover {
  background: var(--color-button-primary-hover, #f0f);
}

.btn-primary:active {
  transform: scale(0.98);
}
</style>
