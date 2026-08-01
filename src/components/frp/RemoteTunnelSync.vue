<script setup lang="ts">
/**
 * 从厂商 API 同步隧道列表
 *
 * 流程：选择厂商 → 自动检查授权状态 → 未授权引导去认证 → 已授权拉取隧道 → 选择性导入本地
 *
 * 导入时将 RemoteTunnelInfo 映射为 CreateTunnelParams，emit 给父组件（TunnelManager）
 * 调用 store.createTunnel 创建本地隧道。
 */
import { ref, computed, watch } from 'vue'
import Button from '@/components/common/Button.vue'
import Select from '@/components/common/Select.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import { fetchTunnels, getAuthStatus } from '@/utils/api/frp-manager'
import { toastError, toastSuccess } from '@/utils/toast'
import type {
  AuthStatus,
  CreateTunnelParams,
  ProviderInfo,
  RemoteTunnelInfo,
  TunnelType,
} from '@/types/frp'
import {
  CloudArrowDownIcon,
  ArrowPathIcon,
  CheckCircleIcon,
  ShieldExclamationIcon,
  ServerIcon,
  GlobeAltIcon,
} from '@heroicons/vue/24/outline'

const props = defineProps<{
  providers: ProviderInfo[]
}>()

const emit = defineEmits<{
  (e: 'import', params: CreateTunnelParams): void
}>()

// 厂商选择
const selectedProviderId = ref('')
const authStatus = ref<AuthStatus | null>(null)
const authChecking = ref(false)
const fetching = ref(false)
const remoteTunnels = ref<RemoteTunnelInfo[]>([])
const importedIds = ref<Set<string>>(new Set())

/** 只显示需要认证且已启用的厂商 */
const authProviders = computed(() =>
  props.providers.filter((p) => p.authType !== 'none' && p.enabled),
)

const providerOptions = computed(() =>
  authProviders.value.map((p) => ({ label: p.name, value: p.id })),
)

const isAuthenticated = computed(() => authStatus.value?.authenticated === true)

/** 选中厂商时检查授权状态 */
watch(selectedProviderId, async (id) => {
  authStatus.value = null
  remoteTunnels.value = []
  importedIds.value.clear()
  if (!id) return
  authChecking.value = true
  try {
    authStatus.value = await getAuthStatus(id)
  } catch (e) {
    toastError('检查授权状态失败: ' + (e instanceof Error ? e.message : String(e)))
  } finally {
    authChecking.value = false
  }
})

async function handleFetch() {
  if (!selectedProviderId.value) return
  fetching.value = true
  try {
    const result = await fetchTunnels(selectedProviderId.value)
    remoteTunnels.value = result.tunnels
    toastSuccess(`拉取成功，共 ${result.tunnels.length} 条隧道`)
  } catch (e) {
    toastError('拉取隧道失败: ' + (e instanceof Error ? e.message : String(e)))
  } finally {
    fetching.value = false
  }
}

/** 将远程隧道映射为本地隧道创建参数并 emit */
function handleImport(tunnel: RemoteTunnelInfo) {
  const tunnelType: TunnelType =
    tunnel.tunnelType === 'tcp' || tunnel.tunnelType === 'udp' ? tunnel.tunnelType : 'tcp'
  const params: CreateTunnelParams = {
    name: tunnel.name,
    providerId: selectedProviderId.value,
    tunnelType,
    localIp: tunnel.localHost || '127.0.0.1',
    localPort: parseInt(tunnel.localPort, 10) || 25565,
    serverAddr: tunnel.serverHost,
    serverPort: parseInt(tunnel.serverPort, 10) || 7000,
    remotePort: parseInt(tunnel.remotePort, 10) || 0,
    token: tunnel.token || undefined,
    useTls: false,
  }
  emit('import', params)
  importedIds.value.add(tunnel.id)
}

function isImported(id: string): boolean {
  return importedIds.value.has(id)
}
</script>

