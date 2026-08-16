<script setup lang="ts">
/**
 * 设置 - 联机 Tab（Scaffolding 收敛版）
 *
 * 组合两个独立卡片：
 * - ApiServerCard：api-server 地址 + 测试连通性 + 云端连接状态（自管理加载状态）
 * - 设备管理：登出 / 清除凭证
 *
 * 已移除旧 WebRTC 架构的 ICE/TURN 服务器配置卡片。
 *
 * 复用：
 * - useOnlineStore：deviceStatus + logout/clear
 * - showConfirm：全局 Modal 服务
 */
import { onMounted, ref, watch, defineAsyncComponent } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { useConfigPage } from '@/composables/useConfigPage'
import { toastSuccess } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const ApiServerCard = defineAsyncComponent(() => import('@/components/settings/ApiServerCard.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
import {
  ArrowRightOnRectangleIcon,
  TrashIcon,
  ServerStackIcon,
} from '@heroicons/vue/24/outline'

const onlineStore = useOnlineStore()

// ============ easytier 公共中继节点 ============
const peersText = ref('')
const peersSaved = ref(false)

const {
  loaded: loadedPeers,
  markDirty: markDirtyPeers,
  flushSave: flushSavePeers,
} = useConfigPage({
  delay: 800,
  errorLabel: 'save easytier peers',
  onLoad: (cfg) => {
    peersText.value = (cfg.onlineEasytierPublicPeers ?? []).join('\n')
  },
})

watch(peersText, () => {
  peersSaved.value = false
  markDirtyPeers('onlineEasytierPublicPeers', parsePeers())
})

function parsePeers(): string[] {
  return peersText.value
    .split('\n')
    .map((l) => l.trim())
    .filter((l) => l.length > 0)
}

async function handleSavePeers() {
  await flushSavePeers()
  peersSaved.value = true
  toastSuccess('已保存')
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

    <!-- easytier 公共中继节点 -->
    <div v-if="!loadedPeers" class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <div class="px-5 py-5">
        <div class="h-4 w-28 bg-gray-200 rounded animate-pulse mb-4" />
        <div class="h-20 bg-gray-100 rounded animate-pulse" />
      </div>
    </div>
    <div v-else class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">easytier 公共中继节点</h3>
      <div class="divide-y divide-gray-200">
        <div class="px-5 py-4">
          <div class="flex items-center justify-between mb-2">
            <div>
              <p class="text-sm font-medium text-gray-900">中继节点列表</p>
              <p class="text-xs text-gray-500 mt-0.5">公网组网时用于穿越 NAT，每行一个节点</p>
            </div>
            <Button type="outline" size="small" @click="handleSavePeers">
              <template #icon><ServerStackIcon class="w-4 h-4" /></template>
              保存
            </Button>
          </div>
          <Input
            v-model="peersText"
            textarea
            :rows="4"
            placeholder="tcp://relay.example.com:11010&#10;udp://relay.example.com:11010"
            class="font-mono"
          />
          <div class="mt-2 flex items-center justify-between">
            <span class="text-xs text-gray-400">
              <template v-if="peersSaved">已保存（对新建的虚拟网络生效）</template>
              <template v-else>留空则不指定 --peers 参数</template>
            </span>
          </div>
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
  </div>
</template>
