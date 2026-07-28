<script setup lang="ts">
/**
 * 加入方面板（阶段二 + 阶段三子任务 5）
 *
 * 显示内容：
 * - 房间信息卡片（房间码、自己的虚拟 IP、房主 MC 版本/端口）
 * - P2P 连接状态徽章
 * - MC 版本匹配提示（如房主版本与自己不同时提示）
 * - 退出房间按钮
 *
 * 加入方无需轮询 answers（房主会主动 confirm），
 * 仅在房间状态异常时由用户主动退出。
 *
 * 数据分发（阶段三子任务 5）：
 * - `useVirtualLan` 启动后端 TUN 桥接 → `onTunPacket` 回调通过 `guestWebrtc.dataChannel.send(raw)` 发给房主
 * - watch `guestWebrtc.dataChannel`：DataChannel 就绪后绑定 `onMessage` → `lan.forwardToTun` 转发到 TUN
 */

import { computed, inject, onMounted, watch } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { useWebRTC } from '@/composables/useWebRTC'
import { useVirtualLan } from '@/composables/useVirtualLan'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import { showConfirm } from '@/utils/modal'
import { toastError, toastSuccess } from '@/utils/toast'
import { decode, CONTROL_SUBTYPE, parseHostMcPortPayload, decodeTurnServersPayload } from '@/utils/online/protocol'
import { importRoomKey } from '@/utils/online/crypto'
import {
  XCircleIcon,
  ClockIcon,
  ServerStackIcon,
  ExclamationTriangleIcon,
  ClipboardDocumentIcon,
} from '@heroicons/vue/24/outline'
import VirtualIpCard from './VirtualIpCard.vue'
import ModpackRequirementCard from './ModpackRequirementCard.vue'

const store = useOnlineStore()
const guestWebrtc = inject('guestWebrtc') as ReturnType<typeof useWebRTC>

/**
 * TUN 桥接：TUN 读到包 → 通过 DataChannel 发给房主
 *
 * 阶段三子任务 8：使用 `guestWebrtc.sendPacket` 走加密通道（若 roomKey 已注入），
 * 不再直接调 `channel.send`。DataChannel 未就绪时 sendPacket 内部静默返回 false。
 */
const lan = useVirtualLan({
  onTunPacket: (raw) => {
    void guestWebrtc.sendPacket(raw)
  },
})

/**
 * 监听 DataChannel 就绪，绑定 onMessage：
 * - Control + HostMcPort：更新本地 store.roomState.hostMcPort（房主开放 LAN 后广播）
 * - Control + TurnServers：更新本地 ICE 配置并尝试 setConfiguration（阶段三子任务 7 阶段 G）
 * - Data（IP 包）：转发到后端 TUN 接口
 * - 其他消息：静默丢弃（不支持的控制子类型或损坏帧）
 *
 * `useWebRTC` 在 `pc.ondatachannel` 触发时填充 `dataChannel.value`，
 * 此处 watch 在 dataChannel 变化时重新绑定 handler。
 *
 * # TURN 服务器更新策略
 *
 * 房主拉取系统 TURN 后通过 DataChannel 广播给所有参与者。加入方收到后：
 * 1. 更新 `store.roomState.iceServers`（影响后续 PC 重建时的 ICE 配置）
 * 2. 调用 `pc.setConfiguration` 更新当前 PC 配置（已建立连接需 ICE restart 完全生效，
 *    此处仅更新配置，不主动触发 restart，避免中断现有连接）
 * 3. 若 PC 尚未建立（negotiating 中），下次 `ensurePeerConnection` 会使用新配置
 *
 * 不主动重建 PC 的原因：
 * - mesh 拓扑下房主为每个参与者生成 Offer，加入方无法单方面触发重新协商
 * - 强制 close + 重新 fetchOfferAndAnswer 需要房主配合重新生成 Offer，链路过长
 * - 当前 TURN 通常在房间初期下发，PC 已建立时 STUN/TURN 已完成 ICE 收集
 */
