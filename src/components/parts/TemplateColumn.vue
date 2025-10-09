<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useTemplateStore } from '@/stores/template'
import { useTaskStore } from '@/stores/task'
import type { Template } from '@/types/dtos'
import { logger, LogTags } from '@/services/logger'

// ==================== Stores ====================
const templateStore = useTemplateStore()
const taskStore = useTaskStore()

// ==================== State ====================
const isCreating = ref(false)
const newTemplateName = ref('')
const newTemplateTitleTemplate = ref('')
const showCreateForm = ref(false)
const isLoading = ref(false)
const creatingFromTemplate = ref<string | null>(null)

// ==================== 初始化 ====================
onMounted(async () => {
  isLoading.value = true
  try {
    await templateStore.fetchAllTemplates()
    logger.info(LogTags.GENERAL, 'Loaded templates', {
      count: templateStore.allTemplates.length,
    })
  } catch (error) {
    logger.error(
      LogTags.GENERAL,
      'Failed to load templates',
      error instanceof Error ? error : new Error(String(error))
    )
  } finally {
    isLoading.value = false
  }
})

// ==================== 事件处理 ====================
async function handleCreateTemplate() {
  if (!newTemplateName.value.trim() || !newTemplateTitleTemplate.value.trim()) {
    return
  }

  isCreating.value = true
  try {
    await templateStore.createTemplate({
      name: newTemplateName.value.trim(),
      title_template: newTemplateTitleTemplate.value.trim(),
    })

    logger.info(LogTags.GENERAL, 'Template created successfully')

    // 重置表单
    newTemplateName.value = ''
    newTemplateTitleTemplate.value = ''
    showCreateForm.value = false
  } catch (error) {
    logger.error(
      LogTags.GENERAL,
      'Failed to create template',
      error instanceof Error ? error : new Error(String(error))
    )
    alert('创建模板失败')
  } finally {
    isCreating.value = false
  }
}

async function handleDeleteTemplate(id: string) {
  if (!confirm('确定要删除此模板吗？')) {
    return
  }

  try {
    await templateStore.deleteTemplate(id)
    logger.info(LogTags.GENERAL, 'Template deleted', { id })
  } catch (error) {
    logger.error(
      LogTags.GENERAL,
      'Failed to delete template',
      error instanceof Error ? error : new Error(String(error))
    )
    alert('删除模板失败')
  }
}

async function handleUseTemplate(template: Template) {
  creatingFromTemplate.value = template.id

  try {
    // 从模板创建任务
    const taskCard = await templateStore.createTaskFromTemplate(template.id)

    // 更新 task store
    taskStore.addOrUpdateTask(taskCard)

    logger.info(LogTags.GENERAL, 'Task created from template', {
      templateId: template.id,
      taskId: taskCard.id,
    })
  } catch (error) {
    logger.error(
      LogTags.GENERAL,
      'Failed to create task from template',
      error instanceof Error ? error : new Error(String(error))
    )
    alert('从模板创建任务失败')
  } finally {
    creatingFromTemplate.value = null
  }
}
</script>

<template>
  <div class="template-column">
    <div class="column-header">
      <h3>模板</h3>
      <button class="create-button" @click="showCreateForm = !showCreateForm">
        {{ showCreateForm ? '取消' : '+' }}
      </button>
    </div>

    <!-- 创建表单 -->
    <div v-if="showCreateForm" class="create-form">
      <input
        v-model="newTemplateName"
        type="text"
        placeholder="模板名称"
        class="form-input"
        @keyup.enter="handleCreateTemplate"
      />
      <input
        v-model="newTemplateTitleTemplate"
        type="text"
        placeholder="标题模板 (可使用 {{变量}})"
        class="form-input"
        @keyup.enter="handleCreateTemplate"
      />
      <button
        class="submit-button"
        :disabled="isCreating || !newTemplateName.trim() || !newTemplateTitleTemplate.trim()"
        @click="handleCreateTemplate"
      >
        {{ isCreating ? '创建中...' : '创建' }}
      </button>
    </div>

    <!-- 加载状态 -->
    <div v-if="isLoading" class="loading-state">加载中...</div>

    <!-- 模板列表 -->
    <div v-else class="templates-list">
      <div
        v-for="template in templateStore.generalTemplates"
        :key="template.id"
        class="template-card"
      >
        <div class="template-header">
          <h4>{{ template.name }}</h4>
          <button class="delete-button" @click="handleDeleteTemplate(template.id)">×</button>
        </div>
        <p class="template-content">{{ template.title_template }}</p>
        <button
          class="use-button"
          :disabled="creatingFromTemplate === template.id"
          @click="handleUseTemplate(template)"
        >
          {{ creatingFromTemplate === template.id ? '创建中...' : '使用模板' }}
        </button>
      </div>

      <div v-if="templateStore.generalTemplates.length === 0" class="empty-state">
        暂无模板，点击 + 创建
      </div>
    </div>
  </div>
</template>

<style scoped>
.template-column {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 1.5rem;
  background-color: var(--color-background-content);
  overflow-y: auto;
}

.column-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1.5rem;
}

.column-header h3 {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.create-button {
  width: 3.2rem;
  height: 3.2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 2rem;
  font-weight: 300;
  color: var(--color-primary);
  background-color: var(--color-primary-bg);
  border: 1px solid var(--color-primary);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.create-button:hover {
  background-color: var(--color-primary);
  color: white;
}

.create-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1.5rem;
  background-color: var(--color-background-hover);
  border-radius: 0.8rem;
  margin-bottom: 1.5rem;
}

.form-input {
  padding: 0.8rem;
  font-size: 1.4rem;
  border: 1px solid var(--color-border-default);
  border-radius: 0.4rem;
  background-color: var(--color-background-content);
  color: var(--color-text-primary);
}

.form-input:focus {
  outline: none;
  border-color: var(--color-primary);
}

.submit-button {
  padding: 0.8rem 1.6rem;
  font-size: 1.4rem;
  font-weight: 500;
  color: white;
  background-color: var(--color-primary);
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.submit-button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
  opacity: 0.6;
}

.submit-button:hover:not(:disabled) {
  opacity: 0.9;
}

.loading-state {
  padding: 2rem;
  text-align: center;
  color: var(--color-text-secondary);
}

.templates-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.template-card {
  padding: 1.2rem;
  background-color: var(--color-background-hover);
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
  transition: all 0.2s ease;
}

.template-card:hover {
  border-color: var(--color-primary);
  box-shadow: 0 2px 8px rgb(0 0 0 / 10%);
}

.template-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.8rem;
}

.template-header h4 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.delete-button {
  width: 2.4rem;
  height: 2.4rem;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 2rem;
  font-weight: 300;
  color: var(--color-text-tertiary);
  background: none;
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.delete-button:hover {
  color: #ff4d4f;
  background-color: rgb(255 77 79 / 10%);
}

.template-content {
  margin: 0 0 1rem;
  font-size: 1.3rem;
  color: var(--color-text-secondary);
  line-height: 1.5;
}

.use-button {
  width: 100%;
  padding: 0.6rem 1.2rem;
  font-size: 1.3rem;
  font-weight: 500;
  color: white;
  background-color: var(--color-primary);
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.use-button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.use-button:hover:not(:disabled) {
  opacity: 0.9;
}

.empty-state {
  padding: 3rem 2rem;
  text-align: center;
  color: var(--color-text-tertiary);
  font-size: 1.4rem;
}
</style>
