<script setup lang="ts">
/**
 * 首页
 */

import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useSdkStore } from '@/stores/sdk'
import { useVersionStore } from '@/stores/version'
import { showWarning, showError, showSuccess } from '@/utils/modal'
import * as tauri from '@/utils/tauri'

const router = useRouter()
const authStore = useAuthStore()
const sdkStore = useSdkStore()
const versionStore = useVersionStore()

onMounted(async () => {
  if (sdkStore.isReady) {
    await versionStore.fetchVersions()
    await versionStore.checkRunningGame()
  }
})

function goToLogin() {
  router.push('/login')
}

function goToVersions() {
  router.push('/versions')
}

// 启动按钮状态
const launchStatus = computed(() => {
  if (!authStore.isLoggedIn) return { icon: 'login', text: '登录后可启动', color: 'text-gray-400' }
  if (versionStore.launching) return { icon: 'loading', text: '取消启动', color: 'text-yellow-600' }
  if (versionStore.runningPid) return { icon: 'stop', text: '停止游戏', color: 'text-red-600' }
  return { icon: 'play', text: '启动游戏', color: 'text-green-600' }
})

async function handleLaunchGame() {
  // 未登录 - toast提示
  if (!authStore.isLoggedIn) {
    showWarning('提示', '请先登录后再启动游戏')
    return
  }

  // 游戏运行中 - 停止
  if (versionStore.runningPid) {
    try {
      await versionStore.stopGame()
      showSuccess('已停止', '游戏进程已终止')
    } catch (e) {
      showError('停止失败', String(e))
    }
    return
  }

  // 启动中 - 取消
  if (versionStore.launching) {
    try {
      await versionStore.cancelLaunch()
      showSuccess('已取消', '启动已取消')
    } catch (e) {
      showError('取消失败', String(e))
    }
    return
  }

  // 获取已安装版本列表
  try {
    const installed = await tauri.listInstalledVersions()
    if (!installed || installed.length === 0) {
      showWarning('提示', '没有已安装的版本，请先下载一个版本')
      return
    }

    // 使用第一个已安装版本
    const versionId = installed[0]

    // 启动游戏
    const pid = await versionStore.launchGame({
      versionId,
      username: authStore.currentUser?.name || 'Player',
      uuid: authStore.currentUser?.uuid || '',
      accessToken: authStore.currentUser?.access_token || '',
    })

    showSuccess('启动成功', `游戏已启动 (PID: ${pid})`)
  } catch (e) {
    showError('启动失败', String(e))
  }
}
</script>

