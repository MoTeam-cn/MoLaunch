<script setup lang="ts">
/**
 * 登录页面
 * 支持离线登录、微软登录（Web Authorization Code Flow）和 authlib 外置登录（yggdrasil 协议）
 *
 * 三种登录方式通过 SubTabBar 切换，与 PCL2 的多登录方式选择一致。
 * 用户可在登录页直接选择外置登录，无需进入版本设置。
 */

import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { open } from '@tauri-apps/plugin-shell'
import Alert from '@/components/common/Alert.vue'
import Button from '@/components/common/Button.vue'
import Input from '@/components/common/Input.vue'
import DeviceCodeModal from '@/components/common/DeviceCodeModal.vue'
import SubTabBar from '@/components/common/SubTabBar.vue'
import ExternalLoginPanel from '@/components/common/ExternalLoginPanel.vue'
import { toastSuccess, toastError } from '@/utils/toast'

const router = useRouter()
const authStore = useAuthStore()

// 当前登录方式 Tab
const activeTab = ref<'offline' | 'microsoft' | 'external'>('offline')
const TABS = [
  { id: 'offline', label: '离线登录' },
  { id: 'microsoft', label: '微软登录' },
  { id: 'external', label: '外置登录' },
]

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
    toastSuccess('登录成功')
    router.push('/apps')
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
  toastSuccess('登录成功')
  router.push('/apps')
}

// 微软登录弹窗关闭
function onMsModalClose() {
  showMsModal.value = false
}

// 外置登录成功
function onExternalLoginSuccess() {
  toastSuccess('登录成功')
  router.push('/apps')
}

// 打开购买页面
function openBuyPage() {
  open('https://www.xbox.com/zh-cn/games/store/minecraft-java-bedrock-edition-for-pc/9nxp44l49shj').catch(() => toastError('打开购买页面失败'))
}

// 打开 Minecraft 官网
function openOfficialSite() {
  open('https://www.minecraft.net/').catch(() => toastError('打开官网失败'))
}
</script>

