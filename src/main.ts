import { createApp } from 'vue'
import { createPinia } from 'pinia'
import naive from 'naive-ui' // 1. 引入 naive-ui

import App from './App.vue'
import router from './router' // 导入路由
import i18n from './i18n'
import './style.css'

const pinia = createPinia()
const app = createApp(App)

app.use(pinia)
app.use(naive) // 2. 全局安装 naive-ui 插件
app.use(i18n)
app.use(router) // 确保已经 use 了 router

app.mount('#app')
