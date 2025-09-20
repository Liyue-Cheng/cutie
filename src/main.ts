import { createApp } from 'vue'
import { createPinia } from 'pinia'
import naive from 'naive-ui' // 1. 引入 naive-ui

import App from './App.vue'
import './style.css'

const pinia = createPinia()
const app = createApp(App)

app.use(pinia)
app.use(naive) // 2. 全局安装 naive-ui 插件

app.mount('#app')
