<script setup lang="ts">
import { computed } from 'vue'
import { RRule } from 'rrule'
import type { TaskRecurrence } from '@/types/dtos'
import { useTemplateStore } from '@/stores/template'

const props = defineProps<{
  recurrence: TaskRecurrence
}>()

const emit = defineEmits<{
  'toggle-active': [id: string, currentStatus: boolean]
  edit: [id: string]
  delete: [id: string]
}>()

const templateStore = useTemplateStore()

// 获取关联的模板
const template = computed(() => {
  return templateStore.getTemplateById(props.recurrence.template_id)
})

// 将 RRULE 字符串转换为人类可读文本
const ruleDescription = computed(() => {
  try {
    const rule = RRule.fromString(props.recurrence.rule)
    return rule.toText()
  } catch (e) {
    return props.recurrence.rule
  }
})

function handleToggleActive() {
  emit('toggle-active', props.recurrence.id, props.recurrence.is_active)
}

function handleEdit() {
  emit('edit', props.recurrence.id)
}

function handleDelete() {
  if (confirm('确定删除这个循环规则吗？已生成的任务不会被删除。')) {
    emit('delete', props.recurrence.id)
  }
}
</script>

<template>
  <div class="recurrence-card" :class="{ inactive: !recurrence.is_active }">
    <div class="card-header">
      <h4 class="template-title">{{ template?.title || '未知模板' }}</h4>
      <div class="status-badge" :class="{ active: recurrence.is_active }">
        {{ recurrence.is_active ? '激活中' : '已暂停' }}
      </div>
    </div>

    <div class="card-body">
      <div class="rule-info">
        <span class="rule-icon">🔄</span>
        <span class="rule-text">{{ ruleDescription }}</span>
      </div>

      <div v-if="recurrence.start_date || recurrence.end_date" class="date-range">
        <span v-if="recurrence.start_date" class="date-item">
          开始: {{ recurrence.start_date }}
        </span>
        <span v-if="recurrence.end_date" class="date-item"> 结束: {{ recurrence.end_date }} </span>
      </div>
    </div>

    <div class="card-actions">
      <button
        @click="handleToggleActive"
        class="btn-action"
        :title="recurrence.is_active ? '暂停' : '激活'"
      >
        {{ recurrence.is_active ? '⏸️ 暂停' : '▶️ 激活' }}
      </button>
      <button @click="handleEdit" class="btn-action" title="编辑循环规则">✏️ 编辑</button>
      <button @click="handleDelete" class="btn-action btn-danger" title="删除">🗑️ 删除</button>
    </div>
  </div>
</template>

<style scoped>
.recurrence-card {
  background: white;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 12px;
  transition: all 0.2s;
}

.recurrence-card:hover {
  box-shadow: 0 2px 8px rgb(0 0 0 / 10%);
}

.recurrence-card.inactive {
  opacity: 0.6;
  background: #f9f9f9;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.template-title {
  margin: 0;
  font-size: 1.1em;
  font-weight: 600;
  color: #333;
}

.status-badge {
  padding: 4px 12px;
  border-radius: 12px;
  font-size: 0.85em;
  font-weight: 500;
  background: #e0e0e0;
  color: #666;
}

.status-badge.active {
  background: #e8f5e9;
  color: #2e7d32;
}

.card-body {
  margin-bottom: 12px;
}

.rule-info {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.rule-icon {
  font-size: 1.2em;
}

.rule-text {
  color: #555;
  font-size: 0.95em;
}

.date-range {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 0.85em;
  color: #777;
  margin-left: 32px;
}

.date-item {
  display: block;
}

.card-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.btn-action {
  padding: 6px 16px;
  border: 1px solid #ddd;
  border-radius: 6px;
  background: white;
  color: #555;
  font-size: 0.9em;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-action:hover {
  background: #f5f5f5;
  border-color: #999;
}

.btn-danger:hover {
  background: #ffebee;
  border-color: #ef5350;
  color: #c62828;
}
</style>
