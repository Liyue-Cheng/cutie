<script setup lang="ts">
import { ref, nextTick, watch, onMounted, onBeforeUnmount } from 'vue'
import { useAiStore } from '@/stores/ai'
import type { MessageImage } from '@/types/ai'
import CuteIcon from '@/components/parts/CuteIcon.vue'
import AiToolCallCard from './AiToolCallCard.vue'

const emit = defineEmits<{
  close: []
}>()

const aiStore = useAiStore()

// ==================== 状态 ====================
const inputText = ref('')
const images = ref<MessageImage[]>([])
const messagesContainerRef = ref<HTMLElement | null>(null)
const fileInputRef = ref<HTMLInputElement | null>(null)
const dialogRef = ref<HTMLElement | null>(null)
const textareaRef = ref<HTMLTextAreaElement | null>(null)

// ==================== 方法 ====================
async function handleSend() {
  const text = inputText.value.trim()
  if (!text && images.value.length === 0) return
  if (aiStore.isLoading) return

  try {
    await aiStore.sendMessage({
      role: 'user',
      text: text || '看看这张图片',
      images: images.value,
    })

    // 清空输入
    inputText.value = ''
    images.value = []

    // 滚动到底部并重新聚焦输入框
    await nextTick()
    scrollToBottom()
    textareaRef.value?.focus()
  } catch (error) {
    console.error('发送消息失败:', error)
  }
}

function handleKeyDown(event: KeyboardEvent) {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    handleSend()
  }
}

function handleClose() {
  emit('close')
}

function handleClearHistory() {
  if (confirm('确定要清空聊天记录吗？')) {
    aiStore.clearMessages()
  }
}

function scrollToBottom() {
  if (messagesContainerRef.value) {
    messagesContainerRef.value.scrollTop = messagesContainerRef.value.scrollHeight
  }
}

// ==================== 图片处理 ====================
function handleImageUpload() {
  fileInputRef.value?.click()
}

async function handleFileChange(event: Event) {
  const target = event.target as HTMLInputElement
  const files = target.files
  if (!files || files.length === 0) return

  const file = files[0]
  if (!file) return

  if (!file.type.startsWith('image/')) {
    alert('请选择图片文件')
    return
  }

  // 文件大小限制 (5MB)
  if (file.size > 5 * 1024 * 1024) {
    alert('图片文件不能超过 5MB')
    return
  }

  // 转换为 base64
  const reader = new FileReader()
  reader.onload = (e) => {
    const dataUrl = e.target?.result as string
    images.value.push({
      kind: 'base64',
      data: dataUrl,
    })
  }
  reader.readAsDataURL(file)

  // 清空 input
  target.value = ''
}

function removeImage(index: number) {
  images.value.splice(index, 1)
}

// ==================== 监听消息变化，自动滚动 ====================
watch(
  () => aiStore.allMessages.length,
  async () => {
    await nextTick()
    scrollToBottom()
  }
)

// ==================== 点击外部关闭 ====================
function handleClickOutside(event: MouseEvent) {
  if (dialogRef.value && !dialogRef.value.contains(event.target as Node)) {
    handleClose()
  }
}

