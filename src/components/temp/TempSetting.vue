<template>
  <div class="modal-overlay" @click.self="emit('close')">
    <CuteCard class="settings-card">
      <div class="settings-wrapper">
        <CutePane class="settings-sidebar">
          <ul class="settings-nav">
            <li :class="{ active: activeTab === 'language' }" @click="activeTab = 'language'">
              Language
            </li>
            <li :class="{ active: activeTab === 'scale' }" @click="activeTab = 'scale'">Scale</li>
            <li :class="{ active: activeTab === 'timezone' }" @click="activeTab = 'timezone'">
              Timezone
            </li>
            <li :class="{ active: activeTab === 'account' }" @click="activeTab = 'account'">
              Account
            </li>
          </ul>
        </CutePane>
        <div class="settings-content">
          <div v-if="activeTab === 'language'">
            <h4>Language Settings</h4>
            <div class="button-group">
              <CuteButton @click="setLocale('en')">English</CuteButton>
              <CuteButton @click="setLocale('zh-CN')">Chinese</CuteButton>
            </div>
          </div>
          <div v-if="activeTab === 'scale'">
            <h4>Display Scaling</h4>
            <div class="preset-buttons">
              <CuteButton @click="setZoom(80)">80%</CuteButton>
              <CuteButton @click="setZoom(90)">90%</CuteButton>
              <CuteButton @click="setZoom(100)">100%</CuteButton>
              <CuteButton @click="setZoom(110)">110%</CuteButton>
              <CuteButton @click="setZoom(120)">120%</CuteButton>
            </div>
          </div>
          <div v-if="activeTab === 'timezone'">
            <h4>Timezone Settings</h4>
            <p>Timezone settings will be here.</p>
          </div>
          <div v-if="activeTab === 'account'">
            <h4>Account Settings</h4>
            <p>Account settings will be here.</p>
          </div>
        </div>
      </div>
    </CuteCard>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import CuteCard from '../ui/CuteCard.vue'
import CutePane from '../ui/CutePane.vue'
import CuteButton from '../ui/CuteButton.vue'

const emit = defineEmits(['close'])

const activeTab = ref('language')
const { locale } = useI18n()

function setLocale(newLocale: string) {
  locale.value = newLocale
}

function setZoom(percentage: number) {
  const newSize = (percentage / 100) * 62.5
  document.documentElement.style.fontSize = `${newSize}%`
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background-color: rgb(0 0 0 / 20%);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 100;
}

.settings-card {
  width: 60rem;
  height: 40rem;
  padding: 0;
  display: flex;
}

.settings-wrapper {
  flex-grow: 1;
  display: flex;
  background-color: var(--c-gray-background);
}

.settings-sidebar {
  width: 18rem;
  flex-shrink: 0;
  background-color: var(--color-background-secondary);
  border: none;
  padding: 1.5rem 0;
}

.settings-nav {
  list-style: none;
  padding: 0;
  margin: 0;
  font-size: 1.5rem;
}

.settings-nav li {
  padding: 0.8rem 2rem;
  cursor: pointer;
  color: var(--color-text-secondary);
  transition: all 0.2s;
}

.settings-nav li.active {
  background-color: var(--c-blue-100);
  color: var(--color-text-accent);
  font-weight: 500;
}

.settings-content {
  flex-grow: 1;
  padding: 2rem;
  background-color: var(--color-background-content);
}

.settings-content h4 {
  font-size: 1.8rem;
  color: var(--color-text-primary);
  margin-top: 0;
  margin-bottom: 2rem;
}

.button-group {
  display: flex;
  gap: 1rem;
}

.preset-buttons {
  display: flex;
  justify-content: center;
  gap: 1rem;
}
</style>
