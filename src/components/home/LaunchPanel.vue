<script setup lang="ts">
/**
 * 启动面板（左侧栏，参考 PCL2）
 * - 顶部：账号类型胶囊指示器
 * - 中部：账号卡片（头像+用户名+hover工具栏）
 * - 底部：版本选择器 + 启动按钮
 */

import { computed } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { useVersionStore } from '@/stores/version'
import { showWarning } from '@/utils/toast'
import AccountSelector from './AccountSelector.vue'
import VersionSelector from './VersionSelector.vue'

const authStore = useAuthStore()
const versionStore = useVersionStore()

// 账号类型标签
const accountTypeLabel = computed(() => {
  if (!authStore.isLoggedIn) return '未登录'
  return authStore.currentUser?.login_type === 'Microsoft' ? '正版账号' : '离线账号'
})

// 启动按钮状态
const launchState = computed(() => {
  if (!authStore.isLoggedIn) return { text: '登录后可启动', color: 'bg-gray-100 text-gray-400 cursor-not-allowed', spin: false }
  if (versionStore.launching) return { text: '取消启动', color: 'bg-yellow-500 text-white hover:bg-yellow-600', spin: true }
  if (versionStore.runningPid) return { text: '停止游戏', color: 'bg-red-500 text-white hover:bg-red-600', spin: false }
  if (!versionStore.selectedVersion) return { text: '选择版本', color: 'bg-gray-100 text-gray-400 cursor-not-allowed', spin: false }
  return { text: '启动游戏', color: 'bg-primary-600 text-white hover:bg-primary-700 shadow-sm', spin: false }
})

const progressPercent = computed(() => {
  if (!versionStore.launchProgress) return 0
  return Math.round(versionStore.launchProgress.overall_progress)
})

async function handleLaunch() {
  if (versionStore.launching) { versionStore.cancelLaunch(); return }
  if (versionStore.runningPid) { versionStore.stopGame(); return }
  if (!authStore.isLoggedIn) { showWarning('请先登录'); return }
  if (!versionStore.selectedVersion) { showWarning('请选择版本'); return }

  const user = authStore.currentUser!
  await versionStore.launchGame({
    versionId: versionStore.selectedVersion,
    username: user.name,
    uuid: user.uuid,
    accessToken: user.access_token,
    loginType: user.login_type,
  })
}
</script>

<template>
  <div class="flex h-full flex-col">
    <!-- 顶部：账号类型胶囊指示器（参考 PCL2 PanTypeOne） -->
    <div class="flex justify-center px-4 pt-4">
      <div class="flex items-center gap-1.5 rounded-full bg-primary-50/60 px-3 py-1">
        <!-- 盾牌图标 -->
        <svg v-if="authStore.isLoggedIn && authStore.currentUser?.login_type === 'Microsoft'" class="h-3.5 w-3.5 text-primary-600" viewBox="0 0 20 20" fill="currentColor">
          <path d="M10 1l7 3v6c0 4.5-3 8-7 9-4-1-7-4.5-7-9V4l7-3z" />
        </svg>
        <!-- 离线图标 -->
        <svg v-else class="h-3.5 w-3.5 text-gray-400" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M12.6 4.4a1 1 0 011.4 1.4L7.4 12.4a1 1 0 01-1.4-1.4l6.6-6.6zM4 10a6 6 0 0111.2-3 1 1 0 11-1.8.9 4 4 0 00-6.4 4.8l1.4-1.4a1 1 0 011.4 1.4l-3 3a1 1 0 01-1.4 0l-.5-.5A6 6 0 014 10z" clip-rule="evenodd" />
        </svg>
        <span class="text-xs font-medium" :class="authStore.isLoggedIn ? 'text-primary-700' : 'text-gray-500'">
          {{ accountTypeLabel }}
        </span>
      </div>
    </div>

    <!-- 中部：账号卡片区域（弹性填充，垂直居中） -->
    <div class="flex flex-1 items-center justify-center overflow-y-auto px-4 py-2">
      <AccountSelector />
    </div>

    <!-- 启动进度（仅启动中显示，浮在版本选择上方） -->
    <div v-if="versionStore.launching && versionStore.launchProgress" class="px-4 pb-2">
      <div class="h-1.5 overflow-hidden rounded-full bg-gray-100">
        <div class="h-full rounded-full bg-primary-500 transition-all duration-300" :style="{ width: progressPercent + '%' }" />
      </div>
      <div class="mt-1 flex items-center justify-between text-xs">
        <span class="text-gray-500">{{ versionStore.launchStageName }}</span>
        <span class="font-medium text-primary-600">{{ progressPercent }}%</span>
      </div>
    </div>

    <!-- 运行中提示 -->
    <div v-else-if="versionStore.runningPid" class="mx-4 mb-2 flex items-center gap-2 rounded-lg bg-green-50 px-3 py-2">
      <div class="h-2 w-2 animate-pulse rounded-full bg-green-500" />
      <span class="text-xs font-medium text-green-700">游戏运行中</span>
      <span class="ml-auto truncate text-xs text-green-600">{{ versionStore.runningVersionId }}</span>
    </div>

    <!-- 版本选择器 -->
    <div class="border-t border-gray-100 px-4 py-3">
      <div class="mb-1.5 px-1 text-xs font-medium text-gray-400">游戏版本</div>
      <VersionSelector />
    </div>

    <!-- 启动按钮（底部） -->
    <div class="px-4 pb-4">
      <button
        class="flex w-full items-center justify-center gap-2 rounded-xl py-3.5 text-sm font-semibold transition-all"
        :class="launchState.color"
        :disabled="!authStore.isLoggedIn && !versionStore.selectedVersion"
        @click="handleLaunch"
      >
        <svg v-if="launchState.spin" class="h-5 w-5 animate-spin" viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
          <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
        </svg>
        <svg v-else-if="versionStore.runningPid" class="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
          <rect x="6" y="6" width="12" height="12" rx="2" />
        </svg>
        <svg v-else class="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M8 5v14l11-7z" />
        </svg>
        {{ launchState.text }}
      </button>
    </div>
  </div>
</template>