<template>
  <div class="rounded-lg border border-gray-200 bg-white p-4 space-y-4">
    <!-- 厂商选择 -->
    <div>
      <label class="mb-1.5 block text-xs font-medium text-gray-700">选择厂商</label>
      <Select
        v-model="selectedProviderId"
        :options="providerOptions"
        placeholder="选择已安装的厂商"
        class="w-full"
      />
    </div>

    <!-- 无可认证厂商提示 -->
    <div v-if="authProviders.length === 0" class="flex flex-col items-center justify-center py-8">
      <ShieldExclamationIcon class="w-10 h-10 text-gray-300 mb-2" />
      <p class="text-sm font-medium text-gray-500">暂无可同步的厂商</p>
      <p class="text-xs text-gray-400 mt-1">请先安装支持认证的厂商</p>
    </div>

    <!-- 授权状态检查中 -->
    <div v-else-if="authChecking" class="flex items-center justify-center py-6">
      <ArrowPathIcon class="w-5 h-5 text-gray-400 animate-spin" />
      <span class="ml-2 text-sm text-gray-500">检查授权状态...</span>
    </div>

    <!-- 未授权提示 -->
    <div v-else-if="selectedProviderId && !isAuthenticated" class="flex flex-col items-center justify-center py-8">
      <ShieldExclamationIcon class="w-10 h-10 text-amber-400 mb-2" />
      <p class="text-sm font-medium text-gray-700">厂商未授权</p>
      <p class="text-xs text-gray-400 mt-1">请先到「认证」页面完成授权后再同步隧道</p>
    </div>

    <!-- 已授权：拉取按钮 -->
    <div v-else-if="isAuthenticated" class="flex items-center justify-between">
      <span class="text-xs text-green-600 flex items-center gap-1">
        <CheckCircleIcon class="w-4 h-4" />
        已认证
      </span>
      <Button
        type="primary"
        size="small"
        :loading="fetching"
        @click="handleFetch"
      >
        <template #icon><CloudArrowDownIcon class="w-4 h-4" /></template>
        拉取隧道
      </Button>
    </div>

    <!-- 远程隧道列表 -->
    <div v-if="remoteTunnels.length > 0" class="space-y-2">
      <div class="text-xs text-gray-500 border-t border-gray-100 pt-3">
        共 {{ remoteTunnels.length }} 条远程隧道
      </div>
      <div
        v-for="tunnel in remoteTunnels"
        :key="tunnel.id"
        class="rounded-md border border-gray-200 p-3 hover:border-primary-300 transition-colors"
      >
        <div class="flex items-start justify-between gap-2">
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 flex-wrap">
              <span class="text-sm font-medium text-gray-900">{{ tunnel.name }}</span>
              <span class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-primary-50 text-primary-700 uppercase">
                {{ tunnel.tunnelType }}
              </span>
              <span v-if="tunnel.status" class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-gray-50 text-gray-500">
                {{ tunnel.status }}
              </span>
            </div>
            <div class="mt-1 flex flex-wrap gap-x-4 gap-y-0.5 text-xs text-gray-500">
              <span v-if="tunnel.serverHost" class="flex items-center gap-1">
                <GlobeAltIcon class="w-3.5 h-3.5" />
                {{ tunnel.serverHost }}{{ tunnel.serverPort ? ':' + tunnel.serverPort : '' }}
              </span>
              <span v-if="tunnel.localPort" class="flex items-center gap-1">
                <ServerIcon class="w-3.5 h-3.5" />
                {{ tunnel.localHost || '127.0.0.1' }}:{{ tunnel.localPort }}
              </span>
              <span v-if="tunnel.remotePort">远程: {{ tunnel.remotePort }}</span>
              <span v-if="tunnel.customDomain" class="text-green-600">{{ tunnel.customDomain }}</span>
            </div>
          </div>
          <div class="shrink-0">
            <Tooltip v-if="!isImported(tunnel.id)" text="导入到本地隧道列表">
              <Button
                type="outline"
                size="mini"
                @click="handleImport(tunnel)"
              >
                <template #icon><CloudArrowDownIcon class="w-3.5 h-3.5" /></template>
                导入
              </Button>
            </Tooltip>
            <span v-else class="inline-flex items-center gap-1 text-xs text-green-600 px-2">
              <CheckCircleIcon class="w-3.5 h-3.5" />
              已导入
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- 拉取后空列表 -->
    <div v-else-if="fetching" class="flex items-center justify-center py-6">
      <ArrowPathIcon class="w-5 h-5 text-gray-400 animate-spin" />
      <span class="ml-2 text-sm text-gray-500">拉取中...</span>
    </div>
  </div>
</template>
