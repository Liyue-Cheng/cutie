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
        path: 'kanban-legacy',
        name: 'kanban-legacy',
        component: () => import('../views/KanbanLegacyView.vue'),
      },
      {
        path: 'daily-overview',
        name: 'daily-overview',
        component: () => import('../views/DailyOverviewView.vue'),
      },
      {
        path: 'staging',
        name: 'staging',
        component: () => import('../views/StagingView.vue'),
      },
      {
        path: 'calendar-legacy',
        name: 'calendar-legacy',
        component: () => import('../views/CalendarView.vue'),
      },
      {
        path: 'daily-planning',
        name: 'daily-planning',
        component: () => import('../views/DailyPlanningView.vue'),
      },
      {
        path: 'daily-shutdown',
        name: 'daily-shutdown',
        component: () => import('../views/DailyShutdownView.vue'),
      },
      {
        path: 'trash',
        name: 'trash',
        component: () => import('../views/TrashView.vue'),
      },
      {
        path: 'upcoming',
        name: 'upcoming',
        component: () => import('../views/UpcomingView.vue'),
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
