<script setup lang="ts">
/**
 * 红石联机 - 创建房间面板
 *
 * 选择中转服务器 + 本地 MC 端口拉起 hongshi 内核创建隧道；
 * status=open 后展示联机地址 <server>:<port>，支持复制/停止/重启。
 */
import { ref, computed, onMounted, onUnmounted, onActivated, onDeactivated, defineAsyncComponent } from 'vue'
import { ArrowPathIcon, ClipboardDocumentIcon, PlayIcon, SignalIcon, StopIcon } from '@heroicons/vue/24/outline'
const Card = defineAsyncComponent(() => import('@/components/common/Card.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const AlertV2 = defineAsyncComponent(() => import('@/components/common/AlertV2.vue'))
import { redstoneGetServers, redstoneStart, redstoneStatus, redstoneStop } from '@/utils/api/redstone'
import { getRunningMcPort } from '@/utils/api/online-manager'
import { copyToClipboard } from '@/utils/clipboard'
import { toastError } from '@/utils/toast'
import type { RedStoneServer, RedStoneStatusResult } from '@/types/redstone'

const POLL_INTERVAL = 2000
const POLL_TIMEOUT = 15000
const CLOSED_MESSAGE = '房间已关闭（长时间无人或服务器维护）'
type CreatePhase = 'idle' | 'creating' | 'open' | 'closed' | 'error'
const servers = ref<RedStoneServer[]>([])
const serverLoading = ref(false)
const serverError = ref('')
const useManualServer = ref(false)
const server = ref('')
const mcPort = ref('')
const phase = ref<CreatePhase>('idle')
const status = ref<RedStoneStatusResult | null>(null)
const errorMessage = ref('')
const creating = ref(false)
const stopping = ref(false)
const restarting = ref(false)
let pollTimer: ReturnType<typeof setInterval> | null = null
let pollStart = 0
let polling = false
let mountedOnce = false

const serverOptions = computed(() =>
  servers.value.map((s) => ({ label: `${s.host}（${s.region}）`, value: s.host })),
)
const address = computed(() => {
  const host = status.value?.server
  const port = status.value?.port
  return host && port != null ? `${host}:${port}` : ''
})
function toMessage(e: unknown): string {
  return e instanceof Error ? e.message : String(e)
}
async function loadServers() {
  serverLoading.value = true
  serverError.value = ''
  try {
    const res = await redstoneGetServers()
    servers.value = res.servers
    if (res.servers.length > 0) {
      useManualServer.value = false
      if (!res.servers.some((s) => s.host === server.value)) server.value = res.servers[0].host
    } else {
      useManualServer.value = true
      serverError.value = '暂无可用的中转服务器，可手动填写服务器地址'
    }
  } catch (e) {
    useManualServer.value = true
    serverError.value = `服务器列表拉取失败：${toMessage(e)}，可手动填写地址`
  } finally {
    serverLoading.value = false
  }
}
async function autoFillPort() {
  try {
    const res = await getRunningMcPort()
    if (res.success && res.ports.length > 0 && !mcPort.value) mcPort.value = String(res.ports[0])
  } catch (e) {
    console.warn('[RedStone] 自动探测 MC 端口失败', e)
  }
}
function validateInputs(): string | null {
  if (!server.value.trim()) return '请先选择或填写中转服务器'
  const portStr = mcPort.value.trim()
  if (!/^\d{1,5}$/.test(portStr)) return 'MC 端口需为 1-65535 的数字'
  const port = Number(portStr)
  if (port < 1 || port > 65535) return 'MC 端口需在 1-65535 范围内'
  return null
}
function clearPollTimer() {
  if (pollTimer) {
    clearInterval(pollTimer)
    pollTimer = null
  }
}

/** 单次状态轮询；返回 true 表示结束轮询 */
async function pollStatus(): Promise<boolean> {
  if (polling) return false
  polling = true
  try {
    const res = await redstoneStatus()
    status.value = res
    if (res.status === 'open') { phase.value = 'open'; errorMessage.value = ''; return true }
    if (res.status === 'closed') { phase.value = 'closed'; errorMessage.value = CLOSED_MESSAGE; return true }
    if (!res.running) {
      phase.value = 'error'
      errorMessage.value = '隧道创建失败：服务器不可达或暂无可分配端口，请稍后重试或更换服务器'
      return true
    }
    if (Date.now() - pollStart > POLL_TIMEOUT) { phase.value = 'error'; errorMessage.value = '隧道建立超时'; return true }
    return false
  } catch (e) {
    phase.value = 'error'
    errorMessage.value = `查询隧道状态失败：${toMessage(e)}`
    return true
  } finally {
    polling = false
  }
}
/** 以 2s 间隔轮询直至 open / 错误 / 超时 */
function startPolling() {
  clearPollTimer()
  pollTimer = setInterval(async () => {
    if (await pollStatus()) clearPollTimer()
  }, POLL_INTERVAL)
}

async function launchTunnel(serverHost: string, mcPortValue: number) {
  creating.value = true
  clearPollTimer()
  try {
    await redstoneStart({ server: serverHost, mc_port: mcPortValue })
    phase.value = 'creating'
    errorMessage.value = ''
    pollStart = Date.now()
    startPolling()
  } catch (e) {
    phase.value = 'error'
    errorMessage.value = `隧道创建失败：${toMessage(e)}`
  } finally {
    creating.value = false
  }
}

function handleCreate() {
  const error = validateInputs()
  if (error) {
    toastError(error)
    return
  }
  void launchTunnel(server.value.trim(), Number(mcPort.value))
}

/** 重启 = 先 stop 再 start（沿用当前服务器与端口） */
async function handleRestart() {
  const error = validateInputs()
  if (error) {
    toastError(error)
    return
  }
  restarting.value = true
  try {
    await redstoneStop()
  } catch (e) {
    toastError(`停止旧隧道失败：${toMessage(e)}`)
  }
  await launchTunnel(server.value.trim(), Number(mcPort.value))
  restarting.value = false
}

async function handleStop() {
  stopping.value = true
  try {
    await redstoneStop()
    clearPollTimer()
    phase.value = 'idle'
    status.value = null
    errorMessage.value = ''
  } catch (e) {
    toastError(`停止隧道失败：${toMessage(e)}`)
  } finally {
    stopping.value = false
  }
}

async function copyAddress() {
  if (address.value) await copyToClipboard(address.value, { toast: true })
}

async function restoreStatus() {
  try {
    const res = await redstoneStatus()
    status.value = res
    if (!res.running) {
      if (res.status === 'closed') { phase.value = 'closed'; errorMessage.value = CLOSED_MESSAGE }
      else { phase.value = 'idle'; errorMessage.value = '' }
      return
    }
    if (res.status === 'open') { phase.value = 'open'; errorMessage.value = ''; return }
    phase.value = 'creating' // 进程存活但隧道未就绪：恢复创建中轮询
    errorMessage.value = ''
    pollStart = Date.now()
    startPolling()
  } catch (e) {
    phase.value = 'idle'
  }
}

onMounted(() => {
  mountedOnce = true
  void restoreStatus()
  void loadServers()
  void autoFillPort()
})
onActivated(() => {
  if (mountedOnce) { // keep-alive 初次激活紧跟 mounted，去重避免重复拉取
    mountedOnce = false
    return
  }
  void restoreStatus()
})
onDeactivated(() => clearPollTimer())
onUnmounted(() => clearPollTimer())
</script>

<template>
  <div class="space-y-4">
    <Card title="中转服务器">
      <div class="space-y-3">
        <AlertV2 v-if="serverError" type="error" :message="serverError" />
        <div v-if="useManualServer" class="flex items-center gap-2">
          <div class="flex-1">
            <Input v-model="server" placeholder="手动填写中转服务器地址（如 hk.hongshi.site）" />
          </div>
          <Button type="outline" :loading="serverLoading" @click="loadServers">
            <template #icon><ArrowPathIcon class="w-4 h-4" /></template>重试
          </Button>
        </div>
        <div v-else class="flex items-center gap-2">
          <div class="flex-1">
            <Select v-model="server" :options="serverOptions" placeholder="选择中转服务器" />
          </div>
          <Button type="outline" :loading="serverLoading" @click="loadServers">
            <template #icon><ArrowPathIcon class="w-4 h-4" /></template>刷新
          </Button>
        </div>
        <AlertV2 type="info" message="服务器列表实时来自红石官网，选择就近节点延迟更低" />
      </div>
    </Card>
    <Card title="本地 MC 服务">
      <div class="space-y-3">
        <Input v-model="mcPort" placeholder="MC 服务监听端口（如 25565）" />
        <AlertV2 type="info" message="内核采用懒连接转发，游戏可不预启动；端口已自动探测回填，可手动修改" />
      </div>
    </Card>
    <Card title="隧道状态">
      <div v-if="phase === 'idle'">
        <Button type="primary" long :loading="creating" @click="handleCreate">
          <template #icon><PlayIcon class="w-4 h-4" /></template>创建隧道
        </Button>
      </div>
      <div v-else-if="phase === 'creating'" class="flex flex-col items-center justify-center py-8">
        <SignalIcon class="w-8 h-8 text-blue-400 animate-pulse" />
        <p class="text-sm text-gray-600 font-medium mt-3">等待隧道建立…</p>
        <p class="text-xs text-gray-400 mt-1">正在连接 {{ server || '中转服务器' }}，通常需要数秒</p>
      </div>
      <div v-else-if="phase === 'open'" class="flex flex-col items-center gap-3 py-2">
        <div class="text-base sm:text-lg font-bold text-gray-800 break-all text-center">{{ address }}</div>
        <div class="flex flex-wrap items-center justify-center gap-2">
          <Button type="secondary" @click="copyAddress">
            <template #icon><ClipboardDocumentIcon class="w-4 h-4" /></template>复制地址
          </Button>
          <Button type="outline" :loading="restarting" @click="handleRestart">
            <template #icon><ArrowPathIcon class="w-4 h-4" /></template>重启
          </Button>
          <Button type="ghost" :loading="stopping" @click="handleStop">
            <template #icon><StopIcon class="w-4 h-4" /></template>停止联机
          </Button>
        </div>
        <p class="text-xs text-gray-400">将地址发给好友即可直连进服；隧道 10 人上限 / 10Mbps</p>
      </div>
      <div v-else class="space-y-3">
        <AlertV2 type="error" :message="errorMessage" />
        <div class="flex items-center gap-2">
          <Button type="outline" :loading="restarting" @click="handleRestart">
            <template #icon><ArrowPathIcon class="w-4 h-4" /></template>重新创建
          </Button>
          <Button type="ghost" :loading="stopping" @click="handleStop">
            <template #icon><StopIcon class="w-4 h-4" /></template>停止隧道
          </Button>
        </div>
      </div>
    </Card>
    <Card title="说明">
      <AlertV2 type="info" message="隧道上限 10 人 / 10Mbps；每次创建地址不同，无需长期保留" />
    </Card>
  </div>
</template>