<script setup lang="ts">
/**
 * 从厂商 API 同步隧道列表
 *
 * 流程：选择厂商 → 自动检查授权状态 → 未授权引导去认证 → 已授权拉取隧道 → 选择性导入本地
 *
 * 导入直接调用 useFrpStore.createTunnel（能拿到成功结果）：
 * - 本地已存在同名隧道的远程隧道显示「已导入」不可重复导入
 * - 导入成功才标记为已导入；全部导入成功后自动 emit close 收起面板
 * - 导入失败不标记、不关闭，错误由 store 的 toast 提示
 */
import { ref, computed, watch } from 'vue'
import Button from '@/components/common/Button.vue'
import Select from '@/components/common/Select.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import Tag from '@/components/common/Tag.vue'
import { useFrpStore } from '@/stores/frp'
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
  (e: 'close'): void
}>()

const store = useFrpStore()

// 厂商选择
const selectedProviderId = ref('')
const authStatus = ref<AuthStatus | null>(null)
const authChecking = ref(false)
const fetching = ref(false)
const remoteTunnels = ref<RemoteTunnelInfo[]>([])
const importingIds = ref<Set<string>>(new Set())

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
  importingIds.value.clear()
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

/** 判断远程隧道是否已导入：按厂商自增 id 匹配本地隧道的 remoteTunnelId（已知）或按名字兜底 */
function isImported(tunnel: RemoteTunnelInfo): boolean {
  if (importingIds.value.has(tunnel.id)) return true
  return store.tunnels.some(
    (t) => t.remoteTunnelId === tunnel.id || (!t.remoteTunnelId && t.name === tunnel.name),
  )
}

/** 隧道显示名：优先用厂商的 remark（用户可读名字），为空时回退 name（真实隧道 id） */
function displayName(tunnel: RemoteTunnelInfo): string {
  return tunnel.remark || tunnel.name
}

/** 将远程隧道映射为本地隧道创建参数并导入 */
async function handleImport(tunnel: RemoteTunnelInfo) {
  if (importingIds.value.has(tunnel.id)) return
  importingIds.value.add(tunnel.id)
  try {
    const tunnelType: TunnelType =
      tunnel.tunnelType === 'tcp' || tunnel.tunnelType === 'udp' ? tunnel.tunnelType : 'tcp'
    const localPort = Number(tunnel.localPort)
    const serverPort = Number(tunnel.serverPort)
    const remotePort = Number(tunnel.remotePort)
    if (!Number.isInteger(serverPort) || serverPort <= 0) {
      toastError(`隧道「${displayName(tunnel)}」未解析到有效的服务端端口号，已取消导入`)
      return
    }
    if (tunnelType === 'tcp' || tunnelType === 'udp') {
      if (!Number.isInteger(remotePort) || remotePort <= 0) {
        toastError(`隧道「${displayName(tunnel)}」未解析到远程端口（remotePort），已取消导入`)
        return
      }
    }
    const params: CreateTunnelParams = {
      name: displayName(tunnel),
      providerId: selectedProviderId.value,
      tunnelType,
      localIp: tunnel.localHost || '127.0.0.1',
      localPort: Number.isInteger(localPort) && localPort > 0 ? localPort : 25565,
      serverAddr: tunnel.serverHost,
      serverPort,
      remotePort,
      token: tunnel.token || undefined,
      useTls: false,
      remoteTunnelId: tunnel.id,
      remoteTunnelName: tunnel.name,
      imported: true,
      rawConfig: tunnel.rawConfig,
    }
    const ok = await store.createTunnel(params)
    if (ok && remoteTunnels.value.every(t => isImported(t))) {
      // 全部导入成功后自动收起面板
      emit('close')
    }
  } finally {
    importingIds.value.delete(tunnel.id)
  }
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
              <span class="text-sm font-medium text-gray-900">{{ displayName(tunnel) }}</span>
              <Tag size="small" color="arcoblue" class="uppercase">{{ tunnel.tunnelType }}</Tag>
              <Tag v-if="tunnel.status" size="small" color="gray">{{ tunnel.status }}</Tag>
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
            <Tooltip v-if="!isImported(tunnel)" text="导入到本地隧道列表">
              <Button
                type="outline"
                size="mini"
                :loading="importingIds.has(tunnel.id)"
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
