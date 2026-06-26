<script setup lang="ts">
/**
 * 首页
 */

import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useSdkStore } from '@/stores/sdk'
import { useVersionStore } from '@/stores/version'

const router = useRouter()
const authStore = useAuthStore()
const sdkStore = useSdkStore()
const versionStore = useVersionStore()

onMounted(async () => {
  // 如果 SDK 就绪，获取版本列表
  if (sdkStore.isReady) {
    await versionStore.fetchVersions()
  }
})

function goToLogin() {
  router.push('/login')
}

function goToVersions() {
  router.push('/versions')
}
</script>

<template>
  <div class="max-w-4xl mx-auto">
    <!-- 欢迎区域 -->
    <div class="card mb-6">
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
            {{ authStore.isLoggedIn ? `欢迎回来，${authStore.username}` : '欢迎使用 MoLaunch' }}
          </h1>
          <p class="text-gray-600 dark:text-gray-400 mt-2">
            {{ authStore.isLoggedIn 
              ? '准备开始你的 Minecraft 冒险之旅' 
              : '现代化的 Minecraft 启动器，让游戏更简单' 
            }}
          </p>
        </div>
        <button
          v-if="!authStore.isLoggedIn"
          @click="goToLogin"
          class="btn-primary"
        >
          立即登录
        </button>
      </div>
    </div>

    <!-- 状态卡片 -->
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
      <!-- SDK 状态 -->
      <div class="card">
        <div class="flex items-center">
          <div
            class="w-10 h-10 rounded-lg flex items-center justify-center"
            :class="sdkStore.isReady ? 'bg-green-100 dark:bg-green-900' : 'bg-yellow-100 dark:bg-yellow-900'"
          >
            <svg
              class="w-6 h-6"
              :class="sdkStore.isReady ? 'text-green-600 dark:text-green-400' : 'text-yellow-600 dark:text-yellow-400'"
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
            <p class="text-sm font-medium text-gray-600 dark:text-gray-400">SDK 状态</p>
            <p class="text-lg font-semibold text-gray-900 dark:text-gray-100">
              {{ sdkStore.isReady ? '就绪' : '加载中' }}
            </p>
          </div>
        </div>
      </div>

      <!-- 版本数量 -->
      <div class="card">
        <div class="flex items-center">
          <div class="w-10 h-10 rounded-lg bg-blue-100 dark:bg-blue-900 flex items-center justify-center">
            <svg class="w-6 h-6 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
            </svg>
          </div>
          <div class="ml-4">
            <p class="text-sm font-medium text-gray-600 dark:text-gray-400">可用版本</p>
            <p class="text-lg font-semibold text-gray-900 dark:text-gray-100">
              {{ versionStore.versions.length }}
            </p>
          </div>
        </div>
      </div>

      <!-- 登录状态 -->
      <div class="card">
        <div class="flex items-center">
          <div
            class="w-10 h-10 rounded-lg flex items-center justify-center"
            :class="authStore.isLoggedIn ? 'bg-green-100 dark:bg-green-900' : 'bg-gray-100 dark:bg-gray-700'"
          >
            <svg
              class="w-6 h-6"
              :class="authStore.isLoggedIn ? 'text-green-600 dark:text-green-400' : 'text-gray-600 dark:text-gray-400'"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
            </svg>
          </div>
          <div class="ml-4">
            <p class="text-sm font-medium text-gray-600 dark:text-gray-400">登录状态</p>
            <p class="text-lg font-semibold text-gray-900 dark:text-gray-100">
              {{ authStore.isLoggedIn ? '已登录' : '未登录' }}
            </p>
          </div>
        </div>
      </div>
    </div>

    <!-- 快速操作 -->
    <div class="card">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
        快速操作
      </h2>
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
        <button
          @click="goToVersions"
          class="flex flex-col items-center p-4 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
        >
          <svg class="w-8 h-8 text-primary-600 dark:text-primary-400 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
          </svg>
          <span class="text-sm font-medium text-gray-700 dark:text-gray-300">下载版本</span>
        </button>
        
        <button
          class="flex flex-col items-center p-4 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
          :disabled="!authStore.isLoggedIn"
        >
          <svg class="w-8 h-8 text-green-600 dark:text-green-400 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span class="text-sm font-medium text-gray-700 dark:text-gray-300">启动游戏</span>
        </button>
        
        <button
          class="flex flex-col items-center p-4 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
        >
          <svg class="w-8 h-8 text-purple-600 dark:text-purple-400 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
          </svg>
          <span class="text-sm font-medium text-gray-700 dark:text-gray-300">皮肤管理</span>
        </button>
        
        <button
          class="flex flex-col items-center p-4 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
        >
          <svg class="w-8 h-8 text-orange-600 dark:text-orange-400 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
          </svg>
          <span class="text-sm font-medium text-gray-700 dark:text-gray-300">Mod 管理</span>
        </button>
      </div>
    </div>

    <!-- 版本列表预览 -->
    <div v-if="versionStore.versions.length > 0" class="card mt-6">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
          最新版本
        </h2>
        <button
          @click="goToVersions"
          class="text-sm text-primary-600 hover:text-primary-700 dark:text-primary-400 dark:hover:text-primary-300"
        >
          查看全部
        </button>
      </div>
      <div class="space-y-2">
        <div
          v-for="version in versionStore.versions.slice(0, 5)"
          :key="version.id"
          class="flex items-center justify-between p-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
        >
          <div>
            <span class="font-medium text-gray-900 dark:text-gray-100">
              {{ version.id }}
            </span>
            <span
              class="ml-2 text-xs px-2 py-0.5 rounded-full"
              :class="{
                'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200': version.version_type === 'release',
                'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200': version.version_type === 'snapshot',
                'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200': version.version_type === 'old_beta' || version.version_type === 'old_alpha',
              }"
            >
              {{ version.version_type === 'release' ? '正式版' : version.version_type === 'snapshot' ? '快照版' : '旧版本' }}
            </span>
          </div>
          <span class="text-sm text-gray-500 dark:text-gray-400">
            {{ new Date(version.release_time * 1000).toLocaleDateString('zh-CN') }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
