<script setup lang="ts">
/**
 * 联机主页（阶段一骨架）
 *
 * 单列布局，根据设备认证状态显示不同内容：
 * - 未注册：注册引导卡片
 * - 已注册未登录：登录卡片 + 设备信息
 * - 已登录：房间创建/加入入口（阶段二占位「功能开发中」）
 *
 * 阶段二会替换「功能开发中」为房间管理界面（房主/加入方）。
 * 阶段三会接入 MC 版本检测与启动流程。
 */

import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useOnlineStore } from '@/stores/online'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import {
  UserPlusIcon,
  ArrowRightOnRectangleIcon,
  UserCircleIcon,
  ServerStackIcon,
  Cog6ToothIcon,
  GlobeAltIcon,
  ClockIcon,
  KeyIcon,
} from '@heroicons/vue/24/outline'
import { formatTimestamp } from '@/utils/format'

const router = useRouter()
const onlineStore = useOnlineStore()

/** 设备状态（null 表示未拉取） */
const status = computed(() => onlineStore.deviceStatus)
/** 是否未注册（含未查询的情况） */
const isUnregistered = computed(() => !status.value || !status.value.registered)
/** 是否已注册但未登录或 JWT 过期 */
const needLogin = computed(
  () => !!status.value && status.value.registered && (!status.value.logged_in || status.value.token_expired),
)
/** 是否已就绪（已注册且 JWT 有效） */
const isReady = computed(
  () => !!status.value && status.value.registered && status.value.logged_in && !status.value.token_expired,
)

onMounted(() => {
  void onlineStore.refreshStatus()
})

function goSettings() {
  router.push('/apps/settings?tab=online')
}

async function handleRegister() {
  await onlineStore.register()
}

async function handleLogin() {
  await onlineStore.login()
}
</script>

<template>
  <div class="h-full overflow-y-auto">
    <div class="mx-auto max-w-3xl px-6 py-6 space-y-6">
      <!-- 顶部标题 + 状态徽章 -->
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-xl font-semibold text-gray-900">联机</h1>
          <p class="text-xs text-gray-500 mt-1">通过 P2P 与好友一起游玩 Minecraft</p>
        </div>
        <div class="flex items-center gap-2">
          <span
            class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium"
            :class="isReady
              ? 'bg-green-50 text-green-700'
              : isUnregistered
                ? 'bg-gray-100 text-gray-600'
                : 'bg-yellow-50 text-yellow-700'"
          >
            <span
class="w-1.5 h-1.5 rounded-full mr-1.5"
              :class="isReady ? 'bg-green-500' : isUnregistered ? 'bg-gray-400' : 'bg-yellow-500'" />
            {{ isReady ? '已就绪' : isUnregistered ? '未注册' : '需登录' }}
          </span>
          <Tooltip text="联机设置">
            <Button type="ghost" size="small" @click="goSettings">
              <template #icon><Cog6ToothIcon class="w-4 h-4" /></template>
            </Button>
          </Tooltip>
        </div>
      </div>

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

      <!-- 未注册：注册引导 -->
      <Card v-else-if="isUnregistered" title="注册设备">
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
      </Card>

      <!-- 已注册未登录：登录卡片 -->
      <Card v-else-if="needLogin" title="设备登录">
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
      </Card>

      <!-- 已就绪：房间入口（阶段二占位） -->
      <Card v-else-if="isReady" title="房间管理">
        <div class="py-8 flex flex-col items-center text-center">
          <div class="mb-4 flex h-14 w-14 items-center justify-center rounded-2xl bg-gray-100">
            <ServerStackIcon class="h-7 w-7 text-gray-400" />
          </div>
          <div class="mb-2 text-sm font-semibold text-gray-700">功能开发中</div>
          <p class="text-xs text-gray-500 max-w-md">
            房间创建、加入、WebRTC 连接、虚拟网卡等功能将在阶段二实现。
            当前阶段仅完成设备认证骨架。
          </p>
        </div>
      </Card>

      <!-- 设备信息（已注册时显示） -->
      <Card v-if="status?.registered" title="设备信息">
        <div class="divide-y divide-gray-100">
          <div class="px-1 py-3 flex items-center justify-between">
            <div class="flex items-center gap-2 text-sm text-gray-600">
              <UserCircleIcon class="w-4 h-4 text-gray-400" />
              <span>设备 ID</span>
            </div>
            <code class="text-xs text-gray-900 bg-gray-50 px-2 py-0.5 rounded">{{ status.device_id || '-' }}</code>
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
      </Card>
    </div>
  </div>
</template>