<template>
  <div class="h-full overflow-y-auto">
    <div class="max-w-4xl mx-auto py-6 px-6">
      <!-- 欢迎区域 -->
      <div class="bg-white border border-gray-200 rounded-lg p-6 mb-6">
        <div class="flex items-center justify-between">
          <div>
            <h1 class="text-2xl font-bold text-gray-900">
              {{ authStore.isLoggedIn ? `欢迎回来，${authStore.username}` : '欢迎使用 MoLaunch' }}
            </h1>
            <p class="text-gray-600 mt-2">
              {{ authStore.isLoggedIn
                ? '准备开始你的 Minecraft 冒险之旅'
                : '现代化的 Minecraft 启动器，让游戏更简单'
              }}
            </p>
          </div>
          <button
            v-if="!authStore.isLoggedIn"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
            @click="goToLogin"
          >
            立即登录
          </button>
        </div>
      </div>

      <!-- 状态卡片 -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
        <!-- SDK 状态 -->
        <div class="bg-white border border-gray-200 rounded-lg p-5">
          <div class="flex items-center">
            <div
              class="w-10 h-10 rounded-lg flex items-center justify-center"
              :class="sdkStore.isReady ? 'bg-green-100' : 'bg-yellow-100'"
            >
              <svg
                class="w-6 h-6"
                :class="sdkStore.isReady ? 'text-green-600' : 'text-yellow-600'"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  v-if="sdkStore.isReady"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                />
                <path
                  v-else
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
            </div>
            <div class="ml-4">
              <p class="text-sm font-medium text-gray-600">SDK 状态</p>
              <p class="text-lg font-semibold text-gray-900">
                {{ sdkStore.isReady ? '就绪' : '加载中' }}
              </p>
            </div>
          </div>
        </div>

        <!-- 版本数量 -->
        <div class="bg-white border border-gray-200 rounded-lg p-5">
          <div class="flex items-center">
            <div class="w-10 h-10 rounded-lg bg-blue-100 flex items-center justify-center">
              <svg class="w-6 h-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
              </svg>
            </div>
            <div class="ml-4">
              <p class="text-sm font-medium text-gray-600">可用版本</p>
              <p class="text-lg font-semibold text-gray-900">
                {{ versionStore.versions.length }}
              </p>
            </div>
          </div>
        </div>

        <!-- 登录状态 -->
        <div class="bg-white border border-gray-200 rounded-lg p-5">
          <div class="flex items-center">
            <div
              class="w-10 h-10 rounded-lg flex items-center justify-center"
              :class="authStore.isLoggedIn ? 'bg-green-100' : 'bg-gray-100'"
            >
              <svg
                class="w-6 h-6"
                :class="authStore.isLoggedIn ? 'text-green-600' : 'text-gray-600'"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
              </svg>
            </div>
            <div class="ml-4">
              <p class="text-sm font-medium text-gray-600">登录状态</p>
              <p class="text-lg font-semibold text-gray-900">
                {{ authStore.isLoggedIn ? '已登录' : '未登录' }}
              </p>
            </div>
          </div>
        </div>
      </div>

      <!-- 设备信息 -->
      <div class="bg-white border border-gray-200 rounded-lg p-6 mb-6">
        <h2 class="text-lg font-semibold text-gray-900 mb-4">设备信息</h2>
        <div class="space-y-3">
          <div class="flex items-center justify-between py-2 border-b border-gray-100">
            <span class="text-gray-600">设备 ID</span>
            <div class="flex items-center">
              <code v-if="sdkStore.deviceId" class="px-3 py-1 bg-gray-100 rounded text-sm font-mono">
                {{ sdkStore.deviceId }}
              </code>
              <span v-else class="text-gray-400">未获取</span>
            </div>
          </div>
          <div class="flex items-center justify-between py-2 border-b border-gray-100">
            <span class="text-gray-600">平台</span>
            <span class="text-gray-900">{{ sdkStore.status?.platform || '未知' }}</span>
          </div>
        </div>
      </div>

      <!-- 快速操作 -->
      <div class="bg-white border border-gray-200 rounded-lg p-6 mb-6">
        <h2 class="text-lg font-semibold text-gray-900 mb-4">快速操作</h2>
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
          <button
            class="flex flex-col items-center p-4 rounded-lg border border-gray-200 hover:bg-gray-50 transition-colors"
            @click="goToVersions"
          >
            <svg class="w-8 h-8 text-primary-600 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
            </svg>
            <span class="text-sm font-medium text-gray-700">下载版本</span>
          </button>

          <button
            class="flex flex-col items-center p-4 rounded-lg border border-gray-200 hover:bg-gray-50 transition-colors relative"
            @click="handleLaunchGame"
          >
            <!-- 图标 -->
            <svg v-if="launchStatus.icon === 'loading'" class="w-8 h-8 text-yellow-600 mb-2 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            <svg v-else-if="launchStatus.icon === 'stop'" class="w-8 h-8 text-red-600 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />
            </svg>
            <svg v-else-if="launchStatus.icon === 'login'" class="w-8 h-8 text-gray-400 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1" />
            </svg>
            <svg v-else class="w-8 h-8 text-green-600 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <!-- 文字 -->
            <span class="text-sm font-medium" :class="launchStatus.color">
              {{ launchStatus.text }}
            </span>
            <!-- 启动进度 -->
            <div v-if="versionStore.launching && versionStore.launchProgress" class="absolute -bottom-10 left-0 right-0 text-center">
              <div class="text-xs text-gray-500">{{ versionStore.launchStageName }}</div>
              <div class="w-full bg-gray-200 rounded-full h-1.5 mt-1">
                <div class="bg-primary-600 h-1.5 rounded-full transition-all" :style="{ width: `${versionStore.launchProgress.overall_progress * 100}%` }"></div>
              </div>
            </div>
          </button>

          <button
            class="flex flex-col items-center p-4 rounded-lg border border-gray-200 hover:bg-gray-50 transition-colors"
          >
            <svg class="w-8 h-8 text-purple-600 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
            <span class="text-sm font-medium text-gray-700">皮肤管理</span>
          </button>

          <button
            class="flex flex-col items-center p-4 rounded-lg border border-gray-200 hover:bg-gray-50 transition-colors"
          >
            <svg class="w-8 h-8 text-orange-600 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
            </svg>
            <span class="text-sm font-medium text-gray-700">Mod 管理</span>
          </button>
        </div>
      </div>

      <!-- 最新版本预览 -->
      <div v-if="versionStore.versions.length > 0" class="bg-white border border-gray-200 rounded-lg p-6">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-lg font-semibold text-gray-900">最新版本</h2>
          <button
            class="text-sm text-primary-600 hover:text-primary-700"
            @click="goToVersions"
          >
            查看全部
          </button>
        </div>
        <div class="space-y-2">
          <div
            v-for="version in versionStore.versions.slice(0, 5)"
            :key="version.id"
            class="flex items-center justify-between p-3 rounded-lg hover:bg-gray-50 transition-colors"
          >
            <div>
              <span class="font-medium text-gray-900">
                {{ version.id }}
              </span>
              <span
                class="ml-2 text-xs px-2 py-0.5 rounded-full"
                :class="{
                  'bg-green-100 text-green-800': version.version_type === 'release',
                  'bg-yellow-100 text-yellow-800': version.version_type === 'snapshot',
                  'bg-gray-100 text-gray-800': version.version_type === 'old_beta' || version.version_type === 'old_alpha',
                }"
              >
                {{ version.version_type === 'release' ? '正式版' : version.version_type === 'snapshot' ? '快照版' : '旧版本' }}
              </span>
            </div>
            <span class="text-sm text-gray-500">
              {{ new Date(version.release_time * 1000).toLocaleString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' }) }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
