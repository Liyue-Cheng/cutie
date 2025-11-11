<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { RRule, Frequency } from 'rrule'
import type { TaskRecurrence } from '@/types/dtos'
import { pipeline } from '@/cpu'

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

    // 🔥 只有当 expiry_behavior 发生变化时才包含该字段
    if (expiryBehavior.value !== props.recurrence.expiry_behavior) {
      payload.expiry_behavior = expiryBehavior.value
    }

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
    alert('更新循环规则失败')
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

      <!-- 过期行为 -->
      <section class="form-section">
        <label class="section-label">过期后的处理方式</label>
        <div class="radio-group">
          <label class="radio-item">
            <input
              type="radio"
              value="CARRYOVER_TO_STAGING"
              v-model="expiryBehavior"
            />
            <span>
              <strong>结转到暂存区</strong>
              <div class="radio-description">如果今天忘记完成，任务会进入暂存区等待处理（如：交水电费）</div>
            </span>
          </label>
          <label class="radio-item">
            <input
              type="radio"
              value="EXPIRE"
              v-model="expiryBehavior"
            />
            <span>
              <strong>自动过期</strong>
              <div class="radio-description">如果今天没完成，任务自动失效，不再提醒（如：每日签到、游戏日常）</div>
            </span>
          </label>
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
  max-width: 600px;
  width: 90%;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 4px 16px rgb(0 0 0 / 20%);
}

h3 {
  margin: 0 0 8px;
  font-size: 1.8rem;
  color: #333;
}

.info-text {
  margin: 0 0 20px;
  font-size: 1.4rem;
  color: #666;
}

.form-section {
  margin-bottom: 20px;
}

.section-label {
  display: block;
  font-size: 1.4rem;
  font-weight: 600;
  color: #555;
  margin-bottom: 12px;
}

.radio-group {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.radio-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
}

.radio-item:hover {
  background: #f5f5f5;
  border-color: #999;
}

.radio-item input[type='radio'] {
  cursor: pointer;
  flex-shrink: 0;
}

.radio-item span {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.radio-description {
  font-size: 0.85em;
  color: #888;
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
  border: 1px solid #ddd;
  border-radius: 6px;
  background: white;
  cursor: pointer;
  transition: all 0.2s;
}

.weekday-btn:hover {
  background: #f0f0f0;
}

.weekday-btn.active {
  background: #4caf50;
  color: white;
  border-color: #4caf50;
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
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 1.4rem;
}

.inline-inputs {
  display: flex;
  gap: 12px;
}

.inline-inputs .select-input {
  flex: 1;
}

.date-inputs {
  display: flex;
  gap: 12px;
}

.date-input-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.date-input-wrapper label {
  font-size: 1.2rem;
  color: #666;
}

.date-input {
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 1.4rem;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
}

.checkbox-label input[type='checkbox'] {
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.preview-section {
  background: #f5f5f5;
  padding: 16px;
  border-radius: 6px;
}

.rule-preview {
  font-size: 1.4rem;
  color: #333;
  line-height: 1.5;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 24px;
}

.btn {
  padding: 10px 24px;
  border: none;
  border-radius: 6px;
  font-size: 1.4rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-secondary {
  background: #e0e0e0;
  color: #555;
}

.btn-secondary:hover {
  background: #d0d0d0;
}

.btn-primary {
  background: #4caf50;
  color: white;
}

.btn-primary:hover {
  background: #45a049;
}
</style>
