import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import MainLayout from '../components/layout/MainLayout.vue'

const routes: Array<RouteRecordRaw> = [
  // 路由组 1: 所有使用 MainLayout 标准布局的页面
  {
    path: '/',
    component: MainLayout,
    children: [
      {
        path: '', // 默认子路由，访问'/'时显示
        name: 'home',
        component: () => import('../views/HomeView.vue'),
      },
      // 可以在这里为未来页面预留位置
      // {
      //   path: 'settings',
      //   name: 'settings',
      //   component: () => import('../views/SettingsView.vue'),
      // }
    ],
  },

  // 路由组 2: 全屏的、不使用 MainLayout 的页面
  {
    path: '/ai-chat',
    name: 'ai-chat',
    component: () => import('../views/AIChatView.vue'),
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
