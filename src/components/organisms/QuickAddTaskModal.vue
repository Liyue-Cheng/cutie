<template>
  <Teleport to="body">
    <div v-if="show" class="quick-add-overlay" @click="handleOverlayClick">
      <div class="quick-add-dialog" @click.stop>
        <div class="dialog-header">
          <h3>{{ $t('quickAdd.title') }}</h3>
          <button class="close-button" @click="close">
            <CuteIcon name="X" :size="18" />
          </button>
        </div>
        <div class="dialog-body">
          <input
            ref="inputRef"
            v-model="taskTitle"
            type="text"
            class="task-input"
            :placeholder="$t('task.placeholder.title')"
            @keydown.enter="handleAdd"
            @keydown.esc="close"
          />
        </div>
        <div class="dialog-footer">
          <button class="cancel-button" @click="close">{{ $t('common.action.cancel') }}</button>
          <button class="add-button" :disabled="!taskTitle.trim()" @click="handleAdd">
            {{ buttonText }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import { pipeline } from '@/cpu'
import { logger, LogTags } from '@/infra/logging/logger'
import { deriveViewMetadata } from '@/services/viewAdapter'

const props = defineProps<{
  show: boolean
  viewKey?: string // 🔥 支持 VIEW_CONTEXT_KEY 规范，默认为 misc::staging
}>()

const emit = defineEmits<{
  close: []
}>()

const { t } = useI18n()
const taskTitle = ref('')
const inputRef = ref<HTMLInputElement | null>(null)

// 计算有效的 viewKey
const effectiveViewKey = computed(() => {
  return props.viewKey || 'misc::staging'
})

// 推导 ViewMetadata
const viewMetadata = computed(() => {
  return deriveViewMetadata(effectiveViewKey.value)
})

// 根据 viewKey 生成按钮文本
const buttonText = computed(() => {
  const metadata = viewMetadata.value
  if (metadata && metadata.type === 'date') {
    const dateConfig = metadata.config as import('@/types/drag').DateViewConfig
    return t('quickAdd.button.addTo', { date: dateConfig.date })
  }
  // 其他视图使用通用文本
  return t('quickAdd.button.add')
})

// 当对话框显示时，自动聚焦到输入框
watch(
  () => props.show,
  async (newShow) => {
    if (newShow) {
      taskTitle.value = ''
      await nextTick()
      inputRef.value?.focus()
    }
  }
)

function close() {
  emit('close')
}

function handleOverlayClick() {
  close()
}

async function handleAdd() {
  const title = taskTitle.value.trim()
  if (!title) return

  try {
    const metadata = viewMetadata.value
    if (!metadata) {
      logger.error(
        LogTags.COMPONENT_KANBAN,
        'Failed to derive view metadata',
        new Error('Metadata is undefined')
      )
      return
    }

    const isDateView = metadata.type === 'date'

    if (isDateView) {
      // 日期视图：使用合并端点一次性创建任务并添加日程
      const dateConfig = metadata.config as import('@/types/drag').DateViewConfig
      const date = dateConfig.date // YYYY-MM-DD

      await pipeline.dispatch('task.create_with_schedule', {
        title,
        estimated_duration: 60, // 默认 60 分钟
        scheduled_day: date,
      })

      logger.info(LogTags.COMPONENT_KANBAN, 'Quick add task with schedule', {
        title,
        date,
        viewKey: effectiveViewKey.value,
      })
    } else {
      // 非日期视图：只创建任务，需要根据 viewKey 提取上下文信息
      const taskData: any = {
        title,
        estimated_duration: 60, // 默认 60 分钟
      }

      // 根据 viewKey 提取上下文信息
      const parts = effectiveViewKey.value.split('::')
      const [type, subtype, identifier] = parts

      if (type === 'misc' && subtype === 'staging' && identifier) {
        // misc::staging::${areaId} - 指定 area 的 staging 任务
        taskData.area_id = identifier
        logger.debug(LogTags.COMPONENT_KANBAN, 'Creating task with area context', {
          areaId: identifier,
          viewKey: effectiveViewKey.value,
        })
      } else if (type === 'area' && subtype) {
        // area::${areaId} - 指定 area 的所有任务
        taskData.area_id = subtype
        logger.debug(LogTags.COMPONENT_KANBAN, 'Creating task with area context', {
          areaId: subtype,
          viewKey: effectiveViewKey.value,
        })
      } else if (type === 'project' && subtype) {
        // project::${projectId} - 指定项目的任务
        taskData.project_id = subtype
        logger.debug(LogTags.COMPONENT_KANBAN, 'Creating task with project context', {
          projectId: subtype,
          viewKey: effectiveViewKey.value,
        })
      }

      await pipeline.dispatch('task.create', taskData)

      logger.info(LogTags.COMPONENT_KANBAN, 'Quick add task', {
        title,
        viewKey: effectiveViewKey.value,
        taskData,
      })
    }

    // 清空输入框并关闭对话框
    taskTitle.value = ''
    close()
  } catch (error) {
    logger.error(
      LogTags.COMPONENT_KANBAN,
      'Failed to quick add task',
      error instanceof Error ? error : new Error(String(error))
    )
  }
}
</script>

<style scoped>
.quick-add-overlay {
  position: fixed;
  inset: 0;
  background-color: var(--color-overlay-heavy, #f0f);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  backdrop-filter: blur(2px);
}

.quick-add-dialog {
  background-color: var(--color-background-content, #f0f);
  border: 1px solid var(--color-border-subtle, #f0f);
  border-radius: 1.2rem;
  box-shadow: var(--shadow-lg);
  width: 90%;
  max-width: 50rem;
  padding: 0;
  overflow: hidden;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.6rem 2rem;
  border-bottom: 1px solid var(--color-border-soft, #f0f);
}

.dialog-header h3 {
  margin: 0;
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary, #f0f);
}

.close-button {
  all: unset;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 3.2rem;
  height: 3.2rem;
  border-radius: 0.6rem;
  cursor: pointer;
  color: var(--color-text-secondary, #f0f);
  transition: all 0.15s ease;
}

.close-button:hover {
  background-color: var(--color-background-hover, #f0f);
  color: var(--color-text-primary, #f0f);
}

.dialog-body {
  padding: 2rem;
}

.task-input {
  width: 100%;
  padding: 1.2rem 1.6rem;
  font-size: 1.6rem;
  border: 2px solid var(--color-border-default, #f0f);
  border-radius: 0.8rem;
  background-color: var(--color-background-primary, #f0f);
  color: var(--color-text-primary, #f0f);
  transition: all 0.15s ease;
  box-sizing: border-box;
}

.task-input:focus {
  outline: none;
  border-color: var(--color-border-focus);
  box-shadow: var(--shadow-focus);
}

.task-input::placeholder {
  color: var(--color-text-tertiary, #f0f);
}

.dialog-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 1rem;
  padding: 1.6rem 2rem;
  border-top: 1px solid var(--color-border-soft, #f0f);
}

.cancel-button,
.add-button {
  padding: 0.8rem 1.6rem;
  font-size: 1.4rem;
  font-weight: 600;
  border-radius: 0.6rem;
  border: none;
  cursor: pointer;
  transition: all 0.15s ease;
}

.cancel-button {
  background-color: transparent;
  color: var(--color-text-secondary, #f0f);
}

.cancel-button:hover {
  background-color: var(--color-background-hover, #f0f);
  color: var(--color-text-primary, #f0f);
}

.add-button {
  background-color: var(--color-button-primary-bg);
  color: var(--color-button-primary-text, #f0f);
}

.add-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.add-button:hover:not(:disabled) {
  background-color: var(--color-button-primary-hover);
}
</style>
