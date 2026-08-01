<script setup lang="ts">
/**
 * 认证中心：管理厂商 OAuth2 / Device Code / API Key 认证。
 * 参见 FRP_MANAGER_DESIGN.md §6.8。
 */
import { onMounted, onUnmounted, computed, ref } from 'vue'
import { useFrpStore } from '@/stores/frp'
import { open } from '@tauri-apps/plugin-shell'
import Button from '@/components/common/Button.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import Input from '@/components/common/Input.vue'
import { showConfirm } from '@/utils/modal'
import { toastInfo, toastError } from '@/utils/toast'
import {
  ShieldCheckIcon,
  ArrowPathIcon,
  KeyIcon,
  ArrowTopRightOnSquareIcon,
  CheckCircleIcon,
  ExclamationCircleIcon,
  XCircleIcon,
} from '@heroicons/vue/24/outline'
import type { AuthStatus, ProviderInfo } from '@/types/frp'

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
  try { await open(url) } catch { toastError('打开链接失败') }
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
</script>

<template>
  <div class="space-y-4">
    <!-- 顶部操作栏 -->
    <div class="flex items-center justify-between flex-wrap gap-2">
      <p class="text-sm text-gray-500">共 {{ providers.length }} 个厂商</p>
      <Tooltip text="刷新认证状态">
        <Button type="ghost" size="small" :loading="loading" @click="handleRefreshAuthStatuses">
          <template #icon><ArrowPathIcon class="w-4 h-4" /></template>
        </Button>
      </Tooltip>
    </div>

    <!-- 厂商认证卡片 -->
    <div v-if="providers.length > 0" class="space-y-3">
      <div
        v-for="provider in providers"
        :key="provider.id"
        class="rounded-lg border border-gray-200 bg-white p-4 hover:border-primary-300 transition-all"
      >
        <div class="flex items-start gap-3">
          <!-- 厂商图标 -->
          <div class="w-10 h-10 rounded-lg bg-primary-50 flex items-center justify-center shrink-0 overflow-hidden">
            <img v-if="provider.icon" :src="provider.icon" :alt="provider.name" class="w-full h-full object-cover" />
            <ShieldCheckIcon v-else class="w-5 h-5 text-primary-600" />
          </div>

          <!-- 厂商信息 + 认证状态 -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 flex-wrap">
              <span class="text-sm font-semibold text-gray-900">{{ provider.name }}</span>
              <span v-if="authBadge(provider.authType)" class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium" :class="authBadge(provider.authType)!.cls">
                {{ authBadge(provider.authType)!.text }}
              </span>
            </div>
            <p class="text-xs text-gray-500 mt-1">{{ provider.description }}</p>

            <!-- 认证状态行 -->
            <div class="flex items-center gap-1.5 mt-2">
              <CheckCircleIcon v-if="statusInfo(store.authStatuses[provider.id]).cls.includes('green')" class="w-4 h-4" :class="statusInfo(store.authStatuses[provider.id]).cls" />
              <ExclamationCircleIcon v-else-if="statusInfo(store.authStatuses[provider.id]).cls.includes('amber')" class="w-4 h-4" :class="statusInfo(store.authStatuses[provider.id]).cls" />
              <XCircleIcon v-else-if="statusInfo(store.authStatuses[provider.id]).cls.includes('red')" class="w-4 h-4" :class="statusInfo(store.authStatuses[provider.id]).cls" />
              <span class="text-xs font-medium" :class="statusInfo(store.authStatuses[provider.id]).cls">
                {{ statusInfo(store.authStatuses[provider.id]).text }}
              </span>
              <span v-if="store.authStatuses[provider.id]?.authenticated && store.authStatuses[provider.id]?.expiresAt" class="text-xs text-gray-400">
                {{ formatRemaining(store.authStatuses[provider.id]?.expiresAt) }}
              </span>
            </div>

            <!-- 权限范围 -->
            <div v-if="store.authStatuses[provider.id]?.scopes?.length" class="flex items-center gap-1 mt-1 flex-wrap">
              <span class="text-xs text-gray-400">权限：</span>
              <span v-for="scope in store.authStatuses[provider.id]!.scopes" :key="scope" class="inline-flex px-1.5 py-0.5 rounded text-xs bg-gray-100 text-gray-600">{{ scope }}</span>
            </div>

            <!-- Device Code 流程展示 -->
            <div v-if="store.deviceCodeInfos[provider.id]" class="mt-3 p-3 bg-purple-50 rounded-md space-y-2">
              <div class="flex items-center gap-2">
                <span class="text-xs text-gray-600">用户码：</span>
                <span class="text-lg font-mono font-bold text-purple-700 tracking-wider">{{ store.deviceCodeInfos[provider.id].userCode }}</span>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-xs text-gray-600">验证链接：</span>
                <a class="text-xs text-blue-600 hover:underline flex items-center gap-0.5 cursor-pointer" @click.prevent="openUrl(store.deviceCodeInfos[provider.id]!.verificationUri)">
                  {{ store.deviceCodeInfos[provider.id].verificationUri }}
                  <ArrowTopRightOnSquareIcon class="w-3 h-3" />
                </a>
              </div>
              <div class="flex items-center justify-between">
                <span class="text-xs text-gray-500">倒计时：{{ formatCountdown(countdowns[provider.id] || 0) }}</span>
                <div class="flex items-center gap-1.5">
                  <span v-if="store.deviceCodePolling[provider.id]" class="flex items-center gap-1 text-xs text-purple-600">
                    <ArrowPathIcon class="w-3 h-3 animate-spin" />轮询中
                  </span>
                  <Button type="ghost" size="mini" @click="handleCancelDeviceCode(provider.id)">取消</Button>
                </div>
              </div>
            </div>
          </div>

          <!-- 操作区 -->
          <div class="shrink-0 flex flex-col items-end gap-1.5">
            <!-- 无需认证 -->
            <span v-if="provider.authType === 'none'" class="text-xs text-gray-400">无需认证</span>

            <!-- OAuth2 未认证 -->
            <Button v-else-if="provider.authType === 'oauth2' && !store.authStatuses[provider.id]?.authenticated" type="primary" size="small" :loading="!!store.authActionLoading[provider.id]" @click="handleStartOAuth2(provider.id)">
              <template #icon><KeyIcon class="w-3.5 h-3.5" /></template>
              开始认证
            </Button>

            <!-- Device Code 未认证 -->
            <Button v-else-if="provider.authType === 'device_code' && !store.authStatuses[provider.id]?.authenticated && !store.deviceCodeInfos[provider.id]" type="primary" size="small" :loading="!!store.authActionLoading[provider.id]" @click="handleStartDeviceCode(provider.id)">
              <template #icon><KeyIcon class="w-3.5 h-3.5" /></template>
              开始认证
            </Button>

            <!-- API Key 输入 -->
            <div v-else-if="provider.authType === 'api_key' && !store.authStatuses[provider.id]?.authenticated" class="flex items-center gap-1.5 w-48">
              <Input :model-value="store.apiKeyInputs[provider.id] || ''" placeholder="输入 API Key" size="small" @update:model-value="(v: string) => store.apiKeyInputs[provider.id] = v" />
              <Button type="primary" size="small" :loading="!!store.authActionLoading[provider.id]" @click="handleSaveApiKey(provider.id)">保存</Button>
            </div>

            <!-- 已认证：刷新 + 撤销 -->
            <template v-else-if="store.authStatuses[provider.id]?.authenticated && provider.authType !== 'none'">
              <Button v-if="provider.authType !== 'api_key'" type="outline" size="mini" :loading="!!store.authActionLoading[provider.id]" @click="handleRefreshToken(provider.id)">
                <template #icon><ArrowPathIcon class="w-3.5 h-3.5" /></template>
                刷新
              </Button>
              <Button type="ghost" size="mini" :loading="!!store.authActionLoading[provider.id]" @click="handleRevokeAuth(provider)">
                撤销
              </Button>
            </template>
          </div>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else-if="!loading" class="flex flex-col items-center justify-center py-16">
      <ShieldCheckIcon class="w-12 h-12 text-gray-300 mb-3" />
      <p class="text-sm font-medium text-gray-500">暂无厂商</p>
      <p class="text-xs text-gray-400 mt-1">系统默认厂商将自动显示，或先安装外部厂商</p>
    </div>

    <!-- 加载中 -->
    <div v-else class="flex items-center justify-center py-16">
      <ArrowPathIcon class="w-6 h-6 text-gray-400 animate-spin" />
    </div>
  </div>
</template>
