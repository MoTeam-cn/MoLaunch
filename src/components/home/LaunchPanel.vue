<script setup lang="ts">
/**
 * 启动面板（左侧栏）
 * - 顶部：账号类型胶囊指示器
 * - 中部：账号卡片（头像+用户名+hover工具栏）
 * - 底部：版本选择 + 版本设置 + 启动按钮
 */

import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { ShieldCheckIcon, ServerStackIcon, UserIcon } from '@heroicons/vue/24/outline'
import { ArrowPathIcon, PlayIcon, StopIcon } from '@heroicons/vue/24/solid'
import { useAuthStore } from '@/stores/auth'
import { useVersionStore } from '@/stores/version'
import { toastError, toastWarning, toastSuccess, toastInfo } from '@/utils/toast'
import AccountSelector from './AccountSelector.vue'
import VersionSelector from './VersionSelector.vue'

const authStore = useAuthStore()
const versionStore = useVersionStore()
const router = useRouter()

/**
 * 顶部胶囊显示信息
 *
 * 优先级：
 * 1. 游戏运行中：绿色背景 + 脉冲圆点 + "运行中 · 版本名"，复用已有胶囊空间，不挤压账号区
 * 2. 账号类型（三分流：Microsoft / AuthlibInjector / Legacy）
 *   - Microsoft 正版账号：ShieldCheckIcon，蓝色，"正版账号"
 *   - AuthlibInjector 外置账号：ServerStackIcon，紫色，"外置账号"
 *   - Legacy 离线账号：UserIcon，灰色，"离线账号"
 *   - 未登录：UserIcon，灰色，"未登录"
 */
const topPillMeta = computed(() => {
  // 游戏运行中：用绿色胶囊替代账号类型，复用空间避免挤压账号卡片区
  if (versionStore.runningPid) {
    const versionId = versionStore.runningVersionId || versionStore.selectedVersion || ''
    return {
      isRunning: true,
      label: versionId ? `运行中 · ${versionId}` : '运行中',
      icon: null,
      bgClass: 'bg-green-50',
      textClass: 'text-green-700',
    }
  }
  const t = authStore.currentUser?.login_type
  if (authStore.isLoggedIn && t === 'Microsoft') {
    return { isRunning: false, label: '正版账号', icon: ShieldCheckIcon, bgClass: 'bg-primary-50/60', textClass: 'text-primary-700', iconClass: 'text-primary-600' }
  }
  if (authStore.isLoggedIn && t === 'AuthlibInjector') {
    return { isRunning: false, label: '外置账号', icon: ServerStackIcon, bgClass: 'bg-primary-50/60', textClass: 'text-primary-700', iconClass: 'text-purple-600' }
  }
  if (authStore.isLoggedIn) {
    return { isRunning: false, label: '离线账号', icon: UserIcon, bgClass: 'bg-primary-50/60', textClass: 'text-primary-700', iconClass: 'text-gray-400' }
  }
  return { isRunning: false, label: '未登录', icon: UserIcon, bgClass: 'bg-primary-50/60', textClass: 'text-gray-500', iconClass: 'text-gray-400' }
})

// 启动按钮状态：白底 + 主题色细边框 + 主题色文字，文字色=边框色
const launchState = computed(() => {
  if (!authStore.isLoggedIn) return { text: '登录后可启动', color: 'border border-gray-300 bg-white/80 text-gray-400 cursor-not-allowed', spin: false }
  if (versionStore.launching) return { text: '取消启动', color: 'border border-yellow-500 bg-white/80 text-yellow-600 hover:bg-yellow-50 hover:border-yellow-600', spin: true }
  if (versionStore.runningPid) return { text: '停止游戏', color: 'border border-red-500 bg-white/80 text-red-600 hover:bg-red-50 hover:border-red-600', spin: false }
  if (!versionStore.selectedVersion) return { text: '选择版本', color: 'border border-gray-300 bg-white/80 text-gray-400 cursor-not-allowed', spin: false }
  // Highlight：主题色边框 + 白底 + 主题色文字，hover 时变亮档（primary-500）+ 极淡底
  return { text: '启动游戏', color: 'border border-primary-600 bg-white/80 text-primary-600 hover:bg-primary-50 hover:border-primary-500 hover:text-primary-500', spin: false }
})

