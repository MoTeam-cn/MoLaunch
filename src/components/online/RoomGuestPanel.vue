<script setup lang="ts">
/**
 * 加入方面板（Scaffolding 收敛版）
 *
 * 进房后显示：
 * - 房间信息（N 段公开标识、房主 MC 版本/端口、备注）
 * - easytier 连接状态徽章（组网中 / 已组网 / 失败）
 * - 进服地址（127.0.0.1:本地 port-forward 端口，可复制）+ 重新探测按钮
 * - 整合包要求（房主关联整合包时自动校验本地）
 * - 退出房间按钮（停 easytier + 清空本地状态）
 *
 * 挂载后自动探测进服地址（scaffolding_client_probe，join 闸门已拿完整码）。
 */
import { computed, onMounted, onUnmounted, ref, defineAsyncComponent } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { getOnlineSession } from '@/composables/online/onlineSession'
import { useEasyTier, type EasyTierStatus } from '@/composables/useEasyTier'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Card = defineAsyncComponent(() => import('@/components/common/Card.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
const AlertV2 = defineAsyncComponent(() => import('@/components/common/AlertV2.vue'))
const ModpackRequirementCard = defineAsyncComponent(() => import('./ModpackRequirementCard.vue'))
const RoomToolsDrawer = defineAsyncComponent(() => import('./RoomToolsDrawer.vue'))
import { showConfirm } from '@/utils/modal'
import { toastError, toastSuccess } from '@/utils/toast'
import { copyToClipboard } from '@/utils/clipboard'
import {
  XCircleIcon,
  ServerStackIcon,
  ExclamationTriangleIcon,
  ClipboardDocumentIcon,
  ArrowPathIcon,
  Cog6ToothIcon,
} from '@heroicons/vue/24/outline'

const store = useOnlineStore()
const easytier = useEasyTier()
const session = getOnlineSession()

const room = computed(() => store.roomState)
/** 探测结果（mcIp/mcPort 由 scaffolding.probe 写入 store.easytierRuntime） */
const entry = computed(() => store.easytierRuntime)

/** 房间工具抽屉开关（检查 MC 服务 / 网络连通性 / 端口自动检测） */
const toolsDrawerOpen = ref(false)

const connStateText = computed(() => {
  switch (easytier.status.value) {
    case 'joined': return '已组网'
    case 'joining': return '组网中…'
    case 'error': return '组网失败'
    case 'stopping': return '断开中…'
    default: return '未组网'
  }
})

const connStateClass = computed(() => {
  switch (easytier.status.value as EasyTierStatus) {
    case 'joined': return 'bg-green-50 text-green-700'
    case 'joining': return 'bg-blue-50 text-blue-700'
    case 'error': return 'bg-red-50 text-red-700'
    case 'stopping': return 'bg-yellow-50 text-yellow-700'
    default: return 'bg-gray-50 text-gray-500'
  }
})

/** 房主 MC 端口轮询周期（5s）与连续失败上限（3 次 = 15s） */
const POLL_INTERVAL_MS = 5000
const POLL_MAX_FAILS = 3

/** 房主 MC 端口轮询定时器与失败计数 */
let portPollTimer: ReturnType<typeof setInterval> | null = null
let pollFailCount = 0

/** 手动指定端口（最高权重：自动轮询不再覆盖；null 为自动模式） */
const manualPort = ref<number | null>(null)
const manualInput = ref('')

function stopPortPolling(): void {
  if (portPollTimer) {
    clearInterval(portPollTimer)
    portPollTimer = null
  }
  pollFailCount = 0
}

async function pollTick(): Promise<void> {
  const res = await session.scaffolding.poll(room.value.roomCode)
  if (!res.ok) {
    pollFailCount += 1
    if (pollFailCount >= POLL_MAX_FAILS) {
      stopPortPolling()
      toastError('房主可能已关闭房间，已停止端口自动更新')
    }
    return
  }
  pollFailCount = 0
  if (res.mcIp) store.setEasyTierRuntime({ mcIp: res.mcIp })
  if (res.mcPort == null || manualPort.value != null) return
  if (res.mcPort !== store.easytierRuntime.mcPort) {
    store.setEasyTierRuntime({ mcPort: res.mcPort })
    toastSuccess(`房主 MC 端口已变更（新端口 ${res.mcPort}），请刷新服务器列表或重新连接`)
  }
}

function startPortPolling(): void {
  stopPortPolling()
  portPollTimer = setInterval(() => {
    void pollTick()
  }, POLL_INTERVAL_MS)
}

function applyManualPort(): void {
  const port = Number(manualInput.value)
  if (!Number.isInteger(port) || port <= 0 || port > 65535) {
    toastError('请输入有效的端口号（1-65535）')
    return
  }
  manualPort.value = port
  toastSuccess(`已手动设置端口 ${port}（自动更新不再覆盖）`)
}

function clearManualPort(): void {
  manualPort.value = null
  manualInput.value = ''
  toastSuccess('已恢复端口自动更新')
}

/** 进服地址（mcIp:有效端口，手动覆盖优先） */
const effectivePort = computed(() => manualPort.value ?? entry.value.mcPort)
const entryAddress = computed(() => {
  if (!entry.value.mcIp || !effectivePort.value) return ''
  return `${entry.value.mcIp}:${effectivePort.value}`
})

/** 重新探测进服地址（scaffolding_client_probe） */
async function reProbe() {
  const res = await session.reconnect.reconnect()
  if (!res.ok) {
    toastError(`探测失败：${res.error ?? '未知错误'}`)
  }
}

/** 首次进入房间：组网 + 探测成功后提示可开始游玩并启动端口轮询 */
async function initialProbe() {
  const res = await session.reconnect.reconnect()
  if (res.ok) {
    toastSuccess('当前成功与主网络组网，可以开始游玩')
    startPortPolling()
  } else {
    toastError(`探测失败：${res.error ?? '未知错误'}`)
  }
}

/** 退出房间：停 easytier + 清空本地状态 */
function handleLeaveRoom() {
  showConfirm(
    '退出房间',
    '退出后将断开虚拟局域网连接。确定退出？',
    async () => {
      stopPortPolling()
      try {
        await easytier.stop()
      } catch (e) {
        toastError(`退出失败：${e instanceof Error ? e.message : String(e)}`)
      } finally {
        store.resetRoomState()
      }
    },
  )
}

async function copyText(text: string) {
  if (!text) return
  await copyToClipboard(text, { toast: true })
}

onMounted(() => {
  if (room.value.role === 'guest' && room.value.roomCode) {
    void initialProbe()
  }
})

onUnmounted(() => {
  stopPortPolling()
})
</script>

<template>
  <div class="space-y-4">
    <AlertV2 type="info" message="联机基于 easytier 虚拟局域网：组网成功后，在 Minecraft「多人游戏 → 直接连接」输入下方进服地址即可进入房主房间" />
    <AlertV2 type="info" message="如遇到违法违规房间，请及时向我们举报" />

    <!-- 房间信息 -->
    <Card title="房间信息">
      <div class="divide-y divide-gray-100">
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ServerStackIcon class="w-4 h-4 text-gray-400" />
            <span>房间标识</span>
          </div>
          <code class="text-sm font-semibold text-primary-600 tracking-wider bg-primary-50 px-2 py-1 rounded">
            {{ room.publicIdentifier }}
          </code>
        </div>
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ServerStackIcon class="w-4 h-4 text-gray-400" />
            <span>房主 MC 版本</span>
          </div>
          <span class="text-xs text-gray-900">{{ room.hostMcVersion || '-' }}</span>
        </div>
        <div v-if="room.remark" class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ServerStackIcon class="w-4 h-4 text-gray-400" />
            <span>房主备注</span>
          </div>
          <span class="text-xs text-gray-900 truncate max-w-[50%]">{{ room.remark }}</span>
        </div>
      </div>
    </Card>

    <!-- 整合包要求（房主关联整合包时显示，自动校验本地是否已装同款） -->
    <ModpackRequirementCard v-if="room.hostModpack" :modpack="room.hostModpack" />

    <!-- 连接状态 -->
    <Card title="连接状态">
      <div class="py-2 flex items-center justify-between">
        <span class="text-xs text-gray-500">easytier 虚拟网络</span>
        <span
          class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium"
          :class="connStateClass"
        >
          {{ connStateText }}
        </span>
      </div>
      <div v-if="easytier.error.value" class="mt-2 p-2 bg-red-50 rounded text-xs text-red-700 flex items-start gap-1.5">
        <ExclamationTriangleIcon class="w-3.5 h-3.5 mt-0.5 shrink-0" />
        <span>{{ easytier.error.value }}</span>
      </div>
    </Card>

    <!-- 进服地址 -->
    <Card title="进服地址">
      <div v-if="entryAddress" class="py-1 flex items-center justify-between gap-2">
        <div class="flex-1 min-w-0">
          <code class="text-base font-mono font-semibold text-green-700 bg-green-50 px-3 py-1.5 rounded block truncate">
            {{ entryAddress }}
          </code>
        </div>
        <Tooltip text="复制进服地址">
          <Button type="ghost" size="small" @click="copyText(entryAddress)">
            <template #icon><ClipboardDocumentIcon class="w-3.5 h-3.5" /></template>
            复制
          </Button>
        </Tooltip>
      </div>
      <div v-else class="py-2 text-xs text-gray-500">
        <div class="flex items-center gap-1.5">
          <ExclamationTriangleIcon class="w-3.5 h-3.5 text-yellow-500 shrink-0" />
          <span>尚未探测到房主 MC 服务，请确认房主已在游戏中开启「对局域网开放」后重新探测</span>
        </div>
      </div>
      <div class="mt-3 pt-3 border-t border-gray-100">
        <Button type="outline" long size="small" :loading="session.scaffolding.probing.value" @click="reProbe">
          <template #icon><ArrowPathIcon class="w-3.5 h-3.5" /></template>
          重新探测进服地址
        </Button>
      </div>
      <!-- 手动端口（最高权重：自动轮询不再覆盖） -->
      <div class="mt-2 pt-3 border-t border-gray-100 space-y-2">
        <div v-if="manualPort" class="flex items-center justify-between text-xs">
          <span class="text-gray-500">已手动设置端口 {{ manualPort }}，自动更新暂停</span>
          <Button type="ghost" size="small" @click="clearManualPort">恢复自动</Button>
        </div>
        <div class="flex items-center gap-2">
          <input
            v-model="manualInput"
            type="number"
            min="1"
            max="65535"
            placeholder="手动指定端口（最高权重）"
            class="flex-1 min-w-0 rounded-md border border-gray-300 px-2 py-1 text-sm"
            @keyup.enter="applyManualPort"
          />
          <Button type="outline" size="small" @click="applyManualPort">手动设置</Button>
        </div>
      </div>
    </Card>

    <!-- 房间工具 -->
    <div class="pt-2">
      <Button type="outline" long @click="toolsDrawerOpen = true">
        <template #icon><Cog6ToothIcon class="w-4 h-4" /></template>
        房间工具
      </Button>
    </div>

    <!-- 退出房间 -->
    <div class="pt-2">
      <Button type="outline" long :loading="store.roomLoading" @click="handleLeaveRoom">
        <template #icon><XCircleIcon class="w-4 h-4" /></template>
        退出房间
      </Button>
    </div>
  </div>

  <RoomToolsDrawer v-model:visible="toolsDrawerOpen" />
</template>
