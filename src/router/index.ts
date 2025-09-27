import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import MainLayout from '../views/MainLayout.vue'

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
      {
        path: 'staging',
        name: 'staging',
        component: () => import('../views/StagingView.vue'),
      },
      {
        path: 'calendar',
        name: 'calendar',
        component: () => import('../views/CalendarView.vue'),
      },
      {
        path: 'daily-shutdown',
        name: 'daily-shutdown',
        component: () => import('../views/DailyShutdownView.vue'),
      },
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
