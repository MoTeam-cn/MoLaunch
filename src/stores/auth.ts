/**
 * 认证状态管理
 * 支持离线登录和微软登录（Web Auth Code Flow / Device Code Flow）
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { AuthResult, LoginStatus, MsAccountInfo, OfflineAccountInfo, DeviceCodeInfo, PollResult } from '@/types/auth'
import * as tauri from '@/utils/tauri'
import { syncOfflineSkins } from '@/utils/default-skin'

const STEP_LABELS: Record<string, string> = {
  exchanging: '授权成功，开始交换 Token...',
  xbl: '正在获取 XBL Token...',
  xsts: '正在获取 XSTS Token...',
  mc_token: '正在获取 Minecraft Token...',
  entitlements: '正在验证游戏所有权...',
  profile: '正在获取玩家档案...',
}

export const useAuthStore = defineStore('auth', () => {
  const currentUser = ref<AuthResult | null>(null)
  const loginStatus = ref<LoginStatus>('idle')
  const error = ref<string | null>(null)
  const msLoginStatus = ref<'idle' | 'requesting' | 'waiting' | 'exchanging' | 'success' | 'error'>('idle')
  const msFlow = ref<'web' | 'device_code' | ''>('')
  const deviceCodeInfo = ref<DeviceCodeInfo | null>(null)
  const msAccounts = ref<MsAccountInfo[]>([])
  const offlineAccounts = ref<OfflineAccountInfo[]>([])
  const msLoginStep = ref('')
  const msLoginStepLabel = computed(() => STEP_LABELS[msLoginStep.value] ?? '')

  let pollTimer: ReturnType<typeof setTimeout> | null = null
  let progressUnlisten: UnlistenFn | null = null
  let codeUnlisten: UnlistenFn | null = null

  const isLoggedIn = computed(() => currentUser.value !== null)
  const username = computed(() => currentUser.value?.name ?? '')
  const isMicrosoftLogin = computed(() => currentUser.value?.login_type === 'Microsoft')
  const isMsLoggingIn = computed(() => ['requesting', 'waiting', 'exchanging'].includes(msLoginStatus.value))

  function cleanup() {
    if (pollTimer) { clearTimeout(pollTimer); pollTimer = null }
    if (progressUnlisten) { progressUnlisten(); progressUnlisten = null }
    if (codeUnlisten) { codeUnlisten(); codeUnlisten = null }
    msLoginStep.value = ''
  }

  async function startProgressListener() {
    progressUnlisten = await listen<string>('ms-login-progress', (e) => {
      msLoginStep.value = e.payload
      if (msLoginStatus.value === 'waiting') msLoginStatus.value = 'exchanging'
    })
  }

  function handleResult(result: PollResult): boolean {
    switch (result.status) {
      case 'Pending': return false
      case 'Success':
        cleanup()
        msLoginStatus.value = 'success'; loginStatus.value = 'success'
        currentUser.value = result.auth; deviceCodeInfo.value = null
        loadMsAccounts()
        return true
      case 'Declined':
        cleanup(); error.value = '授权被拒绝'; msLoginStatus.value = 'error'; return true
      case 'Expired':
        cleanup(); error.value = '设备码已过期，请重新登录'; msLoginStatus.value = 'error'; return true
      case 'Error':
        cleanup(); error.value = result.message; msLoginStatus.value = 'error'; return true
    }
  }

  async function startMsLogin() {
    msLoginStatus.value = 'requesting'
    error.value = null; deviceCodeInfo.value = null; msLoginStep.value = ''
    cleanup()
    try {
      const config = await tauri.msLoginGetConfig()
      msFlow.value = config.flow as 'web' | 'device_code'
      await startProgressListener()
      if (config.flow === 'web') await startWebFlow()
      else await startDeviceCodeFlow()
    } catch (e) {
      error.value = String(e); msLoginStatus.value = 'error'; cleanup(); throw e
    }
  }

  async function startWebFlow() {
    codeUnlisten = await listen<string>('ms-auth-code', async (event) => {
      if (codeUnlisten) { codeUnlisten(); codeUnlisten = null }
      try {
        const result = await tauri.msLoginWebExchange(event.payload)
        handleResult(result)
      } catch (e) {
        cleanup(); error.value = String(e); msLoginStatus.value = 'error'
      }
    })
    await tauri.msLoginWebStart()
    msLoginStatus.value = 'waiting'
  }

  async function startDeviceCodeFlow() {
    const info = await tauri.msLoginRequestDeviceCode()
    deviceCodeInfo.value = info
    msLoginStatus.value = 'waiting'
    startPolling(info.device_code, info.interval * 1000)
  }

  function startPolling(deviceCode: string, intervalMs: number) {
    async function pollOnce() {
      try {
        const result = await tauri.msLoginPoll(deviceCode)
        if (!handleResult(result)) pollTimer = setTimeout(pollOnce, intervalMs)
      } catch (e) {
        cleanup(); error.value = String(e); msLoginStatus.value = 'error'
      }
    }
    pollTimer = setTimeout(pollOnce, 2000)
  }

  function cancelMsLogin() {
    cleanup(); msLoginStatus.value = 'idle'; error.value = null; deviceCodeInfo.value = null
  }

  async function loginOffline(name: string) {
    loginStatus.value = 'loading'; error.value = null
    try {
      currentUser.value = await tauri.loginOffline(name)
      loginStatus.value = 'success'
      await loadOfflineAccounts()
    } catch (e) {
      error.value = String(e); loginStatus.value = 'error'; throw e
    }
  }

  async function refreshMsToken() {
    try {
      currentUser.value = await tauri.msLoginRefresh()
      loginStatus.value = 'success'
    } catch (e) { error.value = String(e); throw e }
  }

  async function loadMsAccounts() {
    try { msAccounts.value = await tauri.getMsAccounts() }
    catch (e) { console.error('Failed to load MS accounts:', e) }
  }

  async function loadOfflineAccounts() {
    try {
      offlineAccounts.value = await tauri.getOfflineAccounts()
      // 同步离线账号皮肤选择到内存缓存
      syncOfflineSkins(offlineAccounts.value)
    }
    catch (e) { console.error('Failed to load offline accounts:', e) }
  }

  async function removeMsAccount(uuid: string) {
    try { await tauri.removeMsAccount(uuid); await loadMsAccounts() }
    catch (e) { error.value = String(e); throw e }
  }

  async function switchMsAccount(uuid: string) {
    loginStatus.value = 'loading'; error.value = null
    try {
      currentUser.value = await tauri.switchMsAccount(uuid)
      loginStatus.value = 'success'
    } catch (e) { error.value = String(e); loginStatus.value = 'error'; throw e }
  }

  async function removeOfflineAccount(uuid: string) {
    try { await tauri.removeOfflineAccount(uuid); await loadOfflineAccounts() }
    catch (e) { error.value = String(e); throw e }
  }

  async function switchOfflineAccount(uuid: string) {
    loginStatus.value = 'loading'; error.value = null
    try {
      currentUser.value = await tauri.switchOfflineAccount(uuid)
      loginStatus.value = 'success'
    } catch (e) { error.value = String(e); loginStatus.value = 'error'; throw e }
  }

  async function restoreSession() {
    try {
      const result = await tauri.getLoginStatus()
      if (result) { currentUser.value = result; loginStatus.value = 'success' }
      await Promise.all([loadMsAccounts(), loadOfflineAccounts()])
    } catch (e) { console.error('Failed to restore session:', e) }
  }

  async function logoutUser() {
    try { await tauri.logout() } catch (e) { console.error('Failed to logout:', e) }
    cleanup(); currentUser.value = null; loginStatus.value = 'idle'
    error.value = null; msLoginStatus.value = 'idle'; deviceCodeInfo.value = null
  }

  return {
    currentUser, loginStatus, error, msLoginStatus, msFlow, deviceCodeInfo,
    msAccounts, offlineAccounts, msLoginStep, msLoginStepLabel,
    isLoggedIn, username, isMicrosoftLogin, isMsLoggingIn,
    loginOffline, startMsLogin, cancelMsLogin, refreshMsToken,
    loadMsAccounts, loadOfflineAccounts, removeMsAccount, switchMsAccount,
    removeOfflineAccount, switchOfflineAccount,
    restoreSession, logout: logoutUser,
  }
})