async function handleLaunch() {
  if (versionStore.launching) { versionStore.cancelLaunch(); toastInfo('已取消启动'); return }
  if (versionStore.runningPid) { versionStore.stopGame(); toastInfo('已停止游戏'); return }
  if (!authStore.isLoggedIn) { toastWarning('请先登录'); return }
  if (!versionStore.selectedVersion) { toastWarning('请选择版本'); return }

  const user = authStore.currentUser!
  try {
    await versionStore.launchGame({
      versionId: versionStore.selectedVersion,
      username: user.name,
      uuid: user.uuid,
      loginType: user.login_type,
    })
    toastSuccess('游戏已启动')
  } catch (e) {
    toastError('启动失败：' + String(e))
  }
}
</script>

<template>
  <div class="flex h-full flex-col">
    <!-- 顶部：账号类型 / 运行中状态胶囊指示器（复用同一空间，运行中时替换为绿色运行状态） -->
    <div class="flex justify-center px-4 pt-4">
      <div
        class="flex max-w-full items-center gap-1.5 rounded-full px-3 py-1"
        :class="topPillMeta.bgClass"
      >
        <!-- 运行中：脉冲圆点 -->
        <span
          v-if="topPillMeta.isRunning"
          class="h-2 w-2 flex-none animate-pulse rounded-full bg-green-500"
        />
        <!-- 账号类型图标 -->
        <component
          :is="topPillMeta.icon"
          v-else-if="topPillMeta.icon"
          class="h-3.5 w-3.5 flex-none"
          :class="topPillMeta.iconClass"
        />
        <span
          class="truncate text-xs font-medium"
          :class="topPillMeta.textClass"
        >
          {{ topPillMeta.label }}
        </span>
      </div>
    </div>

    <!-- 中部：账号卡片区域（弹性填充，垂直居中） -->
    <div data-inner-scroll class="flex flex-1 items-center justify-center overflow-y-auto px-4 py-2">
      <AccountSelector />
    </div>

    <!-- 底部：版本选择 + 版本设置 + 启动按钮 -->
    <div class="flex-none px-5 pb-5 pt-2">
      <!-- Row 3：版本选择 + 版本设置（左右分栏，高 35px、圆角 3px） -->
      <div class="mb-2.5 flex gap-2">
        <VersionSelector />
        <!-- 保留原生 button：自定义尺寸版本设置按钮（h-[35px] w-[80px]），
             Button.vue 的 scoped size 类固定 height 无法被工具类覆盖 -->
        <button
          class="flex h-[35px] w-[80px] flex-none items-center justify-center rounded-[3px] border border-gray-300 bg-white/80 text-[13px] text-gray-600 transition-colors hover:border-primary-500 hover:text-primary-600 hover:bg-primary-50"
          @click="router.push({ path: '/apps/versions/setup', query: versionStore.selectedVersion ? { id: versionStore.selectedVersion } : {} })"
        >
          版本设置
        </button>
      </div>

      <!-- Row 2：启动按钮 + 版本名（LabVersion 叠在按钮内部底部，高 54px、圆角 3px） -->
      <!-- 保留原生 button：启动按钮为特殊布局（h-[54px] + 内部偏移文字 + 版本名绝对定位），
           Button.vue 的 scoped size 类与布局无法承载 -->
      <button
        class="relative flex h-[54px] w-full flex-col items-center justify-center overflow-hidden rounded-[3px] text-[13px] font-normal transition-colors"
        :class="launchState.color"
        :disabled="!authStore.isLoggedIn && !versionStore.selectedVersion"
        @click="handleLaunch"
      >
        <!-- 主文字区（向上偏移以给底部 LabVersion 留出 13px 间距） -->
        <span class="flex items-center gap-2" style="margin-top: -18px;">
          <ArrowPathIcon v-if="launchState.spin" class="h-5 w-5 animate-spin" />
          <StopIcon v-else-if="versionStore.runningPid" class="h-5 w-5" />
          <PlayIcon v-else class="h-5 w-5" />
          <span>{{ launchState.text }}</span>
        </span>
        <!-- 当前版本名（按钮内部底部，11px、灰色、底部 10px） -->
        <span
          v-if="versionStore.selectedVersion"
          class="pointer-events-none absolute bottom-2 left-1/2 max-w-[calc(100%-40px)] -translate-x-1/2 truncate text-center text-[11px] text-gray-400"
        >{{ versionStore.selectedVersion }}</span>
      </button>
    </div>
  </div>
</template>
