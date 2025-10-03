import { createApp } from 'vue'
import { createPinia } from 'pinia'

import App from './App.vue'
import router from './router' // 导入路由
import i18n from './i18n'
import { initializeApiConfig } from '@/composables/useApiConfig'
import './style.css'

const pinia = createPinia()
const app = createApp(App)

app.use(pinia)
app.use(i18n)
app.use(router) // 确保已经 use 了 router

// 初始化API配置
initializeApiConfig()
  .then(async () => {
    console.log('🚀 API configuration initialized')

    // ✅ 在应用启动时加载所有 areas（解决 N+1 查询问题）
    const { useAreaStore } = await import('@/stores/area')
    const areaStore = useAreaStore()
    await areaStore.fetchAreas()
    console.log('✅ All areas loaded')
  })
  .catch((error) => {
    console.error('❌ Failed to initialize API configuration:', error)
  })

app.mount('#app')
