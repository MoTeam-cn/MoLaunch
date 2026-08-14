<script setup lang="ts">
/**
 * 房主面板（阶段三 mesh 拓扑）
 *
 * 显示房间信息 + 加入申请列表（参与者驱动）+ 参与者列表 + P2P 状态 + 关闭按钮。
 *
 * 业务逻辑（信令轮询 / 授权前置 Offer / 自动放行 Answer / 踢出封禁 / 关闭房间）由
 * 全局联机会话 onlineSession 持有（App 级初始化，切页不断连），此处仅消费状态：
 * - joinedRequests / bannedList / handleConfirm / handleKick 等来自会话
 * - WebRTC 实例通过 inject 获取（hostMesh，多 PC 管理器）
 * - TUN 桥接与数据分发由会话统一管理（onTunPacket 按角色路由）
 */

import { ref, computed, inject } from 'vue'
import { useOnlineStore } from '@/stores/online'
import { useWebRTCMesh } from '@/composables/useWebRTCMesh'
import { getOnlineSession } from '@/composables/online/onlineSession'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import Drawer from '@/components/common/Drawer.vue'
import Tag from '@/components/common/Tag.vue'
import AlertV2 from '@/components/common/AlertV2.vue'
import PendingAnswerList from './PendingAnswerList.vue'
import ParticipantList from './ParticipantList.vue'
import BannedList from './BannedList.vue'
import KickConfirmDialog from './KickConfirmDialog.vue'
import WhitelistEditor from './WhitelistEditor.vue'
import HostRoomInfoCard from './HostRoomInfoCard.vue'
import RoomToolsDrawer from './RoomToolsDrawer.vue'
import ConnectionTransportStatus from './ConnectionTransportStatus.vue'
import P2pFailureCard from './P2pFailureCard.vue'
import {
  XCircleIcon,
  UsersIcon,
  ClockIcon,
  ShieldCheckIcon,
  ArrowPathIcon,
  Cog6ToothIcon,
} from '@heroicons/vue/24/outline'

const store = useOnlineStore()
const hostMesh = inject('hostMesh') as ReturnType<typeof useWebRTCMesh>

/** 房主业务逻辑来自全局联机会话（轮询/Offer/交互处理/TUN 均常驻应用生命周期） */
const session = getOnlineSession()
const {
  bannedList,
  banServerTime,
  confirming,
  handleConfirm,
  handleKick,
  handleUnban,
  refreshBans,
  handleCloseRoom,
} = session

const room = computed(() => store.roomState)

/** 待处理加入申请：参与者处于 joined/answered 状态（授权前置，确认后才生成 Offer） */
const joinedRequests = computed(() =>
  store.roomState.participants.filter(
    (p) => p.status === 'joined' || p.status === 'answered',
  ),
)

/** 踢出确认弹窗状态（null=关闭，有值=正在选择封禁时长） */
const kickTarget = ref<{ participantId: string; devicePk: string; virtualIp?: string } | null>(null)

/** 列表抽屉开关：待确认申请 / 参与者 / 封禁列表（详情页仅保留入口按钮） */
const pendingDrawerOpen = ref(false)
const participantDrawerOpen = ref(false)
const banDrawerOpen = ref(false)
/** 房间工具抽屉开关（检查 MC 服务 / 网络连通性 / 端口自动检测） */
const toolsDrawerOpen = ref(false)

function onKick(participantId: string, devicePk: string) {
  const p = room.value.participants.find((x) => x.participantId === participantId)
  kickTarget.value = { participantId, devicePk, virtualIp: p?.virtualIp }
}

function onConfirmKick(banDuration: number | null) {
  const target = kickTarget.value
  kickTarget.value = null
  if (target) void handleKick(target.participantId, target.devicePk, banDuration)
}

function onCloseKick() {
  kickTarget.value = null
}
/** 已联通参与者数（channel open） */
const connectedCount = computed(() => hostMesh.connectedCount())
/** 房主全部参与者 PC（状态行传输方式检测用） */
const hostPcs = computed(() => hostMesh.getConnPcs())
/** 已确认参与者数（status='confirmed'） */
const confirmedCount = computed(
  () => store.roomState.participants.filter((p) => p.status === 'confirmed').length,
)

/** P2P 已联通操作指引（AlertV2 纯文本 message） */
const connectedHintMessage =
  '已联通，请在 Minecraft 内按 Esc → 「开放给局域网」开关。开放后启动器会自动捕获端口并广播给所有参与者，加入方在「多人游戏 → 直接连接」输入你的虚拟 IP 即可加入'

/** 获取参与者连接状态文本（用于 UI 显示） */
function participantStateText(participantId: string): string {
  return hostMesh.getConnState(participantId) ?? 'unknown'
}

/** 获取参与者上报的 NAT 类型（未上报返回 null，参与者列表展示用） */
function participantNatType(participantId: string): string | null {
  return session.participantNatTypes.get(participantId) ?? null
}

