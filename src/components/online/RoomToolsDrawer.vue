<script setup lang="ts">
/**
 * 房间工具抽屉（房主 / 加入方共用）
 *
 * - 检查 MC 服务：复用 serverPing（SLP 协议），房主测本机、加入方测房主虚拟 IP（走 TUN）
 * - 检查网络连通性：复用 tcpCheck，加入方检查与房主的链路（仅加入方可见）
 * - 端口自动检测：监听 MC 局域网发现广播（224.0.2.60:4445），解析 [AD]port[/AD]
 */
import { computed, ref } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { serverPing, tcpCheck } from '@/utils/api/tools'
import type { ServerPingResult, TcpCheckResult } from '@/utils/api/tools'
import { lanPortProbe } from '@/utils/api/online-manager'
import type { LanPortProbeResponse } from '@/types/online'
import Button from '@/components/common/Button.vue'
import Drawer from '@/components/common/Drawer.vue'
import { toastError } from '@/utils/toast'
import {
  BoltIcon,
  CheckCircleIcon,
  XCircleIcon,
  ServerStackIcon,
  WifiIcon,
  MagnifyingGlassIcon,
  ClockIcon,
  TagIcon,
  UsersIcon,
} from '@heroicons/vue/24/outline'

const props = withDefaults(defineProps<{ visible?: boolean }>(), { visible: false })
const emit = defineEmits<{ 'update:visible': [v: boolean] }>()
const store = useOnlineStore()
const room = computed(() => store.roomState)
const isHost = computed(() => room.value.role === 'host')

/** MC 服务检测目标：房主测本机，加入方测房主虚拟 IP */
const mcTarget = computed(() =>
  isHost.value
    ? { host: '127.0.0.1', port: room.value.hostMcPort }
    : { host: room.value.hostVirtualIp || '', port: room.value.hostMcPort },
)

const mcLoading = ref(false)
const mcResult = ref<ServerPingResult | null>(null)
async function checkMcService() {
  if (!mcTarget.value.port) {
    toastError('尚未检测到端口，请先使用「端口自动检测」或手动设置端口')
    return
  }
  mcLoading.value = true
  mcResult.value = null
  try {
    mcResult.value = await serverPing(mcTarget.value.host, mcTarget.value.port)
  } catch (e) {
    toastError(`检测失败：${e instanceof Error ? e.message : String(e)}`)
  } finally {
    mcLoading.value = false
  }
}

const connLoading = ref(false)
const connResult = ref<TcpCheckResult | null>(null)
async function checkConnectivity() {
  if (!room.value.hostVirtualIp || !room.value.hostMcPort) {
    toastError('尚未获取到房主虚拟 IP 或端口')
    return
  }
  connLoading.value = true
  connResult.value = null
  try {
    connResult.value = await tcpCheck(room.value.hostVirtualIp, room.value.hostMcPort)
  } catch (e) {
    toastError(`检测失败：${e instanceof Error ? e.message : String(e)}`)
  } finally {
    connLoading.value = false
  }
}

const probeLoading = ref(false)
const probeResult = ref<LanPortProbeResponse | null>(null)
async function detectPort() {
  probeLoading.value = true
  probeResult.value = null
  try {
    probeResult.value = await lanPortProbe({ timeoutMs: 6000 })
  } catch (e) {
    toastError(`检测失败：${e instanceof Error ? e.message : String(e)}`)
  } finally {
    probeLoading.value = false
  }
}

function latencyColor(ms: number): string {
  if (ms < 50) return 'text-green-500'
  if (ms < 150) return 'text-yellow-500'
  if (ms < 300) return 'text-orange-500'
  return 'text-red-500'
}
</script>

