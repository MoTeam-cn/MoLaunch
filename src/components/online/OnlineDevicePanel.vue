<script setup lang="ts">
/**
 * 联机 - 设备面板（Scaffolding 收敛版）
 *
 * 根据设备认证状态显示：
 * - 未注册：注册引导卡片
 * - 已注册未登录：登录卡片（JWT 过期不触发登录卡片，由后端自动续期兜底）
 * - 已注册：网络环境卡片（NAT 类型检测）+ 设备信息卡片（设备 ID / api-server / 最后登录 / JWT 过期时间）
 *
 * 状态来源：useOnlineStore.deviceStatus（由 Online.vue 在 onMounted 时刷新）
 *           useOnlineStore.natResult（由 Online.vue 进入页面时自动检测，侧边栏切换不丢失）
 */

import { computed, defineAsyncComponent } from 'vue'
import { useOnlineStore } from '@/stores/online'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Card = defineAsyncComponent(() => import('@/components/common/Card.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
const SealedOverlay = defineAsyncComponent(() => import('@/components/common/SealedOverlay.vue'))
import {
  UserPlusIcon,
  ArrowRightOnRectangleIcon,
  UserCircleIcon,
  GlobeAltIcon,
  ClockIcon,
  KeyIcon,
  SignalIcon,
  ArrowPathIcon,
  SignalSlashIcon,
} from '@heroicons/vue/24/outline'
import { formatTimestamp } from '@/utils/format'
import { NAT_TYPE_META, getNatFeasibilityColorClass } from '@/utils/online/nat'
import { stripMcsdkPrefix } from '@/utils/online/device-id'
import { toastSuccess, toastError } from '@/utils/toast'
import { showWarning } from '@/utils/modal'

const onlineStore = useOnlineStore()

/** 云端离线：注册/登录/设备信息均依赖云端，显示封条遮罩；NAT 检测（第三方 STUN）仍可用 */
const offline = computed(() => !onlineStore.cloudConnected && !onlineStore.initializing)

/** 点击封条弹窗告知原因（封存的云端功能因连接失败暂不可用） */
function showSealedReason(label: string) {
  showWarning(
    '功能已封存',
    `「${label}」需要连接云端服务，当前云端连接失败，暂不可用。`,
    onlineStore.cloudError ?? undefined,
  )
}

const status = computed(() => onlineStore.deviceStatus)
const isUnregistered = computed(() => !status.value || !status.value.registered)
/**
 * 是否需要登录卡片：仅「已注册但未登录」时显示。
 *
 * 不判断 token_expired：JWT 过期由后端 `load_creds_with_auto_refresh` 自动
 * refresh 续期 + 前端 onlineManager 1003 降级链兜底，只有静默续期全部失败
 * （业务请求持续 1003）才需要用户手动重新登录，届时各调用方 toast 提示。
 * 本地判断 token 过期就拦截页面会误伤——过期瞬间自动续期即可恢复。
 */
const needLogin = computed(
  () => !!status.value && status.value.registered && !status.value.logged_in,
)

async function handleRegister() {
  await onlineStore.register()
}

async function handleLogin() {
  await onlineStore.login()
}

// ============ NAT 类型检测 ============
// 复用 store 的 natDetecting 状态（与 Online.vue 自动检测共享，避免重复触发）
const detectingNat = computed(() => onlineStore.natDetecting)

async function handleDetectNat() {
  try {
    await onlineStore.forceDetectNat()
    if (onlineStore.natResult?.error) {
      toastError('NAT 检测失败，请检查网络后重试')
    } else {
      toastSuccess('NAT 检测完成')
    }
  } catch {
    toastError('NAT 检测失败，请检查网络后重试')
  }
}
</script>

<template>
  <div class="space-y-4">
    <!-- 加载占位 -->
    <div v-if="onlineStore.refreshing && !status" class="bg-white rounded-lg border border-gray-300 p-8">
      <div class="flex items-center justify-center gap-3 text-gray-400">
        <svg class="h-5 w-5 animate-spin" viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
          <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
        </svg>
        <span class="text-sm">正在加载设备状态...</span>
      </div>
    </div>

    <!-- 未注册：注册引导（云端离线时封存） -->
    <Card v-else-if="isUnregistered" title="注册设备">
      <div class="relative">
        <SealedOverlay
          v-if="offline"
          :reason="onlineStore.cloudError || ''"
          @request="showSealedReason('注册设备')"
        />
        <div class="py-6 flex flex-col items-center text-center">
          <div class="mb-4 flex h-14 w-14 items-center justify-center rounded-2xl bg-primary-50">
            <UserPlusIcon class="h-7 w-7 text-primary-600" />
          </div>
          <div class="mb-2 text-sm font-semibold text-gray-700">注册联机设备</div>
          <p class="mb-5 text-xs text-gray-500 max-w-md">
            注册将为你的设备生成唯一的密钥对，用于联机服务的身份验证。
            密钥保存在本地，不会上传到云端。
          </p>
          <Button type="primary" :loading="onlineStore.loading" @click="handleRegister">
            <template #icon><UserPlusIcon class="w-4 h-4" /></template>
            立即注册
          </Button>
        </div>
      </div>
    </Card>

    <!-- 已注册未登录：登录卡片（云端离线时封存） -->
    <Card v-else-if="needLogin" title="设备登录">
      <div class="relative">
        <SealedOverlay
          v-if="offline"
          :reason="onlineStore.cloudError || ''"
          @request="showSealedReason('设备登录')"
        />
        <div class="py-6 flex flex-col items-center text-center">
          <div class="mb-4 flex h-14 w-14 items-center justify-center rounded-2xl bg-yellow-50">
            <ArrowRightOnRectangleIcon class="h-7 w-7 text-yellow-600" />
          </div>
          <div class="mb-2 text-sm font-semibold text-gray-700">
            {{ status?.token_expired ? '登录已过期' : '设备未登录' }}
          </div>
          <p class="mb-5 text-xs text-gray-500 max-w-md">
            {{ status?.token_expired
              ? 'JWT 已过期，需要重新登录以继续使用联机功能。'
              : '设备已注册但未登录，点击下方按钮登录以获取访问凭证。' }}
          </p>
          <Button type="primary" :loading="onlineStore.loading" @click="handleLogin">
            <template #icon><ArrowRightOnRectangleIcon class="w-4 h-4" /></template>
            登录设备
          </Button>
        </div>
      </div>
    </Card>

    <!-- 网络环境（NAT 检测，已注册时显示；NAT 检测不依赖云端，离线时仍可用） -->
    <Card v-if="status?.registered" title="网络环境">
      <div class="flex items-center justify-between py-1">
        <div class="flex items-center gap-2 text-sm text-gray-600">
          <SignalIcon class="w-4 h-4 text-gray-400" />
          <span>NAT 类型</span>
        </div>
        <div class="flex items-center gap-2">
          <Tooltip
            v-if="onlineStore.natResult"
            :text="NAT_TYPE_META[onlineStore.natResult.type].tooltip"
            position="left"
          >
            <span
              class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium cursor-help"
              :class="getNatFeasibilityColorClass(NAT_TYPE_META[onlineStore.natResult.type].feasibility)"
            >
              {{ NAT_TYPE_META[onlineStore.natResult.type].label }}
            </span>
          </Tooltip>
          <span v-else-if="detectingNat" class="text-xs text-gray-400">检测中...</span>
          <span v-else class="text-xs text-gray-400">未检测</span>
          <Tooltip text="重新检测 NAT 类型">
            <Button type="ghost" size="mini" :loading="detectingNat" @click="handleDetectNat">
              <template #icon><ArrowPathIcon class="w-3.5 h-3.5" /></template>
            </Button>
          </Tooltip>
        </div>
      </div>
      <div v-if="onlineStore.natResult?.error" class="mt-2 text-xs text-red-500">
        <SignalSlashIcon class="w-3.5 h-3.5 inline mr-1" />{{ onlineStore.natResult.error }}
      </div>
    </Card>

    <!-- 设备信息（已注册时显示；云端离线时封存） -->
    <Card v-if="status?.registered" title="设备信息">
      <div class="relative">
        <SealedOverlay
          v-if="offline"
          :reason="onlineStore.cloudError || ''"
          @request="showSealedReason('设备信息')"
        />
        <div class="divide-y divide-gray-100">
          <div class="px-1 py-3 flex items-center justify-between">
            <div class="flex items-center gap-2 text-sm text-gray-600">
              <UserCircleIcon class="w-4 h-4 text-gray-400" />
              <span>设备 ID</span>
            </div>
            <code class="text-xs text-gray-900 bg-gray-50 px-2 py-0.5 rounded">{{ status.device_id ? stripMcsdkPrefix(status.device_id) : '-' }}</code>
          </div>
          <div class="px-1 py-3 flex items-center justify-between">
            <div class="flex items-center gap-2 text-sm text-gray-600">
              <GlobeAltIcon class="w-4 h-4 text-gray-400" />
              <span>api-server</span>
            </div>
            <code class="text-xs text-gray-900 bg-gray-50 px-2 py-0.5 rounded max-w-[260px] truncate">
              {{ status.api_server_url || '-' }}
            </code>
          </div>
          <div class="px-1 py-3 flex items-center justify-between">
            <div class="flex items-center gap-2 text-sm text-gray-600">
              <ClockIcon class="w-4 h-4 text-gray-400" />
              <span>最后登录</span>
            </div>
            <span class="text-xs text-gray-900">
              {{ status.last_login_at ? formatTimestamp(status.last_login_at) : '从未登录' }}
            </span>
          </div>
          <div class="px-1 py-3 flex items-center justify-between">
            <div class="flex items-center gap-2 text-sm text-gray-600">
              <KeyIcon class="w-4 h-4 text-gray-400" />
              <span>JWT 过期时间</span>
            </div>
            <span class="text-xs" :class="status.token_expired ? 'text-red-600' : 'text-gray-900'">
              {{ status.token_expires_at ? formatTimestamp(status.token_expires_at) : '-' }}
            </span>
          </div>
        </div>
      </div>
    </Card>
  </div>
</template>
