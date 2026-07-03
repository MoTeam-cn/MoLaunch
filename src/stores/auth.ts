/**
 * 认证状态管理
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { AuthResult, LoginStatus } from '@/types/auth'
import * as tauri from '@/utils/tauri'

export const useAuthStore = defineStore('auth', () => {
  // 状态
  const currentUser = ref<AuthResult | null>(null)
  const loginStatus = ref<LoginStatus>('idle')
  const error = ref<string | null>(null)

  // 计算属性
  const isLoggedIn = computed(() => currentUser.value !== null)
  const username = computed(() => currentUser.value?.name ?? '')

  // 方法
  async function loginOffline(username: string) {
    loginStatus.value = 'loading'
    error.value = null
    
    try {
      const result = await tauri.loginOffline(username)
      currentUser.value = result
      loginStatus.value = 'success'
    } catch (e) {
      error.value = String(e)
      loginStatus.value = 'error'
      throw e
    }
  }

  async function restoreSession() {
    try {
      const result = await tauri.getLoginStatus()
      if (result) {
        currentUser.value = result
        loginStatus.value = 'success'
      }
    } catch (e) {
      console.error('Failed to restore session:', e)
    }
  }

  async function logoutUser() {
    try {
      await tauri.logout()
    } catch (e) {
      console.error('Failed to logout:', e)
    }
    
    currentUser.value = null
    loginStatus.value = 'idle'
    error.value = null
  }

  return {
    currentUser,
    loginStatus,
    error,
    isLoggedIn,
    username,
    loginOffline,
    restoreSession,
    logout: logoutUser,
  }
})
