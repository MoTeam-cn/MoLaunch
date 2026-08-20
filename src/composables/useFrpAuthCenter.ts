/**
 * 认证中心组合式逻辑：厂商认证状态/时间格式化、Device Code 倒计时轮询定时器与认证动作。
 * 从 components/frp/AuthCenter.vue 拆出，避免 Vue 组件超 300 行。
 *
 * 模板仍直接访问 useFrpStore() 的认证状态（authStatuses / deviceCodeInfos /
 * deviceCodePolling / authActionLoading / apiKeyInputs），本文件只承载派生逻辑与动作。
 */
import { onMounted, onUnmounted, computed, ref } from 'vue'
import { useFrpStore } from '@/stores/frp'
import { openExternal } from '@/utils/openExternal'
import { showConfirm } from '@/utils/modal'
import { toastInfo, toastError } from '@/utils/toast'
import type { AuthStatus, ProviderInfo } from '@/types/frp'

export function useFrpAuthCenter() {
  const store = useFrpStore()
  const providers = computed(() => store.providers)
  const loading = computed(() => store.providersLoading)
  const now = ref(Math.floor(Date.now() / 1000))
  const countdowns = ref<Record<string, number>>({})
  const timers = new Map<string, { countdown: ReturnType<typeof setInterval>; poll: ReturnType<typeof setInterval> }>()

  /** authType 徽章 */
  function authBadge(authType: string): { cls: string; text: string } | null {
    switch (authType) {
      case 'none': return { cls: 'bg-green-50 text-green-700', text: '无需认证' }
      case 'oauth2': return { cls: 'bg-blue-50 text-blue-700', text: 'OAuth2' }
      case 'device_code': return { cls: 'bg-purple-50 text-purple-700', text: 'Device Code' }
      case 'api_key': return { cls: 'bg-yellow-50 text-yellow-700', text: 'API Key' }
      default: return null
    }
  }

  /** 认证状态信息（文案 + 颜色） */
  function statusInfo(status?: AuthStatus): { text: string; cls: string } {
    if (!status || status.authType === 'none') return { text: '—', cls: 'text-gray-400' }
    // 续期中：token 已过期但存在 refresh_token，后端正在/刚尝试静默续期
    if (!status.authenticated && status.refreshing) {
      return { text: '续期中', cls: 'text-blue-600' }
    }
    if (!status.authenticated) {
      if (status.expiresAt) return { text: '已过期', cls: 'text-red-600' }
      return { text: '未认证', cls: 'text-gray-500' }
    }
    if (status.expiresAt) {
      const remaining = status.expiresAt - now.value
      if (remaining < 300) return { text: '即将过期', cls: 'text-amber-600' }
      return { text: '已认证', cls: 'text-green-600' }
    }
    return { text: '已认证', cls: 'text-green-600' }
  }

  /** 格式化剩余时间 */
  function formatRemaining(expiresAt?: number): string {
    if (!expiresAt) return ''
    const remaining = expiresAt - now.value
    if (remaining <= 0) return '已过期'
    const h = Math.floor(remaining / 3600)
    const m = Math.floor((remaining % 3600) / 60)
    if (h > 0) return `剩余 ${h} 小时 ${m} 分钟`
    if (m > 0) return `剩余 ${m} 分钟`
    return `剩余 ${remaining} 秒`
  }

  /** 格式化倒计时（Device Code） */
  function formatCountdown(sec: number): string {
    const m = Math.floor(sec / 60)
    const s = sec % 60
    return `${m}:${s.toString().padStart(2, '0')}`
  }

  /** 打开外部链接 */
  async function openUrl(url: string): Promise<void> {
    try { await openExternal(url) } catch { toastError('打开链接失败') }
  }

  /** 启动 OAuth2 认证 */
  async function handleStartOAuth2(pid: string): Promise<void> {
    await store.startOAuth2Auth(pid)
    toastInfo('认证窗口已在浏览器打开，请完成后返回')
  }

  /** 启动 Device Code 流程 + 自动轮询 */
  async function handleStartDeviceCode(pid: string): Promise<void> {
    const ok = await store.startDeviceCodeAuth(pid)
    if (!ok) return
    const info = store.deviceCodeInfos[pid]
    if (info) startPolling(pid, info.interval, info.expiresIn)
    toastInfo('Device Code 流程已启动，请访问验证链接输入用户码')
  }

  /** 启动倒计时 + 轮询定时器 */
  function startPolling(pid: string, interval: number, expiresIn: number): void {
    clearTimers(pid)
    countdowns.value[pid] = expiresIn
    const countdownTimer = setInterval(() => {
      countdowns.value[pid] = Math.max(0, countdowns.value[pid] - 1)
      if (countdowns.value[pid] === 0) {
        clearTimers(pid)
        store.cancelDeviceCode(pid)
      }
    }, 1000)
    const pollTimer = setInterval(async () => {
      const shouldContinue = await store.pollDeviceCodeAuth(pid)
      if (!shouldContinue) clearTimers(pid)
    }, interval * 1000)
    timers.set(pid, { countdown: countdownTimer, poll: pollTimer })
  }

  /** 清理定时器 */
  function clearTimers(pid: string): void {
    const t = timers.get(pid)
    if (t) {
      clearInterval(t.countdown)
      clearInterval(t.poll)
      timers.delete(pid)
    }
  }

  /** 取消 Device Code 流程 */
  function handleCancelDeviceCode(pid: string): void {
    clearTimers(pid)
    store.cancelDeviceCode(pid)
    toastInfo('已取消 Device Code 流程')
  }

  /** 刷新 token */
  async function handleRefreshToken(pid: string): Promise<void> {
    await store.refreshTokenAuth(pid)
  }

  /** 撤销认证（二次确认） */
  function handleRevokeAuth(p: ProviderInfo): void {
    showConfirm('撤销认证', `确定要撤销「${p.name}」的认证吗？已保存的 token 将被删除。`, async () => {
      await store.revokeAuthAuth(p.id)
    })
  }

  /** 保存 API Key */
  async function handleSaveApiKey(pid: string): Promise<void> {
    const key = store.apiKeyInputs[pid] || ''
    await store.saveApiKeyAuth(pid, key)
  }

  /** 刷新认证状态 */
  async function handleRefreshAuthStatuses(): Promise<void> {
    await store.loadAuthStatuses()
    toastInfo('认证状态已刷新')
  }

  let nowTimer: ReturnType<typeof setInterval> | null = null

  onMounted(async () => {
    await store.loadProviders()
    await store.loadAuthStatuses()
    nowTimer = setInterval(() => { now.value = Math.floor(Date.now() / 1000) }, 1000)
  })

  onUnmounted(() => {
    if (nowTimer) clearInterval(nowTimer)
    timers.forEach(t => { clearInterval(t.countdown); clearInterval(t.poll) })
    timers.clear()
  })

  return {
    providers, loading, countdowns,
    authBadge, statusInfo, formatRemaining, formatCountdown,
    openUrl,
    handleStartOAuth2, handleStartDeviceCode,
    handleCancelDeviceCode, handleRefreshToken, handleRevokeAuth,
    handleSaveApiKey, handleRefreshAuthStatuses,
  }
}