/** 连接状态 failed 的参与者（P2P 组网失败诊断展示用） */
const failedParticipants = computed(() =>
  store.roomState.participants.filter(
    (p) => hostMesh.getConnState(p.participantId) === 'failed',
  ),
)

/** 失败参与者的 NAT 条目（房主侧诊断卡片展示用） */
const failedPeerNats = computed(() =>
  failedParticipants.value.map((p) => ({
    label: `设备 ${p.devicePk.slice(0, 8)}`,
    natType: session.participantNatTypes.get(p.participantId) ?? null,
  })),
)
</script>

<template>
  <div class="space-y-4">
    <AlertV2 type="info" message="P2P联机对房主的网络质量要求较高，如遇连接不上可尝试更换房主" />
    <AlertV2 type="info" message="如遇到违法违规房间，请及时向我们举报" />
    <HostRoomInfoCard />

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
        <ConnectionTransportStatus :pcs="hostPcs" :ice-servers="room.iceServers" />
        <AlertV2 v-if="connectedCount > 0" type="info" :message="connectedHintMessage" />
        <P2pFailureCard
          v-if="failedParticipants.length > 0"
          :self-nat-type="store.natResult?.type ?? null"
          :peers="failedPeerNats"
        />
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

    <!-- 列表抽屉入口：详情页仅保留按钮，待确认申请 / 参与者 / 封禁列表全部收进抽屉 -->
    <Card title="房间管理">
      <div class="grid grid-cols-4 gap-2">
        <Button type="outline" size="small" @click="pendingDrawerOpen = true">
          <template #icon><ClockIcon class="w-3.5 h-3.5" /></template>
          <span class="flex items-center gap-1">
            加入申请
            <Tag v-if="joinedRequests.length > 0" size="small" color="red">{{ joinedRequests.length }}</Tag>
          </span>
        </Button>
        <Button type="outline" size="small" @click="participantDrawerOpen = true">
          <template #icon><UsersIcon class="w-3.5 h-3.5" /></template>
          <span class="flex items-center gap-1">
            参与者
            <Tag size="small">{{ room.participants.length }}</Tag>
          </span>
        </Button>
        <Button type="outline" size="small" @click="banDrawerOpen = true">
          <span class="flex items-center gap-1">
            封禁
            <Tag size="small">{{ bannedList.length }}</Tag>
          </span>
        </Button>
        <Button type="outline" size="small" @click="toolsDrawerOpen = true">
          <template #icon><Cog6ToothIcon class="w-3.5 h-3.5" /></template>
          <span>工具</span>
        </Button>
      </div>
    </Card>

    <!-- 房间工具抽屉：检查 MC 服务 / 网络连通性 / 端口自动检测 -->
    <RoomToolsDrawer v-model:visible="toolsDrawerOpen" />

    <!-- 待确认加入申请抽屉 -->
    <Drawer
      v-model:visible="pendingDrawerOpen"
      title="待处理加入申请"
      placement="right"
      :width="420"
      render-in-place
      popup-container="#app-content"
    >
      <PendingAnswerList :requests="joinedRequests" :busy="confirming" @confirm="handleConfirm" />
    </Drawer>

    <!-- 参与者列表抽屉 -->
    <Drawer
      v-model:visible="participantDrawerOpen"
      title="参与者列表"
      placement="right"
      :width="420"
      render-in-place
      popup-container="#app-content"
    >
      <ParticipantList
        :participants="room.participants"
        :conn-state-text="participantStateText"
        :nat-type-of="participantNatType"
        @kick="onKick"
      />
    </Drawer>

    <!-- 封禁列表抽屉 -->
    <Drawer
      v-model:visible="banDrawerOpen"
      placement="right"
      :width="420"
      render-in-place
      popup-container="#app-content"
    >
      <template #title>
        <div class="flex items-center gap-1">
          <span>封禁列表</span>
          <Tooltip text="刷新封禁列表">
            <Button type="ghost" size="mini" @click="refreshBans">
              <template #icon><ArrowPathIcon class="w-3.5 h-3.5" /></template>
            </Button>
          </Tooltip>
        </div>
      </template>
      <BannedList :bans="bannedList" :server-time="banServerTime" @unban="handleUnban" />
    </Drawer>

    <!-- 踢出确认弹窗（选择封禁时长） -->
    <KickConfirmDialog
      v-if="kickTarget"
      :device-pk="kickTarget.devicePk"
      :virtual-ip="kickTarget.virtualIp"
      @close="onCloseKick"
      @confirm="onConfirmKick"
    />

    <div class="pt-2">
      <Button type="outline" long :loading="store.roomLoading" @click="handleCloseRoom">
        <template #icon><XCircleIcon class="w-4 h-4" /></template>
        关闭房间
      </Button>
    </div>
  </div>
</template>