watch(
  () => guestWebrtc.dataChannel.value,
  (channel) => {
    if (!channel) return
    guestWebrtc.setDataChannelHandlers({
      onMessage: (raw) => {
        const msg = decode(raw)
        if (!msg) return
        if (msg.kind === 'control' && msg.subtype === CONTROL_SUBTYPE.HOST_MC_PORT) {
          const port = parseHostMcPortPayload(msg.payload)
          if (port !== null && port > 0) {
            store.roomState.hostMcPort = port
            console.info(`[Online] 加入方收到房主 MC 端口: ${port}`)
          }
          return
        }
        if (msg.kind === 'control' && msg.subtype === CONTROL_SUBTYPE.TURN_SERVERS) {
          const turnServers = decodeTurnServersPayload(msg.payload)
          if (!turnServers || turnServers.length === 0) {
            console.info('[Online] 加入方收到房主广播的空 ICE 列表，忽略')
            return
          }
          // 更新本地 ICE 服务器配置（影响后续 PC 重建）
          store.roomState.iceServers = turnServers
          // 尝试更新当前 PC 配置（已建立连接需 ICE restart 完全生效）
          const currentPc = guestWebrtc.pc.value
          if (currentPc) {
            try {
              currentPc.setConfiguration({
                iceServers: turnServers.map((entry) => {
                  const server: RTCIceServer = { urls: entry.urls }
                  if (entry.username) server.username = entry.username
                  if (entry.credential) server.credential = entry.credential
                  return server
                }),
                iceTransportPolicy: 'all',
              })
              console.info('[Online] 加入方已更新 PeerConnection ICE 配置（需 ICE restart 完全生效）')
            } catch (e) {
              console.warn('[Online] 加入方更新 PC 配置失败:', e)
            }
          }
          console.info(`[Online] 加入方收到房主广播的 ICE 服务器列表：${turnServers.length} 条`)
          return
        }
        if (msg.kind === 'data') {
          void lan.forwardToTun(raw)
        }
      },
    })
  },
  { immediate: true },
)

const room = computed(() => store.roomState)
const connState = guestWebrtc.connectionState

/** 距过期剩余时间（秒） */
const remainingSeconds = computed(() => {
  if (!room.value.expiresAt) return 0
  return Math.max(0, room.value.expiresAt - Math.floor(Date.now() / 1000))
})

const remainingText = computed(() => {
  const s = remainingSeconds.value
  if (s <= 0) return '已过期'
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  if (h > 0) return `${h}小时${m}分钟`
  return `${m}分钟`
})

/** 退出房间 */
function handleLeaveRoom() {
  showConfirm(
    '退出房间',
    '退出后将断开与房主的 P2P 连接。确定退出？',
    async () => {
      try {
        // 先停止 TUN 桥接，再关 PC，最后调后端退出
        await lan.stop()
        guestWebrtc.close()
        await store.guestLeaveRoom()
      } catch (e) {
        toastError(`退出失败：${e instanceof Error ? e.message : String(e)}`)
        // 即使后端调用失败也清空本地状态
        store.resetRoomState()
      }
    },
  )
}

/** 复制文本到剪贴板（复用项目惯例 navigator.clipboard.writeText） */
async function copyText(text: string, label: string) {
  if (!text) return
  try {
    await navigator.clipboard.writeText(text)
    toastSuccess(`已复制${label}: ${text}`)
  } catch {
    toastError('复制失败')
  }
}

onMounted(() => {
  // 加入方拉取一次房间信息同步元数据
  void store.refreshRoomInfo()

  // 阶段三子任务 8：注入 DataChannel 加密密钥（空字符串表示未启用加密，importRoomKey 返回 null）
  void importRoomKey(store.roomState.roomKey)
    .then((key) => guestWebrtc.setRoomKey(key))
    .catch((e) => console.warn('[Online] 加入方加密密钥导入失败:', e))

  // 启动 TUN 桥接：进入面板即创建 TUN 接口，开始读包 → dataChannel.send
  // 失败仅 toast（如 wintun.dll 缺失 / 无管理员权限），不阻塞信令流程
  void lan.start(store.roomState.selfVirtualIp, store.roomState.subnet).catch((e) => {
    toastError(`虚拟网卡启动失败：${e instanceof Error ? e.message : String(e)}`)
  })
})
</script>

