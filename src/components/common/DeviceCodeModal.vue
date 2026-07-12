<script setup lang="ts">
/**
 * 设备码登录弹窗
 *
 * 展示微软设备码授权信息，自动打开浏览器并轮询授权结果。
 * 支持：复制验证码、重新打开网页、取消登录。
 */

import { ref, watch, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { openUrl } from '@tauri-apps/plugin-shell'

interface Props {
  /** 是否显示 */
  visible: boolean
}

const props = defineProps<Props>()
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'success'): void
}>()

const router = useRouter()
const authStore = useAuthStore()

// 本地状态
const polling = ref(false)
const pollError = ref<string | null>(null)
const countdown = ref(0)

// 轮询定时器
let pollTimer: ReturnType<typeof setTimeout> | null = null
// 倒计时定时器
let countdownTimer: ReturnType<typeof setInterval> | null = null

// 打开浏览器
async function openBrowser() {
  if (authStore.deviceCodeInfo) {
    try {
      await openUrl(authStore.deviceCodeInfo.verification_uri)
    } catch {
      // 降级：忽略错误
    }
  }
}

// 复制验证码
async function copyCode() {
  if (authStore.deviceCodeInfo) {
    try {
      await navigator.clipboard.writeText(authStore.deviceCodeInfo.user_code)
    } catch {
      // 降级：忽略错误
    }
  }
}

// 开始轮询
async function startPolling() {
  if (polling.value) return
  polling.value = true
  pollError.value = null

  // 自动打开浏览器 + 复制验证码
  await openBrowser()
  await copyCode()

  // 启动倒计时
  if (authStore.deviceCodeInfo) {
    countdown.value = authStore.deviceCodeInfo.expires_in
    countdownTimer = setInterval(() => {
      countdown.value--
      if (countdown.value <= 0) {
        stopPolling()
        pollError.value = '设备码已过期，请重新登录'
      }
    }, 1000)
  }

  // 开始轮询
  schedulePoll()
}

// 调度下一次轮询
function schedulePoll() {
  if (!polling.value) return

  const interval = authStore.deviceCodeInfo?.interval ?? 5
  pollTimer = setTimeout(poll, Math.max(interval - 1, 2) * 1000)
}

// 执行轮询
async function poll() {
  if (!polling.value) return

  try {
    const success = await authStore.pollMsLogin()
    if (success) {
      stopPolling()
      emit('success')
      router.push('/')
    } else {
      // 继续轮询
      schedulePoll()
    }
  } catch (e) {
    stopPolling()
    pollError.value = String(e)
  }
}

// 停止轮询
function stopPolling() {
  polling.value = false
  if (pollTimer) {
    clearTimeout(pollTimer)
    pollTimer = null
  }
  if (countdownTimer) {
    clearInterval(countdownTimer)
    countdownTimer = null
  }
}

// 取消
function handleCancel() {
  stopPolling()
  authStore.cancelMsLogin()
  emit('close')
}

// 重试
async function handleRetry() {
  stopPolling()
  pollError.value = null
  try {
    await authStore.startMsLogin()
    startPolling()
  } catch (e) {
    pollError.value = String(e)
  }
}

// 监听 visible 变化
watch(
  () => props.visible,
  async (val) => {
    if (val) {
      // 弹窗打开，开始登录流程
      if (!authStore.deviceCodeInfo) {
        try {
          await authStore.startMsLogin()
        } catch (e) {
          pollError.value = String(e)
          return
        }
      }
      startPolling()
    } else {
      stopPolling()
    }
  }
)

// 组件卸载时清理
onUnmounted(() => {
  stopPolling()
})
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" @click.self="handleCancel">
        <div class="mx-4 w-full max-w-md rounded-2xl bg-white p-6 shadow-xl">
          <!-- 标题 -->
          <div class="mb-4 flex items-center gap-3">
            <!-- 微软四色方块 -->
            <div class="flex h-8 w-8 items-center justify-center">
              <svg viewBox="0 0 23 23" class="h-6 w-6">
                <rect x="1" y="1" width="10" height="10" fill="#F25022" />
                <rect x="12" y="1" width="10" height="10" fill="#7FBA00" />
                <rect x="1" y="12" width="10" height="10" fill="#00A4EF" />
                <rect x="12" y="12" width="10" height="10" fill="#FFB900" />
              </svg>
            </div>
            <h3 class="text-lg font-semibold text-gray-900">微软账号登录</h3>
          </div>

          <!-- 错误状态 -->
          <div v-if="pollError" class="space-y-4">
            <div class="rounded-lg bg-red-50 p-4">
              <p class="text-sm text-red-700">{{ pollError }}</p>
            </div>
            <div class="flex gap-3">
              <button class="btn-primary flex-1 rounded-lg px-4 py-2 text-sm font-medium" @click="handleRetry">
                重新登录
              </button>
              <button class="btn-secondary rounded-lg px-4 py-2 text-sm font-medium" @click="handleCancel">
                取消
              </button>
            </div>
          </div>

          <!-- 正常状态 -->
          <div v-else-if="authStore.deviceCodeInfo" class="space-y-4">
            <!-- 说明 -->
            <p class="text-sm text-gray-600">
              登录网页已自动开启，请在网页中输入以下验证码完成登录：
            </p>

            <!-- 验证码展示 -->
            <div class="rounded-xl bg-gray-50 p-4 text-center">
              <p class="mb-1 text-xs text-gray-400">验证码</p>
              <p class="font-mono text-2xl font-bold tracking-widest text-primary-600 select-all">
                {{ authStore.deviceCodeInfo.user_code }}
              </p>
            </div>

            <!-- 倒计时 -->
            <div v-if="countdown > 0" class="text-center">
              <span class="text-xs text-gray-400">
                有效期剩余 {{ Math.floor(countdown / 60) }}:{{ String(countdown % 60).padStart(2, '0') }}
              </span>
            </div>

            <!-- 轮询状态 -->
            <div v-if="polling" class="flex items-center justify-center gap-2 text-sm text-gray-500">
              <svg class="h-4 w-4 animate-spin text-primary-500" viewBox="0 0 24 24" fill="none">
                <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
                <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
              </svg>
              <span>等待授权中...</span>
            </div>

            <!-- 操作按钮 -->
            <div class="flex gap-3">
              <button class="btn-secondary flex-1 rounded-lg px-3 py-2 text-sm font-medium" @click="openBrowser">
                重新打开网页
              </button>
              <button class="btn-secondary flex-1 rounded-lg px-3 py-2 text-sm font-medium" @click="copyCode">
                复制验证码
              </button>
            </div>

            <button class="w-full rounded-lg px-4 py-2 text-sm font-medium text-gray-400 hover:text-gray-600" @click="handleCancel">
              取消登录
            </button>
          </div>

          <!-- 加载状态 -->
          <div v-else class="flex items-center justify-center py-8">
            <svg class="h-8 w-8 animate-spin text-primary-500" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
              <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
            </svg>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
