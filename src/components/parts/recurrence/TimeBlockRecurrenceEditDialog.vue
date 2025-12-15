<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { RRule, Frequency } from 'rrule'
import { pipeline } from '@/cpu'
import type {
  EditTimeBlockRecurrencePayload,
  TimeBlockRecurrence,
  TimeBlockTemplateInfo,
} from '@/types/dtos'
import { getNowLocalISOStringMinutes } from '@/infra/utils/dateUtils'
import { useUIStore } from '@/stores/ui'
import { dialog } from '@/composables/useDialog'

const props = defineProps<{
  recurrenceId: string
}>()

const emit = defineEmits<{
  close: []
}>()

const { t } = useI18n()
const uiStore = useUIStore()

// 状态
const loading = ref(true)
const saving = ref(false)
const recurrence = ref<TimeBlockRecurrence | null>(null)

// 表单字段
const freq = ref<Frequency>(RRule.DAILY)
const interval = ref(1)
const byweekday = ref<number[]>([])
const bymonthday = ref<number | null>(null)
const bymonth = ref<number | null>(null)
const endDate = ref<string | null>(null)
const deleteFutureInstances = ref(true)

const title = ref<string | null>(null)
const glanceNote = ref<string | null>(null)
const detailNote = ref<string | null>(null)
const durationMinutes = ref<number | null>(null)
const isAllDay = ref(false)
const areaId = ref<string | null>(null)

const templateInfo = computed<TimeBlockTemplateInfo | null>(() => recurrence.value?.template ?? null)
const startTimeLocal = computed(() => templateInfo.value?.start_time_local ?? '--:--:--')
const startDateDisplay = computed(() => recurrence.value?.start_date ?? t('common.label.unknown'))

const canSubmit = computed(() => !loading.value && !saving.value && recurrence.value !== null)

function resetForm() {
  freq.value = RRule.DAILY
  interval.value = 1
  byweekday.value = []
  bymonthday.value = null
  bymonth.value = null
  endDate.value = recurrence.value?.end_date ?? null
  deleteFutureInstances.value = true

  title.value = templateInfo.value?.title ?? null
  glanceNote.value = templateInfo.value?.glance_note_template ?? null
  detailNote.value = templateInfo.value?.detail_note_template ?? null
  durationMinutes.value = templateInfo.value?.duration_minutes ?? null
  isAllDay.value = templateInfo.value?.is_all_day ?? false
  areaId.value = templateInfo.value?.area_id ?? null

  if (recurrence.value?.rule) {
    applyRuleToForm(recurrence.value.rule)
  }
}

function applyRuleToForm(rule: string) {
  try {
    const normalized = rule.startsWith('RRULE:') ? rule : `RRULE:${rule}`
    const parsed = RRule.fromString(normalized)
    freq.value = parsed.options.freq ?? RRule.DAILY
    interval.value = parsed.options.interval ?? 1

    if (Array.isArray(parsed.options.byweekday) && parsed.options.byweekday.length > 0) {
      byweekday.value = parsed.options.byweekday
        .map((weekday) => {
          if (typeof weekday === 'number') return weekday
          if (typeof weekday.weekday === 'number') {
            // rrule 库中：0=MO
            return weekday.weekday
          }
          return null
        })
        .filter((val): val is number => val !== null)
    } else {
      byweekday.value = []
    }

    if (Array.isArray(parsed.options.bymonthday) && parsed.options.bymonthday.length > 0) {
      bymonthday.value = parsed.options.bymonthday[0] ?? null
    } else {
      bymonthday.value = null
    }

    if (Array.isArray(parsed.options.bymonth) && parsed.options.bymonth.length > 0) {
      bymonth.value = parsed.options.bymonth[0] ?? null
    } else {
      bymonth.value = null
    }
  } catch (error) {
    console.warn('Failed to parse RRULE, fallback to defaults', error)
    freq.value = RRule.DAILY
    interval.value = 1
    byweekday.value = []
    bymonthday.value = null
    bymonth.value = null
  }
}

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
  return rule.toString().replace('RRULE:', '')
})

function toggleWeekday(day: number) {
  const index = byweekday.value.indexOf(day)
  if (index > -1) {
    byweekday.value.splice(index, 1)
  } else {
    byweekday.value.push(day)
  }
}

const isExactlyWeekdays = computed(() => {
  if (freq.value !== RRule.WEEKLY) return false
  if (byweekday.value.length !== 5) return false
  const sorted = [...byweekday.value].sort((a, b) => a - b)
  return sorted.join(',') === '0,1,2,3,4'
})

function setWeekdays() {
  freq.value = RRule.WEEKLY
  byweekday.value = [0, 1, 2, 3, 4]
}

function setWeekly() {
  freq.value = RRule.WEEKLY
  byweekday.value = []
}