<template>
  <Drawer
    :visible="props.visible"
    title="房间工具"
    placement="right"
    :width="420"
    render-in-place
    popup-container="#app-content"
    @update:visible="(v: boolean) => emit('update:visible', v)"
  >
    <div class="space-y-4">
      <!-- 检查 MC 服务 -->
      <section class="rounded-lg border border-gray-200 p-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <ServerStackIcon class="w-4 h-4 text-gray-500" />
            <span class="text-sm font-medium text-gray-800">检查 MC 服务</span>
          </div>
          <code class="text-xs text-gray-500">{{ mcTarget.host }}:{{ mcTarget.port || '-' }}</code>
        </div>
        <p class="mt-1 text-xs text-gray-400">
          通过 SLP 协议获取服务器状态（MOTD / 人数 / 版本 / 延迟），验证能否获取到服务内容
        </p>
        <Button class="mt-3" type="primary" size="small" :loading="mcLoading" @click="checkMcService">
          <template #icon><BoltIcon class="w-3.5 h-3.5" /></template>
          {{ mcLoading ? '检测中…' : '开始检测' }}
        </Button>

        <div v-if="mcResult" class="mt-3 rounded-lg bg-gray-50 p-3 space-y-2">
          <div class="flex items-center gap-2">
            <CheckCircleIcon v-if="!mcResult.error" class="w-4 h-4 text-green-500" />
            <XCircleIcon v-else class="w-4 h-4 text-red-500" />
            <span
              class="text-xs font-medium"
              :class="mcResult.error ? 'text-red-600' : 'text-green-700'"
            >
              {{ mcResult.error ? '无法获取服务内容' : '服务器在线' }}
            </span>
            <span
              v-if="!mcResult.error"
              class="ml-auto text-xs font-medium"
              :class="latencyColor(mcResult.latency_ms)"
            >
              <ClockIcon class="w-3 h-3 inline mr-0.5" />{{ mcResult.latency_ms }} ms
            </span>
          </div>
          <div v-if="mcResult.error" class="text-xs text-red-600">{{ mcResult.error }}</div>
          <template v-else>
            <div class="text-xs text-gray-700 whitespace-pre-wrap break-words">{{ mcResult.motd || '（无 MOTD）' }}</div>
            <div class="flex items-center gap-4 text-xs text-gray-500">
              <span class="flex items-center gap-1"><UsersIcon class="w-3.5 h-3.5" />{{ mcResult.online }} / {{ mcResult.max }}</span>
              <span class="flex items-center gap-1"><TagIcon class="w-3.5 h-3.5" />{{ mcResult.version || '未知版本' }}</span>
            </div>
          </template>
        </div>
      </section>

      <!-- 检查网络连通性（仅加入方：与房主的链路） -->
      <section v-if="!isHost" class="rounded-lg border border-gray-200 p-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <WifiIcon class="w-4 h-4 text-gray-500" />
            <span class="text-sm font-medium text-gray-800">检查网络连通性</span>
          </div>
          <code class="text-xs text-gray-500">{{ room.hostVirtualIp || '-' }}:{{ room.hostMcPort || '-' }}</code>
        </div>
        <p class="mt-1 text-xs text-gray-400">通过 TCP 握手检查与房主的 P2P + 虚拟网卡链路是否通畅</p>
        <Button class="mt-3" type="outline" size="small" :loading="connLoading" @click="checkConnectivity">
          <template #icon><BoltIcon class="w-3.5 h-3.5" /></template>
          {{ connLoading ? '检测中…' : '开始检测' }}
        </Button>

        <div v-if="connResult" class="mt-3 rounded-lg bg-gray-50 p-3 space-y-2">
          <div class="flex items-center gap-2">
            <CheckCircleIcon v-if="connResult.reachable" class="w-4 h-4 text-green-500" />
            <XCircleIcon v-else class="w-4 h-4 text-red-500" />
            <span
              class="text-xs font-medium"
              :class="connResult.reachable ? 'text-green-700' : 'text-red-600'"
            >
              {{ connResult.reachable ? '链路可达' : '无法连接房主' }}
            </span>
            <span
              v-if="connResult.reachable"
              class="ml-auto text-xs font-medium"
              :class="latencyColor(connResult.latency_ms)"
            >
              <ClockIcon class="w-3 h-3 inline mr-0.5" />{{ connResult.latency_ms }} ms
            </span>
          </div>
          <div v-if="connResult.error" class="text-xs text-red-600">{{ connResult.error }}</div>
        </div>
      </section>

      <!-- 端口自动检测 -->
      <section class="rounded-lg border border-gray-200 p-4">
        <div class="flex items-center gap-2">
          <MagnifyingGlassIcon class="w-4 h-4 text-gray-500" />
          <span class="text-sm font-medium text-gray-800">端口自动检测</span>
        </div>
        <p class="mt-1 text-xs text-gray-400">
          {{
            isHost
              ? '监听局域网发现广播获取 Minecraft 实际开放的端口（与多人游戏发现房间同源）'
              : '监听本地局域网发现广播获取房间端口（即多人游戏界面中显示的端口）'
          }}
        </p>
        <Button class="mt-3" type="outline" size="small" :loading="probeLoading" @click="detectPort">
          <template #icon><BoltIcon class="w-3.5 h-3.5" /></template>
          {{ probeLoading ? '监听中…' : '开始检测' }}
        </Button>

        <div v-if="probeResult" class="mt-3 rounded-lg bg-gray-50 p-3">
          <template v-if="probeResult.success">
            <div class="flex items-center gap-2">
              <CheckCircleIcon class="w-4 h-4 text-green-500" />
              <span class="text-xs font-medium text-green-700">检测到局域网广播</span>
            </div>
            <div class="mt-2 flex items-center gap-2">
              <span class="text-xs text-gray-500">端口</span>
              <code class="text-base font-semibold text-primary-600">{{ probeResult.port }}</code>
            </div>
            <div v-if="probeResult.motd" class="mt-1 text-xs text-gray-500">房间名：{{ probeResult.motd }}</div>
          </template>
          <div v-else class="flex items-start gap-1.5 text-xs text-red-600">
            <XCircleIcon class="w-4 h-4 shrink-0" />
            <span>{{ probeResult.error }}</span>
          </div>
        </div>
      </section>
    </div>
  </Drawer>
</template>
