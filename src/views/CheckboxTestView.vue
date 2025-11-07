<script setup lang="ts">
import { ref } from 'vue'
import CuteDualModeCheckbox from '@/components/parts/CuteDualModeCheckbox.vue'

type CheckboxState = null | 'completed' | 'present'

const state1 = ref<CheckboxState>(null)
const state2 = ref<CheckboxState>(null)
const state3 = ref<CheckboxState>(null)

const getStateText = (state: CheckboxState) => {
  if (state === 'completed') return '✅ 已完成'
  if (state === 'present') return '🔵 已在场'
  return '⬜ 未选中'
}
</script>

<template>
  <div class="checkbox-test-view">
    <div class="test-container">
      <h1>CuteDualModeCheckbox 测试页面</h1>

      <div class="section">
        <h2>使用说明</h2>
        <ul class="instructions">
          <li><strong>单击</strong>：未选中 → 完成任务；已选中（任何状态）→ 未选中</li>
          <li><strong>长按（0.5秒）</strong>：标记在场</li>
          <li>圆角方形外观，淡雅设计，自动适配主题</li>
          <li>完成状态：玫瑰色边框 + 玫瑰色对钩（Rose Pine Dawn: #d7827e）</li>
          <li>在场状态：青蓝色边框 + 青蓝色圆点（Rose Pine Dawn: #286983）</li>
        </ul>
      </div>

      <div class="section">
        <h2>Small 尺寸（默认）</h2>
        <div class="test-row">
          <div class="test-item">
            <CuteDualModeCheckbox v-model:state="state1" />
            <span class="state-label">{{ getStateText(state1) }}</span>
          </div>
        </div>
      </div>

      <div class="section">
        <h2>Large 尺寸</h2>
        <div class="test-row">
          <div class="test-item">
            <CuteDualModeCheckbox v-model:state="state2" size="large" />
            <span class="state-label">{{ getStateText(state2) }}</span>
          </div>
        </div>
      </div>

      <div class="section">
        <h2>自定义尺寸（3rem）</h2>
        <div class="test-row">
          <div class="test-item">
            <CuteDualModeCheckbox v-model:state="state3" size="3rem" />
            <span class="state-label">{{ getStateText(state3) }}</span>
          </div>
        </div>
      </div>

      <div class="section">
        <h2>所有状态预览</h2>
        <div class="preview-grid">
          <div class="preview-item">
            <CuteDualModeCheckbox :state="null" />
            <span>未选中</span>
          </div>
          <div class="preview-item">
            <CuteDualModeCheckbox :state="'completed'" />
            <span>已完成</span>
          </div>
          <div class="preview-item">
            <CuteDualModeCheckbox :state="'present'" />
            <span>已在场</span>
          </div>
        </div>
      </div>

      <div class="section">
        <h2>手动控制按钮</h2>
        <div class="button-group">
          <button @click="state1 = null" class="btn">重置为未选中</button>
          <button @click="state1 = 'completed'" class="btn btn-success">设为完成</button>
          <button @click="state1 = 'present'" class="btn btn-primary">设为在场</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.checkbox-test-view {
  width: 100%;
  min-height: 100vh;
  padding: 2rem;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  display: flex;
  justify-content: center;
  align-items: flex-start;
}

.test-container {
  max-width: 800px;
  width: 100%;
  background: white;
  border-radius: 1rem;
  padding: 2rem;
  box-shadow: 0 8px 32px rgb(0 0 0 / 10%);
}

h1 {
  color: #333;
  margin-bottom: 2rem;
  font-size: 2rem;
  text-align: center;
}

h2 {
  color: #555;
  margin-bottom: 1rem;
  font-size: 1.5rem;
  border-bottom: 2px solid #667eea;
  padding-bottom: 0.5rem;
}

.section {
  margin-bottom: 3rem;
}

.instructions {
  background: #f8f9fa;
  padding: 1rem 1.5rem;
  border-radius: 0.5rem;
  border-left: 4px solid #667eea;
}

.instructions li {
  margin-bottom: 0.5rem;
  line-height: 1.6;
}

.test-row {
  display: flex;
  gap: 2rem;
  flex-wrap: wrap;
  align-items: center;
}

.test-item {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem;
  background: #f8f9fa;
  border-radius: 0.5rem;
  border: 1px solid #e0e0e0;
}

.state-label {
  font-size: 1.1rem;
  font-weight: 500;
  color: #333;
  min-width: 100px;
}

.preview-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 1.5rem;
}

.preview-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  padding: 1.5rem;
  background: #f8f9fa;
  border-radius: 0.5rem;
  border: 1px solid #e0e0e0;
  transition: transform 0.2s;
}

.preview-item:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgb(0 0 0 / 10%);
}

.preview-item span {
  font-size: 0.9rem;
  color: #666;
}

.button-group {
  display: flex;
  gap: 1rem;
  flex-wrap: wrap;
}

.btn {
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 0.5rem;
  font-size: 1rem;
  cursor: pointer;
  background: #e0e0e0;
  color: #333;
  transition: all 0.2s;
}

.btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgb(0 0 0 / 15%);
}

.btn-success {
  background: #52c41a;
  color: white;
}

.btn-primary {
  background: #1890ff;
  color: white;
}
</style>
