<script setup lang="ts">
/**
 * 启动面板（左侧栏，参考 PCL2 PageLaunchLeft）
 * - 顶部：账号类型胶囊指示器
 * - 中部：账号卡片（头像+用户名+hover工具栏）
 * - 底部：版本选择 + 版本设置 + 启动按钮
 */

import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useVersionStore } from '@/stores/version'
import { showError, showWarning } from '@/utils/toast'
import AccountSelector from './AccountSelector.vue'
import VersionSelector from './VersionSelector.vue'

const authStore = useAuthStore()
const versionStore = useVersionStore()
const router = useRouter()

// 账号类型标签
const accountTypeLabel = computed(() => {
  if (!authStore.isLoggedIn) return '未登录'
  return authStore.currentUser?.login_type === 'Microsoft' ? '正版账号' : '离线账号'
})

// 启动按钮状态（参考 PCL2 MyButton：白底 + 主题色细边框 + 主题色文字，文字色=边框色）
const launchState = computed(() => {
  if (!authStore.isLoggedIn) return { text: '登录后可启动', color: 'border border-gray-300 bg-white/80 text-gray-400 cursor-not-allowed', spin: false }
  if (versionStore.launching) return { text: '取消启动', color: 'border border-yellow-500 bg-white/80 text-yellow-600 hover:bg-yellow-50 hover:border-yellow-600', spin: true }
  if (versionStore.runningPid) return { text: '停止游戏', color: 'border border-red-500 bg-white/80 text-red-600 hover:bg-red-50 hover:border-red-600', spin: false }
  if (!versionStore.selectedVersion) return { text: '选择版本', color: 'border border-gray-300 bg-white/80 text-gray-400 cursor-not-allowed', spin: false }
  // Highlight：主题色边框 + 白底 + 主题色文字，hover 时边框变亮蓝 + 极淡蓝底（参考 PCL2 ColorBrush3 + ColorBrush7）
  return { text: '启动游戏', color: 'border border-primary-600 bg-white/80 text-primary-600 hover:bg-primary-50 hover:border-blue-500 hover:text-blue-500', spin: false }
})

async function handleLaunch() {
  if (versionStore.launching) { versionStore.cancelLaunch(); return }
  if (versionStore.runningPid) { versionStore.stopGame(); return }
  if (!authStore.isLoggedIn) { showWarning('请先登录'); return }
  if (!versionStore.selectedVersion) { showWarning('请选择版本'); return }

  const user = authStore.currentUser!
  try {
    await versionStore.launchGame({
      versionId: versionStore.selectedVersion,
      username: user.name,
      uuid: user.uuid,
      accessToken: user.access_token,
      loginType: user.login_type,
    })
  } catch (e) {
    showError('启动失败', String(e))
  }
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

    <!-- 运行中提示 -->
    <div v-if="versionStore.runningPid" class="mx-4 mb-2 flex items-center gap-2 rounded-lg bg-green-50 px-3 py-2">
      <div class="h-2 w-2 animate-pulse rounded-full bg-green-500" />
      <span class="text-xs font-medium text-green-700">游戏运行中</span>
      <span class="ml-auto truncate text-xs text-green-600">{{ versionStore.runningVersionId }}</span>
    </div>

    <!-- 底部：版本选择 + 版本设置 + 启动按钮（参考 PCL2 PageLaunchLeft） -->
    <div class="flex-none px-5 pb-5 pt-2">
      <!-- Row 3：版本选择 + 版本设置（左右分栏，参考 PCL2 BtnVersion + BtnMore，高 35px、圆角 3px） -->
      <div class="mb-2.5 flex gap-2">
        <VersionSelector />
        <button
          class="flex h-[35px] w-[80px] flex-none items-center justify-center rounded-[3px] border border-gray-300 bg-white/80 text-[13px] text-gray-600 transition-colors hover:border-primary-500 hover:text-primary-600 hover:bg-primary-50"
          @click="router.push({ path: '/apps/versions/setup', query: versionStore.selectedVersion ? { id: versionStore.selectedVersion } : {} })"
        >
          版本设置
        </button>
      </div>

      <!-- Row 2：启动按钮 + 版本名（LabVersion 叠在按钮内部底部，参考 PCL2 BtnLaunch + LabVersion，高 54px、圆角 3px） -->
      <button
        class="relative flex h-[54px] w-full flex-col items-center justify-center overflow-hidden rounded-[3px] text-[13px] font-normal transition-colors"
        :class="launchState.color"
        :disabled="!authStore.isLoggedIn && !versionStore.selectedVersion"
        @click="handleLaunch"
      >
        <!-- 主文字区（向上偏移以给底部 LabVersion 留出 13px 间距） -->
        <span class="flex items-center gap-2" style="margin-top: -18px;">
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
          <span>{{ launchState.text }}</span>
        </span>
        <!-- 当前版本名（按钮内部底部，参考 PCL2 LabVersion：11px、灰色、底部 10px） -->
        <span
          v-if="versionStore.selectedVersion"
          class="pointer-events-none absolute bottom-2 left-1/2 max-w-[calc(100%-40px)] -translate-x-1/2 truncate text-center text-[11px] text-gray-400"
        >{{ versionStore.selectedVersion }}</span>
      </button>
    </div>
  </div>
</template>
