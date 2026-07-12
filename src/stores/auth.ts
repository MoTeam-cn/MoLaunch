/**
 * 认证状态管理
 * 支持离线登录和微软登录（Device Code Flow）
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { AuthResult, LoginStatus, DeviceCodeInfo, MsAccountInfo } from '@/types/auth'
import * as tauri from '@/utils/tauri'

export const useAuthStore = defineStore('auth', () => {
  // 状态
  const currentUser = ref<AuthResult | null>(null)
  const loginStatus = ref<LoginStatus>('idle')
  const error = ref<string | null>(null)

  // 微软登录状态
  const msLoginStatus = ref<'idle' | 'device_code' | 'polling' | 'exchanging' | 'success' | 'error'>('idle')
  const deviceCodeInfo = ref<DeviceCodeInfo | null>(null)
  const msAccounts = ref<MsAccountInfo[]>([])

  // 计算属性
  const isLoggedIn = computed(() => currentUser.value !== null)
  const username = computed(() => currentUser.value?.name ?? '')
  const isMicrosoftLogin = computed(() => currentUser.value?.login_type === 'Microsoft')
  const isMsLoggingIn = computed(() =>
    msLoginStatus.value === 'device_code' ||
    msLoginStatus.value === 'polling' ||
    msLoginStatus.value === 'exchanging'
  )

  // ============================================================
  // 离线登录
  // ============================================================

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

  // ============================================================
  // 微软登录 - Device Code Flow
  // ============================================================

  /** 开始微软登录流程：申请设备码 */
  async function startMsLogin() {
    msLoginStatus.value = 'device_code'
    error.value = null
    deviceCodeInfo.value = null

    try {
      const info = await tauri.msLoginStart()
      deviceCodeInfo.value = info
      msLoginStatus.value = 'polling'
      return info
    } catch (e) {
      error.value = String(e)
      msLoginStatus.value = 'error'
      throw e
    }
  }

  /** 轮询微软登录授权结果 */
  async function pollMsLogin(): Promise<boolean> {
    if (!deviceCodeInfo.value) {
      error.value = '设备码信息缺失'
      msLoginStatus.value = 'error'
      return false
    }

    try {
      const result = await tauri.msLoginPoll(deviceCodeInfo.value.device_code)

      if (result.status === 'Pending') {
        return false // 继续轮询
      }

      // Success
      msLoginStatus.value = 'exchanging'
      currentUser.value = {
        name: result.name,
        uuid: result.uuid,
        access_token: result.access_token,
        client_token: result.client_token,
        login_type: result.login_type,
        profile_json: result.profile_json ?? undefined,
      }
      loginStatus.value = 'success'
      msLoginStatus.value = 'success'
      deviceCodeInfo.value = null

      // 刷新账号列表
      await loadMsAccounts()

      return true
    } catch (e) {
      error.value = String(e)
      msLoginStatus.value = 'error'
      throw e
    }
  }

  /** 取消微软登录 */
  function cancelMsLogin() {
    msLoginStatus.value = 'idle'
    deviceCodeInfo.value = null
    error.value = null
  }

  /** 静默刷新微软 Token */
  async function refreshMsToken() {
    try {
      const result = await tauri.msLoginRefresh()
      currentUser.value = result
      loginStatus.value = 'success'
    } catch (e) {
      error.value = String(e)
      // 刷新失败，可能需要重新登录
      throw e
    }
  }

  // ============================================================
  // 微软账号管理
  // ============================================================

  /** 加载已存储的微软账号列表 */
  async function loadMsAccounts() {
    try {
      msAccounts.value = await tauri.getMsAccounts()
    } catch (e) {
      console.error('Failed to load MS accounts:', e)
    }
  }

  /** 删除已存储的微软账号 */
  async function removeMsAccount(uuid: string) {
    try {
      await tauri.removeMsAccount(uuid)
      await loadMsAccounts()
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  /** 切换到已存储的微软账号 */
  async function switchMsAccount(uuid: string) {
    loginStatus.value = 'loading'
    error.value = null

    try {
      const result = await tauri.switchMsAccount(uuid)
      currentUser.value = result
      loginStatus.value = 'success'
    } catch (e) {
      error.value = String(e)
      loginStatus.value = 'error'
      throw e
    }
  }

  // ============================================================
  // 通用方法
  // ============================================================

  /** 恢复会话 */
  async function restoreSession() {
    try {
      const result = await tauri.getLoginStatus()
      if (result) {
        currentUser.value = result
        loginStatus.value = 'success'
      }
      // 加载微软账号列表
      await loadMsAccounts()
    } catch (e) {
      console.error('Failed to restore session:', e)
    }
  }

  /** 登出 */
  async function logoutUser() {
    try {
      await tauri.logout()
    } catch (e) {
      console.error('Failed to logout:', e)
    }

    currentUser.value = null
    loginStatus.value = 'idle'
    error.value = null
    msLoginStatus.value = 'idle'
    deviceCodeInfo.value = null
  }

  return {
    // 状态
    currentUser,
    loginStatus,
    error,
    msLoginStatus,
    deviceCodeInfo,
    msAccounts,
    // 计算属性
    isLoggedIn,
    username,
    isMicrosoftLogin,
    isMsLoggingIn,
    // 离线登录
    loginOffline,
    // 微软登录
    startMsLogin,
    pollMsLogin,
    cancelMsLogin,
    refreshMsToken,
    // 账号管理
    loadMsAccounts,
    removeMsAccount,
    switchMsAccount,
    // 通用
    restoreSession,
    logout: logoutUser,
  }
})
