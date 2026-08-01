<script setup lang="ts">
/**
 * 微软登录弹窗
 * 支持 Web Auth Code Flow（官方 ID）和 Device Code Flow（自定义 ID）
 */

import { computed, ref, watch, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { open } from '@tauri-apps/plugin-shell'
import { useAuthStore } from '@/stores/auth'
import Button from '@/components/common/Button.vue'
import StepProgressBar from '@/components/common/StepProgressBar.vue'
import { toastSuccess, toastError } from '@/utils/toast'

const props = defineProps<{ visible: boolean }>()
const emit = defineEmits<{ (e: 'close'): void; (e: 'success'): void }>()

const router = useRouter()
const authStore = useAuthStore()

const STEPS = [
  { key: 'xbl', label: '获取 XBL Token' },
  { key: 'xsts', label: '获取 XSTS Token' },
  { key: 'mc_token', label: '获取 Minecraft Token' },
  { key: 'entitlements', label: '验证游戏所有权' },
  { key: 'profile', label: '获取玩家档案' },
] as const

const stepIndex = computed(() => {
  const s = authStore.msLoginStep
  if (!s || s === 'exchanging') return -1
  return STEPS.findIndex((x) => x.key === s)
})

const isWebFlow = computed(() => authStore.msFlow === 'web')

// verification_uri 白名单，防止钓鱼替换
const ALLOWED_URIS = [
  'https://microsoft.com/link',
  'https://login.microsoftonline.com',
  'https://login.live.com',
  'https://www.microsoft.com/link',
]
// 打开登录页失败/被拦截时的提示信息
const uriError = ref('')

async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text)
    toastSuccess('设备码已复制到剪贴板')
    return true
  } catch {
    toastError('复制失败，请手动复制')
    return false
  }
}

async function openBrowser(url: string) {
  try { await open(url) } catch { toastError('打开浏览器失败，请手动访问该链接') }
}

/** 用户主动点击：打开 Microsoft 登录页（带白名单校验） */
function openLoginUrl() {
  if (!authStore.deviceCodeInfo) return
  const uri = authStore.deviceCodeInfo.verification_uri
  const isAllowed = ALLOWED_URIS.some(allowed => uri === allowed || uri.startsWith(allowed + '/'))
  if (!isAllowed) {
    uriError.value = '登录地址不在受信任白名单内，已拦截以防止钓鱼跳转'
    return
  }
  uriError.value = ''
  void openBrowser(uri)
}

/** 用户主动点击：复制设备码到剪贴板 */
async function copyCode() {
  if (!authStore.deviceCodeInfo) return
  const ok = await copyToClipboard(authStore.deviceCodeInfo.user_code)
  if (ok) {
    copyFailed.value = false
    toastSuccess('设备码已复制到剪贴板')
  } else {
    copyFailed.value = true
    toastError('复制失败，请手动复制')
  }
}

function handleCancel() { authStore.cancelMsLogin(); emit('close') }
async function handleRetry() { await authStore.startMsLogin() }

// 设备码自动辅助标志：防止自动复制/自动打开与用户手动点击重复
let autoOpened = false
let autoOpenTimer: ReturnType<typeof setTimeout> | null = null
/** 自动复制是否失败：失败则放弃自动打开网页，并提示用户手动复制 */
const copyFailed = ref(false)

watch(() => props.visible, async (val) => {
  if (val) {
    autoOpened = false
    copyFailed.value = false
    try { await authStore.startMsLogin() } catch { /* handled in store */ }
  } else {
    // 关闭弹窗时取消未触发的自动打开
    if (autoOpenTimer) { clearTimeout(autoOpenTimer); autoOpenTimer = null }
  }
})

watch(() => authStore.deviceCodeInfo, async (info) => {
  // 设备码流程：自动复制设备码 + 延迟 2s 自动打开授权网页（带白名单校验）
  // 避免剪贴板嗅探：只复制官方生成的 user_code，不复制 verification_uri 之外的任何内容
  if (info) {
    uriError.value = ''
    const ok = await copyToClipboard(info.user_code)
    if (!ok) {
      // 复制失败：放弃自动打开网页，提示用户手动复制
      copyFailed.value = true
      toastError('复制设备码失败，请手动复制')
      return
    }
    copyFailed.value = false
    autoOpenTimer = setTimeout(() => {
      if (autoOpened) return
      autoOpened = true
      openLoginUrl()
    }, 2000)
  }
})

watch(() => authStore.msLoginStatus, (status) => {
  if (status === 'success') { toastSuccess('登录成功'); emit('success'); router.push('/apps') }
})

onUnmounted(() => {
  if (autoOpenTimer) { clearTimeout(autoOpenTimer); autoOpenTimer = null }
  authStore.cancelMsLogin()
})
</script>

