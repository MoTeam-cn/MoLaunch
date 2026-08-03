/**
 * 路由配置
 *
 * 业务页面均挂载在 /apps 下并要求登录（meta.requiresAuth），/login 为免认证入口，/ 重定向到 /apps。
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
    {
      path: '/apps/tools',
      name: 'tools',
      component: () => import('@/views/Tools.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/apps/online',
      name: 'online',
      component: () => import('@/views/Online.vue'),
      meta: { requiresAuth: true },
    },
  ],
})

router.beforeEach((to, _from, next) => {
  const authStore = useAuthStore()
  // 会话恢复期间不拦截：restoreSession 是异步的，在 App.vue onMounted 中调用。
  // 若不跳过守卫，应用启动时 currentUser 还是 null，会把已登录用户错误地重定向到 /login。
  // App.vue 有 isRestoring 加载遮罩覆盖整个恢复期，用户看不到路由变化。
  if (authStore.isRestoring) {
    next()
    return
  }
  if (to.meta.requiresAuth && !authStore.isLoggedIn) {
    next('/login')
  } else if (to.path === '/login' && authStore.isLoggedIn && to.query.add !== '1') {
    // 已登录用户访问登录页，重定向到首页
    // 例外：query 带 add=1 时表示用户主动点击「添加账号」，放行进入登录页
    next('/apps')
  } else {
    next()
  }
})

export default router
