<script setup lang="ts">
/**
 * 设置 - 联机 Tab
 *
 * 组合三个独立卡片：
 * - ApiServerCard：api-server 地址 + 测试连通性 + 云端连接状态（自管理加载状态）
 * - ICE 服务器配置：用户自定义 TURN 服务器列表
 * - 设备管理：登出 / 清除凭证
 *
 * 复用：
 * - useConfigPage：ICE 卡片的加载 + 防抖保存 + loaded 守卫
 * - useOnlineStore：deviceStatus + logout/clear + setCustomTurnServers
 * - showConfirm：全局 Modal 服务
 */
import { ref, watch, onMounted, defineAsyncComponent } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { useConfigPage } from '@/composables/useConfigPage'
import { showConfirm } from '@/utils/modal'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const TurnServerEntryEditor = defineAsyncComponent(() => import('@/components/online/TurnServerEntryEditor.vue'))
const ApiServerCard = defineAsyncComponent(() => import('@/components/settings/ApiServerCard.vue'))
import {
  ArrowRightOnRectangleIcon,
  TrashIcon,
  PlusIcon,
  ServerStackIcon,
} from '@heroicons/vue/24/outline'
import type { IceServerEntry } from '@/types/online'

const onlineStore = useOnlineStore()

// ============ ICE 服务器配置 ============
const customTurnServers = ref<IceServerEntry[]>([])

const { loaded, markDirty } = useConfigPage({
  delay: 800,
  errorLabel: 'save online settings',
  onLoad: (cfg) => {
    customTurnServers.value = cfg.onlineCustomTurnServers ?? []
  },
})

// watch deep：任一条目字段变更触发防抖保存 + store 同步
watch(
  customTurnServers,
  (v) => {
    markDirty('onlineCustomTurnServers', v)
    onlineStore.setCustomTurnServers(v)
  },
  { deep: true },
)

function handleAddTurnServer() {
  customTurnServers.value.push({ urls: [], username: undefined, credential: undefined })
}

function handleRemoveTurnServer(index: number) {
  customTurnServers.value.splice(index, 1)
}

function handleUpdateTurnServer(index: number, entry: IceServerEntry) {
  customTurnServers.value[index] = entry
}

// ============ 设备登出 / 清除凭证 ============
async function handleLogout() {
  await onlineStore.logout()
}

function handleClearCredentials() {
  showConfirm(
    '清除设备凭证',
    '此操作将删除本地密钥与 JWT，需要重新注册设备才能继续使用联机功能。\n是否继续？',
    async () => {
      await onlineStore.clear()
    },
  )
}

onMounted(() => {
  void onlineStore.refreshStatus()
})
</script>

<template>
  <div class="space-y-6">
    <!-- api-server 配置（自管理加载状态） -->
    <ApiServerCard />

    <!-- 加载占位（仅 ICE + 设备管理） -->
    <div v-if="!loaded" class="space-y-6">
      <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
        <div class="px-5 py-5">
          <div class="h-4 w-24 bg-gray-200 rounded animate-pulse mb-4" />
          <div class="h-10 bg-gray-100 rounded animate-pulse" />
        </div>
      </div>
    </div>

    <template v-else>
      <!-- ICE 服务器配置 -->
      <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
        <div class="px-5 pt-5 pb-3 flex items-center justify-between">
          <div>
            <h3 class="text-sm font-semibold text-gray-900">ICE 服务器配置</h3>
            <p class="text-xs text-gray-500 mt-0.5">
              自定义 TURN 服务器（P2P 直连失败时的中转备用方案），自动保存（800ms 防抖）
            </p>
          </div>
        </div>
        <div class="divide-y divide-gray-200">
          <!-- 空状态：icon + text 垂直水平居中 -->
          <div
            v-if="customTurnServers.length === 0"
            class="flex flex-col items-center justify-center py-8"
          >
            <ServerStackIcon class="w-8 h-8 text-gray-300 mb-2" />
            <p class="text-xs text-gray-500">未配置自定义 TURN 服务器</p>
            <p class="text-xs text-gray-400 mt-1">系统将仅使用 STUN + 官方 TURN（如可用）</p>
          </div>
          <!-- TURN 服务器条目列表 -->
          <TurnServerEntryEditor
            v-for="(entry, idx) in customTurnServers"
            :key="idx"
            :model-value="entry"
            :index="idx"
            @update:model-value="(v) => handleUpdateTurnServer(idx, v)"
            @remove="handleRemoveTurnServer(idx)"
          />
          <!-- 添加按钮 -->
          <div class="px-5 py-3">
            <Button type="ghost" size="small" long @click="handleAddTurnServer">
              <template #icon><PlusIcon class="w-4 h-4" /></template>
              添加 TURN 服务器
            </Button>
          </div>
        </div>
      </div>

      <!-- 设备管理 -->
      <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
        <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">设备管理</h3>
        <div class="divide-y divide-gray-200">
          <!-- 登出 -->
          <div class="px-5 py-4 flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">登出设备</p>
              <p class="text-xs text-gray-500 mt-0.5">撤销当前 JWT，保留本地密钥，可重新登录</p>
            </div>
            <Button
              type="outline"
              size="small"
              :loading="onlineStore.loading"
              :disabled="!onlineStore.deviceStatus?.logged_in"
              @click="handleLogout"
            >
              <template #icon><ArrowRightOnRectangleIcon class="w-4 h-4" /></template>
              登出
            </Button>
          </div>
          <!-- 清除凭证 -->
          <div class="px-5 py-4 flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">清除设备凭证</p>
              <p class="text-xs text-gray-500 mt-0.5">删除本地密钥与 JWT，需要重新注册设备</p>
            </div>
            <Button
              type="outline"
              size="small"
              :loading="onlineStore.loading"
              :disabled="!onlineStore.deviceStatus?.registered"
              @click="handleClearCredentials"
            >
              <template #icon><TrashIcon class="w-4 h-4" /></template>
              清除
            </Button>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
