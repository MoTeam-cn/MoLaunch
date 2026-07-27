<script setup lang="ts">
/**
 * 房主面板（阶段三 mesh 拓扑）
 *
 * 显示房间信息 + 待确认 Answer 列表 + 参与者列表 + P2P 状态 + 关闭按钮。
 *
 * 业务逻辑全部委托给 `useRoomHost` composable：
 * - 信令轮询（参与者 / Answer / 保活）
 * - 自动为 status='joined' && !hostOfferReady 的参与者生成 per-participant Offer
 * - 确认/拒绝 Answer、踢出参与者、关闭房间
 *
 * WebRTC 实例通过 inject 从父级 RoomManager 获取（hostMesh，多 PC 管理器）。
 *
 * 数据分发（阶段三子任务 5）：
 * - `useVirtualLan` 启动后端 TUN 桥接 → `onTunPacket` 回调调 `hostMesh.broadcastPacket` 下发所有参与者
 * - 每个新参与者的 DataChannel 在 `createOfferFor` 后绑定 `onMessage` → `lan.forwardToTun` 转发到 TUN
 */

import { computed, inject } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { useWebRTCMesh } from '@/composables/useWebRTCMesh'
import { useVirtualLan } from '@/composables/useVirtualLan'
import { useRoomHost } from '@/composables/useRoomHost'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import PendingAnswerList from './PendingAnswerList.vue'
import ParticipantList from './ParticipantList.vue'
import WhitelistEditor from './WhitelistEditor.vue'
import { toastSuccess, toastError } from '@/utils/toast'
import {
  XCircleIcon,
  UsersIcon,
  ClockIcon,
  ServerStackIcon,
  WifiIcon,
  ClipboardDocumentIcon,
  InformationCircleIcon,
  ShieldCheckIcon,
} from '@heroicons/vue/24/outline'

const store = useOnlineStore()
const hostMesh = inject('hostMesh') as ReturnType<typeof useWebRTCMesh>

/** TUN 桥接：TUN 读到包 → 广播给所有已联通参与者 */
const lan = useVirtualLan({
  onTunPacket: (raw) => {
    hostMesh.broadcastPacket(raw)
  },
})

/** 房主业务逻辑（轮询 + Offer 生成 + 交互处理 + timer 管理） */
const {
  pendingAnswers,
  handleConfirm,
  handleKick,
  handleCloseRoom,
} = useRoomHost({ hostMesh, lan })

const room = computed(() => store.roomState)
/** 已联通参与者数（channel open） */
const connectedCount = computed(() => hostMesh.connectedCount())
/** 已确认参与者数（status='confirmed'） */
const confirmedCount = computed(
  () => store.roomState.participants.filter((p) => p.status === 'confirmed').length,
)

const remainingSeconds = computed(() => {
  if (!room.value.expiresAt) return 0
  return Math.max(0, room.value.expiresAt - Math.floor(Date.now() / 1000))
})

const remainingText = computed(() => {
  const s = remainingSeconds.value
  if (s <= 0) return '已过期'
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  return h > 0 ? `${h}小时${m}分钟` : `${m}分钟`
})

/** 获取参与者连接状态文本（用于 UI 显示） */
function participantStateText(participantId: string): string {
  return hostMesh.getConnState(participantId) ?? 'unknown'
}

/** 复制虚拟 IP 到剪贴板（复用项目惯例 navigator.clipboard.writeText） */
async function copyVirtualIp() {
  const ip = room.value.selfVirtualIp
  if (!ip) return
  try {
    await navigator.clipboard.writeText(ip)
    toastSuccess(`已复制: ${ip}`)
  } catch {
    toastError('复制失败')
  }
}
</script>

<template>
  <div class="space-y-4">
    <Card title="房间信息">
      <div class="divide-y divide-gray-100">
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ServerStackIcon class="w-4 h-4 text-gray-400" /><span>房间码</span>
          </div>
          <code class="text-base font-semibold text-primary-600 tracking-wider bg-primary-50 px-3 py-1 rounded">
            {{ room.roomCode }}
          </code>
        </div>
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <WifiIcon class="w-4 h-4 text-gray-400" /><span>虚拟 IP</span>
          </div>
          <div class="flex items-center gap-1">
            <code class="text-xs text-gray-900 bg-gray-50 px-2 py-0.5 rounded">{{ room.selfVirtualIp }}</code>
            <Tooltip text="复制虚拟 IP">
              <Button type="ghost" size="mini" @click="copyVirtualIp">
                <template #icon><ClipboardDocumentIcon class="w-3.5 h-3.5" /></template>
              </Button>
            </Tooltip>
          </div>
        </div>
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ServerStackIcon class="w-4 h-4 text-gray-400" /><span>MC 版本 / 端口</span>
          </div>
          <span class="text-xs text-gray-900">{{ room.hostMcVersion || '-' }} : {{ room.hostMcPort || '-' }}</span>
        </div>
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ClockIcon class="w-4 h-4 text-gray-400" /><span>剩余时间</span>
          </div>
          <span class="text-xs" :class="remainingSeconds < 300 ? 'text-red-600' : 'text-gray-900'">
            {{ remainingText }}
          </span>
        </div>
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <UsersIcon class="w-4 h-4 text-gray-400" /><span>人数</span>
          </div>
          <span class="text-xs text-gray-900">{{ room.participants.length + 1 }} / {{ room.maxPlayers }}</span>
        </div>
      </div>
    </Card>

    <Card title="P2P 连接">
      <div class="py-2 space-y-2">
        <div class="flex items-center justify-between">
          <span class="text-xs text-gray-500">已联通 / 已确认</span>
          <span class="text-xs text-gray-900">{{ connectedCount }} / {{ confirmedCount }}</span>
        </div>
        <div class="flex items-center justify-between">
          <span class="text-xs text-gray-500">总参与者数</span>
          <span class="text-xs text-gray-900">{{ room.participants.length }}</span>
        </div>
        <div
          v-if="connectedCount > 0"
          class="mt-2 p-2 bg-blue-50 rounded text-xs text-blue-700 flex gap-1.5 items-start"
        >
          <InformationCircleIcon class="w-3.5 h-3.5 mt-0.5 shrink-0" />
          <span>已联通，请在 Minecraft 内按 Esc → 「开放给局域网」开关。开放后启动器会自动捕获端口并广播给所有参与者，加入方在「多人游戏 → 直接连接」输入你的虚拟 IP 即可加入</span>
        </div>
      </div>
    </Card>

    <!-- 白名单管理（阶段三子任务 8 安全加强）：房主运行期增删 + 启用开关 -->
    <Card title="白名单管理">
      <template #extra>
        <div class="flex items-center gap-1 text-xs" :class="room.whitelistEnabled ? 'text-primary-600' : 'text-gray-400'">
          <ShieldCheckIcon class="w-3.5 h-3.5" />
          <span>{{ room.whitelistEnabled ? '已启用' : '未启用' }}</span>
        </div>
      </template>
      <WhitelistEditor mode="runtime" :room-code="room.roomCode" />
    </Card>

    <PendingAnswerList
      v-if="pendingAnswers.length > 0"
      :answers="pendingAnswers"
      @confirm="handleConfirm"
    />

    <ParticipantList
      v-if="room.participants.length > 0"
      :participants="room.participants"
      :conn-state-text="participantStateText"
      @kick="handleKick"
    />

    <div class="pt-2">
      <Button type="outline" long :loading="store.roomLoading" @click="handleCloseRoom">
        <template #icon><XCircleIcon class="w-4 h-4" /></template>
        关闭房间
      </Button>
    </div>
  </div>
</template>
