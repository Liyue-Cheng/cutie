import { createI18n } from 'vue-i18n'
import en from '../locales/en'
import zhCN from '../locales/zh-CN'

const i18n = createI18n({
  legacy: false, // Use Composition API
  locale: 'zh-CN', // Set default locale
  fallbackLocale: 'en', // Fallback locale
  messages: {
    en,
    'zh-CN': zhCN,
  },
})

export default i18n
