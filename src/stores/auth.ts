/**
 * 认证状态管理
 * 支持离线登录、微软登录（Web Auth Code Flow / Device Code Flow）和 authlib 外置登录（yggdrasil 协议）
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { AuthlibAccountInfo, DeviceCodeInfo, LocalAuthResult, LoginStatus, MsAccountInfo, OfflineAccountInfo, PollResult } from '@/types/auth'
import * as tauri from '@/utils/tauri'
import { syncOfflineSkins } from '@/utils/default-skin'
import { safeCall } from '@/utils/async'

const STEP_LABELS: Record<string, string> = {
  exchanging: '授权成功，开始交换 Token...',
  xbl: '正在获取 XBL Token...',
  xsts: '正在获取 XSTS Token...',
  mc_token: '正在获取 Minecraft Token...',
  entitlements: '正在验证游戏所有权...',
  profile: '正在获取玩家档案...',
}

export const useAuthStore = defineStore('auth', () => {
  // 使用 LocalAuthResult 而非 AuthResult：后端所有登录方法（离线/微软/authlib）
  // 统一返回 LocalAuthResult，含 server_url / server_name 字段（仅 authlib 账号有值）。
  // AuthResult 结构上可赋值给 LocalAuthResult（额外字段均为可选），赋值不受影响。
  const currentUser = ref<LocalAuthResult | null>(null)
  const loginStatus = ref<LoginStatus>('idle')
  const error = ref<string | null>(null)
  const msLoginStatus = ref<'idle' | 'requesting' | 'waiting' | 'exchanging' | 'success' | 'error'>('idle')
  const msFlow = ref<'web' | 'device_code' | ''>('')
  const deviceCodeInfo = ref<DeviceCodeInfo | null>(null)
  const msAccounts = ref<MsAccountInfo[]>([])
  const offlineAccounts = ref<OfflineAccountInfo[]>([])
  const authlibAccounts = ref<AuthlibAccountInfo[]>([])
  const msLoginStep = ref('')
  const msLoginStepLabel = computed(() => STEP_LABELS[msLoginStep.value] ?? '')
  /** 会话恢复中标志：应用启动时为 true，restoreSession 完成后置为 false。
   *  路由守卫在此期间不拦截 requiresAuth 路由，避免会话还没恢复就把已登录用户踢到 /login */
  const isRestoring = ref(true)

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
    const accounts = await safeCall(() => tauri.getMsAccounts(), 'load MS accounts')
    if (accounts) msAccounts.value = accounts
  }

  async function loadOfflineAccounts() {
    const accounts = await safeCall(async () => {
      const list = await tauri.getOfflineAccounts()
      // 同步离线账号皮肤选择到内存缓存
      syncOfflineSkins(list)
      return list
    }, 'load offline accounts')
    if (accounts) offlineAccounts.value = accounts
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

  // ============================================================
  // authlib 外置登录（yggdrasil 协议）账号管理
  // ============================================================

  /** 加载已保存的 authlib 账号列表 */
  async function loadAuthlibAccounts() {
    const accounts = await safeCall(() => tauri.getAuthlibAccounts(), 'load authlib accounts')
    if (accounts) authlibAccounts.value = accounts
  }

  /** 删除指定 authlib 账号（按 server_url + uuid 定位） */
  async function removeAuthlibAccount(serverUrl: string, uuid: string) {
    try {
      await tauri.removeAuthlibAccount(serverUrl, uuid)
      await loadAuthlibAccounts()
    } catch (e) { error.value = String(e); throw e }
  }

  /** 切换到已保存的 authlib 账号（三步降级：validate → refresh → 用密码重登） */
  async function switchAuthlibAccount(serverUrl: string, uuid: string) {
    loginStatus.value = 'loading'; error.value = null
    try {
      currentUser.value = await tauri.switchAuthlibAccount(serverUrl, uuid)
      loginStatus.value = 'success'
    } catch (e) { error.value = String(e); loginStatus.value = 'error'; throw e }
  }

  // 防重入：App.vue 和 Home.vue 都会在 onMounted 调用 restoreSession，
  // 用 Promise 缓存避免并发触发多次 silent refresh（否则会冲击 Mojang API 触发 429 风控）
  let restoringPromise: Promise<void> | null = null

  async function restoreSession() {
    if (restoringPromise) return restoringPromise
    restoringPromise = (async () => {
      await safeCall(async () => {
        const result = await tauri.getLoginStatus()
        if (result) { currentUser.value = result; loginStatus.value = 'success' }
        await Promise.all([loadMsAccounts(), loadOfflineAccounts(), loadAuthlibAccounts()])
      }, 'restore session')
    })()
    try {
      await restoringPromise
    } finally {
      restoringPromise = null
      isRestoring.value = false
    }
  }

  async function logoutUser() {
    await safeCall(() => tauri.logout(), 'logout')
    cleanup(); currentUser.value = null; loginStatus.value = 'idle'
    error.value = null; msLoginStatus.value = 'idle'; deviceCodeInfo.value = null
  }

  return {
    currentUser, loginStatus, error, msLoginStatus, msFlow, deviceCodeInfo,
    msAccounts, offlineAccounts, authlibAccounts, msLoginStep, msLoginStepLabel,
    isLoggedIn, username, isMicrosoftLogin, isMsLoggingIn, isRestoring,
    loginOffline, startMsLogin, cancelMsLogin, refreshMsToken,
    loadMsAccounts, loadOfflineAccounts, removeMsAccount, switchMsAccount,
    removeOfflineAccount, switchOfflineAccount,
    loadAuthlibAccounts, removeAuthlibAccount, switchAuthlibAccount,
    restoreSession, logout: logoutUser,
  }
})