<template>
  <div class="space-y-4">
    <!-- 房间信息 -->
    <Card title="房间信息">
      <div class="divide-y divide-gray-100">
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ServerStackIcon class="w-4 h-4 text-gray-400" />
            <span>房间码</span>
          </div>
          <code class="text-base font-semibold text-primary-600 tracking-wider bg-primary-50 px-3 py-1 rounded">
            {{ room.roomCode }}
          </code>
        </div>
        <VirtualIpCard :ip="room.selfVirtualIp" label="我的虚拟 IP" />
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ServerStackIcon class="w-4 h-4 text-gray-400" />
            <span>房主 MC 版本</span>
          </div>
          <span class="text-xs text-gray-900">{{ room.hostMcVersion || '-' }}</span>
        </div>
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ServerStackIcon class="w-4 h-4 text-gray-400" />
            <span>房主端口</span>
          </div>
          <span class="text-xs text-gray-900">{{ room.hostMcPort || '-' }}</span>
        </div>
        <div class="px-1 py-3 flex items-center justify-between">
          <div class="flex items-center gap-2 text-sm text-gray-600">
            <ClockIcon class="w-4 h-4 text-gray-400" />
            <span>剩余时间</span>
          </div>
          <span class="text-xs" :class="remainingSeconds < 300 ? 'text-red-600' : 'text-gray-900'">
            {{ remainingText }}
          </span>
        </div>
      </div>
    </Card>

    <!-- 整合包要求（联机大厅阶段 4：房主关联整合包时显示，自动校验本地是否已装同款） -->
    <ModpackRequirementCard v-if="room.hostModpack" :modpack="room.hostModpack" />

    <!-- P2P 连接状态 -->
    <Card title="P2P 连接">
      <div class="py-2 flex items-center justify-between">
        <span class="text-xs text-gray-500">WebRTC 状态</span>
        <span
          class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium"
          :class="{
            'bg-green-50 text-green-700': connState === 'connected',
            'bg-blue-50 text-blue-700': connState === 'connecting' || connState === 'new',
            'bg-yellow-50 text-yellow-700': connState === 'disconnected',
            'bg-red-50 text-red-700': connState === 'failed' || connState === 'closed',
          }"
        >
          {{ connState }}
        </span>
      </div>
      <div v-if="connState === 'connected'" class="mt-2 p-2 bg-green-50 rounded text-xs text-green-700">
        <div class="flex items-start gap-1.5">
          <ExclamationTriangleIcon class="w-3.5 h-3.5 mt-0.5 shrink-0" />
          <div class="flex-1">
            <div>连接已建立，请在 Minecraft 中「多人游戏 → 直接连接」输入房主虚拟 IP 加入</div>
            <div class="mt-1 flex items-center gap-1.5">
              <code class="bg-white px-1.5 py-0.5 rounded text-green-800 border border-green-200">
                {{ room.hostVirtualIp || '（等待房主广播）' }}
              </code>
              <Tooltip text="复制房主虚拟 IP">
                <Button
                  type="ghost"
                  size="mini"
                  :disabled="!room.hostVirtualIp"
                  @click="copyText(room.hostVirtualIp, '房主虚拟 IP')"
                >
                  <template #icon><ClipboardDocumentIcon class="w-3.5 h-3.5" /></template>
                </Button>
              </Tooltip>
            </div>
          </div>
        </div>
      </div>
      <div v-else-if="connState === 'failed'" class="mt-2 p-2 bg-red-50 rounded text-xs text-red-700">
        <ExclamationTriangleIcon class="w-3.5 h-3.5 inline mr-1" />
        连接失败，可能是 NAT 兼容性问题。请检查网络环境后重试
      </div>
    </Card>

    <!-- 退出房间按钮 -->
    <div class="pt-2">
      <Button type="outline" long :loading="store.roomLoading" @click="handleLeaveRoom">
        <template #icon><XCircleIcon class="w-4 h-4" /></template>
        退出房间
      </Button>
    </div>
  </div>
</template>