<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="visible" class="fixed inset-0 z-[10000] flex items-center justify-center p-4" @click.self="handleCancel">
        <div class="absolute inset-0 bg-black/50" />

        <div class="relative w-full max-w-md bg-white rounded-2xl shadow-xl">
          <!-- 标题 -->
          <div class="px-6 pt-6 pb-4 flex items-center gap-3">
            <svg viewBox="0 0 23 23" class="h-6 w-6">
              <rect x="1" y="1" width="10" height="10" fill="#F25022" />
              <rect x="12" y="1" width="10" height="10" fill="#7FBA00" />
              <rect x="1" y="12" width="10" height="10" fill="#00A4EF" />
              <rect x="12" y="12" width="10" height="10" fill="#FFB900" />
            </svg>
            <h3 class="text-lg font-semibold text-gray-900">微软账号登录</h3>
          </div>

          <div class="px-6 pb-6">
            <!-- 请求中 -->
            <div v-if="authStore.msLoginStatus === 'requesting'" class="flex flex-col items-center py-8">
              <div class="h-8 w-8 animate-spin rounded-full border-[3px] border-primary-200 border-t-primary-500" />
              <p class="mt-3 text-sm text-gray-500">正在准备登录...</p>
            </div>

            <!-- 等待用户授权 -->
            <div v-else-if="authStore.msLoginStatus === 'waiting'" class="space-y-4">
              <!-- Web Flow: 等待窗口登录 -->
              <div v-if="isWebFlow" class="space-y-3 py-2">
                <p class="text-sm text-gray-600">请在弹出的浏览器窗口中完成 Microsoft 账号登录。</p>
                <div class="flex items-center justify-center gap-2 text-sm text-gray-500">
                  <div class="h-4 w-4 animate-spin rounded-full border-2 border-primary-200 border-t-primary-500" />
                  <span>等待登录完成...</span>
                </div>
              </div>
              <!-- Device Code Flow: 显示设备码 -->
              <div v-else-if="authStore.deviceCodeInfo" class="space-y-3">
                <div class="text-center">
                  <p class="mb-2 text-sm text-gray-600">
                    {{ copyFailed ? '请点击下方按钮打开 Microsoft 登录页，并手动复制输入以下代码：' : '授权网页已打开（未打开可点下方按钮），输入以下代码：' }}
                  </p>
                  <div class="my-3 select-all rounded-lg bg-gray-100 py-3 text-2xl font-bold tracking-widest text-gray-900">
                    {{ authStore.deviceCodeInfo.user_code }}
                  </div>
                  <p class="text-xs text-gray-400">
                    {{ copyFailed ? '复制失败，请手动长按/右键复制设备码' : '设备码已复制到剪贴板，可直接粘贴' }}
                  </p>
                </div>
                <div class="flex gap-2">
                  <Button type="primary" class="flex-1" @click="openLoginUrl">打开 Microsoft 登录页</Button>
                  <Button type="secondary" @click="copyCode">重新复制</Button>
                </div>
                <p v-if="uriError" class="rounded-lg bg-red-50 px-3 py-2 text-xs text-red-700">{{ uriError }}</p>
                <div class="flex items-center justify-center gap-2 text-sm text-gray-500">
                  <div class="h-4 w-4 animate-spin rounded-full border-2 border-primary-200 border-t-primary-500" />
                  <span>等待授权中...</span>
                </div>
              </div>
              <Button type="text" long @click="handleCancel">取消登录</Button>
            </div>

            <!-- Token 交换中（进度条仅在此阶段显示：伪进度让等待 Token 转换/XBL/XSTS 等阶段的用户心里好受点） -->
            <div v-else-if="authStore.msLoginStatus === 'exchanging'" class="space-y-3 py-2">
              <div class="flex items-center gap-2 text-sm text-gray-500">
                <div class="h-4 w-4 animate-spin rounded-full border-2 border-primary-200 border-t-primary-500" />
                <span>正在登录，请稍候...</span>
              </div>
              <!-- 公共步骤进度条（伪进度动画，不显示步骤名） -->
              <StepProgressBar :steps="STEPS" :current-index="stepIndex" :show-steps="false" />
            </div>

            <!-- 错误 -->
            <div v-else-if="authStore.msLoginStatus === 'error'" class="space-y-4">
              <div class="rounded-lg bg-red-50 p-4"><p class="text-sm text-red-700">{{ authStore.error }}</p></div>
              <div class="flex gap-3">
                <Button type="primary" class="flex-1" @click="handleRetry">重新登录</Button>
                <Button type="secondary" @click="handleCancel">取消</Button>
              </div>
            </div>

            <!-- 兜底加载 -->
            <div v-else class="flex items-center justify-center py-8">
              <div class="h-8 w-8 animate-spin rounded-full border-[3px] border-primary-200 border-t-primary-500" />
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
