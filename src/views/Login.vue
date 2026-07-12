<script setup lang="ts">
/**
 * 登录页面
 * 支持离线登录和微软登录（Web Authorization Code Flow）
 */

import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { open } from '@tauri-apps/plugin-shell'
import Alert from '@/components/common/Alert.vue'
import DeviceCodeModal from '@/components/common/DeviceCodeModal.vue'

const router = useRouter()
const authStore = useAuthStore()

// 离线登录状态
const username = ref('')
const loading = ref(false)
const error = ref<string | null>(null)

// 微软登录弹窗
const showMsModal = ref(false)

// 离线登录
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

// Enter 键提交
function handleKeyPress(event: KeyboardEvent) {
  if (event.key === 'Enter' && !loading.value) {
    handleLogin()
  }
}

// 微软登录
function handleMsLogin() {
  showMsModal.value = true
}

// 微软登录成功
function onMsLoginSuccess() {
  showMsModal.value = false
  router.push('/')
}

// 微软登录弹窗关闭
function onMsModalClose() {
  showMsModal.value = false
}

// 打开购买页面
function openBuyPage() {
  open('https://www.xbox.com/zh-cn/games/store/minecraft-java-bedrock-edition-for-pc/9nxp44l49shj').catch(() => {})
}

// 打开 Minecraft 官网
function openOfficialSite() {
  open('https://www.minecraft.net/').catch(() => {})
}
</script>

<template>
  <div class="flex min-h-[calc(100vh-3.5rem)] items-center justify-center p-4">
    <div class="w-full max-w-md">
      <!-- 标题 -->
      <div class="mb-6 text-center">
        <h1 class="text-2xl font-bold text-gray-900">登录 Minecraft</h1>
        <p class="mt-1 text-sm text-gray-500">选择登录方式以开始游戏</p>
      </div>

      <!-- 登录卡片 -->
      <div class="rounded-2xl bg-white p-6 shadow-lg">
        <!-- 离线登录 -->
        <div class="space-y-3">
          <div>
            <label class="mb-1 block text-sm font-medium text-gray-700">用户名</label>
            <input
              v-model="username"
              type="text"
              maxlength="16"
              class="input w-full"
              placeholder="输入游戏用户名"
              :disabled="loading"
              @keypress="handleKeyPress"
            />
            <p class="mt-1 text-xs text-gray-400">最多 16 个字符，仅支持字母、数字和下划线</p>
          </div>

          <!-- 错误提示 -->
          <div v-if="error || authStore.error" class="rounded-lg bg-red-50 p-3">
            <p class="text-sm text-red-600">{{ error || authStore.error }}</p>
          </div>

          <button
            class="btn-primary w-full rounded-lg py-2.5 text-sm font-medium disabled:opacity-50"
            :disabled="loading || !username.trim()"
            @click="handleLogin"
          >
            <span v-if="loading" class="flex items-center justify-center gap-2">
              <svg class="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
                <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
                <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
              </svg>
              登录中...
            </span>
            <span v-else>离线登录</span>
          </button>
        </div>

        <!-- 分隔线 -->
        <div class="my-4 flex items-center gap-3">
          <div class="h-px flex-1 bg-gray-100"></div>
          <span class="text-xs text-gray-400">或者</span>
          <div class="h-px flex-1 bg-gray-100"></div>
        </div>

        <!-- 微软登录按钮 -->
        <button
          class="flex w-full items-center justify-center gap-2 rounded-lg border border-gray-200 py-2.5 text-sm font-medium text-gray-700 transition hover:bg-gray-50"
          :disabled="authStore.isMsLoggingIn"
          @click="handleMsLogin"
        >
          <!-- 微软四色方块 -->
          <svg viewBox="0 0 23 23" class="h-4 w-4">
            <rect x="1" y="1" width="10" height="10" fill="#F25022" />
            <rect x="12" y="1" width="10" height="10" fill="#7FBA00" />
            <rect x="1" y="12" width="10" height="10" fill="#00A4EF" />
            <rect x="12" y="12" width="10" height="10" fill="#FFB900" />
          </svg>
          微软账号登录
        </button>

        <!-- 购买正版 + 前往官网 -->
        <div class="mt-3 flex gap-2">
          <button
            class="flex-1 rounded-lg border border-gray-200 bg-gray-50 py-2 text-xs font-medium text-gray-500 transition hover:bg-gray-100 hover:text-gray-700"
            @click="openBuyPage"
          >
            购买正版
          </button>
          <button
            class="flex-1 rounded-lg border border-gray-200 bg-gray-50 py-2 text-xs font-medium text-gray-500 transition hover:bg-gray-100 hover:text-gray-700"
            @click="openOfficialSite"
          >
            前往官网
          </button>
        </div>
      </div>

      <!-- 信息提示 -->
      <div class="mt-4">
        <Alert type="info" :truncate="false" message="离线模式仅能进入支持离线登录的服务器。微软账号登录可进入正版服务器并使用皮肤。" />
      </div>
    </div>

    <!-- 微软登录弹窗 -->
    <DeviceCodeModal
      :visible="showMsModal"
      @close="onMsModalClose"
      @success="onMsLoginSuccess"
    />
  </div>
</template>