async function loadRecurrenceDetail() {
  loading.value = true
  try {
    const result = (await pipeline.dispatch('timeblock-recurrence.get', {
      id: props.recurrenceId,
    })) as TimeBlockRecurrence
    recurrence.value = result
    resetForm()
  } catch (error) {
    console.error('Failed to load recurrence detail', error)
    await dialog.alert(t('message.error.loadRecurrenceFailed'))
    emit('close')
  } finally {
    loading.value = false
  }
}

async function handleSave() {
  if (!recurrence.value || !canSubmit.value) return
  saving.value = true
  try {
    const payload: EditTimeBlockRecurrencePayload = {
      id: recurrence.value.id,
      rule: ruleString.value,
      end_date: endDate.value || null,
      time_type: recurrence.value.time_type,
      title: title.value ?? null,
      glance_note_template: glanceNote.value ?? null,
      detail_note_template: detailNote.value ?? null,
      duration_minutes: durationMinutes.value ?? undefined,
      is_all_day: isAllDay.value,
      area_id: areaId.value ?? null,
      local_now: getNowLocalISOStringMinutes(),
      delete_future_instances: deleteFutureInstances.value,
    }

    await pipeline.dispatch('timeblock-recurrence.edit', payload)
    saving.value = false
    await dialog.alert(t('message.success.updateRecurrence'))
    closeDialog()
  } catch (error) {
    saving.value = false
    console.error('Failed to edit recurrence', error)
    await dialog.alert(t('message.error.updateRecurrenceFailed'))
  }
}

function closeDialog() {
  emit('close')
  uiStore.closeTimeBlockRecurrenceEditDialog()
}

watch(
  () => props.recurrenceId,
  () => {
    if (props.recurrenceId) {
      loadRecurrenceDetail()
    }
  },
  { immediate: true }
)
</script>

<template>
  <div class="dialog-backdrop" @click.self="closeDialog">
    <div class="dialog-content">
      <header class="dialog-header">
        <div>
          <h3>{{ t('recurrence.title.timeBlockEdit') }}</h3>
          <p class="subtitle">{{ t('recurrence.title.timeBlockEditDesc') }}</p>
        </div>
        <button class="close-btn" @click="closeDialog">×</button>
      </header>

      <div v-if="loading" class="loading-state">
        {{ t('common.state.loading') }}…
      </div>

      <div v-else class="dialog-body">
        <section class="summary-card">
          <div>
            <strong>{{ t('timeBlock.label.startTime') }}:</strong>
            <span>{{ startTimeLocal }}</span>
          </div>
          <div>
            <strong>{{ t('recurrence.label.startDate') }}:</strong>
            <span>{{ startDateDisplay }}</span>
          </div>
          <div>
            <strong>{{ t('timeBlock.label.duration') }}:</strong>
            <span>{{ durationMinutes ?? '--' }} {{ t('common.unit.minutes') }}</span>
          </div>
        </section>

        <section class="form-section">
          <label class="section-label">{{ t('recurrence.label.frequency') }}</label>
          <div class="radio-group">
            <label class="radio-item">
              <input type="radio" :value="RRule.DAILY" v-model="freq" />
              <span>{{ t('recurrence.freq.daily') }}</span>
            </label>
            <label class="radio-item" @click="setWeekdays">
              <input type="radio" :checked="isExactlyWeekdays" />
              <span>{{ t('recurrence.freq.weekdays') }}</span>
            </label>
            <label class="radio-item" @click="setWeekly">
              <input type="radio" :checked="freq === RRule.WEEKLY && !isExactlyWeekdays" />
              <span>{{ t('recurrence.freq.weekly') }}</span>
            </label>
            <label class="radio-item">
              <input type="radio" :value="RRule.MONTHLY" v-model="freq" />
              <span>{{ t('recurrence.freq.monthly') }}</span>
            </label>
          </div>
        </section>

        <section v-if="freq === RRule.WEEKLY" class="form-section">
          <label class="section-label">{{ t('recurrence.label.selectWeekday') }}</label>
          <div class="weekday-buttons">
            <button
              v-for="(key, index) in ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun']"
              :key="index"
              type="button"
              :class="{ active: byweekday.includes(index) }"
              class="weekday-btn"
              @click="toggleWeekday(index)"
            >
              {{ t(`recurrence.weekday.${key}`) }}
            </button>
          </div>
          <div class="interval-control">
            <label>
              {{ t('recurrence.label.interval') }}
              <input
                type="number"
                v-model.number="interval"
                min="1"
                max="4"
                class="interval-input"
              />
              {{ t('recurrence.label.intervalSuffix') }}
            </label>
          </div>
        </section>

        <section v-if="freq === RRule.MONTHLY" class="form-section">
          <label class="section-label">{{ t('recurrence.label.monthDay') }}</label>
          <select v-model.number="bymonthday" class="select-input">
            <option :value="null" disabled>{{ t('common.action.select') }}</option>
            <option v-for="day in 31" :key="day" :value="day">
              {{ day }} {{ t('recurrence.label.monthDaySuffix') }}
            </option>
          </select>
        </section>

        <section class="form-section">
          <label class="section-label">{{ t('recurrence.label.endDate') }}</label>
          <input type="date" v-model="endDate" class="date-input" />
        </section>

        <section class="form-section">
          <label class="section-label">{{ t('timeBlock.label.duration') }}</label>
          <input
            type="number"
            v-model.number="durationMinutes"
            min="5"
            step="5"
            class="interval-input"
          />
        </section>

        <section class="form-section">
          <label class="section-label">{{ t('recurrence.label.templateTitle') }}</label>
          <input type="text" v-model="title" class="text-input" autocomplete="off" />
        </section>

        <section class="form-section">
          <label class="section-label">{{ t('recurrence.label.shortNote') }}</label>
          <textarea v-model="glanceNote" class="textarea-input" rows="2"></textarea>
        </section>

        <section class="form-section">
          <label class="section-label">{{ t('recurrence.label.detailNote') }}</label>
          <textarea v-model="detailNote" class="textarea-input" rows="4"></textarea>
        </section>

        <section class="form-section">
          <label class="section-label">{{ t('recurrence.label.futureInstances') }}</label>
          <label class="toggle-item">
            <input type="checkbox" v-model="deleteFutureInstances" />
            <span>
              <strong>{{ t('recurrence.action.deleteFutureInstances') }}</strong>
              <div class="radio-description">
                {{ t('recurrence.label.deleteFutureInstancesDesc') }}
              </div>
            </span>
          </label>
        </section>
      </div>

      <footer class="dialog-actions">
        <button class="btn-cancel" @click="closeDialog">{{ t('common.action.cancel') }}</button>
        <button class="btn-primary" :disabled="!canSubmit" @click="handleSave">
          {{ saving ? t('common.state.saving') : t('common.action.save') }}
        </button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.dialog-backdrop {
  position: fixed;
  inset: 0;
  background: var(--color-overlay-heavy);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1300;
}

