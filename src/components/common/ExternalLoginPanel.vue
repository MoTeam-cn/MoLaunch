<script setup lang="ts">
/**
 * authlib-injector 外置登录面板（yggdrasil 协议）
 *
 * 流程：
 * 1. 用户输入 yggdrasil API 根地址（如 https://littleskin.cn/api/yggdrasil）
 * 2. 失焦或点击"获取"按钮时拉取服务器元数据，显示服务器名/注册链接
 * 3. 输入账号密码，点击"外置登录"
 * 4. 单角色 → 直接成功
 * 5. 多角色 → 弹出 ProfileSelectModal 让用户选择，选定后完成登录
 *
 * 与离线/微软登录平级，账号保存后可在账号管理中自由切换。
 */

import { computed, ref, defineAsyncComponent } from 'vue'
import { open } from '@tauri-apps/plugin-shell'
import { authlibFetchServerMeta, authlibLogin, authlibSelectProfile } from '@/utils/api/authlib'
import { normalizeAuthlibServerUrl, willAutoCompletePath } from '@/utils/authlib-url'
import type { AuthlibProfile, AuthlibServerMeta } from '@/types/auth'
import { useAuthStore } from '@/stores/auth'
import { toastWarning, toastSuccess, toastError } from '@/utils/toast'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const ProfileSelectModal = defineAsyncComponent(() => import('./ProfileSelectModal.vue'))

const emit = defineEmits<{ (e: 'success'): void }>()

const authStore = useAuthStore()

// 表单状态
const serverUrl = ref('')
const username = ref('')
const password = ref('')
const loading = ref(false)
const fetchingMeta = ref(false)
const error = ref<string | null>(null)

// 服务器元数据
const serverMeta = ref<AuthlibServerMeta | null>(null)

// 多角色选择弹窗
const showProfileModal = ref(false)
const profileLoading = ref(false)
const pendingProfiles = ref<AuthlibProfile[]>([])

/**
 * 实时计算的规范化 URL（供 UI 提示和实际请求使用）
 *
 * 用户输入 `littleskin.cn` → 显示并请求 `https://littleskin.cn/api/yggdrasil`
 * 用户输入 `https://example.com/custom` → 保留自定义路径
 */
const normalizedUrl = computed(() => normalizeAuthlibServerUrl(serverUrl.value))

/** 是否会自动补全 /api/yggdrasil 路径（用于显示提示） */
const showAutoCompleteHint = computed(() => {
  const raw = serverUrl.value.trim()
  if (!raw) return false
  return willAutoCompletePath(raw)
})

/** 拉取服务器元数据（失焦或点击按钮触发） */
async function fetchMeta() {
  const url = normalizedUrl.value
  if (!url) {
    serverMeta.value = null
    return
  }
  fetchingMeta.value = true
  try {
    serverMeta.value = await authlibFetchServerMeta(url)
  } catch (e) {
    // 拉取失败不阻塞登录，仅清空显示
    serverMeta.value = null
    toastWarning(`无法获取服务器信息：${String(e)}`)
  } finally {
    fetchingMeta.value = false
  }
}

