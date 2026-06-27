<script setup lang="ts">
/**
 * 登录页面
 */

import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const authStore = useAuthStore()

const username = ref('')
const loading = ref(false)
const error = ref<string | null>(null)

async function handleLogin() {
  if (!username.value.trim()) {
    error.value = '请输入用户名'
    return
  }

  loading.value = true
  error.value = null

  try {
    await authStore.loginOffline(username.value.trim())
    router.push('/')
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

function handleKeyPress(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    handleLogin()
  }
}
</script>

<template>
  <div class="h-full overflow-y-auto">
    <div class="max-w-md mx-auto py-12 px-6">
      <div class="bg-white border border-gray-200 rounded-lg p-8">
        <!-- 标题 -->
        <div class="text-center mb-8">
          <h1 class="text-2xl font-bold text-gray-900">
            登录 MoLaunch
          </h1>
          <p class="text-gray-600 mt-2">
            选择登录方式开始游戏
          </p>
        </div>

        <!-- 离线登录 -->
        <div class="mb-6">
          <h2 class="text-lg font-semibold text-gray-900 mb-4">
            离线模式
          </h2>
          <div class="space-y-4">
            <div>
              <label
                for="username"
                class="block text-sm font-medium text-gray-700 mb-2"
              >
                用户名
              </label>
              <input
                id="username"
                v-model="username"
                type="text"
                placeholder="输入你的游戏用户名"
                class="w-full px-3 py-2 bg-gray-50 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 outline-none transition-colors"
                maxlength="16"
                @keypress="handleKeyPress"
              />
              <p class="text-xs text-gray-500 mt-1">
                最多 16 个字符，仅支持字母、数字和下划线
              </p>
            </div>

            <!-- 错误提示 -->
            <div
              v-if="error || authStore.error"
              class="p-3 rounded-lg bg-red-50 border border-red-200"
            >
              <p class="text-sm text-red-600">
                {{ error || authStore.error }}
              </p>
            </div>

            <button
              class="w-full px-4 py-2.5 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              :disabled="loading || !username.trim()"
              @click="handleLogin"
            >
              <span v-if="loading" class="flex items-center justify-center">
                <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                登录中...
              </span>
              <span v-else>离线登录</span>
            </button>
          </div>
        </div>

        <!-- 分隔线 -->
        <div class="relative my-6">
          <div class="absolute inset-0 flex items-center">
            <div class="w-full border-t border-gray-300"></div>
          </div>
          <div class="relative flex justify-center text-sm">
            <span class="px-2 bg-white text-gray-500">
              或者
            </span>
          </div>
        </div>

        <!-- 微软登录 -->
        <div>
          <h2 class="text-lg font-semibold text-gray-900 mb-4">
            微软账号
          </h2>
          <button
            class="w-full flex items-center justify-center px-4 py-2.5 bg-gray-100 text-gray-400 border border-gray-300 rounded-lg cursor-not-allowed"
            disabled
          >
            <svg class="w-5 h-5 mr-2" viewBox="0 0 21 21" xmlns="http://www.w3.org/2000/svg">
              <rect x="1" y="1" width="9" height="9" fill="#f25022"/>
              <rect x="1" y="11" width="9" height="9" fill="#00a4ef"/>
              <rect x="11" y="1" width="9" height="9" fill="#7fba00"/>
              <rect x="11" y="11" width="9" height="9" fill="#ffb900"/>
            </svg>
            微软登录 (即将支持)
          </button>
        </div>

        <!-- 提示信息 -->
        <div class="mt-6 p-4 rounded-lg bg-blue-50 border border-blue-200">
          <div class="flex">
            <svg class="w-5 h-5 text-blue-600 mt-0.5 mr-2 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <div>
              <p class="text-sm text-blue-800 font-medium">
                关于离线模式
              </p>
              <p class="text-sm text-blue-700 mt-1">
                离线模式仅能进入支持离线登录的服务器，无法进入正版服务器。如需进入正版服务器，请使用微软账号登录。
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