onMounted(() => {
  // 自动聚焦输入框
  nextTick(() => {
    textareaRef.value?.focus()
  })

  // 延迟添加监听器，避免立即触发
  setTimeout(() => {
    document.addEventListener('click', handleClickOutside)
  }, 100)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <div ref="dialogRef" class="ai-chat-dialog">
    <!-- 标题栏 -->
    <div class="dialog-header">
      <div class="header-left">
        <CuteIcon name="Sparkles" :size="20" />
        <h2>AI 助手</h2>
      </div>
      <div class="header-actions">
        <button class="icon-btn" title="清空记录" @click="handleClearHistory">
          <CuteIcon name="Trash2" :size="18" />
        </button>
        <button class="icon-btn" title="关闭" @click="handleClose">
          <CuteIcon name="X" :size="18" />
        </button>
      </div>
    </div>

    <!-- 消息列表 -->
    <div ref="messagesContainerRef" class="messages-container">
      <div v-if="!aiStore.hasMessages" class="empty-state">
        <CuteIcon name="MessageSquare" :size="64" />
        <p>开始与 AI 对话吧！</p>
      </div>

      <div v-for="(message, index) in aiStore.allMessages" :key="index" class="message-wrapper">
        <div :class="['message-bubble', message.role]">
          <div class="message-role">{{ message.role === 'user' ? '你' : 'AI' }}</div>
          <div v-if="message.text" class="message-text">{{ message.text }}</div>
          <div
            v-if="
              message.role === 'assistant' && message.tool_calls && message.tool_calls.length > 0
            "
            class="tool-card-list"
          >
            <AiToolCallCard v-for="call in message.tool_calls" :key="call.id" :call="call" />
          </div>
          <!-- 显示用户发送的图片 -->
          <div
            v-if="message.role === 'user' && message.images && message.images.length > 0"
            class="message-images"
          >
            <img
              v-for="(img, imgIndex) in message.images"
              :key="imgIndex"
              :src="img.data"
              alt="上传的图片"
              class="message-image"
            />
          </div>
          <!-- 显示 AI 响应的计时和模型信息 -->
          <div v-if="message.role === 'assistant' && message.response_time_ms" class="message-meta">
            <span class="meta-item">
              <CuteIcon name="Clock" :size="14" />
              {{ message.response_time_ms }}ms
            </span>
            <span v-if="message.model" class="meta-item">
              <CuteIcon name="Cpu" :size="14" />
              {{ message.model }}
            </span>
            <span v-if="message.usage" class="meta-item">
              <CuteIcon name="Activity" :size="14" />
              {{ message.usage.total_tokens }} tokens
            </span>
          </div>
        </div>
      </div>

      <!-- 加载中指示器 -->
      <div v-if="aiStore.isLoading" class="message-wrapper">
        <div class="message-bubble assistant loading">
          <div class="message-role">AI</div>
          <div class="loading-dots">
            <span></span>
            <span></span>
            <span></span>
          </div>
        </div>
      </div>

      <!-- 错误提示 -->
      <div v-if="aiStore.error" class="error-message">
        <CuteIcon name="Info" :size="20" />
        <span>{{ aiStore.error }}</span>
        <button @click="aiStore.resetError">
          <CuteIcon name="X" :size="16" />
        </button>
      </div>
    </div>

    <!-- 图片预览区 -->
    <div v-if="images.length > 0" class="images-preview">
      <div v-for="(img, index) in images" :key="index" class="preview-item">
        <img :src="img.data" alt="待发送图片" />
        <button class="remove-btn" @click="removeImage(index)">
          <CuteIcon name="X" :size="16" />
        </button>
      </div>
    </div>

    <!-- 输入区 -->
    <div class="input-area">
      <button
        class="icon-btn"
        title="上传图片"
        :disabled="aiStore.isLoading"
        @click="handleImageUpload"
      >
        <CuteIcon name="Image" :size="20" />
      </button>
      <input
        ref="fileInputRef"
        type="file"
        accept="image/*"
        style="display: none"
        @change="handleFileChange"
      />
      <textarea
        ref="textareaRef"
        v-model="inputText"
        placeholder="输入消息... (Enter 发送, Shift+Enter 换行)"
        rows="3"
        :disabled="aiStore.isLoading"
        @keydown="handleKeyDown"
      ></textarea>
      <button
        class="send-btn"
        :disabled="aiStore.isLoading || (!inputText.trim() && images.length === 0)"
        @click="handleSend"
      >
        <CuteIcon name="Send" :size="20" />
      </button>
    </div>
  </div>
</template>

<style scoped>
.ai-chat-dialog {
  position: fixed;
  right: 2rem;
  bottom: 8rem;
  width: 42rem;
  height: 60rem;
  background-color: var(--color-background-content);
  border: 1px solid var(--color-border-default);
  border-radius: 1.2rem;
  box-shadow: var(--shadow-xl);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  z-index: 1000;
  animation: slide-in-up 0.2s ease-out;
}

@keyframes slide-in-up {
  from {
    opacity: 0;
    transform: translateY(1rem);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* ==================== 标题栏 ==================== */
.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.2rem 1.6rem;
  border-bottom: 1px solid var(--color-border-default);
  background-color: var(--color-background-secondary);
  color: var(--color-text-primary);
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.header-left h2 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.header-actions {
  display: flex;
  gap: 0.4rem;
}

.icon-btn {
  width: 2.8rem;
  height: 2.8rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: transparent;
  border: none;
  border-radius: 0.6rem;
  cursor: pointer;
  color: var(--color-text-secondary);
  transition: all 0.2s ease;
}

.icon-btn:disabled {
  opacity: var(--opacity-disabled);
  cursor: not-allowed;
}

.icon-btn:hover:not(:disabled) {
  background-color: var(--color-background-hover);
  color: var(--color-text-primary);
}

/* stylelint-disable-next-line no-descending-specificity */
.input-area .icon-btn {
  width: 3.2rem;
  height: 3.2rem;
  flex-shrink: 0;
}

/* ==================== 消息列表 ==================== */
.messages-container {
  flex: 1;
  overflow-y: auto;
  padding: 1.6rem;
  display: flex;
  flex-direction: column;
  gap: 1.2rem;
  background-color: var(--color-background-primary);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--color-empty-text);
  gap: 1rem;
}

.empty-state p {
  font-size: 1.3rem;
  margin: 0;
}

.message-wrapper {
  display: flex;
  flex-direction: column;
}

.message-bubble {
  max-width: 85%;
  padding: 1rem 1.2rem;
  border-radius: 0.8rem;
  position: relative;
  border: 1px solid var(--color-border-light);
}

.message-bubble.user {
  align-self: flex-end;
  background-color: var(--color-background-accent);
  color: var(--color-text-on-accent);
  border-color: transparent;
}

.message-bubble.assistant {
  align-self: flex-start;
  background-color: var(--color-background-content);
  color: var(--color-text-primary);
}

.message-role {
  font-size: 1.1rem;
  font-weight: 600;
  margin-bottom: 0.4rem;
  opacity: 0.7;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.message-text {
  font-size: 1.3rem;
  line-height: 1.5;
  white-space: pre-wrap;
  overflow-wrap: break-word;
}

.tool-card-list {
  margin-top: 1.2rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.message-images {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  margin-top: 1rem;
}

.message-image {
  max-width: 100%;
  max-height: 20rem;
  border-radius: 0.8rem;
  object-fit: contain;
}

/* 消息元信息 */
.message-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.8rem;
  margin-top: 0.6rem;
  padding-top: 0.6rem;
  border-top: 1px solid var(--color-border-light);
  font-size: 1.1rem;
  opacity: 0.6;
}

.message-bubble.user .message-meta {
  border-top-color: rgb(255 255 255 / 20%);
}

.message-bubble.assistant .message-meta {
  border-top-color: var(--color-border-light);
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 0.3rem;
}

/* 加载动画 */
.message-bubble.loading {
  background-color: var(--color-background-content);
  border-color: var(--color-border-light);
}

.loading-dots {
  display: flex;
  gap: 0.4rem;
  padding: 0.4rem 0;
}

.loading-dots span {
  width: 0.6rem;
  height: 0.6rem;
  background-color: var(--color-text-accent);
  border-radius: 50%;
  animation: loading-bounce 1.4s infinite ease-in-out both;
}

.loading-dots span:nth-child(1) {
  animation-delay: -0.32s;
}

.loading-dots span:nth-child(2) {
  animation-delay: -0.16s;
}

@keyframes loading-bounce {
  0%,
  80%,
  100% {
    transform: scale(0.8);
    opacity: 0.3;
  }

  40% {
    transform: scale(1);
    opacity: 1;
  }
}

/* 错误提示 */
.error-message {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  padding: 0.8rem 1rem;
  background-color: var(--color-danger-light);
  color: var(--color-danger-text);
  border: 1px solid var(--color-border-error);
  border-radius: 0.6rem;
  font-size: 1.2rem;
}

.error-message button {
  margin-left: auto;
  background: none;
  border: none;
  cursor: pointer;
  color: var(--color-danger-text);
  padding: 0.2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 0.4rem;
  transition: background-color 0.2s;
}

.error-message button:hover {
  background-color: rgb(0 0 0 / 10%);
}

/* ==================== 图片预览 ==================== */
.images-preview {
  display: flex;
  gap: 0.8rem;
  padding: 1rem 1.6rem;
  border-top: 1px solid var(--color-border-default);
  overflow-x: auto;
  background-color: var(--color-background-secondary);
  flex-shrink: 0;
}

.preview-item {
  position: relative;
  flex-shrink: 0;
}

.preview-item img {
  width: 6rem;
  height: 6rem;
  object-fit: cover;
  border-radius: 0.6rem;
  border: 1px solid var(--color-border-default);
}

.preview-item .remove-btn {
  position: absolute;
  top: -0.4rem;
  right: -0.4rem;
  width: 1.8rem;
  height: 1.8rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-danger);
  color: white;
  border: none;
  border-radius: 50%;
  cursor: pointer;
  box-shadow: var(--shadow-sm);
  transition: transform 0.2s;
}

.preview-item .remove-btn:hover {
  transform: scale(1.1);
}

/* ==================== 输入区 ==================== */
.input-area {
  display: flex;
  align-items: flex-end;
  gap: 0.8rem;
  padding: 1.2rem 1.6rem;
  border-top: 1px solid var(--color-border-default);
  background-color: var(--color-background-secondary);
  flex-shrink: 0;
}

.input-area textarea {
  flex: 1;
  padding: 0.8rem;
  font-size: 1.3rem;
  line-height: 1.5;
  border: 1px solid var(--color-border-input);
  border-radius: 0.6rem;
  resize: none;
  font-family: inherit;
  background-color: var(--color-background-input);
  color: var(--color-text-primary);
  transition: border-color 0.2s;
}

.input-area textarea::placeholder {
  color: var(--color-text-placeholder);
}

.input-area textarea:hover {
  border-color: var(--color-border-input-hover);
}

.input-area textarea:focus {
  outline: none;
  border-color: var(--color-border-input-focus);
  box-shadow: var(--shadow-focus);
}

.input-area textarea:disabled {
  opacity: var(--opacity-disabled);
  cursor: not-allowed;
  background-color: var(--color-background-disabled);
}

.send-btn {
  width: 3.2rem;
  height: 3.2rem;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-button-primary-bg);
  color: var(--color-button-primary-text);
  border: none;
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.send-btn:disabled {
  opacity: var(--opacity-disabled);
  cursor: not-allowed;
}

.send-btn:hover:not(:disabled) {
  background-color: var(--color-button-primary-hover);
  transform: translateY(-1px);
  box-shadow: var(--shadow-sm);
}

.send-btn:active:not(:disabled) {
  transform: translateY(0);
  box-shadow: none;
}
</style>