<template>
  <div class="relative flex min-h-[calc(100vh-3.5rem)] items-center justify-center p-4">
    <!-- 返回主页按钮 -->
    <Button
      type="ghost"
      size="small"
      class="absolute left-4 top-4"
      @click="router.push('/apps')"
    >
      <template #icon>
        <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M12.7 4.3a1 1 0 010 1.4L8.4 10l4.3 4.3a1 1 0 01-1.4 1.4l-5-5a1 1 0 010-1.4l5-5a1 1 0 011.4 0z" clip-rule="evenodd" />
        </svg>
      </template>
      返回主页
    </Button>

    <div class="w-full max-w-md">
      <!-- 标题 -->
      <div class="mb-6 text-center">
        <h1 class="text-2xl font-bold text-gray-900">登录 Minecraft</h1>
        <p class="mt-1 text-sm text-gray-500">选择登录方式以开始游戏</p>
      </div>

      <!-- 登录卡片 -->
      <div class="rounded-2xl bg-white p-6 shadow-lg">
        <!-- 登录方式 Tab -->
        <SubTabBar v-model="activeTab" :tabs="TABS" class="-mx-6 -mt-6 mb-5 rounded-t-2xl" />

        <!--
          登录方式内容切换：用 :key + CSS animation 实现进入动画
          不使用 <Transition> 组件，因为 ExternalLoginPanel 内部含 <Teleport>（ProfileSelectModal），
          Transition mode="out-in" 的 leave 钩子会等待 transitionend 事件，Teleport 可能干扰事件触发，
          导致 leave 卡住、新内容不 enter（表现为切换后空白）。
          改用 :key 变化触发 Vue 重新挂载 + CSS animation 自动播放，不依赖 transitionend 事件。
        -->
        <div :key="activeTab" class="login-tab-enter">
          <!-- 离线登录 -->
          <div v-if="activeTab === 'offline'" class="space-y-3">
            <div>
              <label class="mb-1 block text-sm font-medium text-gray-700">用户名</label>
              <Input
                v-model="username"
                type="text"
                :maxlength="16"
                class="w-full"
                placeholder="输入游戏用户名"
                :disabled="loading"
                @keydown="handleKeyPress"
              />
              <p class="mt-1 text-xs text-gray-400">最多 16 个字符，仅支持字母、数字和下划线</p>
            </div>

            <div v-if="error || authStore.error" class="rounded-lg bg-red-50 p-3">
              <p class="text-sm text-red-600">{{ error || authStore.error }}</p>
            </div>

            <Button
              type="primary"
              long
              :loading="loading"
              :disabled="!username.trim()"
              @click="handleLogin"
            >
              {{ loading ? '登录中...' : '离线登录' }}
            </Button>
          </div>

          <!-- 微软登录 -->
          <div v-else-if="activeTab === 'microsoft'" class="space-y-3">
            <p class="text-sm text-gray-600">使用微软账号登录可进入正版服务器并使用皮肤。</p>
            <Button
              type="outline"
              long
              :disabled="authStore.isMsLoggingIn"
              @click="handleMsLogin"
            >
              <template #icon>
                <!-- 微软四色方块 -->
                <svg viewBox="0 0 23 23" class="h-4 w-4">
                  <rect x="1" y="1" width="10" height="10" fill="#F25022" />
                  <rect x="12" y="1" width="10" height="10" fill="#7FBA00" />
                  <rect x="1" y="12" width="10" height="10" fill="#00A4EF" />
                  <rect x="12" y="12" width="10" height="10" fill="#FFB900" />
                </svg>
              </template>
              微软账号登录
            </Button>

            <div class="flex gap-2">
              <Button type="secondary" size="small" class="flex-1" @click="openBuyPage">
                购买正版
              </Button>
              <Button type="secondary" size="small" class="flex-1" @click="openOfficialSite">
                前往官网
              </Button>
            </div>
          </div>

          <!-- 外置登录（authlib-injector / yggdrasil 协议） -->
          <ExternalLoginPanel
            v-else-if="activeTab === 'external'"
            @success="onExternalLoginSuccess"
          />
        </div>
      </div>

      <!-- 信息提示（与上方内容同步过渡） -->
      <div :key="`${activeTab}-alert`" class="login-tab-enter mt-4">
        <Alert
          v-if="activeTab === 'offline'"
          type="info"
          :truncate="false"
          message="离线模式仅能进入支持离线登录的服务器。微软账号登录可进入正版服务器并使用皮肤。"
        />
        <Alert
          v-else-if="activeTab === 'external'"
          type="info"
          :truncate="false"
          message="外置登录通过 authlib-injector 注入 yggdrasil 协议，支持 LittleSkin 等第三方皮肤站。账号保存后可在账号管理中切换。"
        />
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

<style scoped>
/**
 * 登录方式 Tab 切换动画
 *
 * 实现方式：:key 变化触发 Vue 重新挂载元素 + CSS animation 自动播放
 * 选择原因：ExternalLoginPanel 内部含 <Teleport>（ProfileSelectModal），
 *   <Transition mode="out-in"> 的 leave 钩子会等 transitionend 事件，
 *   Teleport 可能干扰事件触发导致 leave 卡住、新内容不 enter（切换后空白）。
 *   改用 CSS animation 不依赖任何 DOM 事件，稳定可靠。
 *
 * 效果：淡入 + 8px 向上位移，200ms ease-out，与 SubTabBar 指示线动画同步
 */
.login-tab-enter {
  animation: login-tab-in 0.2s ease-out;
}

@keyframes login-tab-in {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 尊重用户 prefers-reduced-motion 设置 */
@media (prefers-reduced-motion: reduce) {
  .login-tab-enter {
    animation: none;
  }
}
</style>
