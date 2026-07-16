/**
 * 路由配置
 *
 * 路径结构：
 * - /login          登录页（无需认证）
 * - /app            首页
 * - /app/versions   版本列表（含社区资源搜索侧栏）
 * - /app/versions/select   选择下载版本
 * - /app/versions/setup    版本设置
 * - /app/settings   全局设置
 * - /apps/downloads  下载管理
 */

import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/apps',
    },
    {
      path: '/login',
      name: 'login',
      component: () => import('@/views/Login.vue'),
    },
    {
      path: '/apps',
      name: 'home',
      component: () => import('@/views/Home.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/apps/versions',
      name: 'versions',
      component: () => import('@/views/Versions.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/apps/versions/select',
      name: 'select',
      component: () => import('@/views/VersionSelect.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/apps/versions/setup',
      name: 'version-settings',
      component: () => import('@/views/VersionSettings.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/apps/settings',
      name: 'settings',
      component: () => import('@/views/Settings.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/apps/downloads',
      name: 'downloads',
      component: () => import('@/views/Downloads.vue'),
      meta: { requiresAuth: true },
    },
  ],
})

router.beforeEach((to, from, next) => {
  const authStore = useAuthStore()
  if (to.meta.requiresAuth && !authStore.isLoggedIn) {
    next('/login')
  } else {
    next()
  }
})

export default router
