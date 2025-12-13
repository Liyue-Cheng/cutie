<template>
  <div class="daily-shutdown-wizard">
    <div class="wizard-header">
      <h1 class="wizard-title">{{ title }}</h1>
      <p class="wizard-subtitle">{{ subtitle }}</p>
    </div>

    <div class="wizard-content">
      <div class="chart-section">
        <div class="chart-wrapper">
          <div class="chart-glow"></div>
          <div class="chart-container">
            <VChart class="donut-chart" :option="chartOption" autoresize />
          </div>
        </div>

        <div class="chart-legend">
          <div class="legend-item">
            <span class="dot completed"></span>
            <span class="legend-name">{{ $t('dailyShutdown.toolbar.completed') }}</span>
            <span class="legend-count">{{ completed }}</span>
          </div>
          <div class="legend-item">
            <span class="dot present"></span>
            <span class="legend-name">{{ $t('task.status.present') }}</span>
            <span class="legend-count">{{ present }}</span>
          </div>
          <div class="legend-item">
            <span class="dot incomplete"></span>
            <span class="legend-name">{{ $t('task.status.incomplete') }}</span>
            <span class="legend-count">{{ incomplete }}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="wizard-navigation">
      <button class="nav-btn next-btn" @click="$emit('next')">
        {{ $t('dailyShutdown.next') }}
      </button>
    </div>

    <div class="wizard-hint">
      <p class="hint-text">{{ hint }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import VChart from 'vue-echarts'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { PieChart } from 'echarts/charts'
import { TooltipComponent, GraphicComponent } from 'echarts/components'
import type { ECBasicOption } from 'echarts/types/dist/shared'

use([CanvasRenderer, PieChart, TooltipComponent, GraphicComponent])

interface Props {
  completed: number
  present: number
  incomplete: number
  title: string
  subtitle: string
  hint: string
}

const props = defineProps<Props>()

const chartBgColor = ref('#f0f')
const chartTextColor = ref('#f0f')
const chartFontFamily = ref(
  'system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif'
)
const completedColor = ref('#f0f')
const presentColor = ref('#f0f')
const incompleteColor = ref('#f0f')

onMounted(() => {
  const root = document.documentElement
  const styles = getComputedStyle(root)
  const contentBg = styles.getPropertyValue('--color-background-content').trim()
  const primaryBg = styles.getPropertyValue('--color-background-primary').trim()
  chartBgColor.value = contentBg || primaryBg || '#f0f'

  const textPrimary = styles.getPropertyValue('--color-text-primary').trim()
  chartTextColor.value = textPrimary || '#f0f'

  const statusCompleted = styles.getPropertyValue('--color-status-completed').trim()
  completedColor.value = statusCompleted || '#f0f'
  const statusPresent = styles.getPropertyValue('--color-status-present').trim()
  presentColor.value = statusPresent || '#f0f'
  const textTertiary = styles.getPropertyValue('--color-text-tertiary').trim()
  incompleteColor.value = textTertiary || '#f0f'

  const bodyFont = getComputedStyle(document.body).fontFamily?.trim()
  if (bodyFont) chartFontFamily.value = bodyFont
})

const total = computed(() => Math.max(0, props.completed + props.present + props.incomplete))
const percent = computed(() => {
  if (total.value === 0) return 0
  return Math.round((props.completed / total.value) * 100)
})

const chartOption = computed<ECBasicOption>(() => {
  const bg = chartBgColor.value
  const empty = total.value === 0

  const data = empty
    ? [
        {
          name: 'empty',
          value: 1,
          itemStyle: {
            color: '#374151',
            borderColor: bg,
            borderWidth: 6,
            borderRadius: 6,
          },
        },
      ]
    : [
        {
          name: 'completed',
          value: props.completed,
          itemStyle: {
            color: completedColor.value,
            borderColor: bg,
            borderWidth: 6,
            borderRadius: 6,
          },
        },
        {
          name: 'present',
          value: props.present,
          itemStyle: {
            color: presentColor.value,
            borderColor: bg,
            borderWidth: 6,
            borderRadius: 6,
          },
        },
        {
          name: 'incomplete',
          value: props.incomplete,
          itemStyle: {
            color: incompleteColor.value,
            borderColor: bg,
            borderWidth: 6,
            borderRadius: 6,
          },
        },
      ]

  return {
    animation: true,
    animationDuration: 500,
    tooltip: { show: false },
    series: [
      {
        type: 'pie',
        radius: ['62%', '86%'],
        center: ['50%', '50%'],
        startAngle: 90,
        label: { show: false },
        labelLine: { show: false },
        data,
      },
    ],
    graphic: [
      {
        type: 'text',
        left: 'center',
        top: 'middle',
        style: {
          text: empty ? '0%' : `${percent.value}%`,
          fill: chartTextColor.value,
          fontSize: 34,
          fontWeight: 700,
          fontFamily: chartFontFamily.value,
          textAlign: 'center',
          textVerticalAlign: 'middle',
        },
      },
    ],
  }
})
</script>

<style scoped>
.daily-shutdown-wizard {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 1rem 3rem 3rem;
  overflow-y: auto;
}

.wizard-header {
  margin-bottom: 2.5rem;
  text-align: center;
}

.wizard-title {
  font-size: 2.4rem;
  font-weight: 600;
  color: var(--color-text-accent, #f0f);
  margin: 0 0 0.8rem;
  line-height: 1.3;
}

.wizard-subtitle {
  font-size: 1.6rem;
  color: var(--color-text-primary, #f0f);
  margin: 0;
  line-height: 1.4;
  opacity: 0.85;
}

.wizard-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 1.6rem;
}

.chart-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2rem;
}

.chart-wrapper {
  position: relative;
  width: 100%;
  max-width: 22rem;
  display: flex;
  justify-content: center;
}

.chart-glow {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 70%;
  height: 70%;
  background: radial-gradient(
    circle,
    var(--color-background-accent-light, #f0f) 0%,
    transparent 70%
  );
  filter: blur(30px);
  pointer-events: none;
  opacity: 0.55;
}

.chart-container {
  width: 100%;
  position: relative;
  z-index: 1;
}

.donut-chart {
  width: 100%;
  height: 220px;
}

.chart-legend {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  width: 100%;
  padding: 0 0.5rem;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  padding: 0.75rem 1rem;
  border-radius: 0.75rem;
  background: transparent;
}

.dot {
  width: 0.8rem;
  height: 0.8rem;
  border-radius: 50%;
  flex-shrink: 0;
}

.dot.completed {
  background: var(--color-status-completed, #f0f);
}

.dot.present {
  background: var(--color-status-present, #f0f);
}

.dot.incomplete {
  background: var(--color-text-tertiary, #f0f);
}

.legend-name {
  flex: 1;
  font-size: 1.5rem;
  font-weight: 500;
  color: var(--color-text-tertiary, #f0f);
  line-height: 1.4;
}

.legend-count {
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
  min-width: 2rem;
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.wizard-navigation {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-top: 2rem;
}

.nav-btn {
  padding: 1rem 2rem;
  border-radius: 0.75rem;
  font-size: 1.4rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  line-height: 1.4;
}

.next-btn {
  flex: 1;
  height: 4.8rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-button-primary-bg, #f0f);
  border: 1px solid transparent;
  color: var(--color-button-primary-text, #f0f);
  box-shadow: var(--shadow-sm, #f0f);
}

.next-btn:hover {
  background-color: var(--color-button-primary-hover, #f0f);
  box-shadow: var(--shadow-md, #f0f);
  transform: translateY(-1px);
}

.wizard-hint {
  margin-top: 2rem;
  padding-top: 1.6rem;
  border-top: 1px solid var(--color-border-subtle, #f0f);
}

.hint-text {
  font-size: 1.35rem;
  color: var(--color-text-accent, #f0f);
  margin: 0;
  line-height: 1.6;
  font-style: italic;
  opacity: 0.85;
}
</style>


