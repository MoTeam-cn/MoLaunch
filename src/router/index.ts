/**
 * 路由配置
 */

import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: () => import('@/views/Home.vue'),
    },
    {
      path: '/login',
      name: 'login',
      component: () => import('@/views/Login.vue'),
    },
    {
      path: '/versions',
      name: 'versions',
      component: () => import('@/views/Versions.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/select',
      name: 'select',
      component: () => import('@/views/VersionSelect.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/version-settings',
      name: 'version-settings',
      component: () => import('@/views/VersionSettings.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/Settings.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/downloads',
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