.dialog-content {
  width: min(720px, 92vw);
  max-height: 90vh;
  background: var(--color-background-content);
  border-radius: 0.8rem;
  border: 1px solid var(--color-border-light);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: var(--shadow-xl);
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 1.6rem 2rem;
  border-bottom: 1px solid var(--color-border-light);
}

.dialog-header h3 {
  margin: 0;
  font-size: 1.8rem;
}

.subtitle {
  margin: 0.4rem 0 0;
  color: var(--color-text-secondary);
  font-size: 1.3rem;
}

.close-btn {
  background: none;
  border: none;
  font-size: 2rem;
  line-height: 1;
  cursor: pointer;
  color: var(--color-text-secondary);
}

.dialog-body {
  padding: 1.6rem 2rem;
  overflow-y: auto;
  flex: 1;
}

.loading-state {
  padding: 2rem;
  text-align: center;
  color: var(--color-text-secondary);
}

.summary-card {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
  background: var(--color-background-secondary);
  padding: 1.2rem;
  border-radius: 0.6rem;
  margin-bottom: 1.8rem;
}

.form-section {
  margin-bottom: 1.8rem;
}

.section-label {
  display: block;
  margin-bottom: 0.8rem;
  font-weight: 600;
  color: var(--color-text-secondary);
}

.radio-group {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

.radio-item,
.toggle-item {
  display: flex;
  gap: 1rem;
  padding: 1rem;
  border: 1px solid var(--color-border-light);
  border-radius: 0.6rem;
}

.radio-item input[type='radio'],
.toggle-item input[type='checkbox'] {
  margin-top: 0.4rem;
}

.radio-description {
  color: var(--color-text-tertiary);
  font-size: 1.2rem;
}

.weekday-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 0.6rem;
}

.weekday-btn {
  padding: 0.6rem 1.2rem;
  border-radius: 0.6rem;
  border: 1px solid var(--color-border-light);
  background: var(--color-background-secondary);
  cursor: pointer;
}

.weekday-btn.active {
  background: var(--color-button-primary-bg);
  color: var(--color-button-primary-text);
  border-color: var(--color-button-primary-bg);
}

.interval-input,
.text-input,
.date-input,
.select-input,
.textarea-input {
  width: 100%;
  padding: 0.8rem 1rem;
  border-radius: 0.6rem;
  border: 1px solid var(--color-border-input);
  background: var(--color-background-input);
}

.textarea-input {
  resize: vertical;
}

.dialog-actions {
  padding: 1.2rem 2rem;
  border-top: 1px solid var(--color-border-light);
  display: flex;
  justify-content: flex-end;
  gap: 1rem;
}

.btn-cancel,
.btn-primary {
  padding: 0.8rem 1.6rem;
  border-radius: 0.6rem;
  border: none;
  cursor: pointer;
  font-weight: 600;
}

.btn-cancel {
  background: var(--color-button-secondary-bg);
  color: var(--color-text-secondary);
}

.btn-primary {
  background: var(--color-button-primary-bg);
  color: var(--color-button-primary-text);
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