/** 外置登录 */
async function handleLogin() {
  const url = normalizedUrl.value
  const user = username.value.trim()
  const pwd = password.value

  if (!url) { error.value = '请输入服务器地址'; return }
  if (!user) { error.value = '请输入账号'; return }
  if (!pwd) { error.value = '请输入密码'; return }

  loading.value = true
  error.value = null

  try {
    const result = await authlibLogin(url, user, pwd)

    if (result.status === 'success') {
      // 单角色，直接成功
      authStore.currentUser = result.user
      authStore.loginStatus = 'success'
      await authStore.loadAuthlibAccounts()
      toastSuccess('外置登录成功')
      emit('success')
      return
    }

    // 多角色，弹出选择弹窗
    pendingProfiles.value = result.available_profiles
    showProfileModal.value = true
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

/** 多角色弹窗：用户选定 profile */
async function onProfileSelect(profile: AuthlibProfile) {
  profileLoading.value = true
  try {
    const user = await authlibSelectProfile(profile)
    authStore.currentUser = user
    authStore.loginStatus = 'success'
    await authStore.loadAuthlibAccounts()
    showProfileModal.value = false
    toastSuccess('外置登录成功')
    emit('success')
  } catch (e) {
    error.value = String(e)
    showProfileModal.value = false
  } finally {
    profileLoading.value = false
  }
}

/** 多角色弹窗：用户取消 */
function onProfileClose() {
  showProfileModal.value = false
  pendingProfiles.value = []
}

/** 打开注册链接 */
function openRegister() {
  if (serverMeta.value?.register_url) {
    open(serverMeta.value.register_url).catch(() => toastError('打开注册页面失败'))
  }
}
</script>

<template>
  <!--
    单一根节点包裹：Vue 的 <Transition> 要求子组件为单一元素根节点，
    否则报 "renders non-element root node that cannot be animated" 警告并导致切换异常。
    ProfileSelectModal 内部使用 <Teleport to="body">，弹窗渲染到 body，外层包裹不影响其定位。
  -->
  <div class="space-y-3">
    <!-- 服务器地址 -->
    <div>
      <label class="mb-1 block text-sm font-medium text-gray-700">服务器地址</label>
      <div class="flex gap-2">
        <Input
          v-model="serverUrl"
          type="text"
          class="flex-1"
          placeholder="如 littleskin.cn 或 https://littleskin.cn"
          :disabled="loading"
          @blur="fetchMeta"
          @keydown.enter="fetchMeta"
        />
        <Button
          type="secondary"
          :loading="fetchingMeta"
          :disabled="loading || !serverUrl.trim()"
          @click="fetchMeta"
        >
          获取
        </Button>
      </div>
      <!-- 自动补全路径提示（用户输入未含 /api/yggdrasil 时显示规范化后的 URL） -->
      <div
        v-if="showAutoCompleteHint && normalizedUrl"
        class="mt-1.5 truncate text-xs text-gray-400"
      >
        将使用：<span class="text-primary-500">{{ normalizedUrl }}</span>
      </div>
      <!-- 服务器信息展示 -->
      <div v-if="serverMeta" class="mt-1.5 flex items-center justify-between text-xs">
        <span class="text-gray-500">
          服务器：<span class="font-medium text-gray-700">{{ serverMeta.server_name }}</span>
        </span>
        <!-- 保留原生 button：注册链接（text-primary-500 文本链接），
             Button.vue 的 scoped size 类与样式不适合文本链接场景 -->
        <button
          v-if="serverMeta.register_url"
          type="button"
          class="text-primary-500 hover:text-primary-600"
          @click="openRegister"
        >
          注册账号
        </button>
      </div>
      <p v-else-if="!showAutoCompleteHint" class="mt-1 text-xs text-gray-400">
        输入域名即可，自动补全协议和 /api/yggdrasil 路径
      </p>
    </div>

    <!-- 账号 -->
    <div>
      <label class="mb-1 block text-sm font-medium text-gray-700">账号</label>
      <Input
        v-model="username"
        type="text"
        class="w-full"
        placeholder="邮箱或用户名"
        :disabled="loading"
      />
    </div>

    <!-- 密码 -->
    <div>
      <label class="mb-1 block text-sm font-medium text-gray-700">密码</label>
      <Input
        v-model="password"
        type="password"
        class="w-full"
        placeholder="输入密码"
        :disabled="loading"
        @keydown.enter="handleLogin"
      />
    </div>

    <!-- 错误提示 -->
    <div v-if="error" class="rounded-lg bg-red-50 p-3">
      <p class="text-sm text-red-600">{{ error }}</p>
    </div>

    <!-- 登录按钮 -->
    <Button
      type="primary"
      long
      :loading="loading"
      :disabled="!serverUrl.trim() || !username.trim() || !password"
      @click="handleLogin"
    >
      {{ loading ? '登录中...' : '外置登录' }}
    </Button>

    <!-- 多角色选择弹窗（Teleport 到 body，不影响外层布局） -->
    <ProfileSelectModal
      :visible="showProfileModal"
      :profiles="pendingProfiles"
      :loading="profileLoading"
      @select="onProfileSelect"
      @close="onProfileClose"
    />
  </div>
</template>
